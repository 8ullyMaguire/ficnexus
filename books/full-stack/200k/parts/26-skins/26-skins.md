# Part 26 — Skins and Customization

> In this chapter you will learn how FicHub lets users change how the site looks — colors, fonts, and layout. We'll build a skin browser, a theme picker, and a layout editor, all while learning how a small Rust backend serves user preferences and a SvelteKit frontend paints them on screen.

---

## Overview

FicHub has two faces: a **modern** mode (the default — slick cards, rounded corners, drop shadows) and an **archive** mode (the AO3/OTW look — serif fonts, bordered tables, that familiar early-2000s feel that fanfiction readers recognize at a glance). Users pick a preset skin — "Classic AO3", "Dark Mode", "Sepia Reader" — and the choice sticks across sessions.

The skin system has three parts:

1. **Backend**: `GET/POST /api/skins` — a list of available skins, and a `PUT /api/me/theme` to save a user's choice.
2. **Backend**: `GET /api/nav` and `GET /api/widgets` — configuration that tells the frontend which navigation items to show and which widgets are active.
3. **Frontend**: a skin browser page, a theme picker dropdown, and a layout editor where you can toggle widgets on or off.

By the end of this chapter you will have built all three layers, from the database table up to the user-facing UI.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database foundations, fetching a single work, searching works, user accounts).
- Your FicHub backend is running on `:8000` and the frontend is serving from `/var/www/fichub`.
- You have the `psql` CLI and `curl` handy.
- For the "Try It Yourself" exercises you need a running browser open to your local FicHub.

---

## Chapter 26.1 — Backend: `GET/POST /api/skins`

### Goal

Create an HTTP endpoint that returns a list of available skins and lets an authenticated user save their favorite.

### Actions

#### 1. The `Skin` struct

In the real codebase, skins are defined in `src/routes/skins.rs`. Each skin is a simple struct with an ID, a display name, and a set of CSS variables that the frontend applies:

```rust
// src/routes/skins.rs (simplified)
use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::error::AppError;
use crate::server::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub preview: String,          // CSS variables as a JSON string
    pub is_default: bool,
    pub category: String,         // "archive", "modern", "dark"
}
```

#### 2. The `GET /api/skins` handler

The handler returns a hardcoded list — skins are baked into the Rust binary, not stored in the database. This keeps the endpoint fast and avoids a DB round-trip on every page load:

```rust
/// `GET /api/skins` — list all available skins.
pub async fn list_skins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let skins = all_skins();
    Ok(Json(json!({ "err": 0, "skins": skins })))
}

fn all_skins() -> Vec<Skin> {
    vec![
        Skin {
            id: "classic-ao3".to_string(),
            name: "Classic AO3".to_string(),
            description: "AO3-style archive look with serif fonts and bordered tables".to_string(),
            preview: r#"{"--font-family-sans": "Georgia, serif", "--color-link": "#990000", "--bg": "#ffffff"}"#.to_string(),
            is_default: true,
            category: "archive".to_string(),
        },
        Skin {
            id: "dark-mode".to_string(),
            name: "Dark Mode".to_string(),
            description: "Dark background with light text — easier on the eyes at night".to_string(),
            preview: r#"{"--font-family-sans": "Inter, sans-serif", "--color-link": "#4da6ff", "--bg": "#121212"}"#.to_string(),
            is_default: false,
            category: "dark".to_string(),
        },
        // ... more skins in the real list
    ]
}
```

#### 3. The `POST /api/skins` handler

Authenticated users can mark a skin as a favorite. The list of favorites is stored per-user in Redis (not the database — it's a lightweight preference that doesn't need persistence guarantees):

