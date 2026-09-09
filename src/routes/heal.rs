//! Admin endpoints for the self-healing loop.
//!
//! * `POST /api/admin/heal?domain=<host>` — diagnose-only trigger (M1).
//! * `GET /api/admin/heal/extractions` — review what the on-the-fly agent
//!   produced (M2). Lists UNTRUSTED extractions so an admin can vet them.
//! * `POST /api/admin/heal/extractions/{id}/trust` — mark an extraction
//!   trusted (M2). Only trusted extractions are reused for replay.
//! * `POST /api/admin/heal/replay-pending` — retry pending exports (M2).
//!   Best-effort: each `pending_exports` row gets the normal lookup +
//!   fetch_chapters path again; successes are marked completed, failures
//!   stay pending with attempts+1. Never blocks.
//!
//! All endpoints require role >= 10 (admin.rs convention: real HTTP 403 via
//! `AppError::Forbidden`).

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::heal::{agent, classifier};
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct HealQuery {
    pub domain: Option<String>,
    pub force: Option<String>,
}

/// docs/AGENTS.md fixer rules — the system prompt for the diagnose call.
const AGENT_SYSTEM_PROMPT: &str = "\
You are the FicNexus self-healing agent. You diagnose fanfiction-scraper \
failures (AO3, FanFiction.net, FictionPress, SpaceBattles, RoyalRoad, etc.) \
and report WHAT changed at the site. Rules:\n\
1. Diagnose only. Do NOT propose code edits or diffs.\n\
2. Name the likely root cause (site DOM change, bot-block, TLS, DNS, site down).\n\
3. If the failure is a parse error, state the most likely changed selector/\
structure.\n\
4. Be concise: max 8 lines.\n";

/// Build the user prompt from the domain's failure rows.
fn build_user_prompt(domain: &str, failures: &[crate::heal::store::ScrapeFailureRow]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Domain: {}", domain));
    lines.push(format!("Failure count (recent window): {}", failures.len()));
    for f in failures.iter().take(12) {
        lines.push(format!(
            "- kind={} url={} msg={} fp={}",
            f.error_kind,
            f.url,
            f.message.as_deref().unwrap_or("(none)"),
            f.fingerprint
        ));
    }
    lines.push("Diagnose what changed at this site. Be concise.".to_string());
    lines.join("\n")
}

/// POST /api/admin/heal?domain=X
pub async fn heal_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<HealQuery>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let domain = params
        .domain
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .ok_or_else(|| AppError::BadRequest("domain query param required".to_string()))?;
    if domain.contains('/') || domain.contains(' ') {
        return Err(AppError::BadRequest("invalid domain".to_string()));
    }

    let window = chrono::Duration::hours(24);
    let failures =
        crate::heal::store::recent_failures(&state.db, domain, chrono::Utc::now() - window)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    let should_heal = classifier::should_heal(&failures, 60);

    // ── Agent diagnose (single call, diagnose-only) ─────────────────
    let cfg = &state.config;
    let agent_available = cfg.agent_enabled && agent::remote_configured(cfg);

    // Cooldown: max `agent_max_runs_per_day` runs/day overall.
    let runs_today = crate::heal::store::count_agent_runs_since(
        &state.db,
        chrono::Utc::now() - chrono::Duration::hours(24),
    )
    .await
    .unwrap_or(0);
    let within_budget = (runs_today as u32) < cfg.agent_max_runs_per_day;
    let _ = agent_available; // kept for later milestones (fix path)

    let mut agent_response = if !cfg.agent_enabled {
        json!({"agent": "disabled", "reason": "AGENT_ENABLED is false"})
    } else if !agent::remote_configured(cfg) {
        json!({"agent": "disabled", "reason": "no agent API key configured"})
    } else if !within_budget {
        json!({"agent": "skipped", "reason": "daily run budget exhausted"})
    } else if !should_heal && params.force.as_deref() != Some("1") {
        json!({"agent": "skipped", "reason": "below structural-failure threshold"})
    } else {
        let system_prompt = AGENT_SYSTEM_PROMPT.to_string();
        let user_prompt = build_user_prompt(domain, &failures);

        // Record a pending run first, then fire the diagnose call.
        let run = crate::heal::store::record_agent_run(
            &state.db,
            "admin_heal",
            None,
            "structural",
            Some(cfg.agent_model.as_str()),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let reply =
            agent::diagnose_remote(&state.http_client, cfg, &system_prompt, &user_prompt).await;
        match reply {
            Ok(text) => {
                let _ = crate::heal::store::update_agent_run(
                    &state.db,
                    run.id,
                    "diagnosed",
                    Some(&text),
                    Some(1),
                    Some(0),
                )
                .await;
                json!({
                    "agent": "diagnosed",
                    "run_id": run.id,
                    "diagnosis": text,
                })
            }
            Err(reason) => {
                let _ = crate::heal::store::update_agent_run(
                    &state.db,
                    run.id,
                    "failed",
                    Some("agent unreachable"),
                    Some(1),
                    Some(0),
                )
                .await;
                json!({
                    "agent": "unavailable",
                    "reason": reason,
                })
            }
        }
    };

    agent_response["should_heal"] = json!(should_heal);
    agent_response["failures"] = json!(
        failures
            .iter()
            .take(25)
            .map(|f| {
                json!({
                    "id": f.id,
                    "url": f.url,
                    "url_id": f.url_id,
                    "error_kind": f.error_kind,
                    "message": f.message,
                    "fingerprint": f.fingerprint,
                    "created_at": f.created_at.to_rfc3339(),
                })
            })
            .collect::<Vec<_>>()
    );
    agent_response["plan"] = json!(format!(
        "diagnose-only: {} failures, class {}",
        failures.len(),
        if failures.is_empty() {
            "n/a".to_string()
        } else {
            classifier::classify(
                &classifier::ErrorKind::from_str(&failures[0].error_kind),
                domain,
                failures[0].message.as_deref().unwrap_or(""),
            )
            .to_string()
        }
    ));

    Ok(Json(agent_response))
}

