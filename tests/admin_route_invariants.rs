//! Invariant: every `/api/admin/*` route is authorization-gated.
//!
//! # Why this exists
//!
//! `admin.rs` had 26 handlers gating on `user.trust_level < 5` while
//! `require_admin_tier` — which reads the `is_admin` flag — sat unused in the
//! same file, 30 lines below a comment explaining why the inline check was
//! wrong. 24 more of the same mistake lived in 10 other route files. None of
//! it was caught by a test, because no test looked.
//!
//! The failure mode is not a broken gate, it is a **missing** one: the next
//! person adds an admin route, writes no check, and it is publicly reachable.
//! 65 call sites of `require_admin_tier` do not make that less likely.
//!
//! # What it checks
//!
//! Parses `src/server.rs` for `.route("/api/admin…", method(handler))`,
//! resolves each handler symbol to its file, extracts the function body by
//! brace matching, and requires that the body contain an authorization gate.
//!
//! **Source-level, not behavioural.** This is a structural guard, and it is
//! honest about the limit: it proves a gate is *present*, not that it is
//! *correct*. `assert_admin_gated` will pass a handler that calls
//! `require_admin()` where it should call `require_admin_tier()`. The
//! behavioural test for that is `admin_api`.
//!
//! What it does catch, which no behavioural test can: a route added with no
//! gate at all, and a gate deleted from a route that had one.
//!
//! # Gate shapes recognised
//!
//! Ten distinct helpers and inline forms exist, all legitimate:
//!
//! | shape | example |
//! |---|---|
//! | `require_admin_tier` | `admin.rs` — the `is_admin` flag (canonical) |
//! | `require_admin` | `backfill.rs` (`auth.level < 10`), `forum_privileges.rs` |
//! | `require_mod` | `subsystems.rs`, `forum.rs` — trust >= 3, moderator |
//! | `require_curator` | `curator_content.rs`, `tags/curator.rs` — trust >= 5 |
//! | `require_forum_category_admin` / `require_forum_moderator` | delegate to `require_admin_at_level` |
//! | `require_trust_resolve` / `require_trust_queue` | `assert_staff_or_min_trust` |
//! | inline `trust_level` | — |
//! | inline `COALESCE(trust, 0)` SQL | `features.rs` — the only reader of `users.trust` |
//!
//! The `features.rs` case is why the matcher accepts SQL as well as calls: it
//! gates with `SELECT COALESCE(trust, 0) … if trust < 4`, which is neither a
//! `require_*` call nor a `trust_level` reference. A first cut of this test
//! that only matched `require_*|is_admin|trust_level` reported those three
//! routes as ungated.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// An authorization gate in a handler body: any `require_*` helper, an
/// `is_admin` reference, a `trust_level` check, or the `users.trust` SQL read
/// that `features.rs` uses.
fn has_gate(body: &str) -> bool {
    body.contains("require_")
        || body.contains("is_admin")
        || body.contains("trust_level")
        || body.contains("COALESCE(trust")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// `crate::routes::forum::admin_hide_post` -> `src/routes/forum.rs`
///
/// Returns `None` for symbols this test cannot resolve. Those are reported
/// rather than skipped, because an unresolvable handler means the check did
/// not run.
fn resolve_handler_file(symbol: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = symbol.split("::").collect();
    let rel = match parts.as_slice() {
        ["crate", "routes", module, ..] => format!("src/routes/{module}.rs"),
        ["routes", module, ..] => format!("src/routes/{module}.rs"),
        ["crate", module, ..] => format!("src/{module}.rs"),
        _ => return None,
    };
    let p = repo_root().join(rel);
    p.exists().then_some(p)
}

/// Extract a function body by brace matching from the opening brace after the
/// signature. Returns `None` if the signature is not found.
fn fn_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    // Anchor on a definition, not a call site: `fn name(` preceded by
    // start-of-line whitespace. A bare `find` can match an earlier
    // `…fn name(`-shaped string and return the wrong body.
    let needle = format!("fn {name}(");
    let mut from = 0usize;
    let idx = loop {
        let rel = src[from..].find(&needle)?;
        let at = from + rel;
        let anchored = at == 0
            || src[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_whitespace() || c == '{' || c == '}');
        if anchored {
            break at;
        }
        from = at + needle.len();
    };

    // Signature may span lines and contain generic brackets, so walk forward
    // to the first `{` that opens the body. `find` returns a byte offset
    // relative to `idx`, hence the addition.
    let open = src[idx..].find('{')? + idx;

    let bytes = src.as_bytes();
    let mut depth = 0i32;
    for i in open..bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    // +1 to include the closing brace. Slicing on byte indices
                    // is safe here because `{` and `}` are ASCII and cannot
                    // appear inside a UTF-8 continuation sequence.
                    return Some(&src[open..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// `/api/admin` routes and the handler symbol each is bound to.
///
/// Hand-rolled rather than a regex, to keep this test dependency-free. The
/// shape is fixed: a quoted path, then a method call whose last argument is a
/// path to a handler function.
fn admin_routes(server_src: &str) -> Vec<(String, String)> {
    const OPEN: &str = "\"/api/admin";
    let mut out = Vec::new();
    let mut cursor = 0usize;

    while let Some(rel) = server_src[cursor..].find(OPEN) {
        let at = cursor + rel;
        let after = &server_src[at..];

        // `after` starts at the opening quote, so skip it and find the
        // closing one.
        let Some(close_rel) = after[1..].find('"') else {
            break;
        };
        let path_len = close_rel + 2; // index just past the closing quote
        let path = after[1..path_len - 1].to_string();

        // From the closing quote, take the shortest span that reaches the
        // handler's closing paren: `, get(handler))` or
        // `, axum::routing::post(handler))`.
        let tail = &after[path_len..];
        let Some(paren) = tail.find('(') else { break };
        let Some(close_rel) = tail[paren..].find(')') else { break };
        let inside = tail[paren + 1..paren + close_rel].trim();

        // A handler is a `::`-separated identifier path. Anything else (a
        // closure, a method chain) is not what this test can resolve, and
        // resolving it would be a guess.
        let is_symbol = !inside.is_empty()
            && inside
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ':');
        if is_symbol {
            out.push((path, inside.to_string()));
        }

        cursor = at + path_len;
    }
    out
}

#[test]
fn every_admin_route_is_authorization_gated() {
    let server = repo_root().join("src/server.rs");
    let server_src = std::fs::read_to_string(&server)
        .unwrap_or_else(|e| panic!("read {}: {e}", server.display()));

    let routes = admin_routes(&server_src);
    assert!(
        routes.len() >= 70,
        "parsed only {} /api/admin routes — the parser regressed, expected ~77. \
         A silently-empty parse would make this test vacuously pass.",
        routes.len()
    );

    let mut ungated: Vec<String> = Vec::new();
    let mut unresolvable: Vec<String> = Vec::new();

    for (path, symbol) in &routes {
        let Some(file) = resolve_handler_file(symbol) else {
            unresolvable.push(format!("{path} -> {symbol} (cannot resolve to a file)"));
            continue;
        };
        let src = std::fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        let leaf = symbol.rsplit("::").next().unwrap_or(symbol);
        match fn_body(&src, leaf) {
            Some(body) if has_gate(body) => {}
            Some(_) => ungated.push(format!("{path} -> {symbol} (no gate in body)")),
            None => unresolvable.push(format!("{path} -> {symbol} (no fn body found)")),
        }
    }

    assert!(
        unresolvable.is_empty(),
        "{} /api/admin route(s) could not be checked, so this test is not \
         covering them. Either the symbol moved or the parser needs updating:\n  {}",
        unresolvable.len(),
        unresolvable.join("\n  ")
    );

    assert!(
        ungated.is_empty(),
        "{} /api/admin route(s) have no authorization gate. An admin route \
         with no gate is publicly reachable. Add `require_admin_tier(&user, \
         &state)?` (the `is_admin` flag — see docs/specs/admin-flag.md), or \
         the correct domain helper (`require_mod`, `require_curator`, \
         `require_forum_*`) if it is deliberately a lower tier:\n  {}",
        ungated.len(),
        ungated.join("\n  ")
    );
}

/// The gate helpers in use, so a rename or a new one is visible in test output
/// rather than only in a diff.
#[test]
fn admin_gate_helpers_are_known() {
    let server_src =
        std::fs::read_to_string(repo_root().join("src/server.rs")).expect("read server.rs");
    let mut helpers: BTreeSet<String> = BTreeSet::new();
    for (_, symbol) in admin_routes(&server_src) {
        let leaf = symbol.rsplit("::").next().unwrap_or(&symbol).to_string();
        helpers.insert(leaf);
    }
    // Sanity: the parse found a plausible number of distinct handlers, and the
    // canonical admin helper is among them.
    assert!(
        helpers.len() >= 40,
        "only {} distinct handlers across /api/admin routes",
        helpers.len()
    );
    // server.rs only references handlers, not the helpers, so the helper's
    // continued existence is checked where it is defined.
    let trust = std::fs::read_to_string(repo_root().join("src/services/trust.rs"))
        .expect("read src/services/trust.rs");
    assert!(
        trust.contains("pub fn require_admin_tier"),
        "the canonical admin gate helper is gone from src/services/trust.rs — \
         if it was renamed or moved, update has_gate() and \
         docs/PLAN-admin-gates-*.md"
    );
}
