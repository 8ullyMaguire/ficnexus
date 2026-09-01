#!/usr/bin/env python3
"""
Populate FicHub with the most popular fics from fic.ai/popular/.

fic.ai/popular (the "Popular Fics" leaderboard) only carries minimal per-fic
metadata (title/author/words/chapters + a link). The correct way to get FULL
metadata (fandom, tags, chapters, body, status) into FicHub is the site's own
scrape/export pipeline — GET /api/epub?q=<source_url>&bypass_cache=1 — which
scrapes via fanfic_scrapers, upserts fic_info, extracts tags, find_or_create's
the works row, and stores the body.

So this script only:
  1. fetches + parses the fic.ai/popular list,
  2. normalizes each fic's source URL (strips ?query and #fragment so the
     url_id hash and stored source match),
  3. passes one URL at a time to the FicHub /api/epub endpoint for full ingest.

Usage:
    python3 scripts/scrape_ficai_popular.py --base http://localhost:8000 --dry-run
    python3 scripts/scrape_ficai_popular.py --base http://localhost:8000 --limit 1
    python3 scripts/scrape_ficai_popular.py --base http://localhost:8000 --up-to 100

Supported source sites come from the fic.ai list (AO3, ffnet, XenForo, RR, ...)
and are ingested via FicHub's own scrapers — no hardcoded table writes here.
"""

import argparse
import re
import subprocess
import sys
import time
import urllib.parse


def fetch_page(url: str) -> str:
    result = subprocess.run(
        ["curl", "-sL", url], capture_output=True, text=True, timeout=30
    )
    result.check_returncode()
    return result.stdout


def parse_fics(page_html: str) -> list[dict]:
    """Parse fic entries from fic.ai/popular/ HTML."""
    fics = []
    blocks = re.split(r"<div>\s*<h3>", page_html)

    for block in blocks[1:]:
        fic = {}

        m = re.search(r"#(\d+)\s*-\s*(.+?)\s*by\s*(.+?)\s*</h", block)
        if not m:
            continue
        fic["rank"] = int(m.group(1))
        fic["title"] = m.group(2).strip()
        fic["author"] = re.sub(r"<[^>]+>", "", m.group(3)).strip()

        desc_m = re.search(r"<p>\s*(.*?)<br\s*/>", block, re.DOTALL)
        fic["description"] = (
            re.sub(r"<[^>]+>", "", desc_m.group(1)).strip() if desc_m else ""
        )

        wc_m = re.search(r"(\d+)\s*words\s+in\s+(\d+)\s*chapters", block)
        fic["words"] = int(wc_m.group(1)) if wc_m else 0
        fic["chapters"] = int(wc_m.group(2)) if wc_m else 0

        dates_m = re.search(
            r"published\s+(\d{4}-\d{2}-\d{2})\s+\d{2}:\d{2}:\d{2}"
            r",?\s*updated\s+(\d{4}-\d{2}-\d{2})\s+\d{2}:\d{2}:\d{2}",
            block,
        )
        fic["published"] = dates_m.group(1) if dates_m else "1970-01-01"
        fic["updated"] = dates_m.group(2) if dates_m else "1970-01-01"

        url_m = re.search(r'href="(https?://[^"]+)"', block)
        if not url_m:
            continue
        fic["url"] = url_m.group(1)

        fics.append(fic)

    return fics


def normalize_url(raw: str) -> str:
    """Strip ?query and #fragment so url_id hash + stored source are clean."""
    clean = raw.split("#", 1)[0]
    parsed = urllib.parse.urlsplit(clean)
    return urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, parsed.path, "", ""))


def ingest(base: str, url: str, timeout_s: int = 180) -> tuple[int, str]:
    """Call FicHub /api/epub?q=<url>&bypass_cache=1. Return (http_code, body_head)."""
    q = urllib.parse.quote(url, safe="")
    target = f"{base}/api/epub?q={q}&bypass_cache=1"
    try:
        result = subprocess.run(
            ["curl", "-sL", "-o", "-", "--write-out", "\n%{http_code}", target],
            capture_output=True, text=True, timeout=timeout_s,
        )
    except subprocess.TimeoutExpired:
        return 408, "timeout"
    out = result.stdout or ""
    if "\n" in out:
        body, code = out.rsplit("\n", 1)
    else:
        body, code = out, "000"
    return int(code), body.strip()[:300]


def main():
    p = argparse.ArgumentParser(description="Populate FicHub from fic.ai/popular")
    p.add_argument("--base", default="http://localhost:8000",
                   help="FicHub base URL (default http://localhost:8000)")
    p.add_argument("--limit", type=int, default=0,
                   help="ingest only the first N fics; 0 = all")
    p.add_argument("--rank", type=int, default=0,
                   help="ingest only this specific rank (1-based); overrides --limit")
    p.add_argument("--delay", type=float, default=1.5,
                   help="seconds between ingests (default 1.5; be gentle)")
    p.add_argument("--max-timeout", type=int, default=180,
                   help="per-fic curl timeout seconds")
    p.add_argument("--dry-run", action="store_true",
                   help="print the normalized URLs and per-fic scrape target but don't ingest")
    p.add_argument("--url", default="https://fic.ai/popular/")
    args = p.parse_args()

    print(f"Fetching {args.url}...", file=sys.stderr)
    page_html = fetch_page(args.url)
    fics = parse_fics(page_html)
    print(f"Found {len(fics)} fics", file=sys.stderr)

    for fic in fics:
        fic["url"] = normalize_url(fic["url"])

    if args.rank:
        picks = [f for f in fics if f["rank"] == args.rank]
    elif args.limit:
        picks = fics[: args.limit]
    else:
        picks = fics

    print(f"Ingesting {len(picks)} fics (rank {picks[0]['rank']}..{picks[-1]['rank']})", file=sys.stderr)

    ok, fail = 0, 0
    for i, fic in enumerate(picks, 1):
        target = f"{args.base}/api/epub?q={urllib.parse.quote(fic['url'], safe='')}&bypass_cache=1"
        print(f"[{i}/{len(picks)}] rank {fic['rank']}: {fic['title']} — {fic['url']}", file=sys.stderr)
        if args.dry_run:
            print(f"  -> {target}")
            continue
        code, body = ingest(args.base, fic["url"], args.max_timeout)
        if code == 200:
            ok += 1
            print(f"  OK ({code})", file=sys.stderr)
        else:
            fail += 1
            print(f"  FAIL ({code}): {body}", file=sys.stderr)
        if i < len(picks):
            time.sleep(args.delay)

    print(f"Done: {ok} ok, {fail} failed, {len(picks) - ok - fail} skipped", file=sys.stderr)
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