```rust
/// `POST /api/skins/favorite` — save a skin as a user favorite.
pub async fn favorite_skin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<FavoriteSkinRequest>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_auth(&state, &headers)?;
    let key = format!("fichub:user:{}:favskins", user_id);

    // Use a Redis SET so each skin appears only once.
    let mut conn = state.redis.clone();
    redis::cmd("SADD")
        .arg(&key)
        .arg(&body.skin_id)
        .query_async::<i64>(&mut conn)
        .await?;

    Ok(Json(json!({ "err": 0, "skin_id": body.skin_id })))
}

#[derive(Debug, Deserialize)]
pub struct FavoriteSkinRequest {
    pub skin_id: String,
}

/// Extract and validate the bearer token, returning the user id.
fn require_auth(state: &Arc<AppState>, headers: &HeaderMap) -> Result<i32, AppError> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing token".into()))?;
    // In the real code auth.rs validates the JWT here.
    crate::auth::verify_token(state, token)
        .map(|claims| claims.user_id)
}
```

#### 4. Register the routes

In `src/server.rs`, the skins routes are registered alongside the other API routes:

```rust
// src/server.rs (simplified snippet from the real registration)
let skins_router = Router::new()
    .route("/api/skins", get(list_skins))
    .route("/api/skins/favorite", post(favorite_skin))
    .route("/api/me/skins/favorites", get(list_favorite_skins));
```

#### 5. Try It Yourself

Start your server, then:

```bash
# Get the list of skins
curl http://localhost:8000/api/skins | jq
```

You should see a JSON object with `"err": 0` and an array of skin objects under `"skins"`.

#### 6. Check

- ✅ `GET /api/skins` returns HTTP 200 with a JSON array of skins.
- ✅ Each skin has `id`, `name`, `description`, `preview`, `is_default`, and `category`.
- ✅ `POST /api/skins/favorite` with a valid JWT returns `"err": 0` and stores the `skin_id` in Redis.

#### 7. Troubleshooting

- **All skins look the same**: The `preview` field is a CSS-variables JSON string. Make sure your frontend parses it and applies each variable to `document.documentElement.style`.
- **Favorites return an error**: Check that you're sending the `Authorization: Bearer <jwt>` header. The token expires in 30 days.

#### 8. What you have now

A backend endpoint that serves a list of skins and lets authenticated users save favorites to Redis.

---

## Chapter 26.2 — Backend: `GET/PUT /api/me/theme`

### Goal

Let each user save their preferred skin, theme mode (light/dark/sephia), and font size. The preference is stored in the database and returned on every page load.

### Actions

#### 1. The `UserTheme` struct

The theme preference is backed by the `user_prefs` table in PostgreSQL:

```rust
// src/routes/customization.rs (simplified)
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserTheme {
    pub user_id: i32,
    pub skin_id: String,
    pub theme_mode: String,    // "light" | "dark" | "sepia"
    pub font_size: i32,        // pixels — 14, 16, 18, 20
    pub custom_css: Option<String>,
}
```

#### 2. The `GET /api/me/theme` handler

Returns the logged-in user's current theme, or the system default if they haven't saved one:

```rust
/// `GET /api/me/theme` — fetch the current user's theme preference.
pub async fn get_user_theme(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let user_id = require_auth(&state, &headers)?;

    let prefs = sqlx::query_as!(
        UserTheme,
        r#"
        SELECT user_id, skin_id, theme_mode, font_size, custom_css
        FROM user_prefs
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    match prefs {
        Some(p) => Ok(Json(json!({
            "err": 0,
            "skin_id": p.skin_id,
            "theme_mode": p.theme_mode,
            "font_size": p.font_size,
            "custom_css": p.custom_css,
        }))),
        None => Ok(Json(json!({
            "err": 0,
            "skin_id": "classic-ao3",
            "theme_mode": "light",
            "font_size": 16,
            "custom_css": null,
        }))),
    }
}
```

#### 3. The `PUT /api/me/theme` handler

Upserts the user's theme preference:

```rust
/// `PUT /api/me/theme` — save the current user's theme preference.
pub async fn set_user_theme(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SetThemeRequest>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_auth(&state, &headers)?;

    sqlx::query!(
        r#"
        INSERT INTO user_prefs (user_id, skin_id, theme_mode, font_size, custom_css, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (user_id) DO UPDATE SET
            skin_id = EXCLUDED.skin_id,
            theme_mode = EXCLUDED.theme_mode,
            font_size = EXCLUDED.font_size,
            custom_css = EXCLUDED.custom_css
        "#,
        user_id,
        body.skin_id,
        body.theme_mode,
        body.font_size,
        body.custom_css,
    )
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "err": 0 })))
}

#[derive(Debug, Deserialize)]
pub struct SetThemeRequest {
    pub skin_id: String,
    pub theme_mode: String,
    pub font_size: i32,
    pub custom_css: Option<String>,
}
```

