"""Writes THIRD_PARTY_NOTICES.txt: the license notices for what the lodger's jars carry.

Adapted from ~/chainv4's gen_notices.py (owner). The jars carry the mind, a native
library compiled from this repository's Cargo workspace, which links llama.cpp; the host's Java
needs nothing beyond the game and its loader. The model is downloaded, not shipped, so its
license is named, not reproduced. Run it after a dependency changes:

    python3 gen_notices.py

It stops with a message when a crate no longer carries what it expects, rather than write an
incomplete file.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
import build_natives  # noqa: E402

XZ_VERSION = re.search(r"(?m)^xz_version=(\S+)", open(os.path.join(ROOT, "host", "gradle.properties"), encoding="utf-8").read()).group(1)
ZERO_BSD = """Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE."""
NOTICES = os.path.join(ROOT, "THIRD_PARTY_NOTICES.txt")
RULE = "-" * 80


def section(title):
    return ["", RULE, title, RULE, ""]


def read(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read().replace("\r\n", "\n").replace("\r", "\n")


def norm_text(text):
    out = []
    for line in (line.rstrip() for line in text.split("\n")):
        if line == "" and (not out or out[-1] == ""):
            continue
        out.append(line)
    while out and out[-1] == "":
        out.pop()

    return "\n".join(out)


meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--format-version", "1", "--locked"],
    cwd=ROOT, check=True, stdout=subprocess.PIPE,
).stdout)


def ours(p):
    return os.path.abspath(p["manifest_path"]).startswith(ROOT + os.sep)


by_key = {(p["name"], p["version"]): p for p in meta["packages"]}
lock = read(os.path.join(ROOT, "Cargo.lock"))
crates, local = {}, set()
for name, version in re.findall(r'\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"', lock):
    p = by_key.get((name, version))
    if p is None:
        sys.exit("missing metadata for %s %s" % (name, version))
    if ours(p):
        local.add(name)
    else:
        crates[(name, version)] = p
crates = sorted(crates.values(), key=lambda p: (p["name"].lower(), p["version"]))
features = {n["id"]: n.get("features", []) for n in meta["resolve"]["nodes"]}


def crate(name):
    for p in crates:
        if p["name"] == name:
            return p
    sys.exit("Cargo.lock no longer holds %s; update gen_notices.py" % name)


def crate_dir(p):
    return os.path.dirname(p["manifest_path"])


def label(p):
    return "%s %s" % (p["name"], p["version"])


def expr(p):
    return re.sub(r"\s*/\s*", " OR ", p["license"] or "")


def choices(p):
    lic = expr(p)
    if " AND " in lic:
        return None

    return [c.strip("() ") for c in lic.split(" OR ")]


LICENSE_NAME = re.compile(r"(?i)^(licen[cs]e|copying|copyright|notice|unlicense)")


def license_files(p):
    d = crate_dir(p)
    files = [f for f in os.listdir(d) if LICENSE_NAME.match(f) and os.path.isfile(os.path.join(d, f))]
    rank = lambda f: (0 if "MIT" in f.upper() else 1 if f.upper() in ("LICENSE", "LICENSE.MD", "LICENSE.TXT") else 2, f)

    return [os.path.join(d, f) for f in sorted(files, key=rank)]


TEMPLATE = re.compile(
    r"(?i)\[yyyy\]|\{yyyy\}|<year>|\[year\]|\{year\}|<copyright|\[name of|\{name of|"
    r"copyright (notice|license|owner|holder|statement|and permission|and license|and related)|"
    r"copyright notices|copyright,? patent|copyright law|copyright interest|subject to copyright|"
    r"copyright assignment|copyright to\b|copyright in\b"
)
COPYRIGHT_LINE = re.compile(r"(?i)^[\s#*/;!-]*((copyright\b|©).*)$")


def copyright_lines(paths):
    out = []
    for path in paths:
        try:
            text = read(path)
        except OSError:
            continue
        for line in text.split("\n"):
            m = COPYRIGHT_LINE.match(line)
            if not m:
                continue
            s = re.sub(r"\s+", " ", m.group(1)).strip().rstrip(",;")
            if TEMPLATE.search(s) or len(s) < 12 or not re.search(r"\d{4}|[A-Z][a-z]", s[9:]):
                continue
            if s not in out:
                out.append(s)

    return out


def authors(p):
    names = [re.sub(r"\s*<[^>]*>", "", a).strip().rstrip(".") for a in p.get("authors") or []]

    return [n for n in names if n]


def needs_full_text(p):
    c = choices(p)
    if c is None:
        parts = re.split(r"\s+(?:AND|OR)\s+", expr(p).replace("(", "").replace(")", ""))

        return any(x not in ("MIT", "Apache-2.0", "Unicode-3.0") for x in parts)
    if "MIT" in c or "Apache-2.0" in c:
        return False

    return c[0] not in ("CC0-1.0", "WTFPL", "Unlicense")


def full_text_files(p):
    out = []
    for path in license_files(p):
        text = read(path)
        if "APACHE" in os.path.basename(path).upper() or re.match(r"\s*Apache License", text):
            continue
        out.append(path)

    return out


NOTES = {
    "r-efi": "Used under the MIT terms. The LGPL-2.1-or-later option is not used.",
    "unicode-ident": "Crate code is used under the MIT terms; its Unicode data is under the\n  Unicode License V3, whose text the crate carries.",
    "llama-cpp-sys-2": "Bundles llama.cpp and ggml, compiled into the native library; see the\n  llama.cpp section.",
}

groups = []
for p in crates:
    if not needs_full_text(p):
        continue
    files = full_text_files(p)
    if not files:
        sys.exit("no license text for %s" % label(p))
    text = "\n\n".join(norm_text(read(f)) for f in files)
    for g in groups:
        if g["text"] == text:
            g["crates"].append(p)
            break
    else:
        groups.append({"text": text, "crates": [p], "expr": expr(p)})


def apache_text():
    counts = {}
    for p in crates:
        for path in license_files(p):
            if "APACHE" not in os.path.basename(path).upper():
                continue
            text = norm_text(read(path))
            stop = text.find("END OF TERMS AND CONDITIONS")
            if text.lstrip().startswith("Apache License") and stop >= 0:
                text = text[: stop + len("END OF TERMS AND CONDITIONS")]
                counts[text] = counts.get(text, 0) + 1
    if not counts:
        sys.exit("no crate in Cargo.lock ships the Apache-2.0 text")

    return max(counts, key=lambda text: (counts[text], text))


def join_names(names):
    if len(names) == 1:
        return names[0]

    return ", ".join(names[:-1]) + " and " + names[-1]


def wrap(text, width=80, indent=""):
    lines, cur = [], indent
    for w in text.split():
        if len(cur) + len(w) + (0 if cur.strip() == "" else 1) > width:
            lines.append(cur.rstrip())
            cur = indent + w
        else:
            cur = cur + ("" if cur.strip() == "" else " ") + w
    if cur.strip():
        lines.append(cur.rstrip())

    return lines


llama = crate("llama-cpp-sys-2")
llama_license = os.path.join(crate_dir(llama), "llama.cpp", "LICENSE")
if not os.path.isfile(llama_license):
    sys.exit("%s no longer bundles llama.cpp/LICENSE; update gen_notices.py" % label(llama))
common = "common" in features.get(llama["id"], [])

out = [
    "Third-party notices",
    "",
    "Distribute this file with the lodger's jars, for Fabric and for NeoForge. It collects the",
    "license notices for the code compiled into them. The lodger's own license is reproduced",
    "first, so this file can stand alone next to a jar.",
    "",
    "Crate versions below are the ones pinned in this repository's Cargo.lock. License texts",
    "were read from those crate versions.",
]

out += section("The lodger and craftkit")
out += ["Source: this repository", "License: MIT", "", norm_text(read(os.path.join(ROOT, "LICENSE")))]

out += section("The Apache License, Version 2.0")
out += wrap("Reproduced once here. It covers every Apache-2.0 crate listed further down.")
out += ["", apache_text()]

out += section("llama.cpp and ggml")
out += wrap(
    "Bundled in %s and compiled into the native library: llama.cpp and ggml, which run the "
    "model. %s"
    % (label(llama), "Its common library is compiled in too." if common else "Its common library, server and tools are not compiled in.")
)
out += ["License: MIT", "", norm_text(read(llama_license))]

out += section("Khronos's Vulkan headers")
out += wrap(
    "Compiled into the Windows and Linux natives (llama.cpp's Vulkan backend is written against "
    "them, vulkan.hpp's inline code among them): Vulkan-Headers %s, under Apache-2.0 OR MIT, used "
    "under the MIT terms reproduced at the top. Copyright 2015-2025 The Khronos Group Inc. The "
    "SPIR-V headers (%s, MIT-style, Copyright (c) 2015-2024 The Khronos Group Inc.) are read at "
    "build time only." % (build_natives.VULKAN_HEADERS, build_natives.SPIRV_HEADERS)
)

out += section("XZ for Java")
out += wrap(
    "Bundled in the host jar to unpack the natives: org.tukaani:xz %s, by Lasse Collin and "
    "the XZ for Java contributors, under the BSD Zero Clause License:" % XZ_VERSION
)
out += ["", ZERO_BSD]

out += section("Microsoft's C and C++ runtime")
out += wrap(
    "Linked statically into the Windows natives, so no Visual C++ Redistributable is needed, as "
    "the Visual Studio license permits for the runtime's static libraries."
)

out += section("The model")
out += wrap(
    "Not distributed with the jars: the first start downloads it from Hugging Face, from its "
    "publisher, after the player accepts its license on the title screen. Qwen2.5 0.5B and 1.5B "
    "Instruct and Qwen3 4B Instruct 2507 are under the Apache License, Version 2.0; Qwen2.5 3B "
    "Instruct is under the Qwen Research License."
)

if groups:
    out += section("Other license texts Cargo.lock requires")
    out += ["These texts are in addition to the MIT and Apache-2.0 texts above."]
    for g in groups:
        out += [""] + wrap(join_names([label(p) for p in g["crates"]])) + ["  " + g["expr"], "", g["text"]]

out += section("Rust crates from Cargo.lock")
out += wrap(
    "Every third-party crate in Cargo.lock is listed below, for every platform, build-time tools "
    "among them. Local crates %s are this repository, covered by the MIT license at the top."
    % join_names(sorted(local))
)
out += [""]
out += wrap(
    "Where a crate offers a choice, this distribution uses MIT if MIT is one of the choices, and "
    "otherwise the license noted for that crate. MIT-licensed crates use the MIT terms reproduced "
    "at the top with the copyright lines given here. Apache-2.0 crates use the Apache License text above."
)
out += ["", "%d crates." % len(crates)]
for p in crates:
    lines = copyright_lines(license_files(p))
    out += ["", label(p), "  " + expr(p)]
    if lines:
        out += ["  " + l for l in lines]
    else:
        names = authors(p)
        if names:
            out += wrap("Authors: " + ", ".join(names) + ". The package has no separate copyright line.", indent="  ")
        else:
            out.append("  %s contributors. The package has no separate copyright line." % p["name"])
    if p["name"] in NOTES:
        out.append("  " + NOTES[p["name"]])

text = "\n".join(out).rstrip("\n") + "\n"
open(NOTICES, "w", encoding="utf-8").write(text)
print(len(crates), "crates,", len(groups), "text groups,", text.count("\n"), "lines")
