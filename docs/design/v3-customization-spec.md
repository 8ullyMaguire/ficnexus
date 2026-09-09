
---

## 11. Extension Platform (v3.1)

Builds on the progression system to let users create, share, and compose custom recommenders, themes, and layouts.

### 11.1 The Power Ladder

Every customization surface offers escalating tiers with different safety + resource envelopes:

| Tier | What the user provides | Runs where | Risk | Resource control |
|---|---|---|---|---|
| **1. Config** | Pick options / weights | Our code | None | Free |
| **2. Recipe** | A JSON composition of existing parts | Our code | None | Free, cached |
| **3. Script** | A small scoring function (WASM) | Sandbox on-host | Medium | Fuel + memory metering |
| **4. Service** | A URL implementing a protocol | Their server | Low | Timeout + circuit breaker + cache |

**The key move:** the most powerful tier (Service) is the cheapest for us because compute lives on the user's machine. The safest tiers (Config/Recipe) cover 90% of what people actually want.

### 11.2 Recipe Builder (P6)

Most users don't want to write code — they want to tune. A recipe is a named blend of existing strategies:

```json
{
  "name": "dark-fic-finder",
  "blend": { "cooccur": 0.3, "tag_graph": 0.4, "embeddings": 0.3 },
  "filters": { "min_words": 30000, "exclude_warnings": ["Major Character Death"] },
  "boost": { "tag": "Dark Harry Potter", "weight": 1.5 }
}
```

This is a `REC_STRATEGIES` weight set + filters — it runs through the existing RRF ranker with zero new engine code. It's a data row, so it's trivially shareable and impossible to abuse.

**DB:** `user_recipes` table (id, user_id, name, description, blend JSONB, filters JSONB, boost JSONB, curator_prior, is_active, is_public, installs, timestamps).

**Feature gates:** recipes.create (L4), recipes.publish (L6), recipes.gallery (L4).

### 11.3 Plugin Manifest (P7)

One table for all shareable objects — recipes, themes, layouts, views:

```json
{
  "kind": "recipe" | "theme" | "layout" | "view",
  "slug": "dark-fic-finder",
  "name": "Dark Fic Finder",
  "author": { "id": 42, "name": "..." },
  "version": 3,
  "tier": "recipe",
  "gate": { "level": 6 },
  "payload": { ... },
  "stats": { "installs": 128, "rating": 0.31 }
}
```

**DB:** `extensions` table (id, kind, slug, name, description, author_id, version, tier, gate_level, payload JSONB, stats JSONB, timestamps).

### 11.4 Theme Design Tokens (P8)

Themes as data, not code — CSS custom properties:

```css
:root {
  --fh-accent: #7c5cff;  --fh-bg: #0f1117;  --fh-surface: #171a23;
  --fh-text: #e8eaf0;    --fh-muted: #8a90a2; --fh-radius: 12px;
  --fh-font: "Inter";    --fh-density: 1;
}
```

Theme = a JSON blob of token overrides. Import/export/share as easily as a recipe.

**Presets:** Default Dark, Default Light, High Contrast, Sepia, Dyslexia-Friendly.

**Reader theme** is separate from app theme (font, width, line height).

**Safety:** themes are token values only, never raw CSS — no injection surface.

### 11.5 Resource Guardrails

| Threat | Guardrail |
|---|---|
| Slow/dead user sidecar | Timeout → fallback; circuit breaker |
| Popular engine hogging compute | Aggressive caching → promote to precomputed |
| Global pipeline pollution | User engines on-demand only, never in batch |
| Arbitrary code on host | Prefer Config/Recipe/Service; WASM gets fuel+memory metering |
| Call spam | Per-user-recommender hourly budget via rate limiter |
| Theme injection | Token-values-only; sanitize raw CSS |
| Marketplace abuse | Level/role gates + shadow-run review |

### 11.6 Build Order

| Phase | Deliverable | Status |
|---|---|---|
|| **P1** | DB migrations, XP engine, feature registry, can() gate | ✅ DONE |
|| **P2** | Ability Tree UI + notifications + onboarding | ✅ DONE |
|| **P3** | Theme + layout customization backend | ✅ DONE |
|| **P4** | Widget-composed home dashboard | ✅ DONE |
|| **P5** | Custom Views + admin feature editor | ✅ DONE |
|| **P6** | Recipe Builder + plugin manifest | ✅ DONE |
|| **P7** | Theme design tokens + presets + import/export | ✅ DONE |
|| **P8** | Unified extension marketplace (`/marketplace`) | ✅ DONE (2026-08-27) |

