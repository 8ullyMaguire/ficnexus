#!/usr/bin/env python3
"""Generate docs-map.json — the stable link contract for in-app docs.

Reads the mdBook HTML output (docs/book/html/*.html) + SUMMARY.md and emits:
  docs-map.json = [
    { slug, page, anchor, title, summary, keywords, feature }
  ]

Every heading (h1-h3) becomes an entry. `anchor` is the heading id from the
built HTML. `slug` is stable: "<page>#<anchor>" (page without .html).
`feature` maps the page to a feature area (used for DocLink placement).

Run AFTER `mdbook build`. Output is written to:
  docs/book/html/docs-map.json
  frontend/static/docs/docs-map.json  (copied for the SPA)
"""
import html
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BOOK_HTML = ROOT / "docs" / "book" / "html"
STATIC_DOCS = ROOT / "frontend" / "static" / "docs"

# Page slug → feature area (drives DocLink placement + palette grouping).
PAGE_FEATURES = {
    "intro": "home",
    "quickstart": "home",
    "what-is-fichub": "home",
    "downloading": "download",
    "searching": "search",
    "bookmarks": "social",
    "ratings": "social",
    "comments": "social",
    "recommendations": "recommend",
    "profile": "social",
    "features": "home",
    "transparency": "admin",
    "faq": "home",
    "anti-bot": "admin",
    "rec-engines": "recommend",
    "self-healing": "admin",
}

# Keywords help the palette/search surface entries ("dark harry", "elo", …).
KEYWORD_OVERRIDES = {
    "searching": ["boolean", "AND OR NOT", "dark harry", "main_char_attr", "query syntax", "filters"],
    "downloading": ["epub", "mobi", "azw3", "kindle", "format", "export"],
    "bookmarks": ["bookmark", "personalize", "recommendations signals"],
    "recommendations": ["recommend", "similar", "co-occurrence", "rec engine"],
    "transparency": ["modlog", "moderation log", "analytics", "transparent"],
    "anti-bot": ["rate limit", "shadowban", "proof of work", "bot"],
    "faq": ["help", "troubleshoot", "question"],
    "quickstart": ["first time", "getting started", "how to"],
}

HEADING_RE = re.compile(r'<h([123]) id="([^"]*)"[^>]*>(.*?)</h\1>', re.S)
TAG_RE = re.compile(r"<[^>]+>")


def clean(text: str) -> str:
    text = html.unescape(TAG_RE.sub("", text))
    return re.sub(r"\s+", " ", text).strip()


def first_paragraph(path: pathlib.Path, anchor: str) -> str:
    """First non-empty paragraph after the heading with this id."""
    text = path.read_text(encoding="utf-8")
    # Find the heading then grab up to ~2 paragraphs of prose
    m = re.search(r'<h[123] id="' + re.escape(anchor) + r'"[^>]*>.*?</h[123]>', text, re.S)
    if not m:
        return ""
    after = text[m.end():]
    paras = re.findall(r"<p>(.*?)</p>", after, re.S)[:2]
    return clean(" ".join(paras))[:220]


def extract_section_body(path: pathlib.Path, anchor: str) -> str:
    """Full section text: heading + all following blocks until the next
    heading of the same or higher level. Strips tags."""
    text = path.read_text(encoding="utf-8")
    m = re.search(r'<h[123] id="' + re.escape(anchor) + r'"[^>]*>.*?</h[123]>', text, re.S)
    if not m:
        return ""
    level = int(m.group(0)[2])
    after = text[m.end():]
    parts = [m.group(0)]
    for block in re.findall(r"<(?:p|li|pre|blockquote|h[1-6])[^>]*>.*?</(?:p|li|pre|blockquote|h[1-6])>", after, re.S):
        tag = re.match(r"<(\w+)", block).group(1)
        if tag.startswith("h"):
            bl = int(tag[1])
            if bl <= level:
                break
        parts.append(block)
    return clean(" ".join(parts))


def extract_page_body(path: pathlib.Path) -> str:
    """Main-content text of a docs page (everything inside <main>)."""
    text = path.read_text(encoding="utf-8")
    m = re.search(r"<main>(.*?)</main>", text, re.S)
    return clean(m.group(1)) if m else clean(text)


