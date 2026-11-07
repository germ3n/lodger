#!/usr/bin/env python3
"""Builds the lodger's two jars, Fabric and NeoForge, for one Minecraft version, each carrying the
mind's native library for all five platforms, xz-compressed (owner). See docs/build.md.

    python3 scripts/build_jars.py 26.3 --natives cross    # build all five here (a Mac)
    python3 scripts/build_jars.py 26.3 --natives DIR      # DIR/<os>-<arch>/<library>, built already
    python3 scripts/build_jars.py 26.3 --natives ci       # the CI run of this commit (`gh`)
    python3 scripts/build_jars.py 26.3 --natives local --allow-missing   # this computer's alone

The five are windows-x86_64, windows-aarch64, linux-x86_64, linux-aarch64 (Vulkan) and
macos-aarch64 (Metal); scripts/build_natives.py builds and checks them. Each goes in the jar as
natives/<os>-<arch>/<library>.xz, half the size of the jar's own deflate; the host unpacks the
one it needs at boot. A jar missing any of the five is an error unless --allow-missing, and a
player on a platform it lacks is told so at boot (FailedScreen).

The Minecraft and loader versions come from versions/<minecraft>.properties, laid over the
gradle.properties files for this build only (`-P`). A version with no file there is refused:
the host is written against one version's code, and another needs its file and, often, a port
(docs/upgrading.md). The jars land in dist/<minecraft>/.
"""

import argparse
import hashlib
import json
import lzma
import os
import platform
import shutil
import subprocess
import sys
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import build_natives  # noqa: E402

PLATFORMS = build_natives.PLATFORMS


def library(plat):
    os_name = plat.split("-")[0]
    return {"macos": "liblodger_core.dylib", "windows": "lodger_core.dll"}.get(os_name, "liblodger_core.so")


def here():
    system = platform.system().lower()
    os_name = "macos" if system == "darwin" else "windows" if system == "windows" else "linux"
    machine = platform.machine().lower()
    arch = "aarch64" if machine in ("arm64", "aarch64") else "x86_64" if machine in ("x86_64", "amd64") else machine

    return "%s-%s" % (os_name, arch)


def run(cmd, cwd=ROOT, env=None):
    print("$", " ".join(cmd), "(in %s)" % os.path.relpath(cwd, ROOT) if cwd != ROOT else "")
    subprocess.run(cmd, cwd=cwd, env=env, check=True)


def versions(minecraft):
    path = os.path.join(ROOT, "versions", minecraft + ".properties")
    if not os.path.isfile(path):
        known = sorted(f[: -len(".properties")] for f in os.listdir(os.path.join(ROOT, "versions")) if f.endswith(".properties"))
        sys.exit("no versions/%s.properties: the versions known are %s. A new one needs its loader versions there, "
                 "and the host ported to it (docs/upgrading.md)." % (minecraft, ", ".join(known)))
    props = {}
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            props[key.strip()] = value.strip()

    return props


def natives_from_ci(staging):
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True).stdout.strip()
    runs = json.loads(subprocess.run(
        ["gh", "run", "list", "--workflow", "ci.yml", "--commit", commit, "--status", "success", "--json", "databaseId", "--limit", "1"],
        cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True).stdout)
    if not runs:
        sys.exit("no successful CI run for %s: push it and wait for the ci workflow, or use --natives local or a directory" % commit[:10])
    # Each artifact in a folder of its name, holding <os>-<arch>/<library>.
    fetched = staging + "-fetched"
    shutil.rmtree(fetched, ignore_errors=True)
    run(["gh", "run", "download", str(runs[0]["databaseId"]), "--pattern", "natives-*", "--dir", fetched])
    for artifact in os.listdir(fetched):
        natives_from_dir(os.path.join(fetched, artifact), staging)
    shutil.rmtree(fetched, ignore_errors=True)


def natives_built(staging, which):
    for plat in which:
        build_natives.build(plat, staging)


def natives_from_dir(source, staging):
    for plat in PLATFORMS:
        found = os.path.join(source, plat, library(plat))
        if os.path.isfile(found):
            os.makedirs(os.path.join(staging, plat), exist_ok=True)
            shutil.copy2(found, os.path.join(staging, plat, library(plat)))


def carried(jar):
    with zipfile.ZipFile(jar) as z:
        names = set(z.namelist())

    return [p for p in PLATFORMS if "natives/%s/%s.xz" % (p, library(p)) in names]


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def compress(staging, have, extra):
    """Each native as <library>.xz: the strongest preset, one stream, which XZ for Java reads.
    Answers the SHA256SUMS lines for each, as built and as packed."""
    sums = []
    for plat in have:
        source = os.path.join(staging, plat, library(plat))
        target = os.path.join(extra, plat, library(plat) + ".xz")
        os.makedirs(os.path.dirname(target), exist_ok=True)
        with open(source, "rb") as f:
            data = f.read()
        packed = lzma.compress(data, format=lzma.FORMAT_XZ, check=lzma.CHECK_CRC64,
                               preset=9 | lzma.PRESET_EXTREME)
        with open(target, "wb") as f:
            f.write(packed)
        print("%s: %.1f MB, %.1f MB packed" % (plat, len(data) / 1e6, len(packed) / 1e6), flush=True)
        sums.append("%s  natives/%s/%s" % (sha256(data), plat, library(plat)))
        sums.append("%s  natives/%s/%s.xz" % (sha256(packed), plat, library(plat)))

    return sums


