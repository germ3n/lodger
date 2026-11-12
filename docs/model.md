# Model

The model file, how it is fetched and loaded, and the worker that scores with it
(`core/src/model/`). This is `~/engine/game/base/src/ai/` with the game taken out: the same
scoring, the same queue, the same download, behind the mind's C ABI instead of a Lua library.

## Which model

**Qwen2.5-Instruct or Qwen3, a `q4_k_m` GGUF, in the size `model.size` picks** (owner: "players can choose between the 0.5b, 1.5b or 3b model"; "please add the 7b and 8b models"):

| `model.size` | File | RAM while loaded | A question of four choices on two threads | For |
| --- | --- | --- | --- | --- |
| `0.5b` | `qwen2.5-0.5b-instruct-q4_k_m.gguf`, 491 MB | about 700 MB | tens to a few hundred milliseconds | a small machine; the file `~/engine` runs, byte for byte; the dev client's choice |
| `1.5b`, the default | `qwen2.5-1.5b-instruct-q4_k_m.gguf`, 1.1 GB | about 1.6 GB | a few times slower | most machines; the owner's default for sharper judgment and for writing its own few words (`brain.text`) |
| `3b` | `qwen2.5-3b-instruct-q4_k_m.gguf`, 2.1 GB | about 2.6 GB | slower again, a second or more | a dedicated server with the headroom; under the **Qwen Research License** (non-commercial use only, which it defines as research or evaluation; commercial use needs a license from Alibaba Cloud), not Apache |
| `4b` | `Qwen3-4B-Instruct-2507-Q4_K_M.gguf` from unsloth's repository, 2.5 GB | about 3.2 GB | about the 3B's | the newest and best reader (owner: "how about we offer this too?"); Apache-2.0; non-thinking only, so no `<think>` block gets in the way of scoring; the vendored llama.cpp (llama-cpp-sys-2 0.1.161) knows its `qwen3` architecture |
| `7b` | Qwen's own `qwen2.5-7b-instruct-q4_k_m`, in two parts (`-00001-of-00002.gguf`, 3.99 GB, and `-00002-of-00002.gguf`, 0.69 GB), 4.7 GB in all | about 5.5 GB | slower again; on a GPU a second or so | a big GPU; Apache-2.0; the two parts are two downloads, saved side by side under the split's own names, and llama.cpp loads the second from the first |
| `8b` | Qwen's own `Qwen3-8B-Q4_K_M.gguf`, 5.0 GB | about 6 GB | the slowest | the largest and best reader, for a big GPU; Apache-2.0; Qwen3-8B can think before it answers in a chat, but the lodger reads no chat template and its writing allows letters and spaces alone, so no `<think>` block ever comes |