def main() -> int:
    if not BOOK_HTML.exists():
        print(f"ERROR: {BOOK_HTML} missing — run `mdbook build` first", file=sys.stderr)
        return 1

    manifest = []
    sections = []  # for Ask-the-Docs (slug, title, body, keywords, feature)
    for page in sorted(BOOK_HTML.glob("*.html")):
        if page.name in ("index.html", "print.html", "404.html"):
            continue
        slug = page.stem
        title = page.read_text(encoding="utf-8")
        m = re.search(r"<title>(.*?)</title>", title, re.S)
        page_title = clean(m.group(1)).replace(" - FicHub Docs", "") if m else slug
        feature = PAGE_FEATURES.get(slug, "home")
        # Collect section bodies for the retrieval index (heading + following
        # paragraphs until next heading).
        section_bodies: dict[str, str] = {}
        for hnum, anchor, raw in HEADING_RE.findall(title):
            heading = clean(raw)
            body = extract_section_body(page, anchor)
            manifest.append({
                "slug": f"{slug}#{anchor}",
                "page": f"{slug}.html",
                "anchor": anchor,
                "title": heading,
                "page_title": page_title,
                "summary": body[:220],
                "keywords": KEYWORD_OVERRIDES.get(slug, []),
                "feature": feature,
                "level": int(hnum),
            })
            section_bodies[f"{slug}#{anchor}"] = body
        sections.append({
            "slug": slug,
            "page": f"{slug}.html",
            "anchor": None,
            "title": page_title,
            "body": extract_page_body(page)[:4000],
            "keywords": KEYWORD_OVERRIDES.get(slug, []),
            "feature": feature,
        })
        for k, v in section_bodies.items():
            if v.strip():
                sections.append({
                    "slug": k,
                    "page": f"{slug}.html",
                    "anchor": k.split("#", 1)[1],
                    "title": next((e["title"] for e in manifest if e["slug"] == k), k),
                    "body": v[:3000],
                    "keywords": KEYWORD_OVERRIDES.get(slug, []),
                    "feature": feature,
                })

    # Also include a top-level "page" entry for each docs page (for the help
    # modal "open full docs" + palette page jumps).
    for slug, feature in PAGE_FEATURES.items():
        p = BOOK_HTML / f"{slug}.html"
        if not p.exists():
            continue
        text = p.read_text(encoding="utf-8")
        m = re.search(r"<title>(.*?)</title>", text, re.S)
        page_title = clean(m.group(1)).replace(" - FicHub Docs", "") if m else slug
        manifest.append({
            "slug": slug,
            "page": f"{slug}.html",
            "anchor": None,
            "title": page_title,
            "page_title": page_title,
            "summary": first_paragraph(p, "top"),
            "keywords": KEYWORD_OVERRIDES.get(slug, []),
            "feature": feature,
            "level": 0,
        })

    # Dedupe by slug (keep first)
    seen = set()
    out = []
    for e in manifest:
        if e["slug"] in seen:
            continue
        seen.add(e["slug"])
        out.append(e)

    BOOK_HTML.joinpath("docs-map.json").write_text(
        json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8"
    )
    if STATIC_DOCS.exists():
        STATIC_DOCS.joinpath("docs-map.json").write_text(
            json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8"
        )

    # doc-sections.json — retrieval index for Ask-the-Docs (slug/title/body/
    # keywords/feature). Consumed by the admin ingest endpoint + /api/docs/ask.
    BOOK_HTML.joinpath("doc-sections.json").write_text(
        json.dumps(sections, indent=1, ensure_ascii=False), encoding="utf-8"
    )
    if STATIC_DOCS.exists():
        STATIC_DOCS.joinpath("doc-sections.json").write_text(
            json.dumps(sections, indent=1, ensure_ascii=False), encoding="utf-8"
        )
    print(f"docs-map.json: {len(out)} entries; doc-sections.json: {len(sections)} sections")
    return 0


if __name__ == "__main__":
    sys.exit(main())
