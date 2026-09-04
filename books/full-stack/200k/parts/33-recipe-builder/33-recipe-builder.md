# Part 33 — Recipe Builder & the v3 Customization Platform

> **What ships in Aug 2026:** FicHub's v3 customization platform landed in three layers — Recipe Builder (P6), the Plugin Manifest / Extension Marketplace (P7, commit `63e6c66`, migration 014), and Theme Design Tokens (P8). Together they let users *mix* their own recommendation recipe, *install* shareable themes/recipes/recipes from a community gallery, and *publish* their own work — all trust-gated so new accounts can't spam the public gallery.
>
> This chapter walks each layer, backend route first then the frontend page that calls it, and ends with hands-on exercises. The v3 design doc that anchors this layout is `docs/v3-customization-spec.md` (sections 11.1–11.8).

Recipes let a user become their own recommender-engine tuner. Each recipe is a little "mixing board" for the recommendation engine: you slide the weights of different strategies (co-occurrence, tag-graph, embeddings), set filters, and tune how loud the human curator's voice counts. When a recipe is **activated**, it overrides the site-wide default strategy weights for that user's recommendations.

Behind the scenes there's one more idea: recipes don't live alone. They're one *kind* of a shared **extension**, which is one row in a single `extensions` table alongside themes, layouts, views, and saved-searches. That table is the Plugin Manifest (P7), and the `/marketplace` gallery is just that table surfaced with filtering + install + 1–5 star ratings + remix.

---

## 33.1 Backend: `GET /api/recipes` & `POST /api/recipes`

Let's start with the two most fundamental operations: **listing** the recipes a user already has, and **creating** a new one. Open `src/routes/recipes.rs` — note the file header (lines 1–12) which lists every route this module serves:

```rust
// src/routes/recipes.rs (lines 1-12)
//! API handlers for Recipe Builder (P1 extension platform).
//!
//! Routes:
//!   GET    /api/recipes              — list user's recipes
//!   POST   /api/recipes              — create recipe
//!   PUT    /api/recipes/:id          — update recipe
//!   DELETE /api/recipes/:id          — delete recipe
//!   POST   /api/recipes/:id/activate — set as active recipe
//!   GET    /api/recipes/active       — get user's active recipe
//!   GET    /api/recipes/gallery      — browse public recipes
//!   POST   /api/recipes/:id/install  — install from gallery
//!   POST   /api/recipes/:id/publish  — make public
```

💡 **Key Concept — the `Recipe` row.** Every recipe is one row in `user_recipes`. The `blend` column is a JSONB object whose keys are recommendation-strategy names and whose values are the weights you want. `curator_prior` (float 0–1) controls how much the human curator's score is blended in. `is_active` marks the one recipe that is currently winning; `is_public` marks recipes that show up in the gallery for others to install.

```rust
// src/routes/recipes.rs (lines 21-47, request types)
#[derive(Debug, Deserialize)]
pub struct CreateRecipeRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub blend: Value,
    #[serde(default)]
    pub filters: Value,
    #[serde(default)]
    pub boost: Value,
    #[serde(default = "default_curator_prior")]
    pub curator_prior: f32,
}

fn default_curator_prior() -> f32 {
    0.1
}
```

### Breakdown

`CreateRecipeRequest` is the JSON body the frontend sends when making a new recipe. Every field maps to a column in the `user_recipes` table (added in migration **053 — `_recipes`**, per `docs/v3-customization-spec.md`):

- **`name`** — a human-readable label like `"Dark & Angsty"` or `"Fluff Focus"`.
- **`description`** — optional, helps you remember what this recipe was for.
- **`blend`** — JSON object with weight values for each recommendation strategy. Example from the spec: `{"cooccur": 0.3, "tag_graph": 0.4, "embeddings": 0.3}`.
- **`filters`** — JSON object for filtering works, e.g. `{"min_words": 30000, "exclude_warnings": ["Major Character Death"]}`.
- **`boost`** — JSON object for boosting certain tags or fandoms.
- **`curator_prior`** — a float from `0.0` to `1.0`; controls how much the human curator's scores influence results. Defaults to `0.1` (10%).

---

Now the **list** handler:

```rust
// src/routes/recipes.rs (lines 91-107, excerpt)
/// GET /api/recipes — list user's recipes
pub async fn list_recipes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let recipes = RecipeService::list_user(&state.db, user_id).await?;
    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: Some(recipes),
        message: None,
    }))
}
```

