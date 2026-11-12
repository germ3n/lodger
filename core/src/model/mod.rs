//! The model layer: Qwen (the size `model.size` picks) scoring fixed choices against a
//! situation, on a worker thread.
//!
//! Ported from `~/engine/game/base/src/ai/`, with the game taken out. [`ask`] hands it a
//! situation and questions with choices, and it answers the choice it finds most probable as the
//! continuation, with a confidence; it also writes the few short lines `brain.text` asks of it
//! (see `docs/model.md`). One
//! [`Model`] per process, owned by the `Lodger`, holding the worker once it has started.

pub mod queue;

#[cfg(feature = "model")]
mod infer;

pub use queue::{Job, Leash, Reply, Scorer, Worker, MAX_QUEUED, SUPERSEDED};

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

/// A question with the choices the model scores. Every choice is text the mod wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub text: String,
    pub choices: Vec<String>,
}

/// The winning choice and its probability among the choices.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub choice: String,
    pub confidence: f32,
}

/// How the model is loaded and asked: where the file is, how many threads, how long a context,
/// how the answer is drawn from the scores, and the seed the draws start from.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub path: PathBuf,
    pub threads: u32,
    pub context: u32,
    /// 0 picks the best choice every time; above 0 draws weighted by the scores, flatter the
    /// higher. See `docs/brain.md`.
    pub temperature: f64,
    /// Where the draws start. Logged, so a run can be replayed.
    pub seed: u64,
    /// The model's layers on the GPU; the rest run on the CPU.
    pub gpu_layers: u32,
    /// The rest between each piece of work, in milliseconds (`model.pause_ms`).
    pub pause_ms: u32,
}

/// How the prompt is fed to the model: this many tokens at a time, with a rest between, so a
/// long situation is not one burst of GPU work the game's frames wait behind.
pub const DECODE_CHUNK: usize = 64;
pub const DECODE_REST: std::time::Duration = std::time::Duration::from_millis(1);
/// How a choice is scored (`docs/model.md`, Scoring): 0 `plain`, its tokens' mean
/// log-probability after `The answer is: `; 1 `spaced`, the same with the space where Qwen's
/// tokens carry it (the prompt ending `The answer is:`, each choice read as ` choice`); 2
/// `calibrated`, spaced, less the same choice's score with no situation, so a choice is judged
/// by how much the situation raises it, not by how common its words are; 3 `listed`, spaced,
/// with the choices named in the question first (`Its choices: a; b; c.`), so it picks among
/// them rather than judging each alone; 4 `half`, spaced, less half the `calibrated` baseline.
/// The baseline is the choice's score after the frame alone, no memory lines. 5 `lettered`: the
/// choices listed under letters and the letter answered scored, in three orders averaged. 6
/// `mixed`: lettered for a question of more than five choices (the menu), spaced for the rest.
/// `mixed` by default: measured against the 1.5B and the 4B, the one way that both reads the
/// situation and picks from the menu to fit it (`docs/model.md`, Scoring); the others stay for
/// `core/tests/frames.rs` to compare.
pub static SCORING: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(6);

/// The rest the config asks for (`model.pause_ms`), for every decode: set as the model starts.
pub static PAUSE_MS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// A small, fast generator for the draws (SplitMix64): no dependency, a few instructions a
/// number, and the same sequence from the same seed on every machine, which a replay needs.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

enum Slot {
    Unstarted,
    Running(Arc<Worker>),
    Closed,
}

/// The model as the mind holds it: settings, and the worker once it has started.
pub struct Model {
    settings: Settings,
    slot: Mutex<Slot>,
    next_id: AtomicU64,
    /// Set while `warm` is loading, so `status` can say so from another thread.
    loading: std::sync::atomic::AtomicBool,
    /// The last loading error, for `status`.
    error: Mutex<String>,
    /// The draws' generator, shared by the asks; the worker draws with it under the lock.
    rng: Arc<Mutex<Rng>>,
}

impl Model {
    pub fn new(settings: Settings) -> Self {
        let rng = Arc::new(Mutex::new(Rng::new(settings.seed)));
        PAUSE_MS.store(
            settings.pause_ms.min(100),
            std::sync::atomic::Ordering::Relaxed,
        );

        Self {
            settings,
            slot: Mutex::new(Slot::Unstarted),
            next_id: AtomicU64::new(1),
            loading: std::sync::atomic::AtomicBool::new(false),
            error: Mutex::new(String::new()),
            rng,
        }
    }

