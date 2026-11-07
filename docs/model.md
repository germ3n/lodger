# Model

The model file, how it is fetched and loaded, and the worker that scores with it
(`core/src/model/`). This is `~/engine/game/base/src/ai/` with the game taken out: the same
scoring, the same queue, the same download, behind the mind's C ABI instead of a Lua library.

## Which model

**Qwen2.5-Instruct, Qwen's own `q4_k_m` GGUF, in the size `model.size` picks** (owner: "players can choose between the 0.5b, 1.5b or 3b model"):

| `model.size` | File | RAM while loaded | A question of four choices on two threads | For |
| --- | --- | --- | --- | --- |
| `0.5b` | `qwen2.5-0.5b-instruct-q4_k_m.gguf`, 491 MB | about 700 MB | tens to a few hundred milliseconds | a small machine; the file `~/engine` runs, byte for byte; the dev client's choice |
| `1.5b`, the default | `qwen2.5-1.5b-instruct-q4_k_m.gguf`, 1.1 GB | about 1.6 GB | a few times slower | most machines; the owner's default for sharper judgment and for writing its own few words (`brain.text`) |
| `3b` | `qwen2.5-3b-instruct-q4_k_m.gguf`, 2.1 GB | about 3 GB | slower again, a second or more | a dedicated server with the headroom; under the **Qwen Research License** (non-commercial use only, which it defines as research or evaluation; commercial use needs a license from Alibaba Cloud), not Apache |
| `4b` | `Qwen3-4B-Instruct-2507-Q4_K_M.gguf` from unsloth's repository, 2.5 GB | about 3.5 GB | about the 3B's | the newest and best reader (owner: "how about we offer this too?"); Apache-2.0; non-thinking only, so no `<think>` block gets in the way of scoring; the vendored llama.cpp (llama-cpp-sys-2 0.1.161) knows its `qwen3` architecture |