### Breakdown

- **`auth: AuthUser`** — the same JWT extractor from Part 8. If the user isn't logged in, `user_id` is `None` and we return `401 Unauthorized`.
- **`RecipeService::list_user`** — lives in `src/services/recipes.rs` (lines 62–73). It's one SQL query:
  ```sql
  SELECT * FROM user_recipes WHERE user_id = $1 ORDER BY updated_at DESC
  ```
- **`RecipeResponse`** — the response shape. It always has `err: 0` on success. When listing, `recipes` is populated and `recipe` (singular) is `None`. This dual-shape response is a FicHub convention — the same struct is reused for single-recipe and multi-recipe responses.

⚠️ **Watch Out** — `RecipeResponse` reuses one struct for both list and single-item calls. The frontend must check which field is `Some(...)` before rendering. It's easy to write `response.recipe.unwrap()` and panic when the handler actually put data in `response.recipes`.

---

Now the **create** handler:

```rust
// src/routes/recipes.rs (lines 109-144, excerpt)
/// POST /api/recipes — create recipe
pub async fn create_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRecipeRequest>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("Recipe name is required".into()));
    }

    let id = RecipeService::create(
        &state.db,
        user_id,
        &req.name,
        req.description.as_deref(),
        req.blend,
        req.filters,
        req.boost,
    )
    .await?;

    // Re-fetch and return the created recipe
    let recipes = RecipeService::list_user(&state.db, user_id).await?;
    let recipe = recipes.into_iter().find(|r| r.id == id);

    Ok(Json(RecipeResponse {
        err: 0,
        recipe,
        recipes: None,
        message: Some(format!("Recipe created with id {id}")),
    }))
}
```

### Breakdown

- **Name validation** — empty/whitespace names get a `400 Bad Request`.
- **`RecipeService::create`** — inserts a row into `user_recipes` and returns the new `id` (see `src/services/recipes.rs` lines 34–60):
  ```sql
  INSERT INTO user_recipes (user_id, name, description, blend, filters, boost)
  VALUES ($1, $2, $3, $4, $5, $6)
  RETURNING id
  ```
  Note that `curator_prior` is **not** passed to `create()` here — it's set by the database column default (`0.1`, matching the Rust default). The `is_active`, `is_public`, and `installs` columns also default (`false`, `false`, `0`) via the schema.
- **Re-fetch and return** — after creating, the handler re-queries the user's recipes and finds the new one by `id`. That means the response includes full DB-set defaults (`is_active: false`, `is_public: false`, `installs: 0`, timestamps).

---

### Check ✅

- The `/api/recipes` routes are registered in `src/server.rs` (lines 282–290):
  ```rust
  // src/server.rs
  .route("/api/recipes", get(crate::routes::recipes::list_recipes))
  .route("/api/recipes", axum::routing::post(crate::routes::recipes::create_recipe))
  .route("/api/recipes/active", get(crate::routes::recipes::get_active_recipe))
  .route("/api/recipes/gallery", get(crate::routes::recipes::browse_gallery))
  .route("/api/recipes/{id}", axum::routing::put(crate::routes::recipes::update_recipe))
  .route("/api/recipes/{id}", axum::routing::delete(crate::routes::recipes::delete_recipe))
  .route("/api/recipes/{id}/activate", axum::routing::post(crate::routes::recipes::activate_recipe))
  .route("/api/recipes/{id}/install", axum::routing::post(crate::routes::recipes::install_recipe))
  .route("/api/recipes/{id}/publish", axum::routing::post(crate::routes::recipes::publish_recipe))
  ```
- Both `list` and `create` require authentication via the `AuthUser` extractor.
- Both return the `RecipeResponse` struct with `err: 0` on success.

### Troubleshooting 🔧

| Problem | Cause | Fix |
|---|---|---|
| `401 Unauthorized: Login required` | No Bearer token in the request | Send `Authorization: Bearer ***`** |
| `Recipe name is required` | `name` is empty or whitespace | Send a non-empty `name` in the JSON body |
| `Failed to list recipes` | Database error | Verify the `user_recipes` table exists; run `sqlx prepare --check` |

---

## 33.2 Backend: `PUT /api/recipes/:id`, `DELETE`, `activate`, and `publish`

Once a recipe exists you need to edit it, switch it on, share it, and remove it. The `update_recipe` handler (lines 146–183) verifies the recipe belongs to the caller before touching it:

