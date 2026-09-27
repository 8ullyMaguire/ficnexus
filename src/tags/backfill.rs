//! Tag-score backfill for content ingested before scrape-time scoring shipped.
//!
//! Scrape-time scoring (SPECIFICATION.md §13, `ExtractedTag`) records the
//! first-listed character tag with score 10 (the main character), secondary
//! characters with 1, the first-listed ship with 5 (primary pairing) and
//! other ships with 1. Content ingested before that shipped has `score = 0`
//! everywhere, which breaks `main_char_attr` (its subquery picks the
//! character tag with the highest score — with all scores tied at 0 the
//! "main character" is arbitrary). This module rewrites those zeros to the
//! scrape-time convention.
//!
//! `main_char_attr` does not read `score` directly: search filters on
//! `fic_tags.role_confidence`, which since migration
//! 095_role_confidence_generated.sql is a **generated** column derived from
//! `score` (>=10 → 1.0, >0 → 0.5, else 0.0). Fixing `score` here therefore fixes
//! the filter, which is the point of this module.
//!
//! Idempotency: a fic's character tags are touched only when *none* of them
//! has a nonzero score (same rule for ships). Once backfilled, re-running
//! leaves the fic alone; user votes (which move scores off zero via the
//! `fic_tag_votes` trigger) also skip it. Freeform/fandom/warning/category
//! tags are never touched — score 0 is their correct value, and
//! `main_char_attr` checks freeform *presence*, not score.

use sqlx::PgPool;

const MAIN_CHAR_SCORE: i16 = 10;
const SIDE_SCORE: i16 = 1; // secondary character AND non-primary ship
const PRIMARY_SHIP_SCORE: i16 = 5;

/// Per-fic outcome of a backfill run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FicScoreSummary {
    pub url_id: String,
    pub main_char: Option<String>,
    pub other_chars: Vec<String>,
    pub primary_ship: Option<String>,
    pub other_ships: Vec<String>,
}

struct TagRow {
    url_id: String,
    tag_id: i32,
    name: String,
}

/// Backfill character (10/1) and ship (5/1) scores for fics whose tags of
/// that type are all zero. Returns one summary per affected fic.
///
/// Never touches fics that already have a nonzero character/ship score
/// (idempotent), and never touches freeform/fandom/warning/category tags.
pub async fn backfill_scores(pool: &PgPool) -> Result<Vec<FicScoreSummary>, sqlx::Error> {
    // Rows are ordered by (url_id, created_at, tag_id) — within a fic, the
    // first row is the earliest-created tag, which is the scrape-time
    // "first-listed" position.
    let char_rows = fetch_zero_scored(pool, 2).await?;
    let ship_rows = fetch_zero_scored(pool, 3).await?;

    let char_groups = group_by_fic(&char_rows);
    let ship_groups = group_by_fic(&ship_rows);

    // Deterministic output ordering (BTreeMap keys on url_id).
    let mut by_fic: std::collections::BTreeMap<String, FicScoreSummary> =
        std::collections::BTreeMap::new();

    for (url_id, rows) in &char_groups {
        let summary = by_fic.entry(url_id.clone()).or_default();
        summary.url_id = url_id.clone();
        summary.main_char = Some(rows[0].name.clone());
        summary.other_chars = rows[1..].iter().map(|r| r.name.clone()).collect();
    }

    for (url_id, rows) in &ship_groups {
        let summary = by_fic.entry(url_id.clone()).or_default();
        summary.url_id = url_id.clone();
        summary.primary_ship = Some(rows[0].name.clone());
        summary.other_ships = rows[1..].iter().map(|r| r.name.clone()).collect();
    }

    if by_fic.is_empty() {
        return Ok(Vec::new());
    }

    // All-or-nothing: a failure mid-way rolls back every score change.
    let mut tx = pool.begin().await?;
    for (url_id, rows) in &char_groups {
        update_score(&mut tx, url_id, rows[0].tag_id, MAIN_CHAR_SCORE).await?;
        for row in &rows[1..] {
            update_score(&mut tx, url_id, row.tag_id, SIDE_SCORE).await?;
        }
    }
    for (url_id, rows) in &ship_groups {
        update_score(&mut tx, url_id, rows[0].tag_id, PRIMARY_SHIP_SCORE).await?;
        for row in &rows[1..] {
            update_score(&mut tx, url_id, row.tag_id, SIDE_SCORE).await?;
        }
    }
    tx.commit().await?;

    Ok(by_fic.into_values().collect())
}

/// Fetch (url_id, tag_id, name) for tags of `tag_type_id` whose fic has
/// *no* nonzero score of that type (i.e. all of them are still 0 — the
/// pre-scoring state). Ordered by url_id, created_at, tag_id.
async fn fetch_zero_scored(pool: &PgPool, tag_type_id: i16) -> Result<Vec<TagRow>, sqlx::Error> {
    sqlx::query_as::<_, (String, i32, String)>(
        r#"SELECT ft.url_id, ft.tag_id, t.name
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE t.tag_type_id = $1
             AND NOT EXISTS (
                 SELECT 1
                 FROM fic_tags f2
                 JOIN tags t2 ON t2.id = f2.tag_id
                 WHERE f2.url_id = ft.url_id
                   AND t2.tag_type_id = $1
                   AND f2.score <> 0
             )
           ORDER BY ft.url_id, ft.created_at, ft.tag_id"#,
    )
    .bind(tag_type_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(url_id, tag_id, name)| TagRow {
                url_id,
                tag_id,
                name,
            })
            .collect()
    })
}

/// Group rows (already ordered by url_id) into per-fic runs, preserving order.
fn group_by_fic(rows: &[TagRow]) -> Vec<(String, Vec<&TagRow>)> {
    let mut out: Vec<(String, Vec<&TagRow>)> = Vec::new();
    for row in rows {
        match out.last_mut() {
            Some((url_id, group)) if *url_id == row.url_id => group.push(row),
            _ => out.push((row.url_id.clone(), vec![row])),
        }
    }
    out
}

async fn update_score(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    url_id: &str,
    tag_id: i32,
    score: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE fic_tags SET score = $1 WHERE url_id = $2 AND tag_id = $3")
        .bind(score)
        .bind(url_id)
        .bind(tag_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_by_fic_preserves_order() {
        let rows = vec![
            TagRow {
                url_id: "a".into(),
                tag_id: 1,
                name: "x".into(),
            },
            TagRow {
                url_id: "a".into(),
                tag_id: 2,
                name: "y".into(),
            },
            TagRow {
                url_id: "b".into(),
                tag_id: 3,
                name: "z".into(),
            },
        ];
        let groups = group_by_fic(&rows);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "a");
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[0].1[0].tag_id, 1);
        assert_eq!(groups[0].1[1].tag_id, 2);
        assert_eq!(groups[1].0, "b");
        assert_eq!(groups[1].1.len(), 1);
    }
}