def main():
    parser = argparse.ArgumentParser(description="Build the lodger's Fabric and NeoForge jars with every platform's native.")
    parser.add_argument("minecraft", help="the Minecraft version, as in versions/<it>.properties (26.3)")
    parser.add_argument("--natives", default="cross", help="cross (build the five here), local (this computer's), ci (the CI run of this commit), "
                        "or a directory of <os>-<arch>/<library>")
    parser.add_argument("--allow-missing", action="store_true", help="build even when a platform's native is missing")
    parser.add_argument("--allow-dirty", action="store_true", help="build from the CI's natives though the mind or the SDK has uncommitted changes")
    args = parser.parse_args()

    # The CI built the commit, not what is changed since: the jars would carry natives of other code.
    if args.natives == "ci" and not args.allow_dirty:
        dirty = subprocess.run(["git", "status", "--porcelain", "--", "core", "sdk", "Cargo.toml", "Cargo.lock"],
                               cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True).stdout.strip()
        if dirty:
            sys.exit("the mind or the SDK has uncommitted changes, and the CI's natives are of the commit; commit and push "
                     "them, or --natives local, or --allow-dirty:\n" + dirty)

    props = versions(args.minecraft)
    gradle_props = ["-P%s=%s" % (k, v) for k, v in props.items()]

    staging = os.path.join(ROOT, "mod", "build", "natives-staging")
    shutil.rmtree(staging, ignore_errors=True)
    os.makedirs(staging)
    if args.natives == "ci":
        natives_from_ci(staging)
    elif args.natives == "cross":
        natives_built(staging, PLATFORMS)
    elif args.natives == "local":
        natives_built(staging, [here()])
    elif os.path.isdir(args.natives):
        natives_from_dir(os.path.abspath(args.natives), staging)
    else:
        sys.exit("--natives is cross, local, ci or a directory; %s is none of them" % args.natives)

    have = [p for p in PLATFORMS if os.path.isfile(os.path.join(staging, p, library(p)))]
    missing = [p for p in PLATFORMS if p not in have]
    print("natives:", ", ".join(have) or "none")
    if missing and not args.allow_missing:
        sys.exit("missing natives for %s (--allow-missing builds without them; players on those are told at boot)" % ", ".join(missing))

    # The natives the jars carry are these and nothing else, compressed.
    extra = os.path.join(ROOT, "mod", "natives-extra")
    shutil.rmtree(extra, ignore_errors=True)
    sums = compress(staging, have, extra)

    # Old jars out of the way: another version's host jar would make two where one is looked
    # for, and another's lodger jar would be checked and copied with these.
    for loader in ("fabric", "neoforge"):
        libs_dir = os.path.join(ROOT, "host", loader, "build", "libs")
        if os.path.isdir(libs_dir):
            for name in os.listdir(libs_dir):
                if name.startswith("craftkit-") and name.endswith(".jar"):
                    os.remove(os.path.join(libs_dir, name))
    shutil.rmtree(os.path.join(ROOT, "mod", "build", "libs"), ignore_errors=True)

    # The full path: Windows does not look in the working directory for a program.
    def gradlew(project):
        return os.path.join(ROOT, project, "gradlew.bat" if os.name == "nt" else "gradlew")
    try:
        run([gradlew("host"), "build", "--no-daemon"] + gradle_props, cwd=os.path.join(ROOT, "host"))
        env = dict(os.environ, CARGO=os.devnull)
        run([gradlew("mod"), "jar", "--no-daemon", "-PnativesOnly=true"] + gradle_props, cwd=os.path.join(ROOT, "mod"), env=env)
    finally:
        # Gone again, so a developer's build afterward packs this computer's own and no stale ones.
        shutil.rmtree(extra, ignore_errors=True)

    out = os.path.join(ROOT, "dist", args.minecraft)
    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(out, exist_ok=True)
    libs = os.path.join(ROOT, "mod", "build", "libs")
    for name in sorted(os.listdir(libs)):
        if name.endswith(".jar") and "-%s-" % props["minecraft_version"] in name:
            jar = os.path.join(libs, name)
            got = carried(jar)
            print("%s: natives for %s" % (name, ", ".join(got)))
            if set(got) != set(have):
                sys.exit("%s carries %s, not the %s staged" % (name, got, have))
            shutil.copy2(jar, os.path.join(out, name))
            with open(jar, "rb") as f:
                sums.append("%s  %s" % (sha256(f.read()), name))
    # Both loaders, every time.
    jars = [n for n in os.listdir(out) if n.endswith(".jar")]
    for loader in ("fabric", "neoforge"):
        if not any(n.startswith("lodger-%s-" % loader) for n in jars):
            sys.exit("no %s jar was built" % loader)
    # Each native as built and as packed in the jars, and the jars: SHA256SUMS beside them, in
    # the format `sha256sum -c` reads for the jars.
    with open(os.path.join(out, "SHA256SUMS"), "w", encoding="utf-8") as f:
        f.write("\n".join(sums) + "\n")
    print("\nSHA-256:\n" + "\n".join(sums))
    print("done:", os.path.relpath(out, ROOT))


if __name__ == "__main__":
    main()