```rust
// src/routes/recipes.rs (lines 146-183, excerpt)
/// PUT /api/recipes/:id — update recipe
pub async fn update_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateRecipeRequest>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    // Extract description strings before passing to service to avoid borrow issues
    let desc_opt = match req.description {
        Some(Some(s)) => Some(s),
        Some(None) => Some(String::new()),
        None => None,
    };
    RecipeService::update(&state.db, user_id, id, req.name.as_deref(),
        desc_opt.as_deref().map(Some), req.blend, req.filters, req.boost, req.curator_prior)
        .await?;
    Ok(Json(RecipeResponse {
        err: 0, recipe: None, recipes: None,
        message: Some("Recipe updated".into()),
    }))
}
```

### Breakdown — partial updates

`UpdateRecipeRequest` (lines 49–57) uses `Option` on every field so the frontend can send **partial updates**:

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateRecipeRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub blend: Option<Value>,
    pub filters: Option<Value>,
    pub boost: Option<Value>,
    pub curator_prior: Option<f32>,
}
```

The tricky one is `description: Option<Option<String>>` — the triple-state Rust idiom for "field not provided / set to null / set to a value":

- `None` → don't change the description
- `Some(None)` → set description to NULL (clear it)
- `Some(Some("text"))` → set description to `"text"`

The service in `src/services/recipes.rs` (lines 122–162) does the same ownership check twice — once explicitly, then again inside the `UPDATE ... WHERE id = $1 AND user_id = $2` clause. Belt and suspenders.

⚠️ **Watch Out** — passing `Option<Value>` (where `Value` is `serde_json::Value`) to SQLx needs a `&bind`. If you hand it an owned `Value` the compiler will ask you to borrow — the service does `.bind(&blend)` / `.bind(&filters)` / `.bind(&boost)` for exactly this reason.

---

**Activate** (`activate_recipe`, lines 205–223) flips a recipe to `is_active = true`. The service (`RecipeService::set_active`, `src/services/recipes.rs` lines 101–115) first sets every recipe for that user to `is_active = false`, *then* flips the target on — guaranteeing there's only ever one active recipe per user. The rec engine reads the single active row.

**Publish** (`publish_recipe`, lines 282–314) flips `is_public`. This is the gateway to the gallery, so it's **trust-gated**:

```rust
// src/routes/recipes.rs (lines 293-298, excerpt)
// Publishing shares the recipe with the whole site, so it sits above the
// bare-write tier (trust-sandboxed accounts may still draft privately).
if req.is_public {
    trust::assert_min_trust(
        &state.db,
        Some(user_id),
        trust::PUBLISH_MIN_TRUST,
        "Publishing recipes",
    )
    .await?;
}
```

The `PUBLISH_MIN_TRUST` constant lives in `src/services/trust.rs` (line 39): `pub const PUBLISH_MIN_TRUST: i16 = 2;`. Any logged-in user can *draft* privately; only Trust Level ≥ 2 (Regular) can push to the public gallery. We'll unpack the trust ladder in Part 19.

---

## 33.3 Backend: Gallery, Install, and the Active Recipe's Strategy Weights

Three routes connect a recipe from "saved" to "live in the feed":

- `GET /api/recipes/gallery` (`browse_gallery`, line 243) — lists public recipes ordered by `installs DESC, updated_at DESC`. This is what powers the **Try It Yourself** picker in the frontend.
- `POST /api/recipes/:id/install` (`install_recipe`, line 258) — copies a public recipe into your own `user_recipes` row (same `blend`/`filters`/`boost`/`curator_prior`), then bumps the source's `installs` by one. Idempotent-ish: installing twice makes two copies, so the frontend disables the button after install.
- `GET /api/recipes/active` (`get_active_recipe`, line 225) — the frontend calls this on startup to learn "which recipe wins today."

⚠️ **Watch Out** — installing is a *copy*, not a reference. Edits you make to your installed copy don't touch the public original, and your changes only affect *your* feed. That's the whole point: recipes are data rows, trivially shareable and impossible to abuse.

The active recipe's weights get handed to the recommendation engine in `RecipeService::to_strategy_weights` (`src/services/recipes.rs`, lines 244–276):

```rust
// src/services/recipes.rs (excerpt)
/// Convert a recipe's `blend` JSON into a strategy-weights string
/// compatible with the rec engine (e.g. "cooccur:0.3,tag_graph:0.4").
/// Falls back to `config::Config::rec_strategies` default when blend
/// is empty or has no recognized strategy names.
pub fn to_strategy_weights(recipe: &Recipe) -> String {
    if let Some(obj) = recipe.blend.as_object() {
        if !obj.is_empty() {
            let parts: Vec<String> = obj
                .iter()
                .filter(|(_, v)| v.as_f64().map_or(false, |w| w > 0.0))
                .map(|(k, v)| {
                    let w = v.as_f64().unwrap_or(1.0);
                    format!("{k}:{w}")
                })
                .collect();
            if !parts.is_empty() {
                return parts.join(",");
            }
        }
    }
    // Fallback: return empty string so caller uses default
    String::new()
}
```

💡 **Key Concept — active recipe overrides the rec engine per-request.** When the rec engine builds a list for you, it asks the recipes service "what's active for this user," calls `to_strategy_weights`, and if the string is non-empty it uses that instead of the site-wide `REC_STRATEGIES` defaults. An empty string (default recipe or no active recipe) means "fall back to the global default blend." This is the per-request override mentioned in the v3 spec — and it's the reason switching your recipe instantly changes what shows up.

---

## 33.4 Backend: The Extension Marketplace (`POST /api/extensions`, `GET /api/extensions`, `install`, `rate`, `remix`)

Recipes are one flavor of a more general idea. The **Plugin Manifest** (P7 in the spec) is a single `extensions` table that holds every shareable object — recipes, themes, layouts, views, saved-searches, and profile widgets. The `/api/extensions` routes (`src/routes/extensions.rs`) expose that table as a gallery. The router is mounted in `src/server.rs` (line 816–817):

```rust
// src/server.rs (lines 816-817)
.route_service("/api/extensions", crate::routes::extensions::router(state.clone()))
```

The module header (lines 1–11) spells out the whole contract:

```rust
// src/routes/extensions.rs (lines 1-11)
//! Extension marketplace API.
//!
//! * `POST /api/extensions`            — publish (or, with `?upsert=1`, update)
//! * `GET  /api/extensions`            — gallery (kind=, q=, limit=)
//! * `GET  /api/extensions/kinds`      — supported kinds
//! * `GET  /api/extensions/{id}`       — fetch one (private owner view if owned)
//! * `POST /api/extensions/{id}/install`
//! * `POST /api/extensions/{id}/rate`  — `{ rating: 1..5 }`
//! * `POST /api/extensions/{id}/remix` — `{ slug, name }`
//!
//! Publishing is trust-gated (TL2+, see `services::trust::PUBLISH_MIN_TRUST`).
```

The seven kinds are defined in `src/services/extensions.rs` (line 13): `["theme", "skin", "recipe", "layout", "view", "saved_search", "profile"]`.

⚠️ **Watch Out** — publishing is trust-gated but **installing/rating/remixing is not**. Any logged-in user can install or remix; only TL2+ can publish to the public gallery. That keeps the barrier to participation low while protecting the shared space.

### Publish

`POST /api/extensions` (lines 61–75) calls `trust::assert_min_trust(... PUBLISH_MIN_TRUST ...)` before delegating to `extensions::publish`. With `?upsert=1` it updates the caller's own existing row (same `kind` + `slug`); without it, a duplicate slug errors and nudges the user toward **remix** instead. The table shipped in **migration 014 (`014_extensions.sql`)**, committed alongside the marketplace in **commit `63e6c66`** (2026-08-27).

### Gallery

`GET /api/extensions?kind=recipe&q=...&limit=48` (lines 51–59) returns the `extensions` rows filtered by kind and a case-insensitive slug/name search, clamped to 100. Each row is an `ExtensionRow` (lines 21–39): `id, kind, slug, name, description, author_id, version, tier, payload, meta, is_public, is_verified, installs, rating, created_at, updated_at`. The **`rating`** column holds the running average of 1–5 star votes; **`installs`** is incremented by the install route.

### Install (one-click, idempotent)

`POST /api/extensions/{id}/install` (lines 100–111) calls `extensions::install`, which is idempotent: installing twice is a no-op (the service checks `INSERT ... ON CONFLICT DO NOTHING` on a `(user_id, extension_id)` uniqueness constraint, then bumps `installs` by one only on first install).

### Rate (1–5 stars, running average)

`POST /api/extensions/{id}/rate` (lines 113–125) accepts `{ rating: 1..5 }` and returns the new average. Ratings live in `extension_ratings` (one row per user+extension); the `rating` column on `extensions` is the running mean, updated in the same transaction. The gallery orders by `rating` then `installs` by default.

### Remix (copy public → private draft)

`POST /api/extensions/{id}/remix` (lines 127–139) copies any public extension into the caller's private drafts — same `payload`/`meta`, a caller-supplied `slug` + `name`, `is_public = false`. Great for forking a theme and then editing it.

---

## 33.5 Theme Design Tokens (P8)

Themes are data, not code. They're JSON blobs of CSS custom-property overrides stored as a `theme`-kind extension in the same `extensions` table. The 21 design tokens ship live in `frontend/src/lib/themes/` and look like this (from `docs/v3-customization-spec.md` section 11.4):

```css
:root {
  --fh-accent: #7c5cff;  --fh-bg: #0f1117;  --fh-surface: #171a23;
  --fh-text: #e8eaf0;    --fh-muted: #8a90a2; --fh-radius: 12px;
  --fh-font: "Inter";    --fh-density: 1;
}
```

💡 **Key Concept — themes are token values only.** A theme is a JSON object whose keys are token names (`--fh-accent`, `--fh-bg`, etc.) and whose values are the scalars/colors you want. Because the app only ever reads token *values* through a fixed allowlist and never accepts raw CSS strings, there's no injection surface. Themes can be imported/exported/shared as easily as a recipe.

Five built-in presets ship baked in: **Default Dark, Default Light, High Contrast, Sepia, Dyslexia-Friendly.** A reader theme (font, line-height, max-width) is tracked separately from the app theme. Changes in the Theme editor update live — flip a token and the whole UI repaints in real time via the preview bridge in `frontend/src/lib/components/ThemeEditor.svelte`.

⚠️ **Watch Out** — don't try to store arbitrary CSS in a theme blob. The backend validates each token against the schema's allowlist (color hex, font-family token, numeric density, etc.) and rejects anything else. If you need a new token, add it to the preset schema first.

---

## 33.6 Frontend: Recipe Builder UI + The `/marketplace` Gallery

The frontend lives two routes:

- `/settings/recipes` — the **recipe builder**. A form with sliders for `blend` (cooccur / tag_graph / embeddings), inputs for `filters` + `boost`, a numeric field for `curator_prior`, and big `Activate` / `Publish` / `Delete` buttons. On load it `GET /api/recipes/active` to highlight which recipe is live.
- `/marketplace` — the **extension gallery**. Lists `extensions` rows with category tabs (Themes · Skins · Recipes · Layouts · Views), per-row install/star/remix buttons, and a search box wired to `?q=`. Clicking install fires `POST /api/extensions/{id}/install` and immediately greys out the button (idempotent, see above).

🧪 **Try It Yourself** — spin up the dev server (`npm run dev`), open `/settings/recipes`, and:

1. Click **New Recipe**.
2. Drag the "embeddings" slider to `0.6`, "cooccur" to `0.4`, leave "tag_graph" at `0.0`.
3. Click **Activate**.
4. Open a new tab to `/recommendations`. Notice how the ordering of fics shifts — that's your recipe's weights being read by the rec engine on every request.

🧪 **Try It Yourself** — remix a theme:

1. Open `/marketplace`, click the **Themes** tab.
2. Find "Default Dark", click **⋮ → Remix**.
3. Edit the remix (it's a private draft now), change `--fh-accent` to `#ff6b6b`.
4. Click **Publish** — but first check your trust level! Only TL2+ can publish. If you're TL0/TL1 the button is disabled and the UI tells you to keep reading and participating.