> **⚠️ Watch Out**: The `user_prefs` table is created in the consolidated `001_initial.sql` migration. Do NOT create a separate migration file for this — the project's convention is to keep all schema in one file. Always run `ALTER OWNER TO fichub` after creating tables in psql.

#### 4. The SQL table

In `references/migrations/001_initial.sql`:

```sql
CREATE TABLE user_prefs (
    user_id      INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    skin_id      TEXT NOT NULL DEFAULT 'classic-ao3',
    theme_mode   TEXT NOT NULL DEFAULT 'light' CHECK (theme_mode IN ('light', 'dark', 'sepia')),
    font_size    SMALLINT NOT NULL DEFAULT 16 CHECK (font_size BETWEEN 8 AND 72),
    custom_css   TEXT,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER OWNER TO fichub;
```

#### 5. Try It Yourself

After applying the migration:

```bash
# Save a dark theme
curl -X PUT http://localhost:8000/api/me/theme \
  -H "Authorization: Bearer <your-jwt>" \
  -H "Content-Type: application/json" \
  -d '{"skin_id":"dark-mode","theme_mode":"dark","font_size":16,"custom_css":null}'
```

Then:

```bash
curl http://localhost:8000/api/me/theme \
  -H "Authorization: Bearer <your-jwt>"
```

#### 6. Check

- ✅ `PUT /api/me/theme` with a valid JWT inserts or updates the `user_prefs` row.
- ✅ `GET /api/me/theme` returns the saved values (or defaults if no row exists).
- ✅ The `theme_mode` CHECK constraint rejects invalid values.

#### 7. Troubleshooting

- **Constraint violation on font_size**: Check that the value is between 8 and 72.
- **`user_prefs` table doesn't exist**: Make sure you ran the migration from `001_initial.sql`.

#### 8. What you have now

A complete theme storage system: the backend serves skins, stores user theme preferences in PostgreSQL, and returns them on request.

---

## Chapter 26.3 — Backend: `GET /api/nav`

### Goal

Give the frontend a navigation configuration — which links appear in the top bar and sidebar — without hardcoding them in the SvelteKit layout. This lets skins rearrange navigation.

### Actions

#### 1. The nav config struct

```rust
// src/routes/customization.rs (simplified)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavConfig {
    pub id: String,
    pub label: String,
    pub href: String,
    pub icon: String,
    #[serde(rename = "children")]
    pub sub_items: Vec<NavConfig>,
}
```

#### 2. The handler

```rust
/// `GET /api/nav` — return the site-wide navigation tree.
pub async fn get_nav_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let nav = vec![
        NavConfig {
            id: "home".to_string(),
            label: "Home".to_string(),
            href: "/".to_string(),
            icon: "home".to_string(),
            sub_items: vec![],
        },
        NavConfig {
            id: "works".to_string(),
            label: "Works".to_string(),
            href: "/works".to_string(),
            icon: "book".to_string(),
            sub_items: vec![
                NavConfig {
                    id: "search".to_string(),
                    label: "Search".to_string(),
                    href: "/search".to_string(),
                    icon: "search".to_string(),
                    sub_items: vec![],
                },
            ],
        },
        // ... forum, fandoms, dashboard, etc.
    ];
    Ok(Json(json!({ "err": 0, "nav": nav })))
}
```

#### 3. Register the route

```rust
// src/server.rs
.route("/api/nav", get(get_nav_config))
```

### Try It Yourself

```bash
curl http://localhost:8000/api/nav | jq '.nav[] | .label'
```

You should see `Home`, `Works`, `Search`, and more — the labels that appear in the top navigation bar.

### What you have now

A backend endpoint that returns navigation configuration as JSON. The frontend can now loop over the array to build menus dynamically, and skins can rearrange the order.

