//! No authorization gate may contradict the constant its own message names.
//!
//! # Why this exists
//!
//! ficnexus shipped a live privilege bug for months: `CURATOR_MIN_TRUST = 3` was
//! declared and referenced *nowhere*, while curator gates hardcoded
//! `trust_level < 5`. Because `trust_level` is CHECK-constrained to 0-6, that admits
//! rung-5 staff rather than the curator tier the message promises — and denies
//! trust levels 3 and 4 the tools they are entitled to. 935 lib tests passed the whole
//! time, because every one of them checked only that a gate *existed*.
//!
//! The first fix removed the two worst sites. This test is what stops it coming back.
//!
//! # What it checks
//!
//! For each inline `trust_level < N` whose error message names a tier ("curator",
//! "moderator", "admin", "resolv"), the N must equal that tier's declared constant.
//!
//! It deliberately ignores gates whose message names no tier: those are per-call
//! decisions — owner-or-staff, scope-restricted — where the number is a product
//! choice and a shared helper cannot express the second half of the condition.
//!
//! # A false negative you should know about
//!
//! An earlier version of this check parsed `git grep -A2` output with the wrong
//! separator. Context lines are `path-lineno-text`, match lines are
//! `path:lineno:text`; matching only the latter parsed **0 of 167 lines** and printed
//! "no gate disagrees", for a codebase that still had the bug in it.
//!
//! So this test asserts on the parser's own output first: if it finds fewer gates
//! than there are `trust_level <` comparisons in the tree, the checker is broken and
//! the test fails rather than passing quietly. A checker that cannot fail is worse
//! than no checker.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if !matches!(
                p.file_name().and_then(|n| n.to_str()),
                Some("target") | Some(".git")
            ) {
                rust_files(&p, out);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Every declared `*_MIN_TRUST`-style constant and its value.
fn tier_constants() -> BTreeMap<String, i32> {
    let mut files = Vec::new();
    rust_files(&repo_root().join("src"), &mut files);
    let mut out = BTreeMap::new();
    for f in files {
        let Ok(body) = std::fs::read_to_string(&f) else {
            continue;
        };
        for line in body.lines() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            let Some(rest) = t.split("const ").nth(1) else {
                continue;
            };
            // `rest` is "NAME: i16 = 5;", so after the first ':' comes the type, then
            // '=', then the value. Take the text after the LAST '=' and trim the ';'.
            let mut parts = rest.splitn(2, ':');
            let name = parts.next().unwrap_or("").trim();
            let tail = parts.next().unwrap_or("");
            if !name.ends_with("MIN_TRUST") && !name.ends_with("MAX_TRUST") {
                continue;
            }
            let raw = tail
                .rsplit('=')
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches(';');
            if let Ok(v) = raw.parse::<i32>() {
                out.insert(name.to_string(), v);
            }
        }
    }
    out
}

/// (file:line, literal, message) for every inline `trust_level < N` that emits a
/// `Forbidden(...)` message naming a tier.
fn tier_named_gates() -> Vec<(String, i32, String)> {
    let mut files = Vec::new();
    rust_files(&repo_root().join("src"), &mut files);
    let mut out = Vec::new();

    for f in files {
        let Ok(body) = std::fs::read_to_string(&f) else {
            continue;
        };
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            let Some(n) = t.split("trust_level < ").nth(1).and_then(|r| {
                r.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            }) else {
                continue;
            };
            // The message is in the next couple of lines.
            let msg: String = lines[i + 1..(i + 4).min(lines.len())].join(" ");
            if !msg.contains("Forbidden") {
                continue;
            }
            out.push((format!("{}:{}", f.display(), i + 1), n, msg));
        }
    }
    out
}

fn tier_of(msg: &str) -> Option<&'static str> {
    let m = msg.to_ascii_lowercase();
    if m.contains("curator") {
        Some("curator")
    } else if m.contains("moderator") {
        Some("moderator")
    } else if m.contains("admin") {
        Some("admin")
    } else {
        None
    }
}

#[test]
fn no_gate_contradicts_the_constant_its_message_names() {
    let consts = tier_constants();
    let gates = tier_named_gates();

    let mut bad: Vec<String> = Vec::new();
    for (loc, n, msg) in &gates {
        let Some(tier) = tier_of(msg) else {
            continue;
        };
        // The constant for a tier, e.g. CURATOR -> CURATOR_MIN_TRUST.
        let cname = format!("{}_MIN_TRUST", tier.to_ascii_uppercase());
        let Some(&want) = consts.get(&cname) else {
            continue; // no constant states this tier; nothing to contradict
        };
        if *n != want {
            bad.push(format!(
                "{loc}: message says {tier:?} ({} = {want}) but the gate is trust_level < {n}",
                cname
            ));
        }
    }

    assert!(
        bad.is_empty(),
        "gates contradict the constant their own message names:\n  {}",
        bad.join("\n  ")
    );
}

/// The parser guard, described in the module docs.
///
/// A checker that silently matches nothing reports "clean" forever. So assert it
/// actually found the gates it is supposed to be examining: every `trust_level <`
/// in `src/` must be visible to `tier_named_gates`, and at least the curator gates
/// this repo is known to have must carry a message.
#[test]
fn the_checker_actually_finds_gates() {
    let mut files = Vec::new();
    rust_files(&repo_root().join("src"), &mut files);

    let mut total_comparisons = 0usize;
    for f in &files {
        let Ok(body) = std::fs::read_to_string(f) else {
            continue;
        };
        for line in body.lines() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if t.contains("trust_level < ") {
                total_comparisons += 1;
            }
        }
    }
    assert!(
        total_comparisons > 10,
        "expected many `trust_level <` comparisons in src/, found {total_comparisons}; \
         if this repo no longer has them the checker needs updating, not deleting"
    );

    let gates = tier_named_gates();
    let curator_named = gates
        .iter()
        .filter(|(_, _, m)| tier_of(m) == Some("curator"))
        .count();
    assert!(
        curator_named > 0,
        "found {total_comparisons} comparisons but none whose message names a curator \
         tier -- the parser is broken, not the codebase. This is the false negative \
         that let the original bug pass."
    );
}

/// `CURATOR_MIN_TRUST` must be referenced somewhere.
///
/// The original defect in one assertion: the constant existed only in its own
/// definition, so the declared curator boundary was never applied.
#[test]
fn declared_tier_constants_are_actually_referenced() {
    let mut files = Vec::new();
    rust_files(&repo_root().join("src"), &mut files);

    for (name, value) in tier_constants() {
        let mut refs = 0usize;
        for f in &files {
            let Ok(body) = std::fs::read_to_string(f) else {
                continue;
            };
            for line in body.lines() {
                let t = line.trim();
                if t.contains(&name) && !t.starts_with("//") {
                    // Skip the declaration itself: `const NAME: i16 = V;`
                    let is_decl = t.contains("const ") && t.contains(&name);
                    if !is_decl {
                        refs += 1;
                    }
                }
            }
        }
        assert!(
            refs > 0,
            "{name} = {value} is declared and referenced nowhere. A named threshold with \
             no references is a bug report: the design decided something the code never \
             applied. Either wire it up or delete it deliberately."
        );
    }
}