⚠️ **Watch Out** — publishing is a network call gated by an *async* trust check (`trust::assert_min_trust`). It's possible for the page to show your old trust level until you refresh. If your "Publish" 403s unexpectedly, hit the reload button next to your username badge — it re-fetches `trust_level` from `GET /api/auth/me`.

---

## 33.7 How Recipes Reach the Reader (the per-request override)

The last piece of the puzzle: how does an active recipe actually change the feed? In `src/recommender/routes.rs` (around line 430) the recommendation handler does roughly:

```rust
// src/recommender/routes.rs (conceptual)
let recipe = RecipeService::get_active(&state.db, user_id).await?;
let weights = recipe
    .as_ref()
    .map(|r| RecipeService::to_strategy_weights(r))
    .unwrap_or_default(); // "" → use site-wide REC_STRATEGIES defaults
let fic_ids = recommend(&state, user_id, &weights, filters, boost).await?;
```

When `weights` is the empty string, `recommend()` falls back to the global `config::Config::rec_strategies` blend. When it's non-empty, the user's recipe *overrides the rec engine per-request*. That override is the moment Part 21's whole story converges: a user's saved recipe becomes their live feed, recomputed every time they open Recommendations.

---

## 33.8 What Ships vs. What's Still Cooking (P9 / deferred)

