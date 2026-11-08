# Building

How the lodger is built, from a developer's first `cargo build` to the two jars that go to
Modrinth. [Development](development.md) has the toolchain, running a dev client and the tests;
this page is the build itself.

## What gets built

| Part | Where | What comes out |
| --- | --- | --- |
| The mind | `core/` (Rust, with the SDK in `sdk/`) | A native library, `liblodger_core.dylib`, `.so` or `lodger_core.dll`, with llama.cpp compiled in |
| The host | `host/` (Java, Fabric and NeoForge) | `craftkit-fabric-<mc>-<version>.jar` and `craftkit-neoforge-<mc>-<version>.jar` |
| The content jars | `mod/` | `lodger-fabric-<mc>-<version>.jar` and `lodger-neoforge-<mc>-<version>.jar`: the natives, the assets and manifests, and the host jar nested inside |

A player installs one content jar, the one for their loader. Everything else is inside it.

## The six natives

One native per platform, with the GPU always on (owner: "one native per os per arch"; Intel Macs too):

| Platform | GPU | Built |
| --- | --- | --- |
| `macos-aarch64` | Metal | natively on a Mac with Apple silicon |
| `macos-x86_64` | Metal | on the same Mac, for Intel (`x86_64-apple-darwin`; it runs under Rosetta there, so it can be tried) |
| `windows-x86_64` | Vulkan | cross-compiled with cargo-xwin |
| `windows-aarch64` | Vulkan | cross-compiled with cargo-xwin |
| `linux-x86_64` | Vulkan | in Docker (`scripts/cross/linux.Dockerfile`) |
| `linux-aarch64` | Vulkan | in Docker |

A player on a platform the jar lacks gets a screen at boot
(`FailedScreen`) naming the platforms it carries.