---

## Chapter 26.4 — Backend: `GET /api/widgets`

### Goal

Return the list of available widgets (notification bell, search bar, reading stats card) so the frontend knows which ones to offer in the layout editor.

### Actions

#### 1. Widget type

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Widget {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_enabled: bool,
    pub position: String,  // "header" | "sidebar" | "footer" | "dashboard"
}
```

#### 2. Handler

```rust
/// `GET /api/widgets` — list all available widgets.
pub async fn get_widgets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let widgets = vec![
        Widget {
            id: "notification-bell".to_string(),
            name: "Notifications".to_string(),
            description: "Bell icon with unread-count badge".to_string(),
            default_enabled: true,
            position: "header".to_string(),
        },
        Widget {
            id: "reading-stats".to_string(),
            name: "Reading Stats".to_string(),
            description: "Words read, login streak, current reading list".to_string(),
            default_enabled: true,
            position: "dashboard".to_string(),
        },
        Widget {
            id: "quick-search".to_string(),
            name: "Quick Search".to_string(),
            description: "Compact search bar with autocomplete".to_string(),
            default_enabled: false,
            position: "header".to_string(),
        },
    ];
    Ok(Json(json!({ "err": 0, "widgets": widgets })))
}
```

### Try It Yourself

```bash
curl http://localhost:8000/api/widgets | jq '.widgets[].name'
```

### What you have now

The backend now serves skins, theme preferences, navigation config, and widget inventory — all the data the frontend needs to render a customizable UI.

---

## Chapter 26.5 — Frontend: Skin Browser, Theme Picker, Layout Editor

### Goal

Build three SvelteKit pages/components that consume the backend endpoints: a skin browser, a theme picker in the settings page, and a layout editor.

### Actions

#### 1. The skin browser page

`src/routes/skins/+page.svelte` — fetches `/api/skins` and displays each skin as a card with a preview. The user clicks "Apply" to call `PUT /api/me/theme`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte.ts';
  import { getPref } from '$lib/prefs';
  import { browser } from '$app/environment';

  interface Skin {
    id: string;
    name: string;
    description: string;
    preview: string;   // JSON string of CSS variables
    is_default: boolean;
    category: string;
  }

  let skins: Skin[] = $state([]);
  let currentSkin: string | null = $state(null);
  let loading = $state(true);

  async function loadSkins() {
    loading = true;
    try {
      const res = await fetch('/api/skins');
      const data = await res.json();
      if (data.err === 0) {
        skins = data.skins || [];
      }
    } catch {
      // ignore
    } finally {
      loading = false;
    }
  }

  async function applySkin(skinId: string) {
    if (!auth.isLoggedIn) {
      // Apply locally even when logged out
      applySkinLocally(skinId);
      return;
    }
    const previews = skins.find((s) => s.id === skinId)?.preview;
    if (previews) {
      const vars = JSON.parse(previews);
      for (const [k, v] of Object.entries(vars)) {
        document.documentElement.style.setProperty(k, String(v));
      }
    }
    currentSkin = skinId;

    // Persist to backend
    const theme = await fetch('/api/me/theme', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${auth.token}` },
      body: JSON.stringify({
        skin_id: skinId,
        theme_mode: skinId.includes('dark') ? 'dark' : 'light',
        font_size: 16,
        custom_css: null,
      }),
    });
    const result = await theme.json();
    if (result.err === 0) {
      alert('Skin applied!');
    }
  }

  function applySkinLocally(skinId: string) {
    const skin = skins.find((s) => s.id === skinId);
    if (skin) {
      const vars = JSON.parse(skin.preview);
      for (const [k, v] of Object.entries(vars)) {
        document.documentElement.style.setProperty(k, String(v));
      }
    }
    currentSkin = skinId;
  }

  onMount(() => {
    auth.init();
    loadSkins();
  });
</script>

<svelte:head><title>Skin Browser — FicHub</title></svelte:head>

