#!/usr/bin/env python3
"""Builds the mind's native library for any of the six platforms a lodger jar carries, each with
the GPU always on: Metal on macOS, Vulkan on Windows and Linux (owner: "one native per os per
arch"). See docs/build.md.

    python3 scripts/build_natives.py all --out DIR          # the six
    python3 scripts/build_natives.py linux-x86_64 --out DIR # one or more by name

Each lands in DIR/<platform>/<library>, checked: it exports the craftkit ABI, and asks for
nothing a player's machine may lack (Windows: system DLLs and vulkan-1.dll alone, the C++
runtime linked in; Linux: glibc 2.34 at most, so Debian 12 and Ubuntu 22.04 load it; macOS:
system libraries alone). Its build tree is deleted after unless --keep, a llama.cpp build being
a gigabyte or more.

How each is built, from a Mac (Apple silicon):

- macos-aarch64, macos-x86_64: cargo, natively and for Intel. Metal comes with llama.cpp on
  Apple.
- windows-x86_64, windows-aarch64: cargo-xwin with clang-cl, against Microsoft's CRT and SDK
  (xwin fetches them once, accepting Microsoft's license) and a Vulkan "SDK" made here from
  Khronos's headers, the SPIR-V headers, an import library for vulkan-1.dll from the headers'
  function list, and this machine's glslc.
- linux-x86_64, linux-aarch64: Docker, in scripts/cross/linux.Dockerfile (Debian 12); x86_64
  under emulation on an Apple chip, so slower.

From Linux, the Linux natives build the same way (Docker) and the Windows ones too if the
tools are there; macOS natives need a Mac.
"""

import argparse
import os
import platform
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PLATFORMS = ["windows-x86_64", "windows-aarch64", "linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"]
TRIPLES = {
    "windows-x86_64": "x86_64-pc-windows-msvc",
    "windows-aarch64": "aarch64-pc-windows-msvc",
    "linux-x86_64": "x86_64-unknown-linux-gnu",
    "linux-aarch64": "aarch64-unknown-linux-gnu",
    "macos-x86_64": "x86_64-apple-darwin",
    "macos-aarch64": "aarch64-apple-darwin",
}
DOCKER = {"linux-x86_64": "amd64", "linux-aarch64": "arm64"}
# Pinned: the headers llama.cpp's Vulkan code is written against (Debian 12's are older).
VULKAN_HEADERS = "v1.4.309"
SPIRV_HEADERS = "vulkan-sdk-1.4.309.0"
# The newest glibc a Linux native may ask for: Ubuntu 22.04 has 2.35, Debian 12 2.36.
NEWEST_GLIBC = (2, 35)
WINDOWS_DLLS = {"kernel32.dll", "advapi32.dll", "ntdll.dll", "bcryptprimitives.dll", "vulkan-1.dll",
                "api-ms-win-core-synch-l1-2-0.dll", "ws2_32.dll", "userenv.dll", "ole32.dll", "shell32.dll"}
LINUX_NEEDED = {"libvulkan.so.1", "libstdc++.so.6", "libgcc_s.so.1", "libm.so.6", "libc.so.6",
                "ld-linux-x86-64.so.2", "ld-linux-aarch64.so.1", "libpthread.so.0", "libdl.so.2"}


def library(plat):
    os_name = plat.split("-")[0]
    return {"macos": "liblodger_core.dylib", "windows": "lodger_core.dll"}.get(os_name, "liblodger_core.so")


def run(cmd, cwd=ROOT, env=None, capture=False):
    print("$", " ".join(cmd), flush=True)
    result = subprocess.run(cmd, cwd=cwd, env=env, check=True, text=True,
                            stdout=subprocess.PIPE if capture else None)

    return result.stdout if capture else None


def need(tool, how):
    found = shutil.which(tool)
    if not found:
        sys.exit("%s is needed: %s" % (tool, how))

    return found


def llvm_bin():
    """Homebrew's LLVM and lld (clang-cl, llvm-dlltool, lld-link) on a Mac, else what is on PATH."""
    dirs = []
    if shutil.which("brew"):
        for formula in ("llvm", "lld"):
            prefix = subprocess.run(["brew", "--prefix", formula], text=True, stdout=subprocess.PIPE,
                                    stderr=subprocess.DEVNULL).stdout.strip()
            if prefix and os.path.isdir(os.path.join(prefix, "bin")):
                dirs.append(os.path.join(prefix, "bin"))

    return dirs