The 0.5B, 1.5B and 4B are Apache-2.0; the 3B is under Qwen's research license, which the
preparing screen names and links before the download (it said Apache for every size until
now). Qwen's own GGUF repository for the 4B answers 401 to a plain download, so its file
comes from unsloth's, which repackages Qwen's model under the same license. Each file's exact
length and SHA-256 are in `ModelSize` (from Hugging Face's `x-linked-size` and `x-linked-etag`),
and the host checks the sum after the download; a file from `model.url` of one's
own is not checked. The scoring code is the same for each; only the file, its size and the
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
- **Why Qwen:** Apache-2.0, so a player may download it without a licence click; GGUF builds by
  Qwen themselves; and it is what the engine already runs, so the scoring code is known to work
  on it. Qwen3-0.6B is also Apache-2.0 and a todo item is to score both on the same log and
  see whether it reads situations better.
- **Why q4_k_m:** Qwen's own quantization, and the smallest at which the engine saw stable
  choices. Community `Q4_K_M` files of the same model are smaller (about 400 MB) and would work;
  the magic check does not care which. The default URL points at Qwen's.

## Downloading

**Picking the size on the screen** (owner): the preparing screen shows a button
with the size it is about to fetch (`Qwen2.5 1.5B, 1117 MB (click for another)`); each press
picks the next of the four, writes `model.size` into the config and starts that download
instead. A model already there skips the screen; the settings page changes it then.

The model is not in the jar. 491 MB is too much to ship with a mod and Hugging Face serves it
well. The mind declares it as a *resource* in its `init` reply (the SDK's `Resource`: the URL
from `model.url`, the path from `model.path` under the game directory, the `GGUF` magic, the
size), and the host fetches it (owner: "lock them out of the main menu until the
model is downloaded and loaded"):

- **On a client,** the first time the title screen opens with the file missing, a screen comes
  up over it: the title, a progress bar, the megabytes so far, and a Quit button; Escape does
  nothing. When the file is there the screen says `Loading the lodger's model` with a sweeping
  bar while the mind loads it (a second or two), and then the title screen is back. On failure,
  the error and a Try again button. A player with the file already there never sees any of it.
  Under the size button, a note on the size chosen: how it differs from the others and the VRAM
  it wants (owner). **Play without it** (owner) closes the screen and writes `model.enabled:
  false`: nothing is downloaded, a part file already there is kept, and the rule brain decides;
  a player who only joins servers needs nothing more, the server's model deciding there. The
  settings' **Use the model** turns it back on, and the screen comes back to download.
- **On a dedicated server,** at startup, before the world opens, blocking, with a log line at
  every ten percent and one when the model is loaded. On failure the server starts anyway and
  the lodger waits for the next start.
- **It must be a GGUF.** The first four bytes must be `GGUF`; anything else, an HTML error page
  above all, fails on those bytes rather than after half a gigabyte.
- **It is written under a temporary name,** `qwen.gguf.part`, and renamed when complete, so a
  failed download never replaces a good model and a half file is never loaded.
- **`http://` or `https://` only.** Redirects are followed. The connect timeout is 15 seconds.
  A slow connection is still a connection, but a quiet one is not: thirty seconds without a
  byte and it is closed, and the download goes on from where it stopped with an HTTP range, up
  to five tries with a pause between (a server that ignores the range sends the whole file
  again). When every try fails the `.part` file stays, and the next start goes on from it; a
  cancel, the wrong magic or a wrong SHA-256 removes it. The 4B's download hung at 2059 of
  2381 MB on a connection gone quiet until now, when reading had no limit at all.
- **`/lodger model status`** says whether the file is there and the model loaded. Deleting the
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

- **The file must be loose.** llama.cpp maps it; a file inside the jar or any archive is
  `qwen.gguf is inside an archive, llama.cpp needs it as a loose file`.
- **Context 512** (`model.context`), batch and ubatch the same, so one decode takes the whole
  situation. **Two threads** (`model.threads`) for decoding and three for the batch, the engine's
  numbers. Raising threads speeds scoring and takes from the server's own threads; the default
  leaves a four-core server its game.
- **CPU only by default.** `model.gpu_layers` is `0`. Metal on macOS and Vulkan elsewhere are
  features of the native build and a config switch, but every server behaves alike on the CPU
  and 0.5B does not need a GPU. The feature exists because `~/engine` has it on macOS; it is an
  experiment here.
- **Memory:** the model's 491 MB mapped plus the context's buffers, about 700 MB resident. The
  JVM does not see it: it is native memory outside the heap, so `-Xmx` is unaffected and the
  machine's own memory is what must have room.
- **Unloading** after `model.idle_minutes` with no target online, and at server stop
  (`craftkit_shutdown`). The context, then the model, then the backend are freed in that order,
  because ggml asserts at process exit if any buffer is alive, which would take the server down
  with it. The engine's `Engine::drop` has the same comment.

## Scoring

`model/infer.rs`, the engine's `decode_questions` unchanged in substance:

1. Clear the KV cache. Tokenize the situation with the beginning-of-text token and decode it,
   without logits.
2. For each question: rewind the cache to the end of the situation, tokenize
   `\n<question>\nThe answer is: ` and decode it, keeping the last position's logits.
3. For each choice: tokenize it (no special tokens). Its score is the mean over its tokens of
   the log-probability of each token given the ones before: the first from the question's
   logits, the rest by decoding the choice one token at a time, then rewinding to the end of
   the question for the next choice. A one-token choice costs no decode.
4. Softmax over the choices' scores. The best is the answer; its probability is the confidence.
   A tie keeps the first choice.

The mean, not the sum, so a two-word choice is not punished for being two words. Choices are
kept short and plain for the same reason: `move a torch`, not `relocate the nearest light
source`.

The unit tests of the pure parts (softmax, log-softmax, the winner, the validation of empty
situations and choices) are the engine's, moved. The scoring itself is tested against the real
model only in the ignored `model::` tests, which need the file (see
[Tests](development.md#tests)).

## The worker and the queue

`model/queue.rs`, the engine's `Worker`:

- **One thread, `lodger-model`,** started on the first ask with a loader closure, which reports
  whether the model loaded before the first job is accepted. A load failure is the asker's error
  and the worker does not stay up.
- **Jobs** carry an id, an optional key, the situation and the questions, and a channel to reply
  on. `MAX_QUEUED` is 64 (`brain.queue`); a full queue refuses the job with a message, so the
  brain can fall back at once rather than wait.
- **Keys.** The worker keeps the newest job id per key. When it takes a job whose key has a newer
  id waiting, it replies `superseded` without scoring. A job already being scored finishes.
- **Stopping** drops the channel, joins the thread, and frees the engine in order.

The mind drains the reply channel at the start of every `tick`, so the brain's
callbacks run on the server thread. A `Scorer` trait stands between the worker and llama.cpp,
and tests give the worker an `Echo` scorer that answers the first choice, or blocks on a gate to
test superseding, as the engine's tests do.
