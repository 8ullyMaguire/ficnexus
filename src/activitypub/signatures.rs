//! HTTP Signatures: draft-cavage + minimal RFC 9421.
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rsa::{pkcs1::DecodeRsaPublicKey, pkcs1::DecodeRsaPrivateKey, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Sha256, Digest};

pub fn digest_header(body: &[u8]) -> String {
    let mut h = Sha256::new(); h.update(body);
    format!("SHA-256={}", B64.encode(h.finalize()))
}

pub fn build_signing_string(method: &str, path_q: &str, host: &str, date: &str, digest: Option<&str>) -> String {
    let mut s = format!("(request-target): {} {}\nhost: {}\ndate: {}", method.to_lowercase(), path_q, host, date);
    if let Some(d) = digest { s.push_str(&format!("\ndigest: {}", d)); }
    s
}

pub fn sign_string(private_pem: &str, signing_string: &str) -> anyhow::Result<String> {
    let key = RsaPrivateKey::from_pkcs1_pem(private_pem)?;
    let hashed = Sha256::digest(signing_string.as_bytes());
    let sig = key.sign(Pkcs1v15Sign::new_unprefixed(), &hashed)?;
    Ok(B64.encode(sig))
}

pub fn verify_signature(public_pem: &str, signing_string: &str, sig_b64: &str) -> anyhow::Result<bool> {
    let key = RsaPublicKey::from_pkcs1_pem(public_pem)?;
    let sig = B64.decode(sig_b64.trim())?;
    let hashed = Sha256::digest(signing_string.as_bytes());
    Ok(key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &sig).is_ok())
}

pub fn build_signature_header(key_id: &str, headers: &str, sig_b64: &str) -> String {
    format!(r#"keyId="{}",algorithm="rsa-sha256",headers="{}",signature="{}""#, key_id, headers, sig_b64)
}

pub fn parse_signature_header(h: &str) -> Option<(String, String, String)> {
    let mut key_id=None; let mut headers=None; let mut sig=None;
    for part in h.split(',') {
        let (k,v)=part.split_once('=')?;
        let v=v.trim().trim_matches('"');
        match k.trim() { "keyId"=>key_id=Some(v.to_string()), "headers"=>headers=Some(v.to_string()), "signature"=>sig=Some(v.to_string()), _=>{} }
    }
    Some((key_id?, headers?, sig?))
}

pub fn sign_request(private_pem: &str, key_id: &str, method: &str, path_q: &str, host: &str, date: &str, body: Option<&[u8]>) -> anyhow::Result<(String,String)> {
    let digest = body.map(digest_header);
    let headers_list = if digest.is_some() { "(request-target) host date digest" } else { "(request-target) host date" };
    let ss = build_signing_string(method, path_q, host, date, digest.as_deref());
    let sig = sign_string(private_pem, &ss)?;
    let header = build_signature_header(key_id, headers_list, &sig);
    Ok((header, digest.unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*; use crate::activitypub::keys::generate_keypair;
    #[test]
    fn roundtrip() {
        let (priv_pem, pub_pem)=generate_keypair().unwrap();
        let ss="(request-target): post /inbox\nhost: example.com\ndate: Thu, 01 Jan 2026 00:00:00 GMT";
        let sig=sign_string(&priv_pem, ss).unwrap();
        assert!(verify_signature(&pub_pem, ss, &sig).unwrap());
        assert!(!verify_signature(&pub_pem, &format!("{}x", ss), &sig).unwrap());
    }
}
