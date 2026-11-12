#!/usr/bin/env python3
"""Writes the craftkit repository (github.com/germ3n/craftkit) from this one: the SDK crate
(`sdk/`), the Java host (`host/`), the SDK guide, the license and the Rust toolchain, with a
workspace, a README and a .gitignore of its own. This repository stays the source; the export
is refreshed from it and pushed (docs/sdk.md, "The craftkit repository").

    python3 scripts/export_craftkit.py ~/craftkit

The destination's .git is kept; everything else in it is written again."""

import os
import re
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LODGER = "https://github.com/germ3n/lodger/blob/master"
CRAFTKIT = "https://github.com/germ3n/craftkit"

# What never goes: build output, caches, the editor's swap files.
SKIP = {"target", "build", ".gradle", ".DS_Store", ".idea", "out"}

WORKSPACE = """# The craftkit SDK: the Rust crate mods are written against (sdk/). The Java host that loads
# them is the Gradle project in host/. Exported from github.com/germ3n/lodger, where both are
# developed beside their first mod.
[workspace]
resolver = "2"
members = ["sdk"]

[workspace.package]
edition = "2021"
license = "MIT"
repository = "{craftkit}"

[workspace.dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"""

README = """# craftkit

Write Minecraft mods in Rust.

craftkit has two halves, both here:

- **`sdk/`, the `craftkit` crate:** what a mod is written against. A mod implements the `Mod`
  trait, exports it with `export_mod!`, and is built as a shared library. It never touches
  Minecraft's classes: once a tick it is handed what the world did and answers with what it
  wants done, so it is tested with the crate's harness in plain `cargo test`. See the
  [crate's README](sdk/README.md) for a whole mod.
- **`host/`, the craftkit host:** a Java mod for Fabric and NeoForge that loads craftkit mods
  through Java's FFM API, hands them the world and carries out their commands. Players need it
  in the game beside a craftkit mod, or nested inside the mod's own jar.

Minecraft Java {mc} on Java 25, with Fabric or NeoForge. The [SDK guide](docs/sdk.md) describes
the protocol, the C ABI and what the host does.

## Building

```sh
cargo test                     # the crate, its harness and its doctests
cd host && ./gradlew build     # the host jars (host/fabric/build/libs, host/neoforge/build/libs) and their tests
```

## Where it is developed

craftkit is developed in [lodger]({lodger_repo}), its first mod, so the two change together;
this repository is exported from there (`scripts/export_craftkit.py`). Issues and pull requests
are welcome here.

## License

MIT.
"""

GITIGNORE = """/target
Cargo.lock
host/.gradle/
host/build/
host/**/build/
.DS_Store
"""


def ignore(directory, names):
    return [n for n in names if n in SKIP or n.endswith(".swp")]


def sdk_guide():
    """docs/sdk.md with its links to the lodger's other pages made absolute."""
    text = open(os.path.join(ROOT, "docs", "sdk.md"), encoding="utf-8").read()

    def absolute(match):
        label, target = match.group(1), match.group(2)
        if re.match(r"[a-z]+://", target) or target.startswith("#") or target.startswith("sdk.md"):
            return match.group(0)
        # From docs/: a page beside it, or a path up into the repository.
        path = os.path.normpath(os.path.join("docs", target.split("#")[0]))
        anchor = "#" + target.split("#", 1)[1] if "#" in target else ""
        if path.startswith("sdk") or path.startswith("host"):
            return f"[{label}](../{path}{anchor})"
        return f"[{label}]({LODGER}/{path}{anchor})"

    return re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", absolute, text)


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: export_craftkit.py <destination>")
    dest = os.path.abspath(os.path.expanduser(sys.argv[1]))
    os.makedirs(dest, exist_ok=True)
    for name in os.listdir(dest):
        if name == ".git":
            continue
        path = os.path.join(dest, name)
        shutil.rmtree(path) if os.path.isdir(path) and not os.path.islink(path) else os.remove(path)

    for part in ["sdk", "host"]:
        shutil.copytree(os.path.join(ROOT, part), os.path.join(dest, part), ignore=ignore)
    for part in ["LICENSE", "rust-toolchain.toml", ".gitattributes"]:
        if os.path.exists(os.path.join(ROOT, part)):
            shutil.copy2(os.path.join(ROOT, part), os.path.join(dest, part))
    os.makedirs(os.path.join(dest, "docs"), exist_ok=True)
    with open(os.path.join(dest, "docs", "sdk.md"), "w", encoding="utf-8") as out:
        out.write(sdk_guide())

    mc = re.search(r"(?m)^minecraft_version=(\S+)", open(os.path.join(ROOT, "host", "gradle.properties")).read()).group(1)
    with open(os.path.join(dest, "Cargo.toml"), "w") as out:
        out.write(WORKSPACE.format(craftkit=CRAFTKIT))
    with open(os.path.join(dest, "README.md"), "w") as out:
        out.write(README.format(mc=mc, lodger_repo="https://github.com/germ3n/lodger"))
    with open(os.path.join(dest, ".gitignore"), "w") as out:
        out.write(GITIGNORE)
    print(f"craftkit written to {dest}")


if __name__ == "__main__":
    main()
