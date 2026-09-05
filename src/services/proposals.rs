//! Universal curator proposals — one consensus queue for every community
//! contribution: translations, UI strings, content fixes, forum/request
//! edits, doc edits, work deletions (docs/plans/translation-everything.md §3).
//!
//! Mirror of the proven `curator_content::vote_fix` math (quorum + net
//! score, no self-vote) generalized to typed proposals with an apply
//! dispatch. Approving a `translate`/`ui_string` applies immediately;
//! legacy kinds are recorded here and applied by their existing endpoints
//! (M6 repoints them).
//!
//! The machine-translation worker (`services::translation`) inserts
//! `source='llm'` rows for human review; dismissing one marks the machine
//! draft `dismissed` so it stops rendering.

use serde_json::{Value, json};
use sqlx::PgPool;

use crate::config::Config;
use crate::error::AppError;

/// Proposal kinds the queue accepts.
pub const KINDS: &[&str] = &[
    "translate",
    "ui_string",
    "content_fix",
    "metadata_fix",
    "post_edit",
    "request_edit",
    "doc_edit",
    "work_deletion",
    "collection_add",
];

pub fn valid_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// Resolve consensus from tallies. Pending until `quorum` total votes are
/// cast; then approve needs ≥1 approve and a non-negative net score,
/// anything else dismisses. Pure — unit tested below.
pub fn decide_status(approves: i64, dismisses: i64, quorum: i64) -> &'static str {
    if approves + dismisses < quorum.max(1) {
        return "pending";
    }
    if approves >= 1 && approves - dismisses >= 0 {
        "approved"
    } else {
        "dismissed"
    }
}

/// Per-kind quorum from instance config.
pub fn quorum_for(config: &Config, kind: &str) -> i64 {
    match kind {
        "translate" | "ui_string" => config.proposal_quorum_translate,
        "content_fix" | "metadata_fix" => config.proposal_quorum_content_fix,
        "post_edit" => config.proposal_quorum_post_edit,
        "request_edit" => config.proposal_quorum_doc_edit, // same weight
        "doc_edit" => config.proposal_quorum_doc_edit,
        "work_deletion" => config.proposal_quorum_work_deletion,
        _ => config.proposal_quorum_content_fix,
    }
}

/// Insert a pending proposal; returns its id. Always modlogged.
pub async fn submit(
    db: &PgPool,
    kind: &str,
    target_type: &str,
    target_id: &str,
    payload: Value,
    proposer: Option<i32>,
    source: &str,
) -> Result<i64, AppError> {
    if !valid_kind(kind) {
        return Err(AppError::BadRequest(format!(
            "unknown proposal kind: {kind}"
        )));
    }
    let id: i64 = sqlx::query_scalar(
        r#"INSERT INTO proposals (kind, target_type, target_id, payload, proposer_id, source)
           VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"#,
    )
    .bind(kind)
    .bind(target_type)
    .bind(target_id)
    .bind(&payload)
    .bind(proposer)
    .bind(if source == "llm" {
        "llm"
    } else if source == "system" {
        "system"
    } else if source == "moderator" {
        "moderator"
    } else {
        "user"
    })
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(format!("proposal insert: {e}")))?;

    crate::modlog::record(
        db,
        proposer,
        proposer.map(|_| "curator".to_string()),
        "proposal_submit",
        kind,
        &format!("{target_type}:{target_id}"),
        json!({"proposal_id": id, "locale": payload.get("locale")}),
    )
    .await;
    Ok(id)
}

/// A pending translate proposal already exists for the same field+locale?
pub async fn pending_translate_exists(
    db: &PgPool,
    target_type: &str,
    target_id: &str,
    field: &str,
    locale: &str,
) -> Result<bool, AppError> {
    let exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
             SELECT 1 FROM proposals
             WHERE kind = 'translate' AND target_type = $1 AND target_id = $2 AND status = 'pending'
               AND payload->>'field' = $3 AND payload->>'locale' = $4
           )"#,
    )
    .bind(target_type)
    .bind(target_id)
    .bind(field)
    .bind(locale)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(exists)
}

