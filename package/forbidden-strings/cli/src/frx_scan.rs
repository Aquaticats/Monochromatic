//! Line-based file scan against the forbidden-regex engine's batch API.
//!
//! Stage two of the engine swap (#384) replaces the aho-corasick/resharp per-file
//! scan with the engine's buffer-batch face.
//!  A file's bytes are split into lines and
//! handed to `RegexSet::line_matches(buf, starts)`,
//!  which resolves per-line rule ids
//! in one SIMD prefilter sweep.
//!  Findings emit as `PATH:LINE rule=N`,
//!  one per
//! line-and-rule pair,
//!  with no column segment (the engine reports per-line rule
//! indices,
//!  not spans).
//!
//! Line mechanics mirror the engine's contract:
//!  split on `\n`,
//!  one trailing `\r` and
//! the terminator excluded by the matcher,
//!  empty lines skipped.
//!  The batch call runs
//! under a `catch_unwind` boundary so an engine panic fails closed as a synthetic
//! finding rather than aborting the scan,
//!  preserving the scanner's fail-closed
//! guarantee against a secret-scanning gate exiting clean on an engine fault.

/// Imports the SIMD newline scan used to build the line-start offsets.
use memchr::memchr_iter;

/// Imports the unwind boundary that turns an engine panic into a fail-closed finding.
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Imports the loaded rule sets scanned against each file.
use crate::frx_load::LoadedRules;
/// Import the canonical result model shared by standalone and embedded callers.
use crate::scan_finding::ScanFinding;

/// Builds the line-start offsets `RegexSet::line_matches` requires for `buf`.
///
/// The engine's precondition is that `starts` ascends,
///  begins at 0,
///  and every offset
/// indexes within `buf`.
///  This returns 0 followed by the offset just past each `\n`,
/// dropping a final offset equal to `buf.len()` (the empty line after a trailing
/// newline):
///  the matcher would skip it anyway,
///  and omitting it keeps every offset a
/// valid in-bounds index.
///  Interior empty lines keep their offset and the matcher
/// skips them,
///  so line numbering stays aligned with the file.
fn line_starts(buf: &[u8]) -> Vec<usize> {
    let mut starts: Vec<usize> = Vec::with_capacity(buf.len() / 32 + 1);
    starts.push(0);
    for newline in memchr_iter(b'\n', buf) {
        let next = newline + 1;
        // A start equal to buf.len() is the empty line past a trailing newline; the
        // matcher skips it, and omitting it keeps every offset in bounds.
        if next < buf.len() {
            starts.push(next);
        }
    }
    return starts
}

/// Renders the stable opaque identity of one matching rule.
///
/// Named rules use their section name;
///  unnamed rules use the set's base offset.
/// This shared formatter keeps content and pathname findings consistent.
pub(crate) fn rule_token(base: usize, names: &[Option<String>], rule_id: usize) -> String {
    // A borrowed name is copied only when a finding is emitted.
    if let Some(name) = names.get(rule_id).and_then(|name| return name.as_deref()) {
        return name.to_string();
    }
    return (base + rule_id).to_string();
}

/// Runs one set's batch matcher under a fail-closed unwind boundary.
///
/// Normal `(line index, rule id)` pairs become typed content findings with one-based
/// lines and the shared opaque rule identity.
///  A caught panic becomes an explicit
/// EngineError record,
///  so neither embedded nor standalone callers can treat it as
/// a clean scan.
///  Terminal rendering happens separately.
fn scan_one_set<Match>(
    base: usize,
    names: &[Option<String>],
    matcher: Match,
) -> Vec<ScanFinding>
where
    Match: FnOnce() -> Vec<(usize, usize)> + std::panic::UnwindSafe,
{
    match catch_unwind(matcher) {
        Ok(pairs) => {
            // Own only location and rule identity; matched bytes never enter a finding.
            let mut findings: Vec<ScanFinding> = Vec::<ScanFinding>::new();
            for (line_index, rule_id) in pairs {
                findings.push(ScanFinding::Content {
                    line: line_index + 1,
                    rule: rule_token(base, names, rule_id),
                });
            }
            return findings
        }
        Err(_) => {
            // Fail closed: a caught engine panic becomes a redacted synthetic finding
            // so the run reports the file and exits non-zero instead of clean.
            return vec![ScanFinding::EngineError]
        }
    }
}

/// Scan exact file bytes against each loaded set,
///  returning only redacted structured records.
///
/// Line splitting happens once.
///  Runtime rules precede the builtin baseline,
/// and each set retains its rule-id offset and configured non-secret names.
/// Empty input yields no findings.
///  The caller supplies candidate identity and
/// a sanitized display path outside this content-matching boundary.
pub(crate) fn scan_content(buf: &[u8], loaded: &LoadedRules) -> Vec<ScanFinding> {
    if buf.is_empty() {
        return Vec::new();
    }
    let starts = line_starts(buf);
    let mut hits: Vec<ScanFinding> = Vec::<ScanFinding>::new();
    for scan_set in loaded.iter_sets() {
        // AssertUnwindSafe permits this borrowed engine boundary. Reuse is supported by
        // RuntimeRules::line_matches and RegexSet::line_matches allocating hits/candidates
        // per call; RegexSet::resolve_matches also owns fresh CheckedFull scratch.
        // The scanner boundary tests provoke a real bounds unwind and compare later scans.
        let matcher = AssertUnwindSafe(|| return scan_set.matcher.line_matches(buf, &starts));
        hits.extend(scan_one_set(scan_set.base, &scan_set.names, matcher));
    }
    return hits
}

/// What:
///  Preserve the standalone text protocol by rendering canonical content records.
/// Why:
///  Embedded callers use scan_content directly,
///  without parsing terminal strings.
///
/// In TS you\'d write (pseudocode):
/// ```ts
/// function scanFile(safeDisplayPath, bytes, loaded): string[];
/// ```
pub fn scan_file(path: &str, buf: &[u8], loaded: &LoadedRules) -> Vec<String> {
    // Vec owns the variable-length terminal records, independently of the borrowed file bytes.
    let mut lines: Vec<String> = Vec::<String>::new();
    for finding in scan_content(buf, loaded) {
        lines.push(finding.render(path));
    }
    return lines;
}

/// Registers the line-splitting edge-case and fail-closed tests (sidecar,
///  lint-exempt).
#[cfg(test)]
#[path = "frx_scan_tests.rs"]
mod tests;
