#!/bin/bash
OUTPUT="FICHUB_BACKEND_100K.md"

# Header
cat > "$OUTPUT" << 'HEADER'
# Building the FicHub Backend

## A Complete Guide to Building a Fanfiction Download Server in Rust

**By the FicHub Project**

*Version 2.0 — 2026*

---

> *"This book teaches you how to build a complete, production-ready backend server in Rust. From web scraping to EPUB generation, from Redis rate limiting to collaborative filtering recommendations — every chapter walks you through real code from a real project."*

---

HEADER

# Concatenate all parts
for f in parts/*/*.md; do
    cat "$f" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "---" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
done

echo "Assembly complete"
