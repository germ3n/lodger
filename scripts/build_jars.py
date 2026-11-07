#!/usr/bin/env python3
"""Builds the lodger's two jars, Fabric and NeoForge, for one Minecraft version, with the mind's
native library for every platform inside (owner):

    python3 scripts/build_jars.py 26.3                 # natives from the CI run of this commit
    python3 scripts/build_jars.py 26.3 --natives local # the ones this computer can build
    python3 scripts/build_jars.py 26.3 --natives DIR   # DIR/<os>-<arch>/<library>

The six platforms are windows, linux and macos, each on x86_64 and aarch64. A Mac builds both
macOS ones itself; Windows and Linux ones need their own machines, which is what the CI's matrix
is (.github/workflows/ci.yml builds each on its own runner), so the default takes them from the
CI's run of the commit checked out (push it first; `gh` must be signed in). A jar missing any of
the six is an error unless --allow-missing, and a player on a platform it lacks is told so at
boot (FailedScreen).

The Minecraft and loader versions come from versions/<minecraft>.properties, laid over the
gradle.properties files for this build only (`-P`). A version with no file there is refused:
the host is written against one version's code, and another needs its file and, often, a port
(docs/upgrading.md). The jars land in dist/<minecraft>/.
"""

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PLATFORMS = ["windows-x86_64", "windows-aarch64", "linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"]
TARGETS = {
    "windows-x86_64": "x86_64-pc-windows-msvc",
    "windows-aarch64": "aarch64-pc-windows-msvc",
    "linux-x86_64": "x86_64-unknown-linux-gnu",
    "linux-aarch64": "aarch64-unknown-linux-gnu",
    "macos-x86_64": "x86_64-apple-darwin",
    "macos-aarch64": "aarch64-apple-darwin",
}


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
    run(["gh", "run", "download", str(runs[0]["databaseId"]), "--pattern", "native-*", "--dir", staging])
    for name in os.listdir(staging):
        if name.startswith("native-"):
            os.rename(os.path.join(staging, name), os.path.join(staging, name[len("native-"):]))


def natives_local(staging):
    mine = here()
    targets = [mine]
    # A Mac builds for both its chips.
    if mine.startswith("macos-"):
        targets = ["macos-aarch64", "macos-x86_64"]
    for plat in targets:
        triple = TARGETS.get(plat)
        if plat != mine:
            run(["rustup", "target", "add", triple])
            run(["cargo", "build", "--release", "-p", "lodger-core", "--target", triple])
            built = os.path.join(ROOT, "target", triple, "release", library(plat))
        else:
            run(["cargo", "build", "--release", "-p", "lodger-core"])
            built = os.path.join(ROOT, "target", "release", library(plat))
        os.makedirs(os.path.join(staging, plat), exist_ok=True)
        shutil.copy2(built, os.path.join(staging, plat, library(plat)))


def natives_from_dir(source, staging):
    for plat in PLATFORMS:
        found = os.path.join(source, plat, library(plat))
        if os.path.isfile(found):
            os.makedirs(os.path.join(staging, plat), exist_ok=True)
            shutil.copy2(found, os.path.join(staging, plat, library(plat)))


def carried(jar):
    with zipfile.ZipFile(jar) as z:
        names = set(z.namelist())

    return [p for p in PLATFORMS if "natives/%s/%s" % (p, library(p)) in names]


def main():
    parser = argparse.ArgumentParser(description="Build the lodger's Fabric and NeoForge jars with every platform's native.")
    parser.add_argument("minecraft", help="the Minecraft version, as in versions/<it>.properties (26.3)")
    parser.add_argument("--natives", default="ci", help="ci (the CI run of this commit, the default), local, or a directory of <os>-<arch>/<library>")
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
    elif args.natives == "local":
        natives_local(staging)
    elif os.path.isdir(args.natives):
        natives_from_dir(os.path.abspath(args.natives), staging)
    else:
        sys.exit("--natives is ci, local or a directory; %s is none of them" % args.natives)

    have = [p for p in PLATFORMS if os.path.isfile(os.path.join(staging, p, library(p)))]
    missing = [p for p in PLATFORMS if p not in have]
    print("natives:", ", ".join(have) or "none")
    if missing and not args.allow_missing:
        sys.exit("missing natives for %s (--allow-missing builds without them; players on those are told at boot)" % ", ".join(missing))

    # The natives the jars carry are these and nothing else.
    extra = os.path.join(ROOT, "mod", "natives-extra")
    shutil.rmtree(extra, ignore_errors=True)
    shutil.copytree(staging, extra)

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
    print("done:", os.path.relpath(out, ROOT))


if __name__ == "__main__":
    main()