The 0.5B, 1.5B, 4B, 7B and 8B are Apache-2.0; the 3B is under Qwen's research license, which the
preparing screen names and links before the download. Qwen's own GGUF repository for the 4B answers 401 to a plain download, so its file
comes from unsloth's, which repackages Qwen's model under the same license. Each file's exact
length and SHA-256 are in `ModelSize` (from Hugging Face's `x-linked-size` and `x-linked-etag`),
and the host checks the sum after the download; a file from `model.url` of one's
own is not checked. The 7B's second part is a resource of its own (`model_part_2`), with its own
length and sum, required as the first is, its terms the first's (no second license screen), and
not fetched for a file of one's own (`model.url` or `model.path`). The scoring code is the same for each; only the file, its size and the
download's length differ. Changing the size downloads the
new file at the next start (the title screen gate again) and keeps the old one until deleted.
The timings are the engine's experience with the small one and estimates for the others; the
measure item in the todo replaces them with numbers.

- **Why this size:** 0.5B scores a 300-token situation and four questions of four choices in
  tens to a few hundred milliseconds on two CPU threads, with about 700 MB resident, which any
  server can spare, and was the first default. The owner moved the default to 1.5B
  with the decision that the model writes its own sign and chat words: three times the memory
  and time, still well inside a tick budget on a worker thread, for judgment that holds up over
  a longer situation text and words that read as words. 0.5B stays a setting for a small
  machine.
- **Why Qwen:** Apache-2.0 for every size but the 3B, so the terms a player accepts are plain; GGUF builds by
  Qwen themselves; and it is what the engine already runs, so the scoring code is known to work
  on it. Qwen3-0.6B is also Apache-2.0 and a todo item is to score both on the same log and
  see whether it reads situations better.
- **Why q4_k_m:** Qwen's own quantization, and the smallest at which the engine saw stable
  choices. Community `Q4_K_M` files of the same model are smaller (about 400 MB) and would work;
  the magic check does not care which. The default URL points at Qwen's.

## Downloading

**Picking the size on the screen** (owner): the preparing screen shows a button
with the size it is about to fetch (`Qwen2.5 1.5B, 1117 MB (click for another)`); each press
picks the next of the six, writes `model.size` into the config and starts that download
instead. A model already there skips the screen; the settings page changes it then.

The model is not in the jar. A gigabyte is too much to ship with a mod and Hugging Face serves it
well. The mind declares it as a *resource* in its `init` reply (the SDK's `Resource`: the URL
from `model.url`, the path from `model.path` under the game directory, by default
`config/lodger/models/` and the size's own file name such as `qwen2.5-1.5b-q4_k_m.gguf`, the
`GGUF` magic, the size and the SHA-256), and the host fetches it (owner: "lock them out of the main menu until the
model is downloaded and loaded"):

- **On a client,** the first time the title screen opens with the file missing, a screen comes
  up over it. First the model's license, its name and text with a link to the full terms, and
  `I accept` (kept in `config/craftkit/accepted.json`, so it is asked once per license); then the
  title, a progress bar, the megabytes so far, and a Quit button; Escape does
  nothing. When the file is there the screen says `Loading the lodger's model` with a sweeping
  bar while the mind loads it (a second or two), and then the title screen is back. On failure,
  the error and a Try again button. A player with the file already there never sees any of it.
  Under the size button, a note on the size chosen: how it differs from the others and the VRAM
  it wants (owner). **Play without it** (owner) closes the screen and writes `model.enabled:
  false`: nothing is downloaded, a part file already there is kept, and the rule brain decides;
  a player who only joins servers needs nothing more, the server's model deciding there. The
  settings' **Use the model** turns it back on, and the screen comes back to download.
- **On a dedicated server,** at startup, before the world opens, blocking, with a log line at
  every ten percent and one when the model is loaded. There is no screen to accept the license
  on, so the log prints it and the key to add to `accepted` in `config/craftkit/accepted.json`;
  until it is there and the server restarts, nothing is downloaded and the rules decide. On a
  failed download the server starts anyway, the rules decide, and the next start tries again.
- **It must be a GGUF.** The first four bytes must be `GGUF`; anything else, an HTML error page
  above all, fails on those bytes rather than after half a gigabyte.
- **It is written under a temporary name,** the file's name with `.part` after it, and renamed when complete, so a
  failed download never replaces a good model and a half file is never loaded.
- **`http://` or `https://` only.** Redirects are followed. The connect timeout is 15 seconds.
  A slow connection is still a connection, but a quiet one is not: thirty seconds without a
  byte and it is closed, and the download goes on from where it stopped with an HTTP range, up
  to five tries with a pause between (a server that ignores the range sends the whole file
  again). When every try fails the `.part` file stays, and the next start goes on from it; a
  cancel, the wrong magic or a wrong SHA-256 removes it. Without that limit, the 4B's download
  once hung at 2059 of 2381 MB on a connection gone quiet.
- **`/craftkit lodger status`** says whether the model or the rules decide, and the debug
  screen's model line whether the file is there and the model loaded. Deleting the
  file makes the next start fetch it again, from `model.url`.

The mind never touches the network: `Downloader.java` in the host does, with the JDK's
`HttpClient`, for any Rust mod's resources. The engine's own `download.rs` was ported first and
then removed in favour of this, so that one downloader serves every mod and the
progress bar has one source.

## Loading

The mind links llama.cpp through the `llama-cpp-2` crate, which builds llama.cpp from source into
the shared library, statically, so the jar's native for a platform is one file. The host asks the
mind to load the model right after the download, through the SDK's `warm` (blocking, on a host
thread, while `status` answers the screen), so the first ask never waits; if it somehow is not
loaded, the first ask loads it on the worker thread and waits the second or two.

- **The file must be loose.** llama.cpp maps it from the path; it cannot read one inside the
  jar or any archive.
- **Context 1536** (`model.context`), batch and ubatch the same. The prompt is decoded 64 tokens
  at a time with a rest of `model.pause_ms` between (`DECODE_CHUNK`), so a long situation is not
  one burst of work the game's frames wait behind. **Two threads** (`model.threads`) for decoding
  and one more for the batch, the engine's numbers. Raising threads speeds scoring and takes from
  the server's own threads; the default leaves a four-core server its game.
- **On the GPU by default.** `model.gpu_share` is `1`: every layer on the GPU, Vulkan on Windows
  and Linux, Metal on macOS, each native built with its backend. `0` keeps it on the CPU, and a
  share between puts that share of the layers (of 24, 28 or 36, by size) on the GPU. With no
  usable GPU llama.cpp runs it all on the CPU.
- **Memory:** the default 1.5B's 1.1 GB mapped plus the context's buffers, about 1.6 GB, in VRAM
  when it is on the GPU and RAM when not (the 0.5B about 700 MB). The JVM does not see it: it is
  native memory outside the heap, so `-Xmx` is unaffected and the machine's own memory is what
  must have room.
- **Unloading** after `model.idle_minutes` with no target online, and at server stop
  (`craftkit_shutdown`). The context, then the model, then the backend are freed in that order,
  because ggml asserts at process exit if any buffer is alive, which would take the server down
  with it. The engine's `Engine::drop` has the same comment.

## Scoring

`model/infer.rs`, the engine's `decode_questions` unchanged in substance:

1. Clear the KV cache. Tokenize the situation with the beginning-of-text token and decode it,
   without logits.
2. For each question: rewind the cache to the end of the situation, tokenize
   `\n<question>\nThe answer is:` and decode it, keeping the last position's logits.
3. A question of more than five choices, the menu, is lettered (below). For the others: tokenize
   each choice with a space before it (` move a torch`; no special tokens). Its score is the mean over its tokens of
   the log-probability of each token given the ones before: the first from the question's
   logits, the rest by decoding the choice one token at a time, then rewinding to the end of
   the question for the next choice. A one-token choice costs no decode.
4. Softmax over the choices' scores. At `brain.temperature` 0 the best is the answer, a tie
   keeping the first choice; above 0 (0.7 by default) the answer is a seeded draw weighted by the
   scores over the temperature, so the same situation is not always answered the same way. The
   answer's probability is the confidence.

The mean, not the sum, so a two-word choice is not punished for being two words. Choices are
kept short and plain for the same reason: `move a torch`, not `relocate the nearest light
source`.

**The space where Qwen's tokens carry it.** Until now the question ended `The answer is: `, a
space last, and each choice was read without one. Qwen's words carry their space at their front
(` knock`), so every choice's first token was one the model almost never sees after a space,
which skewed every score (the writing prompts had the same trouble: a sign said `123456`). Now
the question ends `The answer is:` and each choice is read as ` choice` (`model::SCORING`, 1,
`spaced`, the default).

**What was measured** (owner: "look into the menu wording and scoring"; `core/tests/frames.rs`,
the 1.5B and the 4B, on the GPU): two late-stage situations under three menu orders, and two
`When?` questions whose answer the situation gives away (asleep in bed at night: `while they
sleep`; gone mining far from home: `while they are away`):

| Scoring | Obvious answers right, 1.5B | 4B | The menu's picks |
| --- | --- | --- | --- |
| `plain`, as it was | 1 of 2 | 1 of 2 | one short phrase whatever the situation |
| `spaced` (now) | 2 of 2 | 2 of 2 | the same short phrase |
| `calibrated`: less the choice's score after the frame alone | 0 of 2 | 0 to 1 of 2 | the same |
| `listed`: the choices named in the question | 0 to 1 of 2 | 1 of 2 | the loudest events, by where they sit in the list |
| `half`: listed, less half the baseline | 0 of 2 | 0 of 2 | the 4B the loudest events, but never another |
| `lettered`: the choices under letters, the letter scored | 0 to 2 of 2 | 0 to 2 of 2 | the loud ones, fitting the situation |
| `mixed` (now): lettered past five choices, spaced below | 2 of 2 | 2 of 2 | the loud ones in the late stages, fitting the situation |

**Why a common phrase won.** The mean over a choice's tokens: once ` knock` is read, ` on the
door` is all but certain, so that idiom's mean stays high whatever the situation, and any idiom on
a menu beats the rest. Shortening the words did not help (below), so it was not their length.

**Lettered.** The choices listed under letters, `A. wait`, `B. move a torch`, ..., then `The
answer is:`, and each choice scored by its letter's log-probability: one token each, so no
phrase rides on its own words. A small model leans toward the first letters, so the list is
asked in three orders (turned a third of the way each time) and the scores averaged
(`lettered_scores`). One decode an order rather than one a choice's token, so it is no slower.
The short fixed questions (`Should it do something soon?`, `When?`, `Where?`) stay `spaced`,
which already reads them right and which lettering made worse.

**Fits** (`short_menu_words_against_long_on_situations_with_one_plain_fit`): six situations,
each with one event that plainly fits (asleep in bed: the choke; trading with a villager: the
stab; deep in a cave: the torches flicker; in a dark forest: peeking from the trees; opening their
chest: sorting it; swimming a lake: the figure beneath), on one menu with `knock on the door` and
`move a torch`, three orders each, eighteen asks:

| Scoring | 1.5B, the catalogue's words | 1.5B, short words | 4B, the catalogue's words | 4B, short words |
| --- | --- | --- | --- | --- |
| `spaced` | 0 | 0 | 0 | 0 |
| `mixed` | 6 | 3 | 12 | 12 |

Every `spaced` answer was `knock on the door`. The menu's words were kept as they are: under
`mixed` the 4B fits as well with them as with short ones, and the 1.5B better.

**A three-day stay** (`core/tests/calibration.rs`, seed 7) under `mixed`: the 1.5B 71 decisions,
31 events landed, 23 of its picks `wait`; the 4B 56, 33 and 16. Before, the 1.5B made 49 and
landed 31 with two `wait`s: `wait` lost to `knock on the door` as everything did. As many land
(the day's budget holds them), and it now waits when waiting fits, oftenest early, where the
frame says nothing it does yet could not be explained away. With `mixed` the frame's stage
sentence tells too: the 4B picked the loud events in all six late-stage asks under it, in three
under the old frame.

The unit tests of the pure parts (softmax, log-softmax, the winner, the validation of empty
situations and choices) are the engine's, moved. The scoring itself is tested against the real
model only in the ignored `model::` tests, which need the file (see
[Tests](development.md#tests)).

## The worker and the queue

`model/queue.rs`, the engine's `Worker`:

- **One thread, `lodger-model`,** started on the first ask with a loader closure, which reports
  whether the model loaded before the first job is accepted. A load failure is the asker's error
  and the worker does not stay up.
- **Jobs** carry an id, an optional key, the situation and the questions (or, for a written line,
  the prompt and its leash), and a channel to reply on. `MAX_QUEUED` is 64; a full queue refuses the job with a message, so the
  brain can fall back at once rather than wait.
- **Keys.** The worker keeps the newest job id per key. When it takes a job whose key has a newer
  id waiting, it replies `superseded` without scoring. A job already being scored finishes.
- **Stopping** drops the channel, joins the thread, and frees the engine in order.

The mind drains the reply channel at the start of every `tick`, so the brain's
callbacks run on the server thread. A `Scorer` trait stands between the worker and llama.cpp,
and tests give the worker an `Echo` scorer that answers the first choice, or blocks on a gate to
test superseding, as the engine's tests do.
