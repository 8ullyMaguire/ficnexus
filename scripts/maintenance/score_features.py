#!/usr/bin/env python3
"""Score the user-listed FicHub features and merge them into the ranked CSV.

Reads scripts/feature_inventory.py for the feature list (with impl status).
For NOT-implemented features, computes:
  - heuristic score (category-based priority + community weight)
  - Ollama semantic score (1-10) 'user impact vs implementation effort'
Final Score = heuristic + LLM score (0 if Ollama offline).
Implemented features get a Score of 0 with a 'implemented' marker so they
still appear in the CSV but rank below everything actionable (and get
excluded from the kanban triage selection).
Existing CSV rows are kept unchanged; new rows are appended then sorted.

Usage:
    python scripts/score_features.py [--no-llm] [--model llama3.2:3b]
        [--ollama-url http://localhost:11434] [--max-llm N] [--workers 4]
        [--inventory scripts/feature_inventory.py]
        [--output fichub_issues_ranked.csv]
"""
import argparse
import csv
import json
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

import requests

# Heuristic weights by category — how much a feature moves the needle.
# (mirrors the GitHub issue heuristics in rank_issues.py, adapted to plans)
CATEGORY_WEIGHTS = {
    "utility": 8,       # core promise: getting stories onto devices
    "scraper": 8,       # new sites = new users
    "reading": 7,       # daily driver
    "retention": 5,     # addictiveness
    "discovery": 6,     # find next obsession
    "search": 6,        # findability
    "community": 3,     # social sweetener
    "social": 3,
    "translation": 6,   # global reach
    "consensus": 2,     # roadmap self-improvement
    "admin": 2,         # operator tooling
    "anti-bot": 2,      # hygiene, not user-visible value
    "perf": 3,
    "safety": 4,        # reader safety
}

# Extra points for features called out as P0/crucial in the plans
BONUS_KEYWORDS = [
    (r"send.to.kindle", 3),
    (r"strict.gen", 3),
    (r"websearch_to_tsquery|ts_headline|highlight", 2),
    (r"reciprocal.rank|RRF|hybrid", 2),
    (r"batch|series", 2),
    (r"auto.tagger|machine.suggested", 2),
    (r"offline|PWA", 2),
    (r"nllb|auto.translation", 2),
    (r"blind.date", 2),
    (r"scene", 1),
    (r"import.export", 1),
    (r"data.export", 1),
]


def heuristic_score(feat):
    score = CATEGORY_WEIGHTS.get(feat["cat"], 2)
    title = feat["title"].lower()
    for pat, bonus in BONUS_KEYWORDS:
        if re.search(pat, title):
            score += bonus
    return score


