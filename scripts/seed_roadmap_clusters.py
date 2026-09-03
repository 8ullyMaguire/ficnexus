#!/usr/bin/env python3
"""Seed the Roadmap Consensus Engine's feature_clusters table with real features.

Sources:
  1. scripts/consensus_seed.json  — 15 feature clusters derived from GitHub issues
  2. docs/IDEAS.md                — the project's feature-idea brain dump

For each unique feature it:
  - embeds the representative text via Ollama (nomic-embed-text, 768-d)
  - INSERTs a feature_clusters row (status='open', elo_rating=1500)
  - skips existing rows with the same representative_text (idempotent)

Usage:
    source .env && python3 scripts/seed_roadmap_clusters.py [--dry-run] [--ollama-url http://localhost:11434]
"""
import argparse
import json
import os
import re
import sys
import time
from pathlib import Path

import requests

REPO = Path(__file__).resolve().parent.parent


def parse_ideas_md(text: str):
    """Extract feature lines from IDEAS.md as (title, starred) pairs."""
    features = []
    # Stop at the summary section (not features)
    text = text.split("## Why these ideas hit the 70% mark")[0]
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("- ") and not line.startswith("- **"):
            title = line[2:].strip()
            starred = "⭐" in title
            title = title.replace("⭐", "").strip()
            # Skip bullets that are just prose/intro
            if len(title) > 5 and "–" not in title[:10]:
                features.append((title, starred))
        elif line.startswith("- **") and "**" in line:
            # Bold sub-bullets (e.g. "**⭐ Broader site support** – ...")
            m = re.match(r"- \*\*(⭐)?\s*(.+?)\*\*\s*[–-]?\s*(.*)", line)
            if m:
                starred = bool(m.group(1))
                title = m.group(2).strip()
                desc = m.group(3).strip()
                features.append((title, starred))
    return features


def embed(text: str, ollama_url: str, model: str) -> list:
    res = requests.post(
        f"{ollama_url}/api/embeddings",
        json={"model": model, "prompt": text},
        timeout=60,
    )
    res.raise_for_status()
    emb = res.json().get("embedding", [])
    if not emb:
        raise RuntimeError("empty embedding")
    return emb


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true", help="print features, don't insert")
    ap.add_argument("--ollama-url", default="http://localhost:11434")
    ap.add_argument("--model", default="nomic-embed-text")
    ap.add_argument("--json", default=str(REPO / "scripts" / "consensus_seed.json"))
    ap.add_argument("--ideas", default=str(REPO / "docs" / "IDEAS.md"))
    ap.add_argument("--batch", type=int, default=8, help="sleep between embeds (avoid ollama queue)")
    args = ap.parse_args()

    # Collect features: consensus seed first (high confidence), then IDEAS.md
    features = []
    seen = set()
    with open(args.json, encoding="utf-8") as f:
        seed = json.load(f)
    for c in seed["seed"]:
        title = c["title"]
        if title not in seen:
            features.append((title, c.get("summary", ""), True))
            seen.add(title)

    ideas_text = Path(args.ideas).read_text(encoding="utf-8")
    for title, starred in parse_ideas_md(ideas_text):
        if title not in seen:
            features.append((title, "", starred))
            seen.add(title)

    print(f"Collected {len(features)} unique features (from consensus_seed.json + IDEAS.md)")

    # DB connection via app user (from DATABASE_URL)
    db_url = os.environ.get("DATABASE_URL")
    if not db_url and not args.dry_run:
        print("ERROR: DATABASE_URL not set. Run: source .env && python3 scripts/seed_roadmap_clusters.py", file=sys.stderr)
        sys.exit(1)

    # Verify Ollama reachable
    try:
        requests.get(f"{args.ollama_url}/api/tags", timeout=5)
        print(f"Ollama OK ({args.model})")
    except Exception as e:
        print(f"WARN: Ollama unreachable ({e}) — embedding will fail; use --dry-run to list features", file=sys.stderr)

    if args.dry_run:
        for title, summary, starred in features:
            print(f"  would insert: {title} (starred={starred})")
        print(f"\nDry run — {len(features)} features would be inserted.")
        return

    try:
        import psycopg
        conn = psycopg.connect(db_url)
    except ImportError:
        import psycopg2
        conn = psycopg2.connect(db_url)
    cur = conn.cursor()

    # Existing representative texts to avoid dupes
    cur.execute("SELECT representative_text FROM feature_clusters")
    existing = {r[0] for r in cur.fetchall()}
    print(f"Existing clusters: {len(existing)}")

    inserted = 0
    for title, summary, starred in features:
        rep_text = title if not summary else f"{title}: {summary}"
        if rep_text in existing:
            print(f"  skip (exists): {title}")
            continue
        if args.dry_run:
            print(f"  would insert: {title}")
            continue
        try:
            emb = embed(rep_text, args.ollama_url, args.model)
        except Exception as e:
            print(f"  ERROR embedding {title}: {e} — skipping", file=sys.stderr)
            time.sleep(1)
            continue
        emb_sql = "[" + ",".join(f"{x:.6f}" for x in emb) + "]"
        cur.execute(
            "INSERT INTO feature_clusters (representative_text, embedding, elo_rating) VALUES (%s, %s::vector, 1500.0)",
            (rep_text, emb_sql),
        )
        inserted += 1
        print(f"  inserted: {title} (starred={starred})")
        time.sleep(0.3)

    conn.commit()
    cur.close()
    conn.close()
    print(f"\nDone. Inserted {inserted} clusters. Run: sudo -u postgres psql -d fichub -c 'SELECT id, representative_text FROM feature_clusters ORDER BY id DESC LIMIT 10;' to verify.")


if __name__ == "__main__":
    main()
