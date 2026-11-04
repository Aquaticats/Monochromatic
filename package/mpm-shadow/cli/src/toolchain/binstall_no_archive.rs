//! What:
//! The failure a person reads when `cargo binstall` finds no usable release archive for a crate this repository
//! publishes,
//! with compiling switched off.
//! Why:
//! cargo-binstall prints its reasons as `WARN resolve:` and `ERROR resolve:` lines and then stops with
//! `Fallback to cargo-install is disabled`;
//! alone, that last line names no cause and no way forward.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // binstall-no-archive.ts
//! export function noArchiveFailure(toolId: string, crate: string, version: string, outcome: RunOutcome): Error | undefined
//! ```

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `RunOutcome` from `super::runner`.
use super::runner::RunOutcome;

/// The text cargo-binstall stops with when every way to fetch a prebuilt archive failed and compiling is off.
pub const COMPILE_DISABLED_TEXT: &str = "Fallback to cargo-install is disabled";

/// The markers of the lines in which cargo-binstall explains why one fetcher did not yield an archive.
const REASON_MARKERS: [&str; 2] = ["WARN resolve:", "ERROR resolve:"];

/// The characters cargo-binstall's error report draws around wrapped text.
const REPORT_DRAWING: [&str; 3] = ["\u{2502}", "\u{d7}", "\u{2570}\u{2500}\u{25b6}"];

/// Whether cargo-binstall's output says it stopped because compiling is disabled,
/// however its error report wrapped the sentence.
fn stopped_for_disabled_compile(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().filter(|word| return !REPORT_DRAWING.contains(word)).collect();
    return words.join(" ").contains(COMPILE_DISABLED_TEXT);
}

/// The distinct reason lines of cargo-binstall's output,
/// in the order it printed them.
fn reason_lines(text: &str) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if REASON_MARKERS.iter().any(|marker| return trimmed.contains(marker)) && !reasons.iter().any(|seen| return seen == trimmed) {
            reasons.push(trimmed.to_string());
        }
    }
    return reasons;
}

/// What:
/// The failure for a repository crate whose `cargo binstall` stopped because no archive fit and compiling is off:
/// the tool,
/// the crate,
/// the version,
/// cargo-binstall's own reason lines,
/// and every way forward.
/// `None` when the output says anything else (a network failure,
/// a missing crate),
/// which keeps the generic failure text.
/// Why:
/// A repository crate is installed only from its release archive,
/// so the person needs to know the archive is the thing to fix,
/// and the reasons show whether it is missing,
/// lacks a program,
/// or could not be checked in time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function noArchiveFailure(toolId: string, crate: string, version: string, outcome: RunOutcome): Error | undefined
/// ```
pub fn no_archive_failure(tool_id: &str, crate_name: &str, version: &str, outcome: &RunOutcome) -> Option<ToolchainError> {
    let text: String = format!("{}\n{}", outcome.stdout, outcome.stderr);
    if !stopped_for_disabled_compile(&text) {
        return None;
    }
    let reasons: Vec<String> = reason_lines(&text);
    let mut lines: Vec<String> = vec![
        format!(
            "cargo binstall found no prebuilt release archive it could use for {tool_id} (crate {crate_name}, version {version}) on this platform."
        ),
        "This repository publishes the crate, so it is installed only from its release archive and is never compiled from source."
            .to_string(),
    ];
    if reasons.is_empty() {
        lines.push("cargo-binstall printed no reason line.".to_string());
    } else {
        lines.push("cargo-binstall gave these reasons:".to_string());
        lines.extend(reasons.iter().map(|reason| return format!("  {reason}")));
    }
    lines.push(format!("It then stopped with: {COMPILE_DISABLED_TEXT}."));
    lines.push("Ways to proceed:".to_string());
    lines.push(format!(
        "- Publish a release of the crate whose archive for this platform holds every program the crate declares, \
         then run `mpm-shadow update {tool_id}` and install again."
    ));
    lines.push(format!(
        "- Lock a version whose release archive fits: name that version in the plan's request for {tool_id}, \
         run `mpm-shadow update {tool_id}`, and install again."
    ));
    lines.push(
        "- If a reason above is a timeout or a rate limit, give the install a GitHub token (set GITHUB_TOKEN, or sign in with \
         `gh auth login`) and install again; raising BINSTALL_MAXIMUM_RESOLUTION_TIMEOUT also makes it wait longer for the answer."
            .to_string(),
    );
    return Some(ToolchainError::failure(lines.join("\n")));
}
