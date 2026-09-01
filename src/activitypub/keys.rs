//! RSA-2048 key management for ActivityPub actors.
//! One keypair per (actor_type, actor_id) — actor_type in {"instance","user","category"}.
//! Stored PEM in ap_keys (private_pem, public_pem). Public fetched by keyId URI.

use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey, LineEnding}};
use rand::rngs::OsRng;

/// Generate a fresh 2048-bit RSA keypair, return (private_pem PKCS#1, public_pem PKCS#1).
pub fn generate_keypair() -> anyhow::Result<(String, String)> {
    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let pub_key = RsaPublicKey::from(&priv_key);
    let priv_pem = priv_key.to_pkcs1_pem(LineEnding::LF)?.to_string();
    let pub_pem = pub_key.to_pkcs1_pem(LineEnding::LF)?;
    Ok((priv_pem, pub_pem))
}

/// Extract a public key PEM from a private key PEM (for keyId responses without DB round-trip).
pub fn public_pem_from_private(private_pem: &str) -> anyhow::Result<String> {
    let priv_key = RsaPrivateKey::from_pkcs1_pem(private_pem)?;
    let pub_key = RsaPublicKey::from(&priv_key);
    Ok(pub_key.to_pkcs1_pem(LineEnding::LF)?)
}

/// Ensure a keypair exists for (actor_type, actor_id); return (private_pem, public_pem).
/// Uses INSERT ... ON CONFLICT DO NOTHING + SELECT so concurrent callers don't race.
pub async fn ensure_keypair(db: &sqlx::PgPool, actor_type: &str, actor_id: i64) -> anyhow::Result<(String, String)> {
    if let Some(row) = sqlx::query_as::<_, (String, String)>(
        "SELECT private_pem, public_pem FROM ap_keys WHERE actor_type = $1 AND actor_id = $2"
    ).bind(actor_type).bind(actor_id).fetch_optional(db).await? {
        return Ok(row);
    }
    let (priv_pem, pub_pem) = generate_keypair()?;
    // Insert; if someone else won the race, just read back.
    let _ = sqlx::query("INSERT INTO ap_keys (actor_type, actor_id, private_pem, public_pem) VALUES ($1,$2,$3,$4) ON CONFLICT (actor_type, actor_id) DO NOTHING")
        .bind(actor_type).bind(actor_id).bind(&priv_pem).bind(&pub_pem).execute(db).await;
    if let Some(row) = sqlx::query_as::<_, (String, String)>(
        "SELECT private_pem, public_pem FROM ap_keys WHERE actor_type = $1 AND actor_id = $2"
    ).bind(actor_type).bind(actor_id).fetch_optional(db).await? {
        return Ok(row);
    }
    Ok((priv_pem, pub_pem))
}

/// Fetch the public PEM for an actor (or generate one if missing — self-healing).
pub async fn public_pem(db: &sqlx::PgPool, actor_type: &str, actor_id: i64) -> anyhow::Result<String> {
    let (_, pub_pem) = ensure_keypair(db, actor_type, actor_id).await?;
    Ok(pub_pem)
}

/// Public key for a keyId URI like https://example.com/actor#key or /actor/0#key.
/// Returns None when the key doesn't exist yet.
pub async fn public_pem_by_key_id(db: &sqlx::PgPool, key_id: &str, canonical_origin: &str) -> Option<String> {
    // keyId is origin + path + "#key" — strip fragment and origin.
    let base = key_id.split('#').next().unwrap_or(key_id).trim_end_matches('/');
    let origin = canonical_origin.trim_end_matches('/');
    let path = base.strip_prefix(origin).unwrap_or(base);
    let (actor_type, actor_id) = match path {
        "/actor" | "/actor/0" | "/.well-known/ap-actor" => ("instance", 0i64),
        p if p.starts_with("/actor/") => ("user", p.trim_start_matches("/actor/").parse().ok()?),
        p if p.starts_with("/uid/") => ("user", p.trim_start_matches("/uid/").parse().ok()?),
        p if p.starts_with("/category/") => ("category", p.trim_start_matches("/category/").parse().ok()?),
        _ => return None,
    };
    sqlx::query_scalar::<_, String>("SELECT public_pem FROM ap_keys WHERE actor_type = $1 AND actor_id = $2")
        .bind(actor_type).bind(actor_id).fetch_optional(db).await.ok().flatten()
}