{#if loading}
  <p class="muted">Loading skins…</p>
{:else}
  <div class="skins-grid">
    {#each skins as skin (skin.id)}
      <div class="skin-card" class:selected={currentSkin === skin.id}>
        <h3>{skin.name}</h3>
        <p class="desc">{skin.description}</p>
        <span class="tag">{skin.category}</span>
        <button on:click={() => applySkin(skin.id)}>
          {skin.is_default ? 'Default' : 'Apply'}
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .skins-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 1rem; }
  .skin-card { border: 1px solid var(--color-border, #ddd); border-radius: 8px; padding: 1rem; cursor: pointer; }
  .skin-card.selected { border-color: var(--color-link, #990000); box-shadow: 0 0 0 2px rgba(153,0,0,0.1); }
  .tag { font-size: 0.75rem; background: #f0f0f0; padding: 0.1rem 0.4rem; border-radius: 4px; }
  button { background: var(--color-link, #990000); color: white; border: none; padding: 0.3rem 0.8rem; border-radius: 4px; cursor: pointer; }
</style>
```

#### 2. The theme picker

In the settings page (`src/routes/settings/+page.svelte`), a dropdown shows the user's current theme and lets them switch. The `prefs.ts` store persists the choice locally so it survives page reloads even before the user logs in:

```svelte
<!-- Inside settings/+page.svelte -->
<div class="theme-picker">
  <label>Theme</label>
  <select bind:value={currentSkin} on:change={() => applySkin(currentSkin)}>
    <option value="classic-ao3">Classic AO3</option>
    <option value="dark-mode">Dark Mode</option>
    <option value="sepia">Sepia Reader</option>
  </select>
</div>
```

#### 3. The layout editor

`src/routes/settings/layout/+page.svelte` — fetches `/api/widgets` and shows checkboxes. The user toggles widgets on/off, and the choice is saved to `prefs.ts` (localStorage) for immediate effect, and optionally persisted to the server:

```svelte
<script lang="ts">
  import { getPref, setPref } from '$lib/prefs';
  import { auth } from '$lib/stores/auth.svelte.ts';

  interface Widget {
    id: string;
    name: string;
    description: string;
    default_enabled: boolean;
    position: string;
  }

  let widgets: Widget[] = $state([]);
  let enabled: Record<string, boolean> = $state({});

  async function loadWidgets() {
    const res = await fetch('/api/widgets');
    const data = await res.json();
    if (data.err === 0) {
      widgets = data.widgets || [];
      // Merge with saved preferences
      for (const w of widgets) {
        enabled[w.id] = getPref(`widget:${w.id}:enabled`) ?? w.default_enabled;
      }
    }
  }

  function toggle(id: string) {
    enabled[id] = !enabled[id];
    setPref(`widget:${id}:enabled`, enabled[id]);
    // In archive mode, notify the layout to re-render
    window.dispatchEvent(new CustomEvent('layout-change'));
  }

  loadWidgets();
</script>

<h1>Layout Editor</h1>
{#each widgets as w (w.id)}
  <label>
    <input type="checkbox" checked={enabled[w.id]} on:change={() => toggle(w.id)} />
    <span class="widget-name">{w.name}</span>
    <span class="widget-desc">{w.description}</span>
  </label>
{/each}
```

### Try It Yourself

1. Start the dev server (`npm run dev`) and open `http://localhost:5173/skins`.
2. Click "Apply" on the "Dark Mode" skin — the page should go dark immediately.
3. Visit `/settings/theme` — your saved choice should be pre-selected.
4. Visit `/settings/layout` — toggle "Quick Search" on and watch it appear in the header.

### Check

- ✅ The skin browser page loads and shows at least two skin cards.
- ✅ Clicking "Apply" changes the CSS variables on `document.documentElement` instantly.
- ✅ The theme picker dropdown preserves the selection across page reloads (via `localStorage`).
- ✅ The layout editor checkboxes persist via `getPref`/`setPref` in `prefs.ts`.

### What you have now

A fully working skin system: the backend serves skins, navigation config, and widgets; the frontend applies CSS variables, persists user choices locally and to the server, and supports both modern and archive mode UIs.

### On to the next part

In Part 21 we'll build [Reading Quests and Stats] — the gamification layer that tracks reading streaks and gives XP for completed works.