/// Record a curator vote, run the quorum math, resolve if decided.
pub async fn record_vote(
    db: &PgPool,
    config: &Config,
    proposal_id: i64,
    curator_id: i32,
    curator_role: i16,
    curator_name: &str,
    decision: &str,
) -> Result<Value, AppError> {
    if curator_role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }
    if decision != "approve" && decision != "dismiss" {
        return Err(AppError::BadRequest(
            "decision must be approve|dismiss".into(),
        ));
    }
    let row: Option<(String, String, String, Option<i32>, String)> = sqlx::query_as(
        "SELECT kind, target_type, target_id, proposer_id, status FROM proposals WHERE id = $1",
    )
    .bind(proposal_id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let (kind, target_type, target_id, proposer, status) =
        row.ok_or_else(|| AppError::NotFound("proposal not found".into()))?;
    if status != "pending" {
        return Err(AppError::Conflict(format!("proposal already {status}")));
    }
    if proposer == Some(curator_id) {
        return Err(AppError::BadRequest(
            "you cannot vote on your own proposal".into(),
        ));
    }

    sqlx::query(
        r#"INSERT INTO proposal_votes (proposal_id, curator_id, decision)
           VALUES ($1, $2, $3)
           ON CONFLICT (proposal_id, curator_id) DO UPDATE SET decision = EXCLUDED.decision"#,
    )
    .bind(proposal_id)
    .bind(curator_id)
    .bind(decision)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let (approves, dismisses): (i64, i64) = sqlx::query_as(
        r#"SELECT COUNT(*) FILTER (WHERE decision = 'approve'),
                  COUNT(*) FILTER (WHERE decision = 'dismiss')
           FROM proposal_votes WHERE proposal_id = $1"#,
    )
    .bind(proposal_id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Modest curator incentive per vote (daily-capped in xp_source_defs).
    let _ = crate::services::progression::award_xp(
        db,
        curator_id,
        "translation_reviewed",
        Some(&proposal_id.to_string()),
    )
    .await;

    crate::modlog::record(
        db,
        Some(curator_id),
        Some(curator_name.to_string()),
        "proposal_vote",
        &kind,
        &format!("{target_type}:{target_id}"),
        json!({"proposal_id": proposal_id, "decision": decision, "approves": approves, "dismisses": dismisses}),
    )
    .await;

    let new_status = decide_status(approves, dismisses, quorum_for(config, &kind));
    if new_status == "pending" {
        return Ok(
            json!({"err": 0, "status": "pending", "approves": approves, "dismisses": dismisses, "resolved": false}),
        );
    }

    finalize(
        db,
        config,
        proposal_id,
        &kind,
        &target_type,
        &target_id,
        new_status,
        Some(curator_id),
        Some(curator_name),
        None,
    )
    .await?;
    Ok(
        json!({"err": 0, "status": new_status, "approves": approves, "dismisses": dismisses, "resolved": true}),
    )
}

/// Fast-path: a curator decides now, skipping quorum.
pub async fn decide(
    db: &PgPool,
    config: &Config,
    proposal_id: i64,
    curator_id: i32,
    curator_role: i16,
    curator_name: &str,
    decision: &str,
    note: Option<&str>,
) -> Result<Value, AppError> {
    if curator_role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }
    let final_status = match decision {
        "approve" => "approved",
        "dismiss" => "dismissed",
        _ => {
            return Err(AppError::BadRequest(
                "decision must be approve|dismiss".into(),
            ));
        }
    };
    let row: Option<(String, String, String, String)> =
        sqlx::query_as("SELECT kind, target_type, target_id, status FROM proposals WHERE id = $1")
            .bind(proposal_id)
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let (kind, target_type, target_id, status) =
        row.ok_or_else(|| AppError::NotFound("proposal not found".into()))?;
    if status != "pending" {
        return Err(AppError::Conflict(format!("proposal already {status}")));
    }
    finalize(
        db,
        config,
        proposal_id,
        &kind,
        &target_type,
        &target_id,
        final_status,
        Some(curator_id),
        Some(curator_name),
        note,
    )
    .await?;
    Ok(json!({"err": 0, "status": final_status, "resolved": true}))
}

