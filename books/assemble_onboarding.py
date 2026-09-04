#!/usr/bin/env python3
"""Assemble the FicHub Developer Onboarding book into one Markdown + EPUB.

Usage: python3 books/assemble_onboarding.py
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path("/personal/documents/code/rust/fichub")
PARTS = ROOT / "books" / "onboarding" / "parts"
OUT_DIR = ROOT / "books" / "onboarding"
OUT_MD = OUT_DIR / "FicHub_Dev_Onboarding.md"
OUT_EPUB = OUT_DIR / "FicHub_Dev_Onboarding.epub"
TARGET_WORDS = 20_000


def main() -> int:
    part_files = sorted(PARTS.glob("*.md"))
    if not part_files:
        print("No part files found")
        return 1

    chunks = []
    chunks.append("---")
    chunks.append('title: "FicHub Developer Onboarding"')
    chunks.append('author: "FicHub Contributors"')
    chunks.append('date: "2026-08-11"')
    chunks.append("---")
    chunks.append("")
    chunks.append("# FicHub Developer Onboarding")
    chunks.append("")
    chunks.append("> A ~20,000-word guide for junior developers joining the")
    chunks.append("> FicHub codebase: architecture, repo map, how to run it,")
    chunks.append("> where the key systems live, and where to start contributing.")
    chunks.append("")
    chunks.append("---")
    chunks.append("")

    total_words = 0
    for pf in part_files:
        text = pf.read_text(encoding="utf-8")
        words = len(re.findall(r"\S+", text))
        total_words += words
        chunks.append(f"\n\n---\n\n<!-- part: {pf.name} -->\n\n")
        chunks.append(text)
        print(f"  {pf.name}: {words:,} words")

    OUT_MD.write_text("\n".join(chunks), encoding="utf-8")
    print(f"\nAssembled: {OUT_MD}")
    print(f"Total words: {total_words:,} (target {TARGET_WORDS:,})")

    # Build EPUB
    subprocess.run([
        "pandoc", str(OUT_MD), "-o", str(OUT_EPUB),
        "--toc", "--toc-depth=2",
        "--metadata", "title=FicHub Developer Onboarding",
        "--metadata", "author=FicHub Contributors",
    ], check=True)
    print(f"EPUB: {OUT_EPUB} ({OUT_EPUB.stat().st_size:,} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
