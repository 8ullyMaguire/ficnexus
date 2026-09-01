// ── Send-to-Kindle mailer ─────────────────────────────────────────────
// Thin SMTP wrapper around the `lettre` crate used by the
// POST /api/send-to-kindle route. Kept deliberately small: build a
// MIME message with an EPUB attachment, then hand it to SMTP.
//
// NOTE: with the current Cargo features (`smtp-transport`, `builder`,
// `tokio1`) no TLS backend is compiled in, so the transport is built with
// `builder_dangerous` (plain SMTP). If the deployment relay requires TLS,
// enable `tokio1-rustls-tls` (or `tokio1-native-tls`) and switch to
// `SmtpTransport::starttls_relay` / `relay` instead.

use lettre::message::{Attachment, Mailbox, Message, MultiPart, SinglePart};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::SmtpTransport;
use lettre::Transport;
use std::path::{Path, PathBuf};

/// SMTP settings for the relay (mirrors Config's SMTP_* fields).
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}

impl SmtpConfig {
    /// Build SMTP settings from env (`SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`,
    /// `SMTP_PASS`, `SMTP_FROM`, `SMTP_DIGEST_TO`). Missing host ⇒ empty
    /// (callers treat as "email disabled").
    pub fn from_env() -> Self {
        let host = std::env::var("SMTP_HOST").unwrap_or_default();
        let port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let user = std::env::var("SMTP_USER").unwrap_or_default();
        let pass = std::env::var("SMTP_PASS").unwrap_or_default();
        let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@localhost".into());
        Self {
            host,
            port,
            user,
            pass,
            from,
        }
    }

    /// Recipient for the weekly moderation digest
    /// (`SMTP_DIGEST_TO`, defaulting to the sender).
    pub fn digest_recipient(&self) -> String {
        std::env::var("SMTP_DIGEST_TO").unwrap_or_else(|_| self.from.clone())
    }
}

/// Build + send a plain (no-attachment) email with the given subject/body.
/// Pure send path for notifications and the weekly digest.
pub fn send_digest_email(cfg: &SmtpConfig, to: &str, subject: &str, body: &str) -> Result<(), String> {
    let from: Mailbox = cfg
        .from
        .parse()
        .map_err(|e| format!("invalid from '{}': {}", cfg.from, e))?;
    let to: Mailbox = to
        .parse()
        .map_err(|e| format!("invalid to '{to}': {e}"))?;
    let msg = Message::builder()
        .from(from)
        .to(to)
        .subject(subject.to_string())
        .body(body.to_string())
        .map_err(|e| format!("failed to build email: {e}"))?;
    send_email(cfg, &msg)
}

/// Everything needed to send one Kindle email.
#[derive(Debug, Clone)]
pub struct KindleMail {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub attachment_path: PathBuf,
    pub attachment_name: String,
}

/// Build the MIME message: text body + EPUB attachment.
/// Pure function — no network — so it is unit-testable without a relay.
pub fn build_kindle_message(
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
    attachment_path: &Path,
    attachment_name: &str,
) -> Result<Message, String> {
    let from: Mailbox = from
        .parse()
        .map_err(|e| format!("invalid from address '{}': {}", from, e))?;
    let to: Mailbox = to
        .parse()
        .map_err(|e| format!("invalid to address '{}': {}", to, e))?;

    let attachment_bytes = std::fs::read(attachment_path)
        .map_err(|e| format!("failed to read attachment {}: {}", attachment_path.display(), e))?;

    let content_type: ContentType = ContentType::parse("application/epub+zip")
        .unwrap_or_else(|_| ContentType::TEXT_PLAIN);

    Message::builder()
        .from(from)
        .to(to)
        .subject(subject.to_string())
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(body.to_string()),
                )
                .singlepart(
                    Attachment::new(attachment_name.to_string())
                        .body(attachment_bytes, content_type),
                ),
        )
        .map_err(|e| format!("failed to build email: {}", e))
}

/// Send a pre-built message over the configured SMTP relay.
pub fn send_email(cfg: &SmtpConfig, msg: &Message) -> Result<(), String> {
    let mut builder = SmtpTransport::builder_dangerous(&cfg.host).port(cfg.port);
    if !cfg.user.is_empty() {
        builder = builder.credentials(Credentials::new(cfg.user.clone(), cfg.pass.clone()));
    }
    let transport = builder.build();
    transport
        .send(msg)
        .map(|_| ())
        .map_err(|e| format!("SMTP send failed: {}", e))
}

/// Build and send one Kindle email (synchronous; call via spawn_blocking).
pub fn send_kindle_email(cfg: &SmtpConfig, mail: &KindleMail) -> Result<(), String> {
    let msg = build_kindle_message(
        &cfg.from,
        &mail.to,
        &mail.subject,
        &mail.body,
        &mail.attachment_path,
        &mail.attachment_name,
    )?;
    send_email(cfg, &msg)
}

