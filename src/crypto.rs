//! Symmetric encryption helpers for stored secrets (user-supplied site
//! credentials).
//!
//! Scheme: AES-256-GCM with a random 12-byte nonce prepended to the
//! ciphertext, then base64-encoded. The key is `sha256(JWT_SECRET)` — the
//! same secret the server already uses for auth tokens, so no new config is
//! required. NEVER log or return plaintext passwords.
//!
//! Encrypted form (base64 of): `nonce(12) || ciphertext || tag(16)`.

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Derive the 32-byte AES-256 key from the JWT secret.
fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

/// Encrypt `plaintext` with a fresh random nonce.
///
/// Returns base64(`nonce(12) || ciphertext || tag(16)`).
#[allow(deprecated)] // aes-gcm 0.10 still uses generic-array 0.14's from_slice
pub fn encrypt(plaintext: &str, key: &str) -> String {
    let cipher = Aes256Gcm::new_from_slice(&derive_key(key)).expect("32-byte key");
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("AES-256-GCM encryption failed");

    let mut out = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    base64::engine::general_purpose::STANDARD.encode(out)
}

/// Decrypt a value produced by [`encrypt`]. Returns `None` on any failure
/// (bad base64, wrong key, tampered ciphertext) — never panics, never logs
/// the plaintext.
#[allow(deprecated)] // aes-gcm 0.10 still uses generic-array 0.14's from_slice
pub fn decrypt(ciphertext_b64: &str, key: &str) -> Option<String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64.as_bytes())
        .ok()?;
    if raw.len() < 13 {
        return None;
    }
    let (nonce_bytes, body) = raw.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&derive_key(key)).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(&nonce, body).ok()?;
    String::from_utf8(plaintext).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let ct = encrypt("hunter2", "test-secret");
        assert_ne!(ct, "hunter2");
        assert_eq!(decrypt(&ct, "test-secret").as_deref(), Some("hunter2"));
    }

    #[test]
    fn ciphertext_is_unique_per_call() {
        let key = "test-secret";
        let a = encrypt("same password", key);
        let b = encrypt("same password", key);
        assert_ne!(a, b, "random nonce must make ciphertexts unique");
        assert_eq!(decrypt(&a, key), decrypt(&b, key));
    }

    #[test]
    fn wrong_key_fails() {
        let ct = encrypt("secret", "right-key");
        assert!(decrypt(&ct, "wrong-key").is_none());
    }

    #[test]
    fn garbage_fails_gracefully() {
        assert!(decrypt("not-base64!!", "key").is_none());
        assert!(decrypt("", "key").is_none());
        assert!(decrypt("aGVsbG8=", "key").is_none()); // valid b64, too short
    }
}