/// GET /api/admin/heal/extractions — list UNTRUSTED extractions (the review
/// queue: what the on-the-fly agent produced before an admin trusts it).
pub async fn list_extractions_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let rows = crate::heal::extract::list_extractions(&state.db, false, 100)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(Json(json!({
        "extractions": rows.iter().map(|r| {
            json!({
                "id": r.id,
                "url": r.url,
                "title": r.title,
                "author": r.author,
                "chapters": r.chapters,
                "words": r.words,
                "status": r.status,
                "validated": r.validated,
                "trusted": r.trusted,
                "created_at": r.created_at.to_rfc3339(),
            })
        }).collect::<Vec<_>>(),
    })))
}

/// POST /api/admin/heal/extractions/{id}/trust — mark an extraction trusted
/// so replay can reuse it.
pub async fn trust_extraction_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let trusted = crate::heal::extract::mark_trusted(&state.db, id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if !trusted {
        return Err(AppError::NotFound(format!("extraction {id} not found")));
    }
    Ok(Json(json!({ "trusted": true, "id": id })))
}

/// POST /api/admin/heal/replay-pending — best-effort retry of the pending
/// export queue. For each `pending_exports` row, run the normal scraper
/// lookup + fetch_chapters path; mark completed on success, bump attempts on
/// failure. This mirrors the on-the-fly heal path (trusted extraction →
/// fetch hint) but never blocks and never errors out.
pub async fn replay_pending_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let pending = sqlx::query_as::<_, crate::heal::store::PendingExportRow>(
        r#"SELECT id, url, format, client_ip::text, client_id, status, attempts,
                  error_kind, created_at, completed_at
           FROM pending_exports
           WHERE status = 'pending'
           ORDER BY created_at ASC
           LIMIT 50"#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut completed = 0u32;
    let mut still_pending = 0u32;
    for row in &pending {
        let scraper = state.scraper_registry.find_specific_or_fff(&row.url);
        let outcome = match scraper {
            None => {
                let _ =
                    sqlx::query("UPDATE pending_exports SET attempts = attempts + 1 WHERE id = $1")
                        .bind(row.id)
                        .execute(&state.db)
                        .await;
                "pending"
            }
            Some(s) => {
                let lookup = s.lookup(&state.http_client, &row.url).await;
                match lookup {
                    Ok(meta) => {
                        let chapters = state
                            .scraper_registry
                            .fetch_chapters(&state.http_client, &meta)
                            .await;
                        match chapters {
                            Ok(ch) if !ch.is_empty() => {
                                let _ = sqlx::query(
                                    r#"UPDATE pending_exports
                                       SET status = 'completed',
                                           attempts = attempts + 1,
                                           completed_at = now()
                                       WHERE id = $1"#,
                                )
                                .bind(row.id)
                                .execute(&state.db)
                                .await;
                                "completed"
                            }
                            Ok(_) | Err(_) => {
                                let _ = sqlx::query(
                                    "UPDATE pending_exports SET attempts = attempts + 1 WHERE id = $1",
                                )
                                .bind(row.id)
                                .execute(&state.db)
                                .await;
                                "pending"
                            }
                        }
                    }
                    Err(_) => {
                        let _ = sqlx::query(
                            "UPDATE pending_exports SET attempts = attempts + 1 WHERE id = $1",
                        )
                        .bind(row.id)
                        .execute(&state.db)
                        .await;
                        "pending"
                    }
                }
            }
        };
        match outcome {
            "completed" => completed += 1,
            _ => still_pending += 1,
        }
    }

    Ok(Json(json!({
        "replayed": pending.len(),
        "completed": completed,
        "still_pending": still_pending,
    })))
}
