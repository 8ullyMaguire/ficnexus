#!/usr/bin/env python3
"""Quick word count of assembled Markdown + parts."""
from pathlib import Path

md = Path("BOOK_FULL_STACK_200K.md")
parts_dir = Path("parts")

total = 0
if md.exists():
    total += len(md.read_text().split())
for p in sorted(parts_dir.glob("*/*.md")):
    total += len(p.read_text().split())
print(f"Total words across assembled MD + parts: {total}")