/// Shared resolution: status write + apply + modlog.
async fn finalize(
    db: &PgPool,
    _config: &Config,
    proposal_id: i64,
    kind: &str,
    target_type: &str,
    target_id: &str,
    status: &str,
    decided_by: Option<i32>,
    actor_name: Option<&str>,
    note: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE proposals SET status = $2, decided_by = $3, decided_at = NOW(), note = COALESCE($4, note) WHERE id = $1",
    )
    .bind(proposal_id)
    .bind(status)
    .bind(decided_by)
    .bind(note)
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    apply(db, proposal_id, kind, target_type, target_id, status).await;

    crate::modlog::record(
        db,
        decided_by,
        actor_name.map(|s| s.to_string()),
        if status == "approved" {
            "proposal_approve"
        } else {
            "proposal_dismiss"
        },
        kind,
        &format!("{target_type}:{target_id}"),
        json!({"proposal_id": proposal_id, "note": note}),
    )
    .await;
    Ok(())
}

/// Apply an approved (or dismissed) proposal to its target. Best-effort:
/// an apply failure never blocks the queue bookkeeping, it only logs.
async fn apply(
    db: &PgPool,
    proposal_id: i64,
    kind: &str,
    target_type: &str,
    target_id: &str,
    status: &str,
) {
    let payload: Option<Value> = sqlx::query_scalar("SELECT payload FROM proposals WHERE id = $1")
        .bind(proposal_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    let Some(pl) = payload else { return };

    match (kind, status) {
        ("translate", "approved") => {
            apply_translate(db, proposal_id, target_type, target_id, &pl).await
        }
        ("translate", "dismissed") => {
            // Rejecting an LLM draft (or a bad human one) hides machine text.
            let r = sqlx::query(
                "UPDATE translation_strings SET status = 'dismissed', updated_at = NOW() \
                 WHERE target_type = $1 AND target_id = $2 AND field = $3 AND locale = $4 AND status = 'machine'",
            )
            .bind(target_type)
            .bind(target_id)
            .bind(pl.get("field").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(pl.get("locale").and_then(|v| v.as_str()).unwrap_or(""))
            .execute(db)
            .await;
            if let Err(e) = r {
                tracing::warn!("proposal {proposal_id} translate dismiss: {e}");
            }
        }
        ("ui_string", "approved") => apply_ui_string(db, proposal_id, &pl).await,
        (k, s) => {
            // M6: these legacy kinds will move their apply here; for now the
            // decision is recorded and the original endpoint still applies it.
            tracing::warn!(
                "proposal {proposal_id} ({k} {target_type}:{target_id}) => {s}: legacy apply path"
            );
        }
    }
}

async fn apply_translate(
    db: &PgPool,
    proposal_id: i64,
    target_type: &str,
    target_id: &str,
    pl: &Value,
) {
    let Some(field) = pl.get("field").and_then(|v| v.as_str()) else {
        return;
    };
    let Some(locale) = pl.get("locale").and_then(|v| v.as_str()) else {
        return;
    };
    let Some(text) = pl.get("text").and_then(|v| v.as_str()) else {
        return;
    };
    let source_hash = pl.get("source_hash").and_then(|v| v.as_str()).unwrap_or("");

    // Supersede an existing approved translation for this exact slot.
    let prior: Option<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT id, approved_proposal_id FROM translation_strings \
         WHERE target_type=$1 AND target_id=$2 AND field=$3 AND locale=$4 AND status='approved'",
    )
    .bind(target_type)
    .bind(target_id)
    .bind(field)
    .bind(locale)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    let mut superseded = false;
    if let Some((prior_id, prior_proposal)) = prior {
        let _ = sqlx::query(
            "UPDATE translation_strings SET status='outdated', updated_at=NOW() WHERE id=$1",
        )
        .bind(prior_id)
        .execute(db)
        .await;
        if let Some(old_pid) = prior_proposal {
            let _ = sqlx::query(
                "UPDATE proposals SET status='superseded' WHERE id=$1 AND status='approved'",
            )
            .bind(old_pid)
            .execute(db)
            .await;
            superseded = true;
        }
    }

    let (proposer, source): (Option<i32>, String) =
        sqlx::query_as("SELECT proposer_id, source FROM proposals WHERE id=$1")
            .bind(proposal_id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .unwrap_or((None, "user".into()));

    let origin = if source == "llm" { "llm" } else { "user" };
    let up = sqlx::query(
        r#"INSERT INTO translation_strings
             (target_type, target_id, field, locale, source_hash, text, status, origin, approved_proposal_id, created_by)
           VALUES ($1,$2,$3,$4,$5,$6,'approved',$7,$8,$9)
           ON CONFLICT (target_type, target_id, field, locale) DO UPDATE SET
             text = EXCLUDED.text, source_hash = EXCLUDED.source_hash, status = 'approved',
             origin = EXCLUDED.origin, approved_proposal_id = EXCLUDED.approved_proposal_id,
             created_by = EXCLUDED.created_by, updated_at = NOW()"#,
    )
    .bind(target_type)
    .bind(target_id)
    .bind(field)
    .bind(locale)
    .bind(source_hash)
    .bind(text)
    .bind(origin)
    .bind(proposal_id)
    .bind(proposer)
    .execute(db)
    .await;
    if let Err(e) = up {
        tracing::warn!("proposal {proposal_id} translate apply: {e}");
        return;
    }
    if let Some(uid) = proposer {
        let event = if superseded {
            "translation_improved"
        } else {
            "translation_approved"
        };
        let _ =
            crate::services::progression::award_xp(db, uid, event, Some(&proposal_id.to_string()))
                .await;
    }
}

async fn apply_ui_string(db: &PgPool, proposal_id: i64, pl: &Value) {
    let (Some(locale), Some(ns), Some(key), Some(value)) = (
        pl.get("locale").and_then(|v| v.as_str()),
        pl.get("namespace").and_then(|v| v.as_str()),
        pl.get("key").and_then(|v| v.as_str()),
        pl.get("value").and_then(|v| v.as_str()),
    ) else {
        return;
    };
    let r = sqlx::query(
        r#"INSERT INTO translations (locale_code, namespace, key, value)
           VALUES ($1,$2,$3,$4)
           ON CONFLICT (locale_code, namespace, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
    )
    .bind(locale)
    .bind(ns)
    .bind(key)
    .bind(value)
    .execute(db)
    .await;
    if let Err(e) = r {
        tracing::warn!("proposal {proposal_id} ui_string apply: {e}");
        return;
    }
    let proposer: Option<i32> = sqlx::query_scalar("SELECT proposer_id FROM proposals WHERE id=$1")
        .bind(proposal_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .flatten();
    if let Some(uid) = proposer {
        let _ = crate::services::progression::award_xp(
            db,
            uid,
            "translation_approved",
            Some(&proposal_id.to_string()),
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::decide_status;

    #[test]
    fn pending_below_quorum() {
        assert_eq!(decide_status(1, 0, 2), "pending");
        assert_eq!(decide_status(0, 0, 2), "pending");
    }
    #[test]
    fn net_zero_at_quorum_approves() {
        assert_eq!(decide_status(1, 1, 2), "approved");
    }
    #[test]
    fn majority_dismiss_wins() {
        assert_eq!(decide_status(1, 2, 2), "dismissed");
        assert_eq!(decide_status(0, 2, 2), "dismissed");
    }
    #[test]
    fn approve_dominant_resolves() {
        assert_eq!(decide_status(2, 1, 2), "approved");
        assert_eq!(decide_status(3, 0, 2), "approved");
    }
}
