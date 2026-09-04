#!/usr/bin/env python3
"""Assemble the FicHub full-stack 100k book: concat parts/ in order into
the final Markdown, validate word count, and report. Run from the repo root.

Usage: python3 books/assemble.py
"""
import pathlib
import re
import sys

ROOT = pathlib.Path("/personal/documents/code/rust/fichub")
BOOK = ROOT / "books" / "full-stack" / "100k"
PARTS = BOOK / "parts"
OUT = BOOK / "FicHub_FULL_STACK_100K.md"
TARGET_WORDS = 200_000


def main() -> int:
    part_dirs = sorted(
        (p for p in PARTS.iterdir() if p.is_dir()),
        key=lambda p: int(p.name.split("-")[0]),
    )
    if not part_dirs:
        print("No part directories found under", PARTS)
        return 1

    chunks = []
    # YAML metadata block
    chunks.append("---")
    chunks.append('title: "Building FicHub — Full-Stack 100K Guide"')
    chunks.append('author: "FicHub Contributors"')
    chunks.append('date: "2026-08-12"')
    chunks.append("---")
    chunks.append("")
    chunks.append("# Building FicHub — Full-Stack Code-Along")
    chunks.append("")
    chunks.append("> A 200k-word, hands-on guide to building the FicHub fanfiction")
    chunks.append("> platform from scratch. Written for junior developers. Follow")
    chunks.append("> along with the real source in `/personal/documents/code/rust/fichub`.")
    chunks.append("")
    chunks.append("---")
    chunks.append("")

    total_words = 0
    for p in part_dirs:
        md_files = sorted(p.glob("*.md"))
        if not md_files:
            print(f"WARN: no .md in {p.name}")
            continue
        for mf in md_files:
            text = mf.read_text(encoding="utf-8")
            words = len(re.findall(r"\S+", text))
            total_words += words
            chunks.append(f"\n\n---\n\n<!-- part: {p.name} -->\n\n")
            chunks.append(text)
            print(f"  {p.name}/{mf.name}: {words:,} words")

    OUT.write_text("\n".join(chunks), encoding="utf-8")
    print(f"\nAssembled: {OUT}")
    print(f"Total words: {total_words:,} (target {TARGET_WORDS:,}, "
          f"{'OK' if abs(total_words - TARGET_WORDS) <= TARGET_WORDS * 0.1 else 'OFF ±10%'})")
    print(f"Parts: {len(part_dirs)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
