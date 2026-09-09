use axum::{Json, extract::State};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Personal reading statistics; always scoped to the authenticated user.
pub async fn personal_reading_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    Ok(Json(
        queries::get_personal_reading_analytics(&state.db, user_id).await?,
    ))
}

/// Public author analytics derived from reading history and reading_stats.
pub async fn author_analytics_handler(
    axum::extract::Path(id): axum::extract::Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(queries::get_author_analytics(&state.db, id).await?))
}

/// Analytics dashboard: GET /api/analytics
pub async fn analytics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    // Get daily stats for last 30 days
    let daily_stats = queries::get_daily_stats(&state.db, 30).await?;

    // Get total unique visitors
    let total_visitors_7d = queries::get_total_unique_visitors(&state.db, 7).await?;
    let total_visitors_30d = queries::get_total_unique_visitors(&state.db, 30).await?;

    // Get return visitor rate
    let (total_visitors, return_visitors) = queries::get_return_visitor_rate(&state.db, 30).await?;
    let return_rate = if total_visitors > 0 {
        (return_visitors as f64 / total_visitors as f64 * 100.0).round()
    } else {
        0.0
    };

    // Get popular fics
    let popular_fics_7d = queries::get_popular_fics(&state.db, 7, 10).await?;
    let popular_fics_30d = queries::get_popular_fics(&state.db, 30, 10).await?;

    // Get format breakdown
    let format_breakdown = queries::get_format_breakdown(&state.db, 30).await?;

    Ok(Json(json!({
        "unique_visitors": {
            "last_7_days": total_visitors_7d,
            "last_30_days": total_visitors_30d
        },
        "return_rate": {
            "percentage": return_rate,
            "total_visitors": total_visitors,
            "returning_visitors": return_visitors
        },
        "daily_stats": daily_stats.iter().map(|(day, visitors, requests)| {
            json!({
                "date": day.format("%Y-%m-%d").to_string(),
                "unique_visitors": visitors,
                "total_requests": requests
            })
        }).collect::<Vec<_>>(),
        "popular_fics": {
            "last_7_days": popular_fics_7d.iter().map(|(url_id, requests, downloads)| {
                json!({
                    "url_id": url_id,
                    "requests": requests,
                    "downloads": downloads
                })
            }).collect::<Vec<_>>(),
            "last_30_days": popular_fics_30d.iter().map(|(url_id, requests, downloads)| {
                json!({
                    "url_id": url_id,
                    "requests": requests,
                    "downloads": downloads
                })
            }).collect::<Vec<_>>()
        },
        "format_breakdown": format_breakdown.iter().map(|(format, count)| {
            json!({
                "format": format,
                "count": count
            })
        }).collect::<Vec<_>>()
    })))
}

/// User stats: GET /api/analytics/user/:client_id
pub async fn user_stats_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(client_id): axum::extract::Path<String>,
) -> Result<Json<Value>, AppError> {
    let (total_requests, unique_fics, downloads) =
        queries::get_user_stats(&state.db, &client_id).await?;

    Ok(Json(json!({
        "client_id": client_id,
        "total_requests": total_requests,
        "unique_fics": unique_fics,
        "downloads": downloads
    })))
}

// ═══════════════════════════════════════════════════════════════════
// Usage tracking middleware + admin analytics dashboard
// ═══════════════════════════════════════════════════════════════════

/// Paths that count as ACTIONS (something beyond browsing). Exports,
/// downloads, votes, comments, bookmarks, etc.
const ACTION_PREFIXES: &[&str] = &[
    "/api/epub",
    "/api/meta", // fetch metadata = a concrete lookup action
    "/api/download",
    "/cache/",
    "/api/bookmarks",
    "/api/comments",
    "/api/votes",
    "/api/tags",
    "/api/vote",
    "/api/suggest",
    "/api/requests",
    "/api/recommendations/suggest",
    "/api/recommendations/vote",
    "/api/roadmap/vote",
    "/api/roadmap/suggest",
    "/api/auth/register",
    "/api/auth/login",
    "/api/curator",
    "/api/follow",
    "/api/rate",
    "/api/review",
    "/api/send-to-kindle",
    "/api/ask",
];

fn is_action(path: &str) -> bool {
    ACTION_PREFIXES.iter().any(|p| path.starts_with(p))
}

