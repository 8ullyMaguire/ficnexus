#!/usr/bin/env python3
"""
Batch embedding generation for FicHub works using sentence-transformers.

Fetches works without embeddings (or with stale embeddings) from the database,
generates embeddings from (title || description || tags), and stores them as
pgvector-compatible vectors.

Usage:
    python scripts/generate_embeddings.py [--batch-size 100] [--force]
"""

import argparse
import os
import sys
import time
from datetime import datetime, timezone

import psycopg2
import psycopg2.extras
from sentence_transformers import SentenceTransformer

# ── Config ──────────────────────────────────────────────────────────────

DEFAULT_BATCH_SIZE = 100
MODEL_NAME = "sentence-transformers/all-MiniLM-L6-v2"  # 384-dim embeddings

DATABASE_URL = os.environ.get(
    "DATABASE_URL",
    "postgresql://fichub:fichub@localhost:5432/fichub",
)


def parse_args():
    parser = argparse.ArgumentParser(
        description="Generate embeddings for FicHub works"
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=DEFAULT_BATCH_SIZE,
        help=f"Number of works to process per batch (default: {DEFAULT_BATCH_SIZE})",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Re-generate embeddings for ALL works, not just missing/stale ones",
    )
    parser.add_argument(
        "--where",
        type=str,
        default=None,
        help="Additional SQL WHERE clause (e.g. 'AND w.id = 42')",
    )
    return parser.parse_args()


def connect_db():
    """Connect to PostgreSQL."""
    conn = psycopg2.connect(DATABASE_URL)
    conn.autocommit = False
    return conn


def fetch_works(conn, batch_size: int, force: bool, extra_where: str | None):
    """Yield batches of (work_id, title, description, tags_array) from the works table.

    Tags are aggregated from the fic_tags/tags tables via fic_info.

    Returns rows as dicts with keys: id, canonical_title, description, tags.
    """
    where_clauses = []
    if not force:
        where_clauses.append("(w.embedding IS NULL OR w.embedding_updated_at IS NULL)")
    if extra_where:
        where_clauses.append(extra_where)

    where_sql = " AND ".join(where_clauses) if where_clauses else "TRUE"

    query = f"""
        SELECT w.id,
               w.canonical_title,
               w.description,
               COALESCE(
                   ARRAY_AGG(DISTINCT t.name) FILTER (WHERE t.name IS NOT NULL),
                   ARRAY[]::text[]
               ) AS tags
        FROM works w
        LEFT JOIN fic_info fi ON fi.work_id = w.id
        LEFT JOIN fic_tags ft ON ft.url_id = fi.id
        LEFT JOIN tags t ON t.id = ft.tag_id
        WHERE {where_sql}
        GROUP BY w.id, w.canonical_title, w.description
        ORDER BY w.id
    """

    offset = 0
    while True:
        with conn.cursor(cursor_factory=psycopg2.extras.DictCursor) as cur:
            cur.execute(f"{query} LIMIT %s OFFSET %s", (batch_size, offset))
            rows = cur.fetchall()

        if not rows:
            break

        yield rows
        offset += batch_size


def generate_embedding_text(row) -> str:
    """Build the text to embed from a work row."""
    parts = []
    if row["canonical_title"]:
        parts.append(row["canonical_title"])
    if row["description"]:
        parts.append(row["description"])
    if row.get("tags"):
        parts.append(" ".join(row["tags"]))
    return " ".join(parts)


def store_embeddings(conn, work_ids, embeddings):
    """Batch-upsert embeddings into the works table."""
    if not work_ids:
        return

    now = datetime.now(timezone.utc)
    with conn.cursor() as cur:
        for wid, emb in zip(work_ids, embeddings):
            emb_list = emb.tolist()
            cur.execute(
                "UPDATE works SET embedding = %s::vector, embedding_updated_at = %s WHERE id = %s",
                (emb_list, now, wid),
            )

    conn.commit()


def main():
    args = parse_args()
    print(f"Loading model: {MODEL_NAME}")
    model = SentenceTransformer(MODEL_NAME)
    print(f"Model loaded. Output dimension: {model.get_sentence_embedding_dimension()}")

    conn = connect_db()
    total_processed = 0
    total_elapsed = 0.0

    try:
        for batch_idx, rows in enumerate(fetch_works(
            conn, args.batch_size, args.force, args.where
        )):
            batch_start = time.time()

            work_ids = []
            texts = []
            for row in rows:
                work_ids.append(row["id"])
                texts.append(generate_embedding_text(row))

            if not texts:
                continue

            # Generate embeddings
            embeddings = model.encode(texts, show_progress_bar=False)

            # Store in database
            store_embeddings(conn, work_ids, embeddings)

            batch_time = time.time() - batch_start
            total_processed += len(work_ids)
            total_elapsed += batch_time

            print(
                f"Batch {batch_idx}: {len(work_ids)} works "
                f"({total_processed} total) in {batch_time:.2f}s "
                f"({len(work_ids)/batch_time:.1f} works/s)"
            )

    except KeyboardInterrupt:
        print("\nInterrupted. Cleaning up...")
        conn.rollback()
    finally:
        conn.close()

    if total_processed > 0:
        print(f"\nDone. Processed {total_processed} works in {total_elapsed:.1f}s "
              f"({total_processed/total_elapsed:.1f} works/s)")
    else:
        print("No works to process.")


if __name__ == "__main__":
    main()