The v3 spec (`docs/v3-customization-spec.md`, section 11.6) is explicit about what's deferred to P9:

| Feature | Status |
|---|---|
| Recipe Builder (P6) | ✅ DONE |
| Plugin Manifest / Extension Marketplace (P7) | ✅ DONE — commit `63e6c66`, migration 014 |
| Theme Design Tokens (P8) | ✅ DONE |
| Multi-tenant sidecars | 🚧 deferred P9 |
| WASM sandbox for Scripts (P3 tier) | 🚧 deferred P9 |
| Full marketplace with ratings/reviews + engagement leaderboard | 🚧 deferred P9 |
| Nav/layout editor (drag-drop reorder) + layout presets (mobile vs desktop) | 🚧 deferred P9 |
| Community recommender leaderboard (Elo from `rec_impressions`) | 🚧 deferred P9 |
| Recipe → precomputed promotion | 🚧 deferred P9 |
| XP from customization reputation | 🚧 deferred P9 |
| Seasonal event themes | 🚧 deferred P9 |
| Full marketplace curator-review pipeline + verified badges | 🚧 deferred P9 |

The shipped v3.1 platform is Config (P1) + Recipe (P2) + Plugin Manifest (P7) + Theme Tokens (P8). Scripts (P3) and Services (P4) wait for the WASM sandbox.