def clone(url, tag, into):
    if not os.path.isdir(into):
        run(["git", "clone", "-q", "--depth", "1", "--branch", tag, url, into])


def windows_sdk(arch, env):
    """target/vk-sdk/windows-<arch>: what ggml's Vulkan build looks for under VULKAN_SDK."""
    sdk = os.path.join(ROOT, "target", "vk-sdk", "windows-" + arch)
    lib = os.path.join(sdk, "Lib", "vulkan-1.lib")
    if os.path.isfile(lib):
        return sdk
    src = os.path.join(ROOT, "target", "vk-src")
    clone("https://github.com/KhronosGroup/Vulkan-Headers.git", VULKAN_HEADERS, os.path.join(src, "Vulkan-Headers"))
    clone("https://github.com/KhronosGroup/SPIRV-Headers.git", SPIRV_HEADERS, os.path.join(src, "SPIRV-Headers"))
    shutil.rmtree(sdk, ignore_errors=True)
    shutil.copytree(os.path.join(src, "Vulkan-Headers", "include"), os.path.join(sdk, "Include"))
    build = os.path.join(src, "spirv-build-" + arch)
    run(["cmake", "-S", os.path.join(src, "SPIRV-Headers"), "-B", build, "-DCMAKE_INSTALL_PREFIX=" + sdk,
         "-DSPIRV_HEADERS_ENABLE_TESTS=OFF"], capture=True)
    run(["cmake", "--install", build], capture=True)
    # The loader's exports, as the headers declare them: ggml calls a few of the core ones, all of
    # which every vulkan-1.dll exports.
    names = set()
    for header in ("vulkan_core.h", "vulkan_win32.h"):
        text = open(os.path.join(sdk, "Include", "vulkan", header), encoding="utf-8").read()
        names |= set(re.findall(r"VKAPI_ATTR \w+ VKAPI_CALL (vk\w+)\(", text))
    definition = os.path.join(sdk, "vulkan-1.def")
    with open(definition, "w", encoding="utf-8") as f:
        f.write("LIBRARY vulkan-1.dll\nEXPORTS\n" + "".join("    %s\n" % n for n in sorted(names)))
    os.makedirs(os.path.dirname(lib), exist_ok=True)
    run(["llvm-dlltool", "-m", "i386:x86-64" if arch == "x86_64" else "arm64", "-d", definition, "-l", lib,
         "-D", "vulkan-1.dll"], env=env)
    os.makedirs(os.path.join(sdk, "Bin"), exist_ok=True)
    os.symlink(need("glslc", "install shaderc (brew install shaderc) or the Vulkan SDK"),
               os.path.join(sdk, "Bin", "glslc"))

    return sdk


def build_macos(plat, tree):
    if platform.system() != "Darwin":
        sys.exit("%s needs a Mac" % plat)
    triple = TRIPLES[plat]
    run(["rustup", "target", "add", triple], capture=True)
    run(["cargo", "build", "--release", "-p", "lodger-core", "--target", triple, "--target-dir", tree])

    return os.path.join(tree, triple, "release", library(plat))


def build_windows(plat, tree):
    triple = TRIPLES[plat]
    env = dict(os.environ)
    env["PATH"] = os.pathsep.join(llvm_bin() + [env.get("PATH", "")])
    for tool in ("clang-cl", "lld-link", "llvm-lib", "llvm-dlltool"):
        if not shutil.which(tool, path=env["PATH"]):
            sys.exit("%s is needed for the Windows natives: brew install llvm lld" % tool)
    if not shutil.which("cargo-xwin"):
        sys.exit("cargo-xwin is needed for the Windows natives: cargo install cargo-xwin --locked")
    sdk = windows_sdk(plat.split("-")[1], env)
    env["VULKAN_SDK"] = sdk
    env["CMAKE_PREFIX_PATH"] = sdk
    run(["rustup", "target", "add", triple], capture=True)
    run(["cargo", "xwin", "build", "--release", "-p", "lodger-core", "--features", "vulkan", "--target", triple,
         "--target-dir", tree], env=env)

    return os.path.join(tree, triple, "release", library(plat))


