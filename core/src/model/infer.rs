//! llama.cpp, through `llama-cpp-2`: loading the model and scoring choices.
//!
//! Scoring, as the engine does it: the situation is decoded once; for each question its text
//! and `The answer is: ` are decoded on top and the next-token logits kept; each choice's score
//! is the mean log-probability of its tokens given the ones before, decoding one token at a
//! time and rewinding the cache between choices. Softmax over the choices gives the answer and
//! its confidence. No sampling, no temperature, no generated text. See `docs/model.md`.

use super::{answer_for, log_softmax_at, Answer, Leash, Question, Rng, Scorer, Settings};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::LlamaToken;
use llama_cpp_2::vocab::LlamaVocab;
use std::mem::ManuallyDrop;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

/// Threads for the prompt's batch decode: one more than the per-token threads, as the engine.
fn batch_threads(threads: u32) -> i32 {
    i32::try_from(threads).unwrap_or(2).saturating_add(1)
}

struct Engine {
    ctx: ManuallyDrop<LlamaContext<'static>>,
    model: *mut LlamaModel,
    #[allow(dead_code)]
    backend: LlamaBackend,
    n_ctx: u32,
    temperature: f64,
    rng: Arc<Mutex<Rng>>,
    /// Per token id, whether its text is letters and spaces alone (`[0]`), or letters, digits
    /// and spaces (`[1]`): the only tokens generation may pick. Built once, on the first write.
    plain_tokens: Option<[Vec<bool>; 2]>,
}

// The engine lives on the worker thread alone; the raw model pointer never crosses threads.
unsafe impl Send for Engine {}

impl Drop for Engine {
    fn drop(&mut self) {
        // ggml asserts at process exit if any buffers are still alive, so the context, then the
        // model, then the backend must be freed in this order.
        unsafe {
            ManuallyDrop::drop(&mut self.ctx);
            drop(Box::from_raw(self.model));
        }
    }
}

pub struct LlamaScorer(Engine);

impl Scorer for LlamaScorer {
    fn score(&mut self, situation: &str, questions: &[Question]) -> Result<Vec<Answer>, String> {
        decode_questions(&mut self.0, situation, questions)
    }

    fn generate(&mut self, prompt: &str, leash: Leash) -> Result<String, String> {
        generate_plain(&mut self.0, prompt, leash)
    }
}

/// Loads the model at the settings' path, or says what is wrong with it.
pub fn load(settings: &Settings, rng: Arc<Mutex<Rng>>) -> Result<LlamaScorer, String> {
    load_engine(settings, rng).map(LlamaScorer)
}

fn load_engine(settings: &Settings, rng: Arc<Mutex<Rng>>) -> Result<Engine, String> {
    let path = &settings.path;

    if !path.is_file() {
        return Err(format!(
            "missing {}, fetch it with /craftkit lodger model download",
            path.display()
        ));
    }

    let mut backend = LlamaBackend::init().map_err(|err| err.to_string())?;
    backend.void_logs();
    let model = LlamaModel::load_from_file(&backend, path, &LlamaModelParams::default())
        .map_err(|err| format!("{}: {err}", path.display()))?;
    let model = Box::into_raw(Box::new(model));
    let model_ref: &'static LlamaModel = unsafe { &*model };
    let ctx = match model_ref.new_context(&backend, context_params(settings)) {
        Ok(ctx) => ctx,
        Err(err) => {
            unsafe { drop(Box::from_raw(model)) };

            return Err(err.to_string());
        }
    };
    log::info!("[model] loaded {}", path.display());

    Ok(Engine {
        ctx: ManuallyDrop::new(ctx),
        model,
        backend,
        n_ctx: settings.context,
        temperature: settings.temperature,
        rng,
        plain_tokens: None,
    })
}

/// Whether a token's text may appear in a generated line: letters of either case (the check
/// lower-cases them; forbidding `Here` pushes the model toward whatever else is left), spaces,
/// digits when `digits`, nothing else, and not empty.
fn is_plain_piece(piece: &[u8], digits: bool) -> bool {
    !piece.is_empty()
        && piece
            .iter()
            .all(|b| b.is_ascii_alphabetic() || *b == b' ' || (digits && b.is_ascii_digit()))
}