**P8 detail (shipped):** `migrations/014_extensions.sql` upgrades the baseline
`extensions` table in place — wider kind set (`skin`, `saved_search`, `profile`
join `recipe`/`theme`/`layout`/`view`), **per-category** unique slugs,
`is_public` / `is_verified` flags, first-class `installs` / `rating` columns
(`stats` jsonb kept for legacy readers), restored id sequence, plus
`extension_installs` (dedup per user) and `extension_ratings` (1–5 stars).
Backend: `services/extensions.rs` + `routes/extensions.rs` mounted at
`/api/extensions` — publish (trust-gated TL2+), gallery with kind/search
filters, idempotent install, ratings with running average, remix into a
private draft. Publishing to the existing recipes and skins galleries is now
gated the same way (`services::trust::PUBLISH_MIN_TRUST`).

### 11.7 Deferred (Roadmap)

These are worth considering but not yet:

| Feature | Why deferred |
|---|---|
| Multi-tenant sidecars (user-hosted rec engines) | Needs outbound HTTP safety; compute on user's server |
| WASM sandbox | Heavy dependency (wasmtime); fuel+memory metering needed; only if demand proves out |
| ~~Full marketplace with ratings/reviews~~ | ✅ Shipped 2026-08-27 — see P8 above |
| Nav/layout editor (drag-drop) | Partially built; iterate on existing widget dashboard |
| Layout presets (mobile vs desktop) | Nice but low priority; users can save multiple layouts via user_views |
| Community recommender leaderboard | Needs rec_impressions data to accumulate first |
| Recipe → precomputed promotion | Only when shared recipes get popular enough to warrant batch compute |
| Per-surface rec recipes | Home vs fic-page vs Next-Up each use different blends; extends recipe activation |
| Rec subscription | Follow another user's recipe, auto-propagate updates on change |
| Theme remixing | One-click fork from gallery, tweak, re-share |
| Reduced-motion accessibility preset | CSS prefers-reduced-motion; ships alongside high-contrast |
| Custom shelf colors and icons | Visual library organization; extends shelves metadata |
| Command palette for customization | Universal launcher for saved views, themes, recipes (extends Ctrl+K) |
| User-remappable keyboard shortcuts | Exportable as shareable JSON profiles |
| Skip locked widgets on layout install | Graceful degradation when installing shared layouts |
| Blind-date as unlockable widget | Extends existing Blind Date to widget registry |
| Forum posting level gate | Anti-spam for fresh accounts; configurable threshold |
| Daily quests + reading streaks | Gamified engagement loop; feeds XP toward unlocks |
| Rec explainability | Strategy name + reason on every recommendation card |
| People-like-you row | Taste clusters → explain recommendations by group |
| Curator prior personal slider | User-adjustable taste shaping (exposes REC_CURATOR_PRIOR) |
| Customization profile export | One portable JSON backup of all prefs, themes, recipes, layouts |
| Feature toggle API | Power users script setup via API tokens |
| ~~Curator-verified badge~~ | ✅ `is_verified` column + marketplace flag shipped (2026-08-27); curator review UI still pending |
| ~~Level gate + curator review for publishing~~ | ✅ Publishing gated at trust TL2 across recipes/skins/extensions (2026-08-27) |
| Widget marketplace with live previews | Preview before install; render from layout JSON |
| Rec shown-to-engaged ratio | Social proof per shared recipe from rec_impressions |
| Per-feature notification granularity | Mute notifications for features you disabled |
| Circuit breaker for dead sidecars | Fallback to cooccur when user sidecar is unreachable |

---

## 12. Open Questions

1. **Sidecar compute location:** User's server (Tier 4) recommended over on-host — zero RAM cost to M720q.
2. **Recipe scope:** Per-user global or per-surface (home vs fic-page)? Recommendation: global for simplicity; per-surface (#106) as future extension.
3. **Marketplace moderation:** Shadow-run engagement as gate, or human curator review, or both.
4. **XP from customization:** "Your theme was installed 50 times → reputation" — makes customization itself a progression loop.
5. **Manifest degradation:** Skip missing features or show locked teasers (aspiration pattern).
6. **Rec subscription propagation:** When a followed recipe updates, do followers auto-get the new blend, or is it a snapshot?
7. **Forum level gate threshold:** Configurable per-instance (default L3) or fixed site-wide?
8. **Daily quest scope:** Reading goals only, or mixed (read + bookmark + rate + comment)?
