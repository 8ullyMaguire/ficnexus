#!/usr/bin/env python3
"""Populate FicHub DB from RoyalRoad best-rated list via the local API.

Usage: python3 populate_royalroad.py [limit]
"""
import json
import sys
import time
import urllib.parse
import urllib.request

BASE = "http://localhost:8000/api/epub"

# RoyalRoad best-rated fics (from https://www.royalroad.com/fictions/best-rated)
URLS = [
    "https://www.royalroad.com/fiction/21220/mother-of-learning",
    "https://www.royalroad.com/fiction/36735/the-perfect-run",
    "https://www.royalroad.com/fiction/21410/super-minion",
    "https://www.royalroad.com/fiction/107917/sky-pride",
    "https://www.royalroad.com/fiction/26675/a-journey-of-black-and-red",
    "https://www.royalroad.com/fiction/39408/beware-of-chicken",
    "https://www.royalroad.com/fiction/43318/the-butcher-of-gadobhra",
    "https://www.royalroad.com/fiction/48402/magical-girl-gunslinger",
    "https://www.royalroad.com/fiction/62125/ghost-in-the-city-cyberpunk-gamer-si",
    "https://www.royalroad.com/fiction/65058/pale-lights",
    "https://www.royalroad.com/fiction/65629/the-game-at-carousel-a-horror-movie-litrpg",
    "https://www.royalroad.com/fiction/72498/sublight-drive-star-wars",
    "https://www.royalroad.com/fiction/92144/the-legend-of-william-oh",
    "https://www.royalroad.com/fiction/92820/phantom-star",
    "https://www.royalroad.com/fiction/104434/the-elf-who-would-become-a-dragon-story-complete",
    "https://www.royalroad.com/fiction/106438/magical-girl-mechanical-heart",
    "https://www.royalroad.com/fiction/132259/lost-and-found-warhammer-40k-si",
    "https://www.royalroad.com/fiction/151748/clara-casewell-attorney-to-the-villainess-vol",
    "https://www.royalroad.com/fiction/173313/headless-over-heels-a-dark-fantasy-romancebook",
    "https://www.royalroad.com/fiction/54508/the-unexpected-engagement-of-the-marvelous-mr",
]

def scrape(url: str) -> dict:
    q = urllib.parse.quote(url, safe="")
    req = urllib.request.Request(f"{BASE}?q={q}", headers={"User-Agent": "fichub-populate/1.0"})
    try:
        with urllib.request.urlopen(req, timeout=180) as resp:
            return json.loads(resp.read().decode())
    except Exception as e:  # noqa: BLE001
        return {"err": -99, "msg": str(e)}

def main():
    limit = int(sys.argv[1]) if len(sys.argv) > 1 else len(URLS)
    ok = fail = 0
    for i, url in enumerate(URLS[:limit], 1):
        print(f"[{i}/{min(limit, len(URLS))}] {url}")
        result = scrape(url)
        if result.get("err") == 0:
            info = result.get("info", "")[:80]
            print(f"  OK  {info}")
            ok += 1
        else:
            print(f"  FAIL err={result.get('err')} {result.get('msg', '')[:120]}")
            fail += 1
        # Be polite: delay between scrapes
        time.sleep(3)
    print(f"\nDone: {ok} OK, {fail} failed")

if __name__ == "__main__":
    main()
