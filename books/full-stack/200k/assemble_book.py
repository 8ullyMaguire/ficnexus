# Assemble the full FicHub tutorial book from all canonical parts
# Usage: python3 assemble_book.py
import os
import re
import glob
from datetime import date

parts_dir = '/home/alvaro/code/rust/fichub/books/full-stack/200k/parts'
output_md = '/home/alvaro/code/rust/fichub/books/full-stack/200k/FicHub_full-stack_200k.md'
output_epub = '/home/alvaro/code/rust/fichub/books/full-stack/200k/FicHub_full-stack_200k.epub'

# Gather all parts in order (by directory number prefix)
parts = []
for d in sorted(os.listdir(parts_dir)):
    full = os.path.join(parts_dir, d)
    if not os.path.isdir(full):
        continue
    m = re.match(r'(\d{2})-.+', d)
    if not m:
        continue
    md_files = glob.glob(os.path.join(full, '*.md'))
    if md_files:
        parts.append((int(m.group(1)), d, md_files[0]))

print(f"Found {len(parts)} parts to assemble")

# Read all parts content
content_parts = []
for num, name, path in parts:
    with open(path) as f:
        text = f.read()
    content_parts.append(text)
    print(f"  Part {num:02d} ({name}): {len(text.split())} words")

# Assemble with a clear separator between parts
separator = "\n\n" + "=" * 80 + "\n\n"
full_md = separator.join(content_parts)

# Prepend YAML metadata block
today = date.today().isoformat()
yaml_block = f"""---
title: "Building FicHub — Full-Stack 200K Tutorial"
author: "FicHub Contributors"
date: "{today}"
---

"""
full_md = yaml_block + full_md

total_words = len(full_md.split())
with open(output_md, 'w') as f:
    f.write(full_md)
print(f"\nAssembled: {len(parts)} parts, {total_words} words, {len(full_md)} chars")
print(f"Output: {output_md}")
print(f"File size: {os.path.getsize(output_md)} bytes")

# Generate EPUB with pandoc
import subprocess
print("\nGenerating EPUB with pandoc...")
result = subprocess.run(
    ['pandoc', output_md, '-o', output_epub,
     '--toc', '--toc-depth=2',
     '--metadata', f'title=Building FicHub — Full-Stack 200K Tutorial',
     '--metadata', 'author=FicHub Contributors'],
    capture_output=True, text=True
)
if result.returncode == 0:
    print(f"EPUB generated: {output_epub}")
    print(f"EPUB file size: {os.path.getsize(output_epub)} bytes")
else:
    print(f"EPUB generation failed: {result.stderr}")
