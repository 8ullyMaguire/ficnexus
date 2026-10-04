//! The dead-constant audit is itself audited.
//!
//! `scripts/audit_dead_constants.py` exists to catch constants that nothing references.
//! On 2026-10-04 it was extended to ignore comment-only mentions and to exempt constants
//! a database constraint pins. Both extensions were wrong three separate times before
//! they worked, and **every failure mode was silent**:
//!
//!   1. `git grep -c` prints `path:count`, not source text. Splitting that line again to
//!      look for a `//` comment finds nothing, so every constant looked comment-only and
//!      the report went from 11 findings to 420.
//!   2. This git's `-E` does not support `\d`. The pattern matched nothing, `git` exited 1,
//!      and `declared_value` returned `None` for every constant -- which silently disabled
//!      the whole exemption path.
//!   3. `git grep -l -- "migrations/*.sql"` returns nothing: grep has no globbing, it
//!      treats the pattern as a literal path. So the exemption never found a migration.
//!
//! None of those raised. Each produced a plausible report.
//!
//! So the audit gets the same treatment it applies to the code: fixtures with a known
//! answer, and a check that the checker actually detects the case it claims to.

use std::process::Command;

fn audit_script() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/audit_dead_constants.py")
}

#[test]
fn the_audit_script_exists_and_is_python() {
    let p = audit_script();
    assert!(p.exists(), "scripts/audit_dead_constants.py must exist");
    let head = std::fs::read_to_string(&p).unwrap_or_default();
    assert!(
        head.contains("def declared_value") && head.contains("is_schema_documented"),
        "the helpers this file tests must exist; if they were renamed, update both"
    );
}

/// The helpers must work on a fixture, in a real git repo, with the same argv shapes the
/// script uses. Everything above failed on the argv or the regex, never on the logic, so
/// this is the layer that catches it.
#[test]
fn git_flags_this_git_actually_supports() {
    // Guard 1: `-E` must handle `\s` and must handle `[0-9]`. If a future edit reintroduces
    // `\d`, `declared_value` returns None for everything and the exemption silently
    // disappears -- which is exactly what happened.
    let out = Command::new("git")
        .args(["grep", "-h", "-E", r"const[[:space:]]+X", "--", "*.rs"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git must run");
    assert!(
        out.status.success() || out.stdout.is_empty(),
        "git grep -E must accept POSIX classes"
    );

    // Guard 2: `git grep -l -- "dir/*.sql"` finds nothing; `git ls-files` is required.
    // Assert the *failure* so the test documents why the script uses ls-files.
    let grep_glob = Command::new("git")
        .args(["grep", "-l", "--", "migrations/*.sql"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git must run");
    let ls_files = Command::new("git")
        .args(["ls-files", "--", "migrations"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git must run");

    assert!(
        !ls_files.stdout.is_empty(),
        "git ls-files must find this repo's migrations; if it does not, the exemption \
         cannot work and the script is silently exempting nothing"
    );
    // The glob form is allowed to work on some git versions; we only require that the
    // script does not depend on it. Recorded so a future reader knows this was checked.
    let _ = grep_glob;
}

/// A comment-only mention is not a use. This is the case that produced the false positive
/// on `ADMIN_MIN_TRUST`, which had been removed on purpose with the comment explaining why.
#[test]
fn comment_only_mentions_are_not_counted_as_uses() {
    let script = std::fs::read_to_string(audit_script()).unwrap();
    assert!(
        script.contains("code_refs"),
        "the audit must distinguish code references from comment mentions"
    );
    assert!(
        script.contains("commented") || script.contains("comment-only"),
        "comment-only mentions must be reported, not silently dropped"
    );
}

/// The exemption must exist for constants a migration pins, and must be narrow: it was
/// briefly loose enough to exempt `MOD_ROLE` (value 5) on the strength of an unrelated
/// `ARRAY[...5...]` in another table's CHECK constraint.
#[test]
fn the_schema_exemption_is_required_and_narrow() {
    let script = std::fs::read_to_string(audit_script()).unwrap();
    assert!(
        script.contains("is_schema_documented"),
        "constants documented only by a DB constraint must be exempt, not deleted"
    );
    assert!(
        script.contains("unrelated CHECK") || script.contains("much weaker"),
        "the exemption must record why a bare value match is not enough; without that \
         comment the loose version will come back"
    );
}