def build_linux(plat, tree):
    need("docker", "install Docker and start it")
    arch = DOCKER[plat]
    image = "lodger-linux:" + arch
    run(["docker", "build", "-q", "--platform", "linux/" + arch, "-t", image, "-f",
         os.path.join("scripts", "cross", "linux.Dockerfile"), os.path.join("scripts", "cross")], capture=True)
    rel = os.path.relpath(tree, ROOT)
    run(["docker", "run", "--rm", "--platform", "linux/" + arch, "-v", ROOT + ":/src",
         "-v", "lodger-cargo-" + arch + ":/usr/local/cargo/registry", "-w", "/src",
         "-e", "CARGO_TARGET_DIR=/src/" + rel, image,
         "cargo", "build", "--release", "-p", "lodger-core", "--features", "vulkan"])

    return os.path.join(tree, "release", library(plat))


def check(plat, path):
    """Refuses a native a player's machine could not load, or that is no craftkit mod."""
    os_name = plat.split("-")[0]
    if os_name == "macos":
        exports = run(["nm", "-gU", path], capture=True)
        # The first line is the file, the second its own name; the rest what it links.
        links = [l.strip().split(" ")[0] for l in run(["otool", "-L", path], capture=True).splitlines()[2:]]
        foreign = [l for l in links if not l.startswith(("/usr/lib/", "/System/"))]
        if foreign:
            sys.exit("%s links outside the system: %s" % (plat, foreign))
    elif os_name == "windows":
        env = dict(os.environ, PATH=os.pathsep.join(llvm_bin() + [os.environ.get("PATH", "")]))
        dump = run(["llvm-objdump", "-p", path], env=env, capture=True)
        exports = dump
        dlls = {d.lower() for d in re.findall(r"DLL Name: (\S+)", dump)}
        foreign = sorted(d for d in dlls if d not in WINDOWS_DLLS and not d.startswith("api-ms-win-core-"))
        if foreign:
            sys.exit("%s needs DLLs a player may lack: %s (is the C runtime static? .cargo/config.toml)"
                     % (plat, foreign))
        if "vulkan-1.dll" not in dlls:
            sys.exit("%s has no Vulkan in it" % plat)
    else:
        rel = os.path.relpath(path, ROOT)
        arch = DOCKER[plat]
        dump = run(["docker", "run", "--rm", "--platform", "linux/" + arch, "-v", ROOT + ":/src",
                    "lodger-linux:" + arch, "sh", "-c", "objdump -T /src/%s; objdump -p /src/%s" % (rel, rel)],
                   capture=True)
        exports = dump
        glibc = max((tuple(int(x) for x in v.split(".")) for v in re.findall(r"GLIBC_([0-9.]+)", dump)), default=(0,))
        if glibc > NEWEST_GLIBC:
            sys.exit("%s asks for glibc %s, newer than %s" % (plat, ".".join(map(str, glibc)),
                                                              ".".join(map(str, NEWEST_GLIBC))))
        needed = set(re.findall(r"NEEDED\s+(\S+)", dump))
        if needed - LINUX_NEEDED:
            sys.exit("%s needs libraries a player may lack: %s" % (plat, sorted(needed - LINUX_NEEDED)))
        if "libvulkan.so.1" not in needed:
            sys.exit("%s has no Vulkan in it" % plat)
    if "craftkit_abi_version" not in exports:
        sys.exit("%s does not export craftkit_abi_version" % plat)


def build(plat, out, keep=False):
    tree = os.path.join(ROOT, "target", "cross", plat)
    built = {"macos": build_macos, "windows": build_windows, "linux": build_linux}[plat.split("-")[0]](plat, tree)
    check(plat, built)
    kept = os.path.join(out, plat, library(plat))
    os.makedirs(os.path.dirname(kept), exist_ok=True)
    shutil.copy2(built, kept)
    if not keep:
        shutil.rmtree(tree, ignore_errors=True)
    print("%s: %s (%.1f MB)" % (plat, kept, os.path.getsize(kept) / 1e6), flush=True)


def main():
    parser = argparse.ArgumentParser(description="Build the lodger's natives, the GPU always on.")
    parser.add_argument("platforms", nargs="+", help="all, or any of " + ", ".join(PLATFORMS))
    parser.add_argument("--out", required=True, help="where DIR/<platform>/<library> go")
    parser.add_argument("--keep", action="store_true", help="keep the build trees under target/cross/")
    args = parser.parse_args()
    wanted = PLATFORMS if args.platforms == ["all"] else args.platforms
    unknown = [p for p in wanted if p not in PLATFORMS]
    if unknown:
        sys.exit("no such platform: %s (the six are %s)" % (", ".join(unknown), ", ".join(PLATFORMS)))
    for plat in wanted:
        build(plat, os.path.abspath(args.out), args.keep)


if __name__ == "__main__":
    main()
