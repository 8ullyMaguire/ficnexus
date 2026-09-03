#!/usr/bin/env python3
"""Seed feature_clusters with the user-listed FicHub features — implemented OR not.

The user wants every listed feature to appear in the Roadmap Consensus Engine
so users can vote on it and see whether it's actually useful — regardless of
whether it's already implemented.

Reads scripts/feature_inventory.py (FEATURES list with impl flags) and
INSERTs any missing feature_clusters rows (idempotent, skip-if-representative
text exists). Representative text for implemented features is prefixed with
"[Implemented] " so voters can tell status; proposed features get "[Idea] ".

Usage:
    source .env && python3 scripts/seed_feature_clusters.py [--dry-run]
"""
import argparse
import os
import sys
import time
from pathlib import Path

import requests

REPO = Path(__file__).resolve().parent.parent


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
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--ollama-url", default="http://localhost:11434")
    ap.add_argument("--model", default="nomic-embed-text")
    ap.add_argument("--inventory", default=str(REPO / "scripts" / "feature_inventory.py"))
    args = ap.parse_args()

    ns = {}
    with open(args.inventory) as f:
        exec(compile(f.read(), args.inventory, "exec"), ns)
    features = ns["FEATURES"]

    db_url = os.environ.get("DATABASE_URL")
    if not db_url and not args.dry_run:
        print("ERROR: DATABASE_URL not set. Run: source .env && python3 scripts/seed_feature_clusters.py",
              file=sys.stderr)
        sys.exit(1)

    try:
        requests.get(f"{args.ollama_url}/api/tags", timeout=5)
        print(f"Ollama OK ({args.model})")
    except Exception as e:
        print(f"WARN: Ollama unreachable ({e})", file=sys.stderr)

    if args.dry_run:
        for f in features:
            tag = "IMPLEMENTED" if f["impl"] else "idea"
            print(f"  would insert [{tag}]: {f['title']}")
        print(f"\nDry run — {len(features)} features")
        return

    import psycopg2
    conn = psycopg2.connect(db_url)
    cur = conn.cursor()

    cur.execute("SELECT representative_text FROM feature_clusters")
    existing = {r[0] for r in cur.fetchall()}
    print(f"Existing clusters: {len(existing)}")

    inserted = 0
    for f in features:
        tag = "[Implemented] " if f["impl"] else "[Idea] "
        rep_text = tag + f["title"]
        if rep_text in existing:
            print(f"  skip (exists): {f['title'][:60]}")
            continue
        try:
            emb = embed(rep_text, args.ollama_url, args.model)
        except Exception as e:
            print(f"  ERROR embedding {f['title'][:60]}: {e} — skipping", file=sys.stderr)
            time.sleep(1)
            continue
        emb_sql = "[" + ",".join(f"{x:.6f}" for x in emb) + "]"
        cur.execute(
            "INSERT INTO feature_clusters (representative_text, embedding, elo_rating) "
            "VALUES (%s, %s::vector, 1500.0)",
            (rep_text, emb_sql),
        )
        inserted += 1
        print(f"  inserted [{('IMPLEMENTED' if f['impl'] else 'idea')}]: {f['title'][:70]}")
        time.sleep(0.3)

    conn.commit()
    cur.close()
    conn.close()
    print(f"\nDone. Inserted {inserted} clusters.")


if __name__ == "__main__":
    main()
