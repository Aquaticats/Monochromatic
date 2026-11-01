//! Stage two of the builtin-baseline generation chain:
//!  rewrites the stage-one
//! intermediate into the forbidden-regex dialect and writes the embedded baseline.
//!
//! What:
//!  reads the tail-format intermediate the TS porter (stage one,
//! `package/forbidden-strings/cli/src/mise.port-betterleaks.ts`) wrote to
//! `<repo>/.cache/forbidden-strings-builtin-stage1.txt`,
//!  rewrites every
//! `/PATTERN/FLAGS` line into the restricted dialect while passing every other line
//! (section headers,
//!  comments,
//!  blanks) through byte-identically on its original
//! 1-based line,
//!  verifies each ported rule through strict
//! `forbidden_regex::RegexSet::new`,
//!  proves the whole output loads through the
//! scanner's own tail-format loader,
//!  and writes
//! `<repo>/package/forbidden-strings/cli/data/builtin-rules.txt`.
//!  Why:
//!  the committed
//! baseline stays reproducible from the upstream TOML;
//!  run both stages via
//! `mise run //package/forbidden-strings/cli:generate:rules`.
//!  This bin was first
//! written for the one-shot #376 port,
//!  removed after that cutover,
//!  and recovered as
//! the standing stage two when the baseline adopted the tail rule-file format.
//!
//! It reuses the sidecar's `normalize` (dialect normalizer) and `port` (its `class_end`
//! span helper) modules;
//!  the porting passes themselves live in the sibling
//! `src/dialectport/` modules,
//!  split by role to honor the max-lines budget.

/// Registers the shared dialect normalizer module (POSIX classes,
///  case flags,
///  capturing
/// groups,
///  quantifier bounding).
#[path = "../normalize.rs"]
mod normalize;

/// Registers the shared porter module;
///  only its `class_end` span helper is reused here,
///  but
/// `normalize` depends on the module so it must be compiled in.
///  This bin exercises a subset
/// of the module,
///  so its context-stripping `port` path is dead here.
#[allow(dead_code)]
#[path = "../port.rs"]
mod port;

/// Registers the case-expansion module:
///  the three-casing expander that rewrites inline `(?i)`
/// scopes into case-sensitive dialect before normalization runs.
#[path = "../caseexpand.rs"]
mod caseexpand;

/// Registers the escape and verbose-whitespace rewrite pass.
#[path = "../dialectport/escapefix.rs"]
mod escapefix;

/// Registers the single-atom operand wrapping and byte-cursor helpers.
#[path = "../dialectport/atomwrap.rs"]
mod atomwrap;

/// Registers the per-pattern porting pass and line-level classification.
#[path = "../dialectport/portpass.rs"]
mod portpass;

/// Imports the strict ruleset compiler used as the fail-closed verifier.
use forbidden_regex::RegexSet;

/// Imports the scanner's rule loader,
///  used to prove the written output loads
/// end-to-end through the same tail-format path the runtime uses.
use forbidden_strings::compile_from_text;

/// Imports the exit-code type returned from `main`.
use std::process::ExitCode;

/// Imports the whole-source porting pass and the per-rule classification record.
use crate::portpass::{port_source, Ported};

/// Strict-compiles every ported rule individually,
///  attributing failures to source lines.
///
/// What:
///  validates the flags slot,
///  then compiles each ported body through strict
/// `RegexSet::new` on a one-element slice (never `compile_lenient`),
///  across a pool of worker
/// threads pulling from a shared index,
///  recording the line and error of any rejection.
///  Why:
/// the port is fail-closed (every rule proven on its own,
///  zero silently dropped);
///  the
/// faithful full-context rules are individually costly to determinize,
///  so the per-rule proofs
/// are fanned out to keep the run bounded.
fn verify(rules: &[&Ported]) -> Vec<(usize, String)> {
    let failures: std::sync::Mutex<Vec<(usize, String)>> = std::sync::Mutex::new(Vec::new());
    for rule in rules {
        if rule.flags.chars().any(|f| return f != 'm' && f != 'x') {
            let mut guard = failures.lock().expect("failures mutex is not poisoned");
            guard.push((rule.line, format!("unexpected flag(s) '{}'", rule.flags)));
        }
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(1, |n| return n.get());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(rule) = rules.get(index) else {
                        break;
                    };
                    let start = std::time::Instant::now();
                    let outcome = RegexSet::new(std::slice::from_ref(&rule.ported));
                    let secs = start.elapsed().as_secs_f64();
                    if secs > 1.0 {
                        eprintln!("SLOW line {} took {secs:.1}s", rule.line);
                    }
                    if let Err(error) = outcome {
                        let mut guard = failures.lock().expect("failures mutex is not poisoned");
                        guard.push((rule.line, format!("{error}")));
                    }
                }
            });
        }
    });
    let mut out = failures.into_inner().expect("failures mutex is not poisoned");
    out.sort_by_key(|(line, _)| return *line);
    return out;
}