/// Axum middleware that records a usage event (view or action) per request
/// carrying an X-Client-ID header. Best-effort + non-blocking: the insert is
/// spawned on a background task so it never adds latency to the response.
pub async fn track_usage(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let client_id = req
        .headers()
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() <= 128);

    let path = req.uri().path().to_string();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(256).collect::<String>());

    let db = state.db.clone();
    if let Some(cid) = client_id {
        let event_type = if is_action(&path) { "action" } else { "view" };
        // Fire-and-forget: record after the response, never block the caller.
        tokio::spawn(async move {
            crate::db::queries::insert_usage_event(
                &db,
                &cid,
                &path,
                event_type,
                user_agent.as_deref(),
            )
            .await;
        });
    }

    next.run(req).await
}

/// Per-endpoint usage breakdown: GET /api/admin/endpoint-usage?days=7&limit=20 (role >= 10)
pub async fn endpoint_usage_handler(
    auth: crate::routes::auth::AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id.is_none() {
        return Err(AppError::Unauthorized("Login required".to_string()));
    }
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    let days: i32 = params
        .get("days")
        .and_then(|v| v.parse().ok())
        .unwrap_or(7)
        .clamp(1, 90);
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .clamp(1, 100);
    let rows = queries::get_endpoint_usage(&state.db, days, limit).await?;
    let groups = queries::get_endpoint_group_usage(&state.db, days).await?;
    Ok(Json(json!({
        "days": days,
        "endpoints": rows.iter().map(|(path, etype, cnt)| json!({
            "path": path,
            "event_type": etype,
            "count": cnt,
            "group": queries::endpoint_group(path),
        })).collect::<Vec<_>>(),
        "groups": groups.iter().map(|(g, cnt)| json!({"group": g, "count": cnt})).collect::<Vec<_>>(),
    })))
}

/// Admin analytics dashboard: GET /api/admin/analytics (role >= 10).
/// Returns unique daily/weekly/monthly visitors, active users (performed
/// actions) vs view-only visitors, totals, and a recent action timeline.
pub async fn admin_analytics_handler(
    auth: crate::routes::auth::AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id.is_none() {
        return Err(AppError::Unauthorized("Login required".to_string()));
    }
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let daily = queries::get_daily_unique_visitors(&state.db, 30).await?;
    let weekly = queries::get_weekly_unique_visitors(&state.db, 12).await?;
    let monthly = queries::get_monthly_unique_visitors(&state.db, 12).await?;

    // Active vs view-only for 1d / 7d / 30d windows.
    let (active_1d, _) = queries::get_active_users(&state.db, 1).await?;
    let (active_7d, actions_7d) = queries::get_active_users(&state.db, 7).await?;
    let (active_30d, actions_30d) = queries::get_active_users(&state.db, 30).await?;

    let view_only_1d = queries::get_view_only_users(&state.db, 1).await?;
    let view_only_7d = queries::get_view_only_users(&state.db, 7).await?;
    let view_only_30d = queries::get_view_only_users(&state.db, 30).await?;

    let total_events_7d = queries::get_total_events(&state.db, 7).await?;
    let total_events_30d = queries::get_total_events(&state.db, 30).await?;

    let recent = queries::get_recent_events(&state.db, 50).await?;

    Ok(Json(json!({
        "unique_visitors": {
            "daily": daily.iter().map(|(d, n)| json!({"date": d.format("%Y-%m-%d").to_string(), "visitors": n})).collect::<Vec<_>>(),
            "weekly": weekly.iter().map(|(d, n)| json!({"week_start": d.format("%Y-%m-%d").to_string(), "visitors": n})).collect::<Vec<_>>(),
            "monthly": monthly.iter().map(|(d, n)| json!({"month": d.format("%Y-%m").to_string(), "visitors": n})).collect::<Vec<_>>(),
        },
        "engagement": {
            "active_users": { "1d": active_1d, "7d": active_7d, "30d": active_30d },
            "view_only_users": { "1d": view_only_1d, "7d": view_only_7d, "30d": view_only_30d },
            "action_events": { "7d": actions_7d, "30d": actions_30d },
            "total_events": { "7d": total_events_7d, "30d": total_events_30d },
        },
        "recent_events": recent.iter().map(|(ts, cid, path, etype, ua)| {
            json!({
                "at": ts.to_rfc3339(),
                "client_id": cid,
                "path": path,
                "type": etype,
                "user_agent": ua,
            })
        }).collect::<Vec<_>>(),
    })))
}