---

## 33.9 Exercises

**🧪 Try It Yourself — Write a recipe by hand.**

Using `curl`, POST a recipe directly to the API (you'll need a bearer token from `POST /api/auth/login`):

```bash
curl -X POST https://fichub.net/api/recipes \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Short & Sweet",
    "description": "Only fics under 20k words",
    "blend": {"cooccur": 0.5, "embeddings": 0.5},
    "filters": {"max_words": 20000},
    "boost": {},
    "curator_prior": 0.05
  }'
```

Then activate it:

```bash
curl -X POST "https://fichub.net/api/recipes/$(jq .recipe.id ~/.tmp/new_recipe.json)/activate" \
  -H "Authorization: Bearer <token>"
```

Verify the override: call `GET /api/recommendations/suggest` twice — once with the recipe active, once after `DELETE`ing it. Do the orderings differ? (They should!)

⚠️ **Watch Out** — the gallery and the active-recipe endpoints are separate. Activating a recipe does **not** automatically publish it. Activate = "use this for me"; publish = "let others install this."

---

## 33.10 Recap

- A **recipe** is one row in `user_recipes`; it's a JSON `blend` of strategy weights + `filters` + `boost` + a `curator_prior` float.
- **One** recipe per user is active (`is_active`). The rec engine reads it and calls `RecipeService::to_strategy_weights` to turn the `blend` JSON into a `cooccur:0.3,tag_graph:0.4` string — the per-request override.
- **Publishing** is trust-gated at TL2+ (`trust::PUBLISH_MIN_TRUST` in `src/services/trust.rs`). TL0/1 can still draft privately.
- Recipes are one kind in the **Plugin Manifest** (`extensions` table, migration 014, commit `63e6c66`). The `/api/extensions` router adds publish/gallery/install/rate(1–5)/remix, all gated appropriately.
- **Theme Design Tokens** are JSON blobs of CSS custom properties, validated against a token allowlist — safe by construction, with 5 presets and a live preview.
- The `/marketplace` frontend page and `/settings/recipes` builder page are the two surfaces users touch.
- Full detail: `docs/v3-customization-spec.md` (the authoritative spec) + `src/routes/recipes.rs`, `src/services/recipes.rs`, `src/routes/extensions.rs`, `src/services/extensions.rs`.

> *Next up: Part 19 — Moderation and Admin. We'll see how the trust ladder gates publishing, how every moderation action is recorded in a transparent modlog, and how curators vote on proposals through the consensus hub.*
