#!/usr/bin/env python3
"""Find chapters with stale or missing roadmap/v3 content.
Scans assembled MD + parts for keywords that must reflect the current codebase."""
from pathlib import Path

md = Path("BOOK_FULL_STACK_200K.md")
parts_dir = Path("parts")

text_parts = []
if md.exists():
    text_parts.append(md.read_text())

for p in sorted(parts_dir.glob("*/*.md")):
    text_parts.append(p.read_text())

full = "\n".join(text_parts)

# Anchors that should appear given the 2026-08 shipped state
anchors = [
    "Extension marketplace",
    "recipe builder",
    "theme design tokens",
    "plugin manifest",
    "extensions table",
    "extensions.rs",
    "marketplace",
    "trust-gated publishing",
    "TL2",
    "extensions gallery",
    "Entity recommendations",
    "GET /api/v0/recommendations/entities",
    "fandom landing page",
    "/fandoms",
    "bulk admin actions",
    "main_char_attr",
    "removed",
    "progression system",
    "100 levels",
    "10 ranks",
    "ability tree",
    "widget dashboard",
    "Trust system",
    "TL0",
    "TL5",
    "Community Moderators",
    "trust_promote",
    "badge-wiring",
    "reputation + daily login streaks",
    "community bounties",
    "forum depth phase 2",
    "read-state",
    "pin/lock",
    "metamod",
    "ReactionsPicker",
    "emoji reactions",
    "forum-core crate",
    "SymbolSquares",
    "SymbolsGuideModal",
    "symbols.ts",
    "rating G/T/M/E",
    "search-history chips",
    "GET /api/search/history/chips",
    "history chips",
    "OPDS",
    "Readium Web Publication",
    "RWPM",
    "GET /opds/manifest",
    "consensus hub",
    "/curator/consensus",
    "ConsensusAuthFix",
    "Docs hub fix",
    "Request AnswerRow",
    "search_queries.user_id",
    "lfm2.5:8b",
    "21.7",
    "token/s",
    "qwen3.5:9b",
    "self-healing",
    "heal_extract",
    "on-the-fly",
    "Cookie ingestion",
    "fannish_next_of_kin",
    "username history",
    "user_past_usernames",
    "per-fic hit counter",
    "view count",
    "reading history",
    "comment-reply inbox",
    "known issues",
    "status page",
    "bot vs human",
    "cohort retention",
    "conversion funnel",
    "zero-result query",
    "scraper health board",
    "content coverage",
    "feature adoption",
    "modlog diff",
    "reply inbox",
    "mute/block",
    "site-wide",
    "blocked_users",
    "admin announcement banner",
    "dismissed",
    "per-fic language",
    "Extensibility",
    "Extension marketplace shipped",
    "63e6c66",
    "014",
    "052",
    "053",
    "100-level",
]

import re
def fuzzy(line):
    return re.sub(r"[^a-z0-9 ]", " ", line.lower())
    return re.sub(r"\s+", " ", ...).strip()

hits = set()
missing = []
found_any = False

for a in anchors:
    pat = re.compile(re.escape(a.lower()), re.IGNORECASE)
    if pat.search(full):
        hits.add(a)
    else:
        missing.append(a)

print(f"Anchors: {len(anchors)}")
print(f"Found: {len(hits)}")
print("Missing ({}):".format(len(missing)))
for m in missing:
    print("  -", m)
