//! The curator threshold is `CURATOR_MIN_TRUST`, not a literal `5`.
//!
//! # Why this exists
//!
//! `src/tags/curator.rs` and `src/routes/curator_content.rs` each gated on
//! `user.trust_level < 5` while `CURATOR_MIN_TRUST` sat at 3 in
//! `src/services/trust.rs` — referenced *nowhere outside its own definition*.
//! Trust level is CHECK-constrained to 0-6, so 5 is a staff rung, not the
//! curator boundary the constant names.
//!
//! The live effect: trust-level 3 and 4 users were refused curator tools they
//! are entitled to. That is a live authorisation bug, not a tidy-up, and it
//! survived 935 passing lib tests because no test looked at the threshold.
//!
//! # What this pins
//!
//! 1. `require_curator` admits at `CURATOR_MIN_TRUST` and refuses below it.
//! 2. No source file gates on a bare `trust_level < 5` for curator tools.
//!
//! The second half matters more than the first. A test on the helper alone
//! passes while some call site keeps its own literal, which is exactly how the
//! two diverged in the first place.

use fichub::routes::auth::AuthUser;
use fichub::services::trust::{CURATOR_MIN_TRUST, require_curator};

fn user_with_trust(trust_level: i16) -> AuthUser {
    AuthUser {
        user_id: Some(1),
        username: Some("tester".into()),
        trust_level,
        level: 50,
        is_admin: false,
    }
}

#[test]
fn curator_gate_admits_at_the_constant() {
    assert!(
        require_curator(&user_with_trust(CURATOR_MIN_TRUST)).is_ok(),
        "trust level {CURATOR_MIN_TRUST} is the stated curator minimum and must be admitted"
    );
}

#[test]
fn curator_gate_refuses_below_the_constant() {
    let err = require_curator(&user_with_trust(CURATOR_MIN_TRUST - 1))
        .expect_err("one below the minimum must be refused");
    assert!(
        err.to_string().contains("Curator"),
        "expected a curator-specific message, got: {err}"
    );
}

#[test]
fn curator_gate_refuses_anonymous() {
    let anon = AuthUser {
        user_id: None,
        ..Default::default()
    };
    assert!(
        require_curator(&anon).is_err(),
        "an anonymous caller must not pass a curator gate"
    );
}

/// A curator is not an administrator.
///
/// This is why `require_curator` is a separate function rather than a call to
/// `require_admin_tier`: routing the curator gates through the admin helper would
/// promote curators to the admin tier. `trust_level` is bounded 0-6 and `is_admin`
/// is a separate token claim, so the two can disagree -- and here they do.
///
/// The reverse direction is checked in `admin_api`, which reaches real admin routes
/// with a token. Here the claim is enough: a caller carrying `is_admin` but no trust
/// is exactly the shape a forged-or-mistoken flag takes.
#[test]
fn a_curator_carries_no_admin_claim() {
    let curator = user_with_trust(CURATOR_MIN_TRUST);
    assert!(require_curator(&curator).is_ok());
    assert!(
        !curator.is_admin,
        "a trust-level curator must not carry the admin claim; if this fails the \
         tiers have merged"
    );
}

#[test]
fn an_admin_flag_does_not_imply_curator_trust() {
    // Admin is a flag, curator is a rung. An admin with low trust still passes the
    // admin gate; this asserts the curator helper is not quietly reading is_admin,
    // which is the mistake the shared-helper refactor invited.
    let admin_low_trust = AuthUser {
        user_id: Some(1),
        username: Some("admin".into()),
        trust_level: 0,
        level: 100,
        is_admin: true,
    };
    assert!(
        require_curator(&admin_low_trust).is_err(),
        "require_curator must read trust_level, not is_admin"
    );
}

/// No curator gate may hardcode the number 5.
///
/// The bug was two literals in two files disagreeing with one constant in a third.
/// A test on the helper alone passes while a call site keeps its own literal, which
/// is exactly how the two diverged. So this checks the call sites.
#[test]
fn no_curator_gate_hardcodes_a_trust_literal() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders: Vec<String> = Vec::new();

    for rel in ["src/tags/curator.rs", "src/routes/curator_content.rs"] {
        let Ok(body) = std::fs::read_to_string(root.join(rel)) else {
            panic!("cannot read {rel}; the check must not silently pass");
        };
        for (i, line) in body.lines().enumerate() {
            if line.contains("trust_level <") {
                offenders.push(format!("{rel}:{}: {}", i + 1, line.trim()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "curator gates must call require_curator, not compare trust_level: {offenders:?}"
    );
}