/// Lightweight email-address validation. `lettre`'s parser is stricter than
/// this (it accepts display names, etc.), but for user-supplied Kindle
/// addresses we want a quick early check that returns a friendly error
/// before any EPUB work happens.
pub fn validate_email(address: &str) -> Result<(), String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Err("email address is empty".into());
    }
    if trimmed.len() > 254 {
        return Err("email address is too long".into());
    }
    let mut parts = trimmed.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if local.is_empty() || domain.is_empty() {
        return Err(format!("'{address}' is not a valid email address"));
    }
    if local.contains(' ') || domain.contains(' ') {
        return Err(format!("'{address}' is not a valid email address"));
    }
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err(format!("'{address}' is not a valid email address"));
    }
    Ok(())
}

/// Abstraction over the email transport so handlers can be tested with a
/// mock that records sends instead of requiring a real SMTP relay.
pub trait Mailer: Send + Sync {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String>;

    /// Clone as a boxed trait object (for handing to spawn_blocking).
    fn clone_box(&self) -> Box<dyn Mailer>;
}

impl Clone for Box<dyn Mailer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Mailer for Box<dyn Mailer> {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String> {
        (**self).send_kindle(mail)
    }

    fn clone_box(&self) -> Box<dyn Mailer> {
        (**self).clone_box()
    }
}

/// Real SMTP-backed mailer. Constructed once from `Config`'s SMTP_* values.
#[derive(Debug, Clone)]
pub struct SmtpMailer {
    pub cfg: SmtpConfig,
}

impl Mailer for SmtpMailer {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String> {
        send_kindle_email(&self.cfg, mail)
    }

    fn clone_box(&self) -> Box<dyn Mailer> {
        Box::new(self.clone())
    }
}

/// Test double: records every send it is asked to perform so integration
/// tests can assert on recipient, subject, and attachment without a relay.
#[derive(Debug, Default)]
pub struct MockMailer {
    pub sent: std::sync::Mutex<Vec<KindleMail>>,
    fail_count: std::sync::Mutex<u32>,
}

impl MockMailer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of the mails sent so far (cloned for assertion safety).
    pub fn sent_mails(&self) -> Vec<KindleMail> {
        self.sent.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Force the next `send_kindle` call to fail (simulates relay errors).
    pub fn fail_next(&self) {
        *self.fail_count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
    }

    fn take_failure(&self) -> bool {
        let mut c = self.fail_count.lock().unwrap_or_else(|e| e.into_inner());
        if *c > 0 {
            *c -= 1;
            true
        } else {
            false
        }
    }
}

