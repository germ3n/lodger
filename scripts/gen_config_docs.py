#!/usr/bin/env python3
"""Writes the content jar's key descriptions, `META-INF/craftkit-config.json`, from the table of
keys in docs/config.md: the settings screen's All keys tab shows each as its tooltip. Run it after
docs/config.md changes; a test in core/src/config.rs fails while the two disagree."""

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOCS = os.path.join(ROOT, "docs", "config.md")
OUT = os.path.join(ROOT, "mod", "src", "main", "resources", "META-INF", "craftkit-config.json")


def plain(text):
    """Markdown to the words a tooltip shows: no backticks, links as their text."""
    text = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", text)
    text = text.replace("`", "").replace("**", "")
    return re.sub(r"\s+", " ", text).strip()


def main():
    docs = {}
    for line in open(DOCS, encoding="utf-8"):
        row = re.match(r"^\| `([a-z_.]+)` \| (.*?) \| (.*) \|\s*$", line)
        if row:
            key, default, about = row.groups()
            docs[key] = plain(about) + " Default: " + plain(default) + "."
    with open(OUT, "w", encoding="utf-8") as out:
        json.dump(docs, out, indent=1, ensure_ascii=False, sort_keys=True)
        out.write("\n")
    print("%d keys -> %s" % (len(docs), os.path.relpath(OUT, ROOT)))


if __name__ == "__main__":
    main()
