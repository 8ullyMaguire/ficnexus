#!/usr/bin/env python3
"""Rank GitHub issues (open + closed) of a repo by hybrid heuristic + Ollama semantic score.

Usage:
    python scripts/rank_issues.py [--repo FicHub/fichub.net] [--token PAT]
        [--output fichub_issues_ranked.csv] [--no-llm] [--model llama3.2:3b]
        [--ollama-url http://localhost:11434] [--max-llm N] [--limit N]
        [--workers 4]

Heuristic score:
    +15 p0/critical/blocker label
    +5  bug label
    +3  enhancement label
    +2  per thumbs-up reaction
    +3  per heart reaction
    -5  per thumbs-down reaction
    +1  per comment
Semantic score (Ollama, optional): 0-10 "user impact vs implementation effort".
Final = heuristic + semantic (0 if Ollama offline or --no-llm).
"""
import argparse
import csv
import json
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

import requests

API = "https://api.github.com"
UA = {"Accept": "application/vnd.github.v3+json"}


def fetch_all_issues(repo, token=None, limit=None):
    headers = dict(UA)
    if token:
        headers["Authorization"] = f"token {token}"
    issues, url = [], f"{API}/repos/{repo}/issues?state=all&per_page=100"
    page = 0
    while url:
        page += 1
        res = requests.get(url, headers=headers, timeout=30)
        if res.status_code == 403 and "rate limit" in res.text.lower():
            reset = res.headers.get("X-RateLimit-Reset")
            wait = max(0, int(reset) - int(time.time())) + 5 if reset else 60
            print(f"  rate-limited; sleeping {wait}s...", file=sys.stderr)
            time.sleep(wait)
            continue
        res.raise_for_status()
        data = res.json()
        issues.extend(i for i in data if "pull_request" not in i)
        print(f"  page {page}: {len(data)} items, {len(issues)} issues so far", file=sys.stderr)
        if limit and len(issues) >= limit:
            issues = issues[:limit]
            break
        url = res.links.get("next", {}).get("url")
        if url:
            time.sleep(0.5)  # be polite to the API
    return issues


def calculate_heuristic_score(issue):
    score = 0
    labels = [l["name"].lower() for l in issue.get("labels", [])]
    if any(l in ("p0", "critical", "blocker") for l in labels):
        score += 15
    if "bug" in labels:
        score += 5
    if "enhancement" in labels:
        score += 3
    if "question" in labels:
        score += 1
    if "good first issue" in labels:
        score += 1
    r = issue.get("reactions", {}) or {}
    score += r.get("+1", 0) * 2
    score += r.get("heart", 0) * 3
    score -= r.get("-1", 0) * 5
    score += issue.get("comments", 0) * 1
    return score


def get_ollama_score(title, body, model, ollama_url):
    prompt = (
        "You are a product manager. Score this GitHub issue from 1 to 10 based on "
        "User Impact (high = good) vs Implementation Effort (low = good). "
        "A high-impact, low-effort issue scores near 10; a low-impact, high-effort "
        "issue scores near 1.\n"
        f"Title: {title}\nBody: {(body or '')[:500]}\n"
        'Return ONLY a JSON object: {"score": <int 1-10>, "reason": "<short explanation>"}'
    )
    try:
        res = requests.post(
            f"{ollama_url}/api/generate",
            json={"model": model, "prompt": prompt, "format": "json",
                  "stream": False, "keep_alive": "5m", "options": {"temperature": 0}},
            timeout=60,
        )
        res.raise_for_status()
        raw = res.json().get("response", "")
        data = json.loads(raw)
        score = int(data.get("score", 0))
        return max(1, min(10, score)), str(data.get("reason", ""))[:300]
    except Exception as e:
        return 0, f"Ollama error: {e}"


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--repo", default="FicHub/fichub.net")
    ap.add_argument("--token", default=None, help="GitHub PAT (only for private repos)")
    ap.add_argument("--output", default="fichub_issues_ranked.csv")
    ap.add_argument("--no-llm", action="store_true", help="Skip Ollama semantic scoring")
    ap.add_argument("--model", default="llama3.2:3b")
    ap.add_argument("--ollama-url", default="http://localhost:11434")
    ap.add_argument("--max-llm", type=int, default=0,
                    help="Cap number of issues sent to Ollama (0 = all). "
                         "Issues are pre-sorted by heuristic when capped.")
    ap.add_argument("--limit", type=int, default=0, help="Cap total issues fetched")
    ap.add_argument("--workers", type=int, default=4, help="Parallel Ollama workers")
    args = ap.parse_args()

    print(f"Fetching issues from {args.repo} (state=all)...", file=sys.stderr)
    issues = fetch_all_issues(args.repo, args.token, args.limit or None)
    print(f"Fetched {len(issues)} issues (PRs filtered out).", file=sys.stderr)

    ollama_up = not args.no_llm
    if ollama_up:
        try:
            requests.get(f"{args.ollama_url}/api/tags", timeout=5)
        except Exception:
            ollama_up = False
            print("Ollama offline — heuristic scores only.", file=sys.stderr)

    rows = []
    for i, issue in enumerate(issues, 1):
        h = calculate_heuristic_score(issue)
        rows.append({
            "heuristic": h,
            "llm": 0,
            "reason": "",
            "issue": issue,
        })

    if ollama_up:
        targets = sorted(rows, key=lambda r: r["heuristic"], reverse=True)
        if args.max_llm:
            targets = targets[: args.max_llm]
        print(f"Scoring {len(targets)} issues with Ollama ({args.model}, {args.workers} workers)...",
              file=sys.stderr)
        done = 0
        with ThreadPoolExecutor(max_workers=args.workers) as ex:
            futs = {
                ex.submit(get_ollama_score, r["issue"]["title"], r["issue"].get("body") or "",
                          args.model, args.ollama_url): r
                for r in targets
            }
            for fut in as_completed(futs):
                r = futs[fut]
                r["llm"], r["reason"] = fut.result()
                done += 1
                if done % 25 == 0 or done == len(futs):
                    print(f"  {done}/{len(futs)} scored", file=sys.stderr)

    ranked = []
    for r in rows:
        issue = r["issue"]
        labels = ", ".join(l["name"] for l in issue.get("labels", []))
        ranked.append({
            "Score": r["heuristic"] + r["llm"],
            "State": issue["state"],
            "Title": issue["title"],
            "Labels": labels,
            "Comments": issue.get("comments", 0),
            "URL": issue["html_url"],
            "LLM_Score": r["llm"] or "",
            "LLM_Reasoning": r["reason"],
        })
    ranked.sort(key=lambda x: x["Score"], reverse=True)

    with open(args.output, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=list(ranked[0].keys()))
        writer.writeheader()
        writer.writerows(ranked)

    print(f"\nWrote {len(ranked)} issues to {args.output}")
    print("\nTop 10 by score:")
    for row in ranked[:10]:
        print(f"  {row['Score']:>5}  [{row['State'][:1].upper()}]  {row['Title'][:80]}")


if __name__ == "__main__":
    main()
