# Model

The model file, how it is fetched and loaded, and the worker that scores with it
(`core/src/model/`). This is `~/engine/game/base/src/ai/` with the game taken out: the same
scoring, the same queue, the same download, behind the mind's C ABI instead of a Lua library.

## Which model

**Qwen2.5-0.5B-Instruct, Qwen's own `q4_k_m` GGUF, 491 MB** (`qwen2.5-0.5b-instruct-q4_k_m.gguf`
from `Qwen/Qwen2.5-0.5B-Instruct-GGUF` on Hugging Face). It is the file in `~/engine/models/`
byte for byte, so what the engine learned about it holds here.

- **Why this size:** it scores a 300-token situation and four questions of four choices in tens
  to a few hundred milliseconds on two CPU threads, with about 700 MB resident, which a server
  can spare and a player's own machine running an integrated server can spare. The next size
  up, 1.5B, is three times the memory and time for judgment the lodger does not need.
- **Why Qwen:** Apache-2.0, so a player may download it without a licence click; GGUF builds by
  Qwen themselves; and it is what the engine already runs, so the scoring code is known to work
  on it. Qwen3-0.6B is also Apache-2.0 and a todo item is to score both on the same log and
  see whether it reads situations better.
- **Why q4_k_m:** Qwen's own quantization, and the smallest at which the engine saw stable
  choices. Community `Q4_K_M` files of the same model are smaller (about 400 MB) and would work;
  the magic check does not care which. The default URL points at Qwen's.

## Downloading

The model is not in the jar. 491 MB is too much to ship with a mod, Hugging Face serves it well,
and a mod that downloads half a gigabyte should say so and be told to, so:

- **`/lodger model download [url]`**, an operator command, fetches from the URL given or
  `model.url` to `model.path` (`config/lodger/models/qwen.gguf` in the game directory). The
  command answers at once and says when the download finished or failed, in chat to whoever ran
  it and in the log.
- **One at a time.** A second download while one runs is refused with
  `a model download is already running`.
- **It must be a GGUF.** The first four bytes must be `GGUF`; anything else, an HTML error page
  above all, fails with `not a GGUF model file` as soon as those bytes arrive and the connection
  is dropped.
- **It is written under a temporary name,** `qwen.gguf.part`, and renamed when complete, so a
  failed download never replaces a good model and a half file is never loaded.
- **`http://` or `https://` only.** Redirects are followed. The connect timeout is 15 seconds;
  reading has none, since a slow connection is still a connection, but the operator can stop the
  server.
- **A loaded model keeps being used** until the model is unloaded (idle, or server stop), so a
  download over a running model takes effect later.

The body never touches the file. It asks the mind to download, and the mind's download thread
does it with `ureq`, as the engine's does.

## Loading

The mind links llama.cpp through the `llama-cpp-2` crate, which builds llama.cpp from source into
the shared library, statically, so the jar's native for a platform is one file. The model is
loaded on the first ask, on the worker thread, and the asker's reply waits for it: a second or
two the first time, then nothing.

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