    /// Restarts the draws from `seed`: when a world opens, its seed joins the start's.
    pub fn reseed(&self, seed: u64) {
        *self.rng.lock().unwrap_or_else(|err| err.into_inner()) = Rng::new(seed);
    }

    /// The seed the draws started from, for the log.
    pub fn seed(&self) -> u64 {
        self.settings.seed
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Whether the model file is there to load. Says nothing about whether it is loaded.
    pub fn file_present(&self) -> bool {
        self.settings.path.is_file()
    }

    /// Whether the worker is running with the model loaded.
    pub fn loaded(&self) -> bool {
        matches!(&*self.lock(), Slot::Running(_))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Slot> {
        self.slot.lock().unwrap_or_else(|err| err.into_inner())
    }

    #[cfg(feature = "model")]
    fn start_worker(&self) -> Result<Worker, String> {
        let settings = self.settings.clone();
        let rng = Arc::clone(&self.rng);

        Worker::start(move || infer::load(&settings, rng))
    }

    #[cfg(not(feature = "model"))]
    fn start_worker(&self) -> Result<Worker, String> {
        Err("the model is not built".to_string())
    }

    fn worker(&self) -> Result<Arc<Worker>, String> {
        let mut slot = self.lock();

        match &*slot {
            Slot::Running(worker) => return Ok(Arc::clone(worker)),
            Slot::Closed => return Err("the model has been shut down".to_string()),
            Slot::Unstarted => {}
        }

        let worker = Arc::new(self.start_worker()?);
        *slot = Slot::Running(Arc::clone(&worker));

        Ok(worker)
    }

    /// Loads the model now rather than on the first question, blocking for the second or two it
    /// takes. An error says why it cannot, and is kept for [`Model::status_line`].
    pub fn warm(&self) -> Result<(), String> {
        self.loading.store(true, Ordering::SeqCst);
        let result = self.worker().map(|_| ());
        self.loading.store(false, Ordering::SeqCst);
        *self.error.lock().unwrap_or_else(|err| err.into_inner()) = match &result {
            Ok(()) => String::new(),
            Err(err) => err.clone(),
        };

        result
    }

    pub fn loading(&self) -> bool {
        self.loading.load(Ordering::SeqCst)
    }

    /// The last error `warm` met, or empty.
    pub fn error(&self) -> String {
        self.error
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    /// One line for a screen: what the model is doing.
    pub fn status_line(&self) -> String {
        if self.loaded() {
            "The model is loaded".to_string()
        } else if self.loading() {
            "Loading the model...".to_string()
        } else if !self.file_present() {
            format!(
                "The model is not downloaded ({})",
                self.settings.path.display()
            )
        } else {
            let error = self.error();

            if error.is_empty() {
                "The model is not loaded yet".to_string()
            } else {
                format!("The model failed to load: {error}")
            }
        }
    }

    /// Unloads the model and stops the worker. The context, the model and the backend are freed
    /// on the worker thread in that order (see `infer.rs`). A later question starts it again.
    pub fn unload(&self) {
        let worker = {
            let mut slot = self.lock();

            match std::mem::replace(&mut *slot, Slot::Unstarted) {
                Slot::Running(worker) => Some(worker),
                _ => None,
            }
        };

        if let Some(worker) = worker {
            log::info!("[model] unloading");
            worker.stop();
        }
    }

    /// Unloads and refuses every later question: the process is ending.
    pub fn shutdown(&self) {
        self.unload();
        *self.lock() = Slot::Closed;
    }

    /// Scores `questions` against `situation` on the worker, answering at once with the job's
    /// id; the reply arrives on `reply` later. A newer job with the same `key` supersedes one
    /// still queued. Refused, without a reply, when the queue is full, the model is not built,
    /// the file is missing, or the model is shut down.
    pub fn ask_async(
        &self,
        situation: &str,
        questions: Vec<Question>,
        key: Option<String>,
        reply: Sender<Reply>,
    ) -> Result<u64, String> {
        validate(situation, &questions)?;
        let worker = self.worker()?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        worker.submit(Job {
            id,
            key,
            situation: situation.to_string(),
            questions,
            generate: None,
            reply,
        })?;

        Ok(id)
    }

    /// Has the model write a few words after `prompt` on the worker (`brain.text = generate`):
    /// within `leash` (its tokens, letters and spaces, digits if it says), to a line's end. The reply's
    /// `text` carries them. Refused as `ask_async` is.
    pub fn generate_async(
        &self,
        prompt: &str,
        leash: Leash,
        key: Option<String>,
        reply: Sender<Reply>,
    ) -> Result<u64, String> {
        if prompt.is_empty() {
            return Err("prompt is empty".to_string());
        }
        let worker = self.worker()?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        worker.submit(Job {
            id,
            key,
            situation: prompt.to_string(),
            questions: Vec::new(),
            generate: Some(Leash {
                max_tokens: leash.max_tokens.max(1),
                ..leash
            }),
            reply,
        })?;

        Ok(id)
    }

    /// [`Model::generate_async`] and waits. For tests and tools.
    pub fn generate(&self, prompt: &str, leash: Leash) -> Result<String, String> {
        let (tx, rx) = channel();
        self.generate_async(prompt, leash, None, tx)?;
        let reply = rx
            .recv()
            .map_err(|_| "the model worker stopped".to_string())?;
        reply.result?;

        Ok(reply.text.unwrap_or_default())
    }

    /// [`Model::ask_async`] and waits. Inference takes tens to hundreds of milliseconds, so
    /// this is for tests and tools, never for a tick.
    pub fn ask(&self, situation: &str, questions: &[Question]) -> Result<Vec<Answer>, String> {
        validate(situation, questions)?;
        let (tx, rx) = channel();
        self.ask_async(situation, questions.to_vec(), None, tx)?;

        rx.recv()
            .map_err(|_| "the model worker stopped".to_string())?
            .result
    }

    /// Where the model file is, for messages.
    pub fn path(&self) -> &Path {
        &self.settings.path
    }
}

impl Drop for Model {
    /// A dropped model stops its worker and so frees the context, the model and llama.cpp's
    /// backend, on the worker thread, in that order. Without this a `Lodger` dropped at
    /// `craftkit_shutdown` left the worker alive and the backend initialized, and the next
    /// `init` in the same process (a second test, or a client opening the mod again) could not
    /// load: `BackendAlreadyInitialized`.
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// The model file a developer keeps under `core/models/` for the real-model tests: the 1.5B
/// (`qwen-1.5b.gguf`, the default size) when it is there, else the small one (`qwen.gguf`).
/// Neither is in the repository.
#[cfg(test)]
pub(crate) fn local_model_path() -> Option<PathBuf> {
    ["qwen-1.5b.gguf", "qwen.gguf"]
        .iter()
        .map(|name| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("models")
                .join(name)
        })
        .find(|path| path.is_file())
}

/// One lock for every test that loads the real model or touches the exported static lodger:
/// llama.cpp's backend is one per process, and the exported functions keep one mod, so such
/// tests take turns.
#[cfg(test)]
pub(crate) static TEST_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The model's one per-process slot in tests: a fake scorer installed in place of llama.cpp.
#[cfg(test)]
impl Model {
    pub(crate) fn with_worker(settings: Settings, worker: Worker) -> Self {
        let model = Self::new(settings);
        *model.lock() = Slot::Running(Arc::new(worker));

        model
    }
}

fn validate(situation: &str, questions: &[Question]) -> Result<(), String> {
    if situation.is_empty() {
        return Err("situation is empty".to_string());
    }

    if questions.is_empty() {
        return Err("questions are empty".to_string());
    }

    for question in questions {
        if question.text.is_empty() {
            return Err("question is empty".to_string());
        }

        if question.choices.is_empty() {
            return Err("choices are empty".to_string());
        }

        for choice in &question.choices {
            if choice.is_empty() {
                return Err("choice is empty".to_string());
            }
        }
    }

    Ok(())
}

/// Probabilities from scores. Empty in, empty out; equal scores share evenly.
pub fn softmax(scores: &[f32]) -> Vec<f32> {
    if scores.is_empty() {
        return Vec::new();
    }

    let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut weights = Vec::with_capacity(scores.len());
    let mut sum = 0.0f32;

    for score in scores {
        let weight = (*score - max).exp();
        weights.push(weight);
        sum += weight;
    }

    if sum == 0.0 {
        let even = 1.0 / scores.len() as f32;

        return vec![even; scores.len()];
    }

    for weight in &mut weights {
        *weight /= sum;
    }

    weights
}

/// The log-probability of `token` under `logits`.
pub fn log_softmax_at(logits: &[f32], token: i32) -> Result<f32, String> {
    let idx = usize::try_from(token).map_err(|_| "token id is negative".to_string())?;
    let logit = *logits
        .get(idx)
        .ok_or_else(|| "token is outside the vocab".to_string())?;
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;

    for value in logits {
        sum += (*value - max).exp();
    }

    if sum == 0.0 {
        return Err("logits are empty".to_string());
    }

    Ok(logit - max - sum.ln())
}

/// The index of the largest probability; a tie keeps the first.
pub fn winning_index(probs: &[f32]) -> usize {
    let mut best = 0usize;

    for idx in 1..probs.len() {
        if probs[idx] > probs[best] {
            best = idx;
        }
    }

    best
}

/// The answer for a question from its choices' scores. The confidence is always the chosen
/// choice's probability under the plain softmax, whatever the temperature, so a log reads the
/// same way at any setting.
///
/// With `temperature` 0 the best choice wins, every time. Above 0 the pick is a draw weighted
/// by `softmax(scores / temperature)`: at 1 the model's own odds, below 1 sharper (the best
/// choice most of the time), above 1 flatter. This is what keeps the same situation from
/// getting the same answer on every start, which the engine did (owner).
pub fn answer_for(choices: &[String], scores: &[f32], temperature: f64, rng: &mut Rng) -> Answer {
    let probs = softmax(scores);
    let index = if temperature <= 0.0 || choices.len() < 2 {
        winning_index(&probs)
    } else {
        let scaled: Vec<f32> = scores
            .iter()
            .map(|score| (*score as f64 / temperature) as f32)
            .collect();
        draw(&softmax(&scaled), rng.next_f64())
    };

    Answer {
        choice: choices[index].clone(),
        confidence: probs[index],
    }
}

/// The index a uniform draw `u` in [0, 1) lands on under `weights`, which sum to one.
fn draw(weights: &[f32], u: f64) -> usize {
    let mut cumulative = 0.0f64;

    for (idx, weight) in weights.iter().enumerate() {
        cumulative += *weight as f64;

        if u < cumulative {
            return idx;
        }
    }

    weights.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings {
            path: PathBuf::from("/nowhere/qwen.gguf"),
            threads: 2,
            context: 512,
            temperature: 0.0,
            seed: 1,
            gpu_layers: 0,
            pause_ms: 1,
        }
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn temperature_zero_always_picks_the_best_and_a_tie_the_first() {
        let mut rng = Rng::new(7);
        for _ in 0..20 {
            let answer = answer_for(&names(&["a", "b", "c"]), &[1.0, 3.0, 2.0], 0.0, &mut rng);
            assert_eq!(answer.choice, "b");
        }
        let tie = answer_for(&names(&["a", "b"]), &[1.0, 1.0], 0.0, &mut rng);
        assert_eq!(tie.choice, "a");
        assert!((tie.confidence - 0.5).abs() < 1.0e-5);
    }

    #[test]
    fn a_warm_draw_follows_the_scores_and_the_same_seed_repeats_it() {
        let choices = names(&["rare", "common"]);
        let scores = [0.0, 2.0];
        let mut rng = Rng::new(42);
        let mut common = 0;
        for _ in 0..2000 {
            if answer_for(&choices, &scores, 1.0, &mut rng).choice == "common" {
                common += 1;
            }
        }
        // softmax([0, 2]) gives the common choice about 88%.
        assert!((1600..=1950).contains(&common), "{common}");

        let mut a = Rng::new(9);
        let mut b = Rng::new(9);
        let mut c = Rng::new(10);
        let run = |rng: &mut Rng| -> Vec<String> {
            (0..30)
                .map(|_| answer_for(&choices, &[0.0, 0.5], 1.0, rng).choice)
                .collect()
        };
        let first = run(&mut a);
        assert_eq!(first, run(&mut b), "the same seed draws the same");
        assert_ne!(first, run(&mut c), "another seed draws differently");
    }

    #[test]
    fn the_confidence_is_the_plain_probability_whatever_the_temperature() {
        let choices = names(&["a", "b"]);
        let mut rng = Rng::new(3);
        let answer = answer_for(&choices, &[0.0, 0.0], 1.5, &mut rng);
        assert!((answer.confidence - 0.5).abs() < 1.0e-5);
    }

    #[test]
    fn a_draw_at_the_very_end_lands_on_the_last_choice() {
        assert_eq!(draw(&[0.5, 0.5], 0.999_999), 1);
        assert_eq!(draw(&[0.5, 0.5], 0.0), 0);
        // Rounding can leave the weights a hair under one; the last choice takes the rest.
        assert_eq!(draw(&[0.3, 0.3], 0.95), 1);
    }

    #[test]
    fn softmax_sums_to_one_and_keeps_order() {
        let probs = softmax(&[1.0, 2.0, 3.0]);
        let sum: f32 = probs.iter().sum();

        assert!((sum - 1.0).abs() < 1.0e-5);
        assert!(probs[2] > probs[1] && probs[1] > probs[0]);
    }

    #[test]
    fn softmax_of_nothing_is_nothing_and_equal_scores_share_evenly() {
        assert!(softmax(&[]).is_empty());
        let probs = softmax(&[2.0, 2.0]);
        assert!((probs[0] - 0.5).abs() < 1.0e-5 && (probs[1] - 0.5).abs() < 1.0e-5);
    }

    #[test]
    fn log_softmax_is_a_distribution() {
        let logits = [1.0, 3.0];
        let low = log_softmax_at(&logits, 0).unwrap();
        let high = log_softmax_at(&logits, 1).unwrap();

        assert!(high > low);
        assert!((low.exp() + high.exp() - 1.0).abs() < 1.0e-5);
        assert!(log_softmax_at(&logits, 2).is_err());
        assert!(log_softmax_at(&logits, -1).is_err());
    }

    #[test]
    fn a_tie_keeps_the_first_choice() {
        assert_eq!(winning_index(&[0.5, 0.5]), 0);
        assert_eq!(winning_index(&[0.2, 0.8]), 1);
    }

    #[test]
    fn empty_situations_questions_and_choices_are_refused_by_name() {
        let model = Model::new(settings());
        let question = |choices: &[&str]| {
            vec![Question {
                text: "Move?".to_string(),
                choices: choices.iter().map(|s| s.to_string()).collect(),
            }]
        };

        assert_eq!(
            model.ask("", &question(&["a"])).unwrap_err(),
            "situation is empty"
        );
        assert_eq!(model.ask("close", &[]).unwrap_err(), "questions are empty");
        assert_eq!(
            model.ask("close", &question(&[])).unwrap_err(),
            "choices are empty"
        );
        assert_eq!(
            model.ask("close", &question(&["a", ""])).unwrap_err(),
            "choice is empty"
        );
        let blank = vec![Question {
            text: String::new(),
            choices: vec!["a".to_string()],
        }];
        assert_eq!(model.ask("close", &blank).unwrap_err(), "question is empty");
    }

    #[test]
    fn a_missing_file_is_reported_when_asked_not_before() {
        let model = Model::new(settings());

        assert!(!model.file_present());
        assert!(!model.loaded());
        let err = model
            .ask(
                "close",
                &[Question {
                    text: "Move?".to_string(),
                    choices: vec!["a".to_string()],
                }],
            )
            .unwrap_err();
        assert!(
            err.contains("qwen.gguf") || err.contains("not built"),
            "{err}"
        );
    }

    #[test]
    fn the_status_line_says_what_the_model_is_doing() {
        let model = Model::new(settings());
        assert!(
            model.status_line().contains("not downloaded"),
            "{}",
            model.status_line()
        );

        // A file that is there but is not a model: warm fails and the line says why.
        let dir = std::env::temp_dir().join(format!("lodger-status-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("qwen.gguf");
        std::fs::write(&path, b"not a model").unwrap();
        let model = Model::new(Settings {
            path,
            threads: 2,
            context: 512,
            temperature: 0.0,
            seed: 1,
            gpu_layers: 0,
            pause_ms: 1,
        });
        assert!(
            model.status_line().contains("not loaded yet"),
            "{}",
            model.status_line()
        );
        let err = model.warm().unwrap_err();
        assert!(!model.loading());
        assert_eq!(model.error(), err);
        assert!(
            model.status_line().contains("failed to load"),
            "{}",
            model.status_line()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_fake_worker_answers_through_the_model_and_shutdown_refuses_after() {
        struct First;

        impl Scorer for First {
            fn score(&mut self, _: &str, questions: &[Question]) -> Result<Vec<Answer>, String> {
                Ok(questions
                    .iter()
                    .map(|q| Answer {
                        choice: q.choices[0].clone(),
                        confidence: 0.9,
                    })
                    .collect())
            }
        }

        let model = Model::with_worker(settings(), Worker::start(|| Ok(First)).unwrap());
        let questions = [Question {
            text: "Move?".to_string(),
            choices: vec!["stay".to_string(), "go".to_string()],
        }];

        assert!(model.loaded());
        assert_eq!(model.ask("close", &questions).unwrap()[0].choice, "stay");

        model.shutdown();
        assert!(!model.loaded());
        assert!(model
            .ask("close", &questions)
            .unwrap_err()
            .contains("shut down"));
    }
}