impl Mailer for MockMailer {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String> {
        if self.take_failure() {
            return Err("mock SMTP failure".into());
        }
        self.sent
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(mail.clone());
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Mailer> {
        Box::new(MockMailer {
            sent: self.sent.lock().unwrap_or_else(|e| e.into_inner()).clone().into(),
            fail_count: std::sync::Mutex::new(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a throwaway EPUB-ish temp file.
    fn temp_epub() -> (PathBuf, TempDir) {
        let dir = TempDir::new();
        let path = dir.path().join("test.epub");
        let bytes: Vec<u8> = vec![0x50, 0x4B, 0x03, 0x04, b'f', b'a', b'k', b'e'];
        std::fs::write(&path, &bytes).unwrap();
        (path, dir)
    }

    #[test]
    fn build_kindle_message_sets_recipient_subject_and_attachment() {
        let (path, _dir) = temp_epub();
        let msg = build_kindle_message(
            "fichub@example.com",
            "my.kindle@example.com",
            "Test Story by Test Author",
            "Sent from FicNexus.",
            &path,
            "test.epub",
        )
        .expect("message builds");

        // Envelope recipient
        let to = msg.envelope().to();
        assert_eq!(to.len(), 1);
        assert_eq!(to[0].to_string(), "my.kindle@example.com");

        // Subject header lives in the headers map
        let headers = msg.headers();
        let subject = headers
            .get::<lettre::message::header::Subject>()
            .expect("subject header");
        assert_eq!(subject.as_ref(), "Test Story by Test Author");

        // Message must carry an attachment (multipart/mixed).
        // The rendered message includes the Content-Type header.
        let rendered = msg.formatted();
        let text = String::from_utf8_lossy(&rendered);
        assert!(
            text.contains("multipart/mixed"),
            "expected multipart/mixed in message headers"
        );

        // Body text should be present in the rendered message
        assert!(text.contains("Sent from FicNexus."));
        assert!(text.contains("test.epub"));
        assert!(text.contains("application/epub+zip"));
    }

    #[test]
    fn build_kindle_message_rejects_bad_addresses() {
        let (path, _dir) = temp_epub();
        assert!(build_kindle_message(
            "not-an-address",
            "my.kindle@example.com",
            "s",
            "b",
            &path,
            "test.epub",
        )
        .is_err());
        assert!(build_kindle_message(
            "fichub@example.com",
            "not-an-address",
            "s",
            "b",
            &path,
            "test.epub",
        )
        .is_err());
    }

    #[test]
    fn build_kindle_message_missing_attachment_fails() {
        assert!(build_kindle_message(
            "fichub@example.com",
            "my.kindle@example.com",
            "s",
            "b",
            Path::new("/nonexistent/definitely/missing.epub"),
            "test.epub",
        )
        .is_err());
    }

    #[test]
    fn smtp_transport_builds_with_dangerous_builder() {
        // No network happens here — builder_dangerous only constructs the
        // transport object. Verifies the wrapper's transport construction.
        let cfg = SmtpConfig {
            host: "127.0.0.1".into(),
            port: 2525,
            user: "user".into(),
            pass: "pass".into(),
            from: "fichub@example.com".into(),
        };
        let mut builder = SmtpTransport::builder_dangerous(&cfg.host).port(cfg.port);
        builder = builder.credentials(Credentials::new(cfg.user.clone(), cfg.pass.clone()));
        let transport = builder.build();
        // Transport object exists and is usable (no connection attempted).
        let _ = &transport;
    }

    #[test]
    fn validate_email_accepts_typical_addresses() {
        assert!(validate_email("my.kindle@free.kindle.com").is_ok());
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("  spaced@example.com  ").is_ok(), "trims whitespace");
    }

    #[test]
    fn validate_email_rejects_bad_addresses() {
        assert!(validate_email("").is_err(), "empty");
        assert!(validate_email("   ").is_err(), "whitespace only");
        assert!(validate_email("not-an-address").is_err(), "no @");
        assert!(validate_email("@example.com").is_err(), "empty local");
        assert!(validate_email("user@").is_err(), "empty domain");
        assert!(validate_email("user@nodot").is_err(), "domain without dot");
        assert!(validate_email("user with space@example.com").is_err(), "space in local");
        assert!(validate_email("user@exa mple.com").is_err(), "space in domain");
    }

    #[test]
    fn mock_mailer_records_recipient_subject_and_attachment() {
        let (path, _dir) = temp_epub();
        let mock = MockMailer::new();
        let mail = KindleMail {
            to: "my.kindle@example.com".into(),
            subject: "Test Story by Test Author".into(),
            body: "Sent from FicNexus.".into(),
            attachment_path: path.clone(),
            attachment_name: "test.epub".into(),
        };
        mock.send_kindle(&mail).expect("mock send succeeds");

        let sent = mock.sent_mails();
        assert_eq!(sent.len(), 1, "exactly one mail recorded");
        assert_eq!(sent[0].to, "my.kindle@example.com");
        assert_eq!(sent[0].subject, "Test Story by Test Author");
        assert_eq!(sent[0].attachment_name, "test.epub");
        assert_eq!(sent[0].attachment_path, path);
    }

    #[test]
    fn mock_mailer_can_simulate_failure() {
        let mock = MockMailer::new();
        mock.fail_next();
        let mail = KindleMail {
            to: "my.kindle@example.com".into(),
            subject: "s".into(),
            body: "b".into(),
            attachment_path: PathBuf::from("/nonexistent"),
            attachment_name: "x.epub".into(),
        };
        assert!(mock.send_kindle(&mail).is_err(), "first send fails");
        assert!(
            mock.send_kindle(&mail).is_ok(),
            "second send succeeds (one-shot failure)"
        );
        assert_eq!(mock.sent_mails().len(), 1);
    }

    #[test]
    fn mock_mailer_clone_box_preserves_records() {
        let mock = MockMailer::new();
        let mail = KindleMail {
            to: "a@example.com".into(),
            subject: "s".into(),
            body: "b".into(),
            attachment_path: PathBuf::from("/nonexistent"),
            attachment_name: "x.epub".into(),
        };
        mock.send_kindle(&mail).unwrap();
        let cloned: Box<dyn Mailer> = mock.clone_box();
        cloned.send_kindle(&mail).unwrap();
        assert_eq!(mock.sent_mails().len(), 1, "original mock unchanged");
    }
}

/// Tiny tempdir helper so the tests don't depend on the `tempfile` crate.
#[cfg(test)]
struct TempDir(PathBuf);

#[cfg(test)]
impl TempDir {
    fn new() -> Self {
        let dir =
            std::env::temp_dir().join(format!("fichub_mailer_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

#[cfg(test)]
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