/// Only the likeliest tokens making up this share of the probability are drawn from.
const TOP_P: f32 = 0.9;

/// A token drawn from `candidates` (id, logit) at `temperature`, among the likeliest making up
/// `top_p` of the probability, with `r` in 0..1 as the draw. None for no candidates.
fn sample(candidates: &mut [(usize, f32)], temperature: f32, top_p: f32, r: f64) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
    let best = candidates[0].1;
    let weights: Vec<f64> = candidates
        .iter()
        .map(|(_, l)| (((l - best) / temperature.max(1e-3)) as f64).exp())
        .collect();
    let total: f64 = weights.iter().sum();
    // Keep the head of the list up to `top_p` of the mass, at least one.
    let mut kept = 0;
    let mut mass = 0.0;
    while kept < weights.len() && (kept == 0 || mass < top_p as f64 * total) {
        mass += weights[kept];
        kept += 1;
    }
    let mut at = r * mass;
    for (i, w) in weights.iter().take(kept).enumerate() {
        if at < *w {
            return Some(candidates[i].0);
        }
        at -= w;
    }

    Some(candidates[kept - 1].0)
}

/// Whether `piece` would start a word past the `words`th of the line written so far: a piece
/// opening with a space after text, when the line already has that many words.
fn starts_word_past(written: &[u8], piece: &[u8], words: usize) -> bool {
    let so_far = written
        .split(|b| *b == b' ')
        .filter(|w| !w.is_empty())
        .count();

    piece.first() == Some(&b' ') && so_far >= words
}

/// Continues `prompt` greedily under the plain-token mask for `leash`, at most its tokens,
/// stopping at an end-of-generation token, at a token no longer allowed (a newline,
/// punctuation), or when the line would run past its tokens. The caller checks the words.
fn generate_plain(engine: &mut Engine, prompt: &str, leash: Leash) -> Result<String, String> {
    let model = unsafe { &*engine.model };
    let vocab = model.vocab();
    let n_vocab = usize::try_from(model.n_vocab()).unwrap_or(0);
    if engine.plain_tokens.is_none() {
        let pieces: Vec<Option<Vec<u8>>> = (0..n_vocab)
            .map(|id| {
                let token = LlamaToken(id as i32);
                (!vocab.is_eog(token)).then(|| vocab.token_to_piece(token, false, None))
            })
            .collect();
        let mask = |digits: bool| -> Vec<bool> {
            pieces
                .iter()
                .map(|p| p.as_deref().is_some_and(|p| is_plain_piece(p, digits)))
                .collect()
        };
        engine.plain_tokens = Some([mask(false), mask(true)]);
    }
    let mask = &engine.plain_tokens.as_ref().unwrap()[usize::from(leash.digits)];

    engine.ctx.clear_kv_cache();
    let prompt_tokens = tokenize(&vocab, prompt, true);
    if prompt_tokens.is_empty() {
        return Err("prompt produced no tokens".to_string());
    }
    let mut batch = LlamaBatch::new(engine.n_ctx as usize, 1);
    decode_tokens(&mut engine.ctx, &mut batch, &prompt_tokens, 0, true)?;
    let start = i32::try_from(prompt_tokens.len()).unwrap_or(i32::MAX);
    let mut out: Vec<u8> = Vec::new();

    for pos in (start..).take(leash.max_tokens) {
        let logits = logits_row(&engine.ctx, batch.n_tokens() - 1);
        // The best allowed token; nothing allowed means the model wants to stop.
        let mut best: Option<(usize, f32)> = None;
        for (id, logit) in logits.iter().enumerate().take(mask.len()) {
            if mask[id] && best.map(|(_, b)| *logit > b).unwrap_or(true) {
                best = Some((id, *logit));
            }
        }
        let Some((id, best_logit)) = best else { break };
        // If the model's own first choice is a stop (end of line, punctuation, end token), the
        // line is done: the mask only ever picks among what is allowed, so this is read off the
        // unmasked best.
        let unmasked_best = logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap_or(id);
        if unmasked_best != id && logits[unmasked_best] - best_logit > 2.0 {
            break;
        }
        // A draw among the likely allowed tokens, so one moment can give different lines.
        let id = if leash.temperature > 0.0 {
            let mut candidates: Vec<(usize, f32)> = logits
                .iter()
                .enumerate()
                .take(mask.len())
                .filter(|(i, l)| mask[*i] && **l > best_logit - 8.0)
                .map(|(i, l)| (i, *l))
                .collect();
            let r = engine
                .rng
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .next_f64();
            sample(&mut candidates, leash.temperature, TOP_P, r).unwrap_or(id)
        } else {
            id
        };
        let token = LlamaToken(id as i32);
        let piece = vocab.token_to_piece(token, false, None);
        if starts_word_past(&out, &piece, leash.words) {
            break;
        }
        out.extend(piece);
        decode_tokens(&mut engine.ctx, &mut batch, &[token], pos, true)?;
    }

    Ok(String::from_utf8_lossy(&out).trim().to_string())
}