def get_ollama_score(title, note, model, ollama_url, timeout=120):
    prompt = (
        "You are a product manager for FicHub, a fanfiction archive/download "
        "platform. Score this proposed feature from 1 to 10 based on User Impact "
        "(high = good) vs Implementation Effort (low = good). Return ONLY a JSON "
        f'object: {{"score": <int>, "reason": "<short explanation>"}}\n'
        f"Feature: {title}\n"
        f"Context: {note or ''}"
    )
    try:
        res = requests.post(
            f"{ollama_url}/api/generate",
            json={"model": model, "prompt": prompt, "format": "json", "stream": False},
            timeout=timeout,
        )
        data = json.loads(res.json()["response"])
        score = int(data.get("score", 0))
        return max(1, min(10, score)), data.get("reason", "")
    except Exception as e:
        return 0, f"Ollama offline ({e.__class__.__name__})"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-llm", action="store_true", help="skip Ollama scoring")
    ap.add_argument("--model", default="llama3.2:3b")
    ap.add_argument("--ollama-url", default="http://localhost:11434")
    ap.add_argument("--max-llm", type=int, default=60, help="max LLM calls")
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--inventory", default="scripts/feature_inventory.py")
    ap.add_argument("--output", default="fichub_issues_ranked.csv")
    args = ap.parse_args()

    # Load inventory
    ns = {}
    with open(args.inventory) as f:
        code = f.read()
        exec(compile(code, args.inventory, "exec"), ns)
    features = ns["FEATURES"]
    print(f"inventory: {len(features)} features", file=sys.stderr)

    unimplemented = [f for f in features if not f["impl"]]
    implemented = [f for f in features if f["impl"]]
    print(f"  unimplemented: {len(unimplemented)}, implemented: {len(implemented)}",
          file=sys.stderr)

    # Load existing CSV rows
    existing = []
    try:
        with open(args.output, newline="") as f:
            existing = list(csv.DictReader(f))
        print(f"existing CSV rows: {len(existing)}", file=sys.stderr)
    except FileNotFoundError:
        print("existing CSV not found; starting fresh", file=sys.stderr)

    existing_titles = {r["Title"].strip().lower() for r in existing}

    # UPDATE existing rows whose impl status changed (inventory is source of truth)
    updated = 0
    for r in existing:
        t = r["Title"].strip().lower()
        for f in features:
            if f["title"].strip().lower() == t:
                new_impl = "yes" if f["impl"] else "no"
                if r.get("Impl") != new_impl:
                    r["Impl"] = new_impl
                    if f["impl"]:
                        r["State"] = "closed"
                        r["Score"] = 0
                        r["LLM_Reasoning"] = f"already implemented ({f['cat']}) — {f['note'] or ''}".strip()
                    else:
                        r["State"] = "open"
                        r["Score"] = heuristic_score(f)
                        r["LLM_Reasoning"] = f"plan feature ({f['cat']}) — {f['note'] or ''}".strip()
                    updated += 1
                break
    print(f"  impl-status updates: {updated}", file=sys.stderr)
    dupes = 0
    new_rows = []
    for f in unimplemented:
        t = f["title"].strip().lower()
        if t in existing_titles:
            dupes += 1
            continue
        h = heuristic_score(f)
        new_rows.append({
            "Score": h,
            "State": "open",
            "Title": f["title"],
            "Labels": f"feature-plan;{f['cat']}",
            "Comments": 0,
            "URL": "",
            "LLM_Score": 0,
            "LLM_Reasoning": f"plan feature ({f['cat']}) — {f['note'] or ''}".strip(),
            "Impl": "no",
        })

    # Also add implemented features with Score 0 so they appear (marked)
    for f in implemented:
        t = f["title"].strip().lower()
        if t in existing_titles:
            continue
        new_rows.append({
            "Score": 0,
            "State": "closed",
            "Title": f["title"],
            "Labels": f"feature-plan;{f['cat']};implemented",
            "Comments": 0,
            "URL": "",
            "LLM_Score": 0,
            "LLM_Reasoning": f"already implemented ({f['cat']}) — {f['note'] or ''}".strip(),
            "Impl": "yes",
        })
    print(f"  skipped dupes: {dupes}, added new: {len(new_rows)}", file=sys.stderr)

    # Ollama semantic scoring for the top unimplemented candidates
    if not args.no_llm:
        todo = [r for r in new_rows if r["Impl"] == "no"]
        todo = todo[: args.max_llm]
        print(f"  LLM-scoring {len(todo)} unimplemented features...", file=sys.stderr)

        def score_row(row):
            s, reason = get_ollama_score(row["Title"], row["LLM_Reasoning"],
                                         args.model, args.ollama_url)
            row["LLM_Score"] = s
            row["LLM_Reasoning"] = reason
            row["Score"] = int(row["Score"]) + s
            return row

        done = 0
        with ThreadPoolExecutor(max_workers=args.workers) as ex:
            futs = {ex.submit(score_row, r): r for r in todo}
            for fut in as_completed(futs):
                fut.result()
                done += 1
                if done % 5 == 0:
                    print(f"    {done}/{len(todo)} scored", file=sys.stderr)

    # Merge: keep existing rows as-is, add new rows, sort by Score desc
    fieldnames = ["Score", "State", "Title", "Labels", "Comments", "URL",
                  "LLM_Score", "LLM_Reasoning", "Impl"]
    all_rows = existing + new_rows
    # Normalize Score to int for sorting (existing may be str)
    for r in all_rows:
        try:
            r["Score"] = int(float(r["Score"]))
        except (ValueError, TypeError):
            r["Score"] = 0
    all_rows.sort(key=lambda r: r["Score"], reverse=True)

    with open(args.output, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(all_rows)

    print(f"WROTE {args.output}: {len(all_rows)} rows "
          f"({len(existing)} existing + {len(new_rows)} new)", file=sys.stderr)
    # Console summary: top 15 actionable
    actionable = [r for r in all_rows if r.get("Impl") == "no" and r["Score"] > 0]
    actionable.sort(key=lambda r: r["Score"], reverse=True)
    print("\n=== TOP 15 NOT-IMPLEMENTED (actionable) ===")
    for r in actionable[:15]:
        print(f"{r['Score']:>5} | {r['Title'][:75]}")
    print(f"\ntotal actionable: {len(actionable)}")


if __name__ == "__main__":
    main()