/// Prints one file's change breakdown and a per-changed-rule dump to stdout.
///
/// What:
///  counts rewritten and semantically-changed rules,
///  prints per-category totals,
///  and
/// emits one `CHANGE` line per changed rule for authoring the review doc.
///  Why:
///  the review
/// doc must list every semantically changed rule with before and after.
fn report(name: &str, rules: &[Ported]) {
    let rewritten = rules.iter().filter(|r| return r.ported != r.source).count();
    let semantic = rules
        .iter()
        .filter(|r| return r.case || r.quant || r.reshape || r.crlf)
        .count();
    let case = rules.iter().filter(|r| return r.case).count();
    let quant = rules.iter().filter(|r| return r.quant).count();
    let reshape = rules.iter().filter(|r| return r.reshape).count();
    let crlf = rules.iter().filter(|r| return r.crlf).count();
    let zanchor = rules.iter().filter(|r| return r.zanchor).count();
    let leadstrip = rules.iter().filter(|r| return r.leadstrip).count();
    let flags = rules.iter().filter(|r| return !r.flags.is_empty()).count();
    println!(
        "[{name}] rules={} rewritten={rewritten} semantic={semantic} (case={case} quant={quant} reshape={reshape} crlf={crlf}) zanchor={zanchor} leadstrip={leadstrip} flag_dropped={flags}",
        rules.len(),
    );
    for rule in rules {
        if rule.case || rule.quant || rule.reshape || rule.crlf || rule.zanchor || rule.leadstrip {
            let mut cats: Vec<&str> = Vec::new();
            if rule.case {
                cats.push("case");
            }
            if rule.quant {
                cats.push("quant");
            }
            if rule.reshape {
                cats.push("reshape");
            }
            if rule.crlf {
                cats.push("crlf");
            }
            if rule.zanchor {
                cats.push("zanchor");
            }
            if rule.leadstrip {
                cats.push("leadstrip");
            }
            println!(
                "CHANGE\t{name}\t{}\t{}\t{}\t{}",
                rule.line,
                cats.join(","),
                rule.source,
                rule.ported,
            );
        }
    }
}

/// Returns the repository root,
///  derived from this crate's compile-time manifest directory.
///
/// What:
///  the manifest dir is `<repo>/package/forbidden-strings/core.bench`;
///  its third
/// ancestor is the repo root.
///  Why:
///  the bin writes to fixed repo-relative paths regardless of
/// its working directory.
fn repo_root() -> std::path::PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .ancestors()
        .nth(3)
        .expect("manifest dir has a repo-root ancestor")
        .to_path_buf();
}

/// Ports the stage-one intermediate,
///  verifies the set,
///  and writes the live baseline.
///
/// What:
///  reads the tail-format intermediate stage one wrote under `.cache/`,
///  ports every
/// `/PATTERN/FLAGS` body,
///  strict-compiles each ported rule with per-line attribution,
/// proves the whole output loads through the scanner's own tail-format loader (headers,
/// name uniqueness,
///  per-section classification),
///  and only then overwrites the committed
/// `data/builtin-rules.txt`.
///  Why:
///  the embedded baseline must stay reproducible and the
/// write must be fail-closed at both the per-rule and whole-file layers.
fn main() -> ExitCode {
    let root = repo_root();
    let stage1_path = root.join(".cache/forbidden-strings-builtin-stage1.txt");
    let stage1_text = std::fs::read_to_string(&stage1_path).unwrap_or_else(|error| {
        panic!(
            "read stage-one intermediate {} (run the TS porter first): {error}",
            stage1_path.display(),
        )
    });

    let (builtin_ported, builtin_rules) = port_source(&stage1_text);

    report("builtin", &builtin_rules);

    // Strict-compile every ported rule (fail-closed); attribute any failure to a line.
    let combined: Vec<&Ported> = builtin_rules.iter().collect();
    let start = std::time::Instant::now();
    let failures = verify(&combined);
    let secs = start.elapsed().as_secs_f64();

    if !failures.is_empty() {
        for (line, error) in &failures {
            eprintln!("FAIL line {line}: {error}");
        }
        return ExitCode::FAILURE;
    }

    // Whole-file proof through the scanner's own loader: catches header grammar
    // breaks, duplicate section names, and per-section classification errors the
    // per-rule compile pass cannot see. The redacted error carries no rule text.
    let set = match compile_from_text(&builtin_ported) {
        Ok(set) => set,
        Err(error) => {
            eprintln!("FAIL tail-format load of ported output: {error}");
            return ExitCode::FAILURE;
        }
    };
    // Every section holds exactly one regex rule, so the loaded set's size must
    // equal the ported rule count; a mismatch means a section was dropped or split.
    if set.len() != builtin_rules.len() {
        eprintln!(
            "FAIL loaded rule count {} != ported rule count {}",
            set.len(),
            builtin_rules.len(),
        );
        return ExitCode::FAILURE;
    }

    let builtin_out = root.join("package/forbidden-strings/cli/data/builtin-rules.txt");
    std::fs::write(&builtin_out, &builtin_ported).expect("write builtin baseline file");

    println!(
        "OK: all {} ported rules compile strictly and the tail file loads ({secs:.1}s wall, parallel); wrote {}",
        combined.len(),
        builtin_out.display(),
    );
    return ExitCode::SUCCESS;
}
