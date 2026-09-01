//! Recommendations for *entities* (non-work objects): similar tags/fandoms,
//! authors, collections (reading lists), and users. Each strategy reuses an
//! existing signal table so no new training data is introduced — this is a
//! pure query layer over the materialized graphs.

use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct EntityRecQuery {
    pub kind: String,
    #[serde(default)]
    pub seed: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// A scored, self-describing recommendation.
#[derive(Debug, serde::Serialize)]
pub struct EntityRec {
    pub id: Value,
    pub name: String,
    pub score: f64,
    pub reason: String,
}

/// `GET /api/recommendations/entities?kind=tag|fandom|author|collection|user&seed=...`
pub async fn entity_recs(db: &PgPool, q: EntityRecQuery) -> AppResult<serde_json::Value> {
    let limit = q.limit.unwrap_or(20).clamp(1, 50);
    let rows = match q.kind.as_str() {
        "tag" | "fandom" => recommend_tags(db, q.seed.as_deref(), limit).await?,
        "author" => recommend_authors(db, q.seed.as_deref(), limit).await?,
        "collection" | "list" => recommend_collections(db, q.seed.as_deref(), limit).await?,
        "user" => recommend_users(db, q.seed.as_deref(), limit).await?,
        other => {
            return Err(AppError::BadRequest(format!(
                "kind must be one of: tag, fandom, author, collection, user (got '{other}')"
            )));
        }
    };
    Ok(json!({ "err": 0, "kind": q.kind, "items": rows }))
}

/// Similar tags (or fandoms, which are a tag type). Jaccard over the set of
/// works carrying each tag. `seed` is a tag name; when omitted, popular tags.
async fn recommend_tags(db: &PgPool, seed: Option<&str>, limit: i64) -> AppResult<Vec<EntityRec>> {
    if let Some(name) = seed {
        let rows: Vec<(String, f64)> = sqlx::query_as(
            r#"WITH seed AS (
                   SELECT DISTINCT ft.url_id
                     FROM fic_tags ft
                     JOIN tags t ON t.id = ft.tag_id
                    WHERE t.name = $1
               ),
               cand AS (
                   SELECT ft.tag_id,
                          COUNT(*) FILTER (WHERE ft.url_id IN (SELECT url_id FROM seed)) AS inter,
                          COUNT(*) AS total
                     FROM fic_tags ft
                    WHERE ft.tag_id <> (SELECT id FROM tags WHERE name = $1 LIMIT 1)
                    GROUP BY ft.tag_id
               )
               SELECT t.name,
                      (inter::float8 / NULLIF((SELECT COUNT(*) FROM seed), 0)
                       + inter::float8 / NULLIF(total, 0)) AS score
                 FROM cand c
                 JOIN tags t ON t.id = c.tag_id
                WHERE inter > 0
                ORDER BY inter DESC NULLS LAST
                LIMIT $2"#,
        )
                .bind(name)
        .bind(limit)
        .fetch_all(db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(n, s)| EntityRec {
                id: json!(null),
                name: n,
                score: s,
                reason: "shared fandoms/works".into(),
            })
            .collect())
    } else {
        // Popular, well-used tags.
        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT t.name, COUNT(*) AS uses
                 FROM fic_tags ft
                 JOIN tags t ON t.id = ft.tag_id
                GROUP BY t.name
                HAVING COUNT(*) > 1
                ORDER BY uses DESC
                LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(n, uses)| EntityRec {
                id: json!(null),
                name: n,
                score: uses as f64,
                reason: "popular tag".into(),
            })
                        .collect())
    }
}

/// Similar authors via the materialized `rec_author_graph`. `seed` = author name.
async fn recommend_authors(db: &PgPool, seed: Option<&str>, limit: i64) -> AppResult<Vec<EntityRec>> {
    let s = seed.ok_or_else(|| AppError::BadRequest("seed author required".into()))?;
    let rows: Vec<(String, f64)> = sqlx::query_as(
        r#"SELECT CASE WHEN author_a = $1 THEN author_b ELSE author_a END AS neighbour,
                  (cooccur_count::float8 * (1.0 + tag_jaccard)) AS score
           FROM rec_author_graph
          WHERE author_a = $1 OR author_b = $1
          ORDER BY score DESC
          LIMIT $2"#,
    )
    .bind(s)
    .bind(limit)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(n, score)| EntityRec {
            id: json!(null),
            name: n,
            score,
            reason: format!("readers of {s} also read this author"),
                    })
        .collect())
}

/// Similar collections (reading lists) via shared works in reading_list_items.
async fn recommend_collections(db: &PgPool, seed: Option<&str>, limit: i64) -> AppResult<Vec<EntityRec>> {
    let s = seed.ok_or_else(|| AppError::BadRequest("seed collection title required".into()))?;
    let rows: Vec<(String, f64)> = sqlx::query_as(
        r#"WITH seed AS (
                 SELECT DISTINCT rli.work_id
                   FROM reading_lists rl
                   JOIN reading_list_items rli ON rli.list_id = rl.id
                  WHERE rl.title = $1 AND rl.is_public
             ),
             cand AS (
                 SELECT rl2.id AS list_id,
                        COUNT(*) FILTER (WHERE rli2.work_id IN (SELECT work_id FROM seed)) AS inter,
                        COUNT(*) AS total
                   FROM reading_list_items rli2
                   JOIN reading_lists rl2 ON rl2.id = rli2.list_id
                  WHERE rl2.is_public AND rl2.title <> $1
                  GROUP BY rl2.id
             )
             SELECT rl.title, (inter::float8 / NULLIF(total, 0)) * 2.0 AS score
               FROM cand c
               JOIN reading_lists rl ON rl.id = c.list_id
              WHERE inter > 0
              ORDER BY score DESC
              LIMIT $2"#,
    )
    .bind(s)
    .bind(limit)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(n, score)| EntityRec {
            id: json!(null),
            name: n,
            score,
            reason: format!("shares works with \"{s}\""),
        })
        .collect())
}

/// Similar users by shared positively-rated works (rec_user_signals).
/// `seed` is an integer user_id.
async fn recommend_users(db: &PgPool, seed: Option<&str>, limit: i64) -> AppResult<Vec<EntityRec>> {
    let s: i32 = seed
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("seed user_id (integer) required".into()))?;
    let rows: Vec<(String, f64)> = sqlx::query_as(
        r#"WITH seed AS (
                 SELECT DISTINCT work_id
                   FROM rec_user_signals
                  WHERE user_id = $1 AND signal_weight > 0
             ),
             cand AS (
                 SELECT rs2.user_id,
                        COUNT(*) FILTER (WHERE rs2.work_id IN (SELECT work_id FROM seed)) AS inter,
                        COUNT(*) AS total
                   FROM rec_user_signals rs2
                  WHERE rs2.user_id <> $1 AND rs2.signal_weight > 0
                  GROUP BY rs2.user_id
             )
             SELECT u.username, (inter::float8 / NULLIF(total, 0)) AS score
               FROM cand c
               JOIN users u ON u.id = c.user_id
              WHERE inter > 0
              ORDER BY score DESC
              LIMIT $2"#,
    )
    .bind(s)
    .bind(limit)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(n, score)| EntityRec {
            id: json!(null),
            name: n,
            score,
            reason: "read the same works".into(),
        })
        .collect())
}