Each native is checked before it goes in a jar (`scripts/build_natives.py`'s `check`):

- It exports `craftkit_abi_version`, so it is a craftkit mod.
- **Windows:** it imports only Windows' own DLLs and `vulkan-1.dll`. Microsoft's C and C++
  runtime is linked in (`.cargo/config.toml`: `crt-static` for Rust, `LLAMA_STATIC_CRT` for
  llama.cpp), so no Visual C++ Redistributable is needed.
- **Linux:** it asks for glibc 2.35 at most (in practice 2.34), so it loads on Debian 12 and
  Ubuntu 22.04 and newer, and links only glibc, libstdc++, libgcc and `libvulkan.so.1`.
- **macOS:** it links only the system's libraries and frameworks (Metal among them).

**Vulkan at run time.** The Windows and Linux natives link Vulkan's loader. Every GPU driver
installs it (`vulkan-1.dll` on Windows; Mesa and the vendors' drivers on Linux), and with no
GPU llama.cpp runs on the CPU. A headless Linux server often lacks it: `apt install libvulkan1`
(Debian, Ubuntu), `dnf install vulkan-loader` (Fedora), `pacman -S vulkan-icd-loader` (Arch).
The host's error says so when that is why the native would not load.

**Compressed.** In a release jar each native is `natives/<os>-<arch>/<library>.xz`, xz at its
strongest preset: about half of what the jar's own deflate gives (the Vulkan natives carry
their compiled shaders and are 30 to 50 MB unpacked). At boot the host reads its platform's,
unpacks it with XZ for Java (bundled in the host jar: Fabric's `include`, NeoForge's
`jarJar`), and writes it to `config/craftkit/natives/lodger/<os>-<arch>/`, where it is loaded;
one already there with the same bytes is left alone. A developer's jar carries its native plain,
and the host takes either.

## Building a release

On a Mac with Apple silicon:

```sh
python3 scripts/build_jars.py 26.3
```

That builds the six natives here (`--natives cross`, the default), compresses them, builds the
host with its tests and both content jars, checks each jar carries all six and that both
loaders' jars are there, and puts them in `dist/26.3/` with `SHA256SUMS`: the SHA-256 of each
native as built (`natives/<os>-<arch>/<library>`), as packed in the jars (`.xz`), and of each jar,
also printed at the end (`cd dist/26.3 && sha256sum -c --ignore-missing SHA256SUMS` checks the
jars). The first run takes a while: llama.cpp for each platform, the Docker image, the
x86_64 Linux build under emulation (Rosetta), and xwin's one-time download of Microsoft's CRT
and SDK. Each native's build tree is deleted as soon as its library is copied out, since each is
a gigabyte or more.

Other sources for the natives:

| `--natives` | Where they come from |
| --- | --- |
| `cross` (default) | Built here by `scripts/build_natives.py all` |
| `DIR` | `DIR/<os>-<arch>/<library>`, built already (the CI's jar job uses this) |
| `ci` | The CI's run of the commit checked out, through `gh` (signed in; the commit pushed and the workflow run by hand) |
| `local` | This computer's own platform alone, with `--allow-missing` |

`--allow-missing` builds the jars without some natives; players on those platforms are told so.
`--allow-dirty` lets `ci` pack natives of the commit beside uncommitted changes to the mind.

**The Minecraft version** is the script's argument. `versions/<minecraft>.properties` holds
every pin that changes with it (Minecraft, NeoForm, Fabric API and loader, Loom, NeoForge,
ModDevGradle, Java), laid over the gradle.properties files for that build only (`-P`). A version
with no file is refused: the host is written against one version's code, and another needs its
file and usually a port ([Upgrading](upgrading.md)). Today: 26.3.

## Building natives alone

```sh
python3 scripts/build_natives.py all --out natives            # the six
python3 scripts/build_natives.py windows-x86_64 --out natives # one or more by name
```

Each lands in `natives/<os>-<arch>/<library>`, checked. `--keep` keeps the build trees under
`target/cross/`.

### What the cross-builds need (a Mac)

| For | Install |
| --- | --- |
| macOS | Xcode's command line tools, Rust (`rust-toolchain.toml` pins 1.98.1) |
| Windows | `brew install llvm lld shaderc` (clang-cl, lld-link, llvm-dlltool, glslc), `cargo install cargo-xwin --locked` |
| Linux | Docker Desktop, running |
| All | git, cmake, Python 3 |

**How Windows is cross-built.** cargo-xwin compiles with clang-cl against Microsoft's CRT and
Windows SDK, which it fetches once into `~/Library/Caches/cargo-xwin` (about 1 GB) and so
accepts Microsoft's license for them. llama.cpp's Vulkan build wants a Vulkan SDK; the script
makes a small one in `target/vk-sdk/windows-<arch>/`: Khronos's Vulkan headers (`v1.4.309`), the
SPIR-V headers, an import library for `vulkan-1.dll` made with `llvm-dlltool` from every
function the headers declare (ggml calls four core ones, which every loader exports), and this
machine's `glslc` to compile the shaders. Clang, not MSVC's own compiler, also matters on ARM:
llama.cpp refuses MSVC there.

**How Linux is built.** `scripts/cross/linux.Dockerfile` is Debian 12 with Rust 1.98.1, cmake,
clang (for the bindings), Vulkan's loader to link against, glslc, the SPIR-V headers, and
Khronos's Vulkan headers (Debian 12's are older than llama.cpp's Vulkan code). Debian 12 for its
glibc: Ubuntu 24.04's 2.39 would not load on Debian 12 or Ubuntu 22.04. The repository is
mounted at `/src`, the build tree goes to `target/cross/<platform>/`, and Cargo's downloads are
kept in a Docker volume (`lodger-cargo-<arch>`). On an Apple chip the x86_64 build runs under
emulation and takes several times longer.

**Disk.** A whole cross-build wants about 15 GB free while it runs: the Docker images and their
caches, xwin's CRT and SDK, and one llama.cpp build tree at a time. `docker system prune` and
deleting `target/` give it back.

## The CI

`.github/workflows/ci.yml` runs by hand only: Actions, ci, Run workflow, with the release's tag
and whether it is a pre-release as its two inputs. Its jobs:

1. **checks** on macOS, Ubuntu 22.04 and Windows: `cargo fmt --check`, clippy with warnings as
   errors, `cargo test --workspace`.
2. **natives-mac-windows** on macOS: `build_natives.py macos-aarch64 macos-x86_64
   windows-x86_64 windows-aarch64`, four natives, as on a developer's Mac (Homebrew's LLVM, lld and shaderc; cargo-xwin).
3. **natives-linux**, one runner per chip (`ubuntu-22.04`, `ubuntu-22.04-arm`): `build_natives.py
   linux-<arch>` in the same Docker image, natively.
4. **jars**, once the rest pass: the six natives merged into one folder, then
   `build_jars.py 26.3 --natives natives`, which builds the host with its JUnit tests and both
   content jars. The jars and their `SHA256SUMS` are the run's `lodger-jars` artifact, and the
   sums are printed in the log and on the run's summary page; each native job prints its own
   native's too, as built.

5. **release**: a GitHub release of the run, tagged as asked when it was started (empty:
   `v<mod version>-<run number>`; a pre-release if ticked). It holds the six natives on their
   own, each named by its platform (`lodger_core-windows-x86_64.dll`, ...), both jars and
   `SHA256SUMS`, and its notes list every file's SHA-256.

So a jar from the CI and one from a developer's Mac are built by the same scripts the same way.

## A developer's build

```sh
cargo build --release -p lodger-core   # this machine's native, in target/release/
cd host && ./gradlew build             # the host jars and their tests
cd mod && ./gradlew jar                # the content jars, this machine's native in them, plain
```

`./gradlew jar` in `mod/` runs `cargo build --release -p lodger-core` itself (`cargoBuild`), and
puts the native in as `natives/<os>-<arch>/<library>` uncompressed, with whatever is in
`mod/natives-extra/<os>-<arch>/` beside it. That native is CPU only on Windows and Linux (no
Vulkan SDK assumed); on a Mac it has Metal. For a dev client, see
[Development](development.md#building-the-host-and-the-content-jar).