fn context_params(settings: &Settings) -> LlamaContextParams {
    LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(settings.context))
        .with_n_batch(settings.context)
        .with_n_ubatch(settings.context)
        .with_n_threads(i32::try_from(settings.threads).unwrap_or(2))
        .with_n_threads_batch(batch_threads(settings.threads))
}

fn tokenize(vocab: &LlamaVocab, text: &str, add_special: bool) -> Vec<LlamaToken> {
    vocab.tokenize(text.as_bytes(), add_special, false)
}

fn rewind(ctx: &mut LlamaContext, pos: i32) -> Result<(), String> {
    let pos = u32::try_from(pos).map_err(|_| "bad position".to_string())?;
    ctx.kv_cache_seq_rm(0, Some(pos), None)
        .map_err(|err| err.to_string())?;

    Ok(())
}

fn decode_tokens(
    ctx: &mut LlamaContext,
    batch: &mut LlamaBatch,
    tokens: &[LlamaToken],
    pos: i32,
    want_logits: bool,
) -> Result<(), String> {
    if tokens.is_empty() {
        return Err("nothing to decode".to_string());
    }

    let len = i32::try_from(tokens.len()).map_err(|_| "too many tokens".to_string())?;
    let end = pos
        .checked_add(len)
        .ok_or_else(|| "context exceeded".to_string())?;

    if end > i32::try_from(ctx.n_ctx()).unwrap_or(i32::MAX) {
        return Err("context exceeded".to_string());
    }

    let width = usize::try_from(ctx.n_batch()).unwrap_or(1).max(1);
    let mut offset = 0usize;

    while offset < tokens.len() {
        let count = (tokens.len() - offset).min(width);
        let last = offset + count == tokens.len();
        batch.clear();

        for idx in 0..count {
            let at = pos + i32::try_from(offset + idx).unwrap_or(i32::MAX);
            let logits = want_logits && last && idx + 1 == count;
            batch
                .add(tokens[offset + idx], at, &[0], logits)
                .map_err(|err| err.to_string())?;
        }

        ctx.decode(batch).map_err(|err| err.to_string())?;
        offset += count;
    }

    Ok(())
}

fn logits_row(ctx: &LlamaContext, index: i32) -> Vec<f32> {
    ctx.get_logits_ith(index).to_vec()
}

fn mean_logprob(
    ctx: &mut LlamaContext,
    batch: &mut LlamaBatch,
    prefix: &[f32],
    choice: &[LlamaToken],
    pos: i32,
) -> Result<f32, String> {
    if choice.is_empty() {
        return Err("choice produced no tokens".to_string());
    }

    let mut total = log_softmax_at(prefix, choice[0].0)?;

    if choice.len() == 1 {
        return Ok(total);
    }

    for (cursor, idx) in (pos..).zip(0..choice.len() - 1) {
        decode_tokens(ctx, batch, &[choice[idx]], cursor, true)?;
        let row = logits_row(ctx, batch.n_tokens() - 1);
        total += log_softmax_at(&row, choice[idx + 1].0)?;
    }

    Ok(total / choice.len() as f32)
}

fn decode_questions(
    engine: &mut Engine,
    situation: &str,
    questions: &[Question],
) -> Result<Vec<Answer>, String> {
    // The context keeps its model private in this version; the engine holds the same model.
    let vocab = unsafe { &*engine.model }.vocab();
    engine.ctx.clear_kv_cache();
    let situation_tokens = tokenize(&vocab, situation, true);

    if situation_tokens.is_empty() {
        return Err("situation produced no tokens".to_string());
    }

    let mut batch = LlamaBatch::new(engine.n_ctx as usize, 1);
    decode_tokens(&mut engine.ctx, &mut batch, &situation_tokens, 0, false)?;
    let sit_end = i32::try_from(situation_tokens.len()).unwrap_or(i32::MAX);
    let mut answers = Vec::with_capacity(questions.len());

    for (question_idx, question) in questions.iter().enumerate() {
        if question_idx > 0 {
            rewind(&mut engine.ctx, sit_end)?;
        }

        let suffix = format!("\n{}\nThe answer is: ", question.text);
        let suffix_tokens = tokenize(&vocab, &suffix, false);

        if suffix_tokens.is_empty() {
            return Err("question produced no tokens".to_string());
        }

        decode_tokens(&mut engine.ctx, &mut batch, &suffix_tokens, sit_end, true)?;
        let q_end = sit_end + i32::try_from(suffix_tokens.len()).unwrap_or(i32::MAX);
        let prefix_logits = logits_row(&engine.ctx, batch.n_tokens() - 1);
        let mut scores = Vec::with_capacity(question.choices.len());
        let mut extended = false;

        for choice in &question.choices {
            if extended {
                rewind(&mut engine.ctx, q_end)?;
                extended = false;
            }

            let choice_tokens = tokenize(&vocab, choice, false);

            if choice_tokens.is_empty() {
                return Err(format!("choice {choice} produced no tokens"));
            }

            let score = mean_logprob(
                &mut engine.ctx,
                &mut batch,
                &prefix_logits,
                &choice_tokens,
                q_end,
            )?;

            if choice_tokens.len() > 1 {
                extended = true;
            }

            scores.push(score);
        }

        let answer = {
            let mut rng = engine.rng.lock().unwrap_or_else(|err| err.into_inner());

            answer_for(&question.choices, &scores, engine.temperature, &mut rng)
        };
        answers.push(answer);
    }

    Ok(answers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_piece_is_letters_and_spaces_and_digits_only_when_counting() {
        assert!(is_plain_piece(b" here", false));
        assert!(is_plain_piece(b"Here", false), "the check lower-cases");
        assert!(!is_plain_piece(b"123", false));
        assert!(is_plain_piece(b" 14", true));
        assert!(!is_plain_piece(b"\n", true));
        assert!(!is_plain_piece(b"!", true));
        assert!(!is_plain_piece(b"", true));
    }

    #[test]
    fn a_draw_keeps_to_the_likely_tokens_and_a_low_temperature_is_nearly_greedy() {
        let mut c = vec![(1, 0.0), (2, 5.0), (3, 4.5), (4, -20.0)];
        assert_eq!(
            sample(&mut c.clone(), 0.01, 0.9, 0.99),
            Some(2),
            "near greedy"
        );
        let mut seen = std::collections::HashSet::new();
        for i in 0..100 {
            seen.insert(sample(&mut c, 1.0, 0.9, i as f64 / 100.0).unwrap());
        }
        assert!(
            seen.contains(&2) && seen.contains(&3),
            "both likely ones: {seen:?}"
        );
        assert!(!seen.contains(&4), "never the unlikely one");
        assert_eq!(sample(&mut [], 1.0, 0.9, 0.5), None);
    }

    #[test]
    fn the_decode_stops_before_a_word_past_the_leash() {
        assert!(!starts_word_past(b"", b" the", 1), "the first word");
        assert!(!starts_word_past(b" the", b"re", 1), "a word continued");
        assert!(starts_word_past(b" the other one", b" again", 3));
        assert!(!starts_word_past(b" the other", b" one", 3));
    }
}

/// Tests against the real model, which must be at `core/models/qwen.gguf` (ignored otherwise):
/// `cargo test -p lodger-core -- --ignored model::`.
#[cfg(test)]
mod real {
    use super::super::{Model, Question, Settings};

    fn model() -> Option<Model> {
        super::super::local_model_path().map(|path| {
            Model::new(Settings {
                path,
                threads: 2,
                context: 512,
                temperature: 0.0,
                seed: 1,
            })
        })
    }

    fn question(text: &str, choices: &[&str]) -> Question {
        Question {
            text: text.to_string(),
            choices: choices.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    #[ignore = "needs core/models/qwen-1.5b.gguf or qwen.gguf, not in the repository"]
    fn a_player_who_walks_toward_the_figure_wants_to_be_looked_at() {
        let _turn = super::super::TEST_TURN
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(model) = model() else { return };
        let situation = "You are a quiet presence living unseen in a player's house in a game.\n\
            What you know:\nThe player has seen the figure twice and walked toward it both times, \
            and stood looking at it for a long while.\n";
        let answers = model
            .ask(
                situation,
                &[question(
                    "What does it want from this player?",
                    &[
                        "a place of its own",
                        "to be looked at",
                        "one thing of theirs",
                    ],
                )],
            )
            .unwrap();

        assert_eq!(answers[0].choice, "to be looked at", "{answers:?}");
        model.shutdown();
    }

    #[test]
    #[ignore = "needs core/models/qwen-1.5b.gguf or qwen.gguf, not in the repository"]
    fn a_second_model_in_one_process_loads_after_the_first_is_dropped() {
        let _turn = super::super::TEST_TURN
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(first) = model() else { return };
        first.warm().unwrap();
        assert!(first.loaded());
        drop(first);

        let second = model().unwrap();
        second
            .warm()
            .unwrap_or_else(|err| panic!("the second load failed: {err}"));
        assert!(second.loaded());
    }

    #[test]
    #[ignore = "needs core/models/qwen-1.5b.gguf or qwen.gguf, not in the repository"]
    fn two_questions_on_one_situation_agree_with_each_asked_alone() {
        let _turn = super::super::TEST_TURN
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(model) = model() else { return };
        let situation = "The player lit every room yesterday and logged off early.";
        let one = question(
            "Should it do something soon?",
            &["nothing yet", "something small", "something it will notice"],
        );
        let two = question("When?", &["while they are away", "while they sleep", "now"]);

        let together = model.ask(situation, &[one.clone(), two.clone()]).unwrap();
        let alone_one = model.ask(situation, &[one]).unwrap();
        let alone_two = model.ask(situation, &[two]).unwrap();

        assert_eq!(together[0].choice, alone_one[0].choice);
        assert_eq!(together[1].choice, alone_two[0].choice);
        assert!((together[0].confidence - alone_one[0].confidence).abs() < 1.0e-3);
        model.shutdown();
    }

    /// The situation of the first written line seen in the dev client (2026-11-09), where the
    /// model put `123456` on a sign.
    const DEV_STAY: &[&str] = &[
        "The chest 60 blocks north of the house is opened most, 1 times so far. It holds a bread, an oak sapling, an apple.",
        "Today it did open the door.",
        "Today it did stop the music.",
        "Today it did close the door.",
        "Today they killed their first pig.",
        "Around the house: water 31 blocks across 277 blocks west of the house; sand 294 blocks west of the house; trees 266 blocks west of the house.",
        "They have killed 4 sheep so far.",
        "Now the player is outdoors, walking.",
    ];

    #[test]
    #[ignore = "needs core/models/qwen-1.5b.gguf or qwen.gguf, not in the repository"]
    fn every_kind_of_line_the_model_writes_passes_its_own_check() {
        use crate::brain::text::{self, Kind};
        let _turn = super::super::TEST_TURN
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(model) = model() else { return };
        let lines: Vec<String> = DEV_STAY.iter().map(|l| l.to_string()).collect();
        let mut failed = Vec::new();
        for kind in Kind::ALL {
            let examples: Vec<String> = kind
                .examples()
                .iter()
                .map(|e| text::fill(e, "sheep", 4))
                .collect();
            let refs: Vec<&str> = examples.iter().map(String::as_str).collect();
            let prompt = crate::brain::situation::writing_prompt(&lines, kind.purpose(), &refs);
            let written = model.generate(&prompt, kind.leash()).unwrap();
            let checked = text::check(kind, &written);
            eprintln!("{kind:?}: {written:?} -> {checked:?}");
            if checked.is_none() {
                failed.push(format!("{kind:?} wrote {written:?}"));
            }
        }

        assert!(failed.is_empty(), "{failed:?}");
        model.shutdown();
    }
}
