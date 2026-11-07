//! What:
//! Turns a scan into the text the shadow writes to standard error before it runs pnpm.
//! Why:
//! The message is the whole product of the shadow,
//! so it names the affected files,
//! says what is allowed,
//! says what
//! the shadow does and does not do,
//! and says how to make it stop (fix the manifests;
//! there is no switch).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // report.ts
//! export function renderReport(root: string, report: ScanReport): string | undefined { ... }
//! ```

/// Imports the borrowed path type.
use std::path::Path;

/// Imports the scan result.
use crate::scan::ScanReport;

/// The prefix every line carries.
/// It does not name the wrapper:
/// the wrapper names itself only after `--version` and
/// `--help` (`shadow_notice.rs`),
/// and these lines name the manifests and the rule instead.
pub const LINE_PREFIX: &str = "warning: ";

/// What:
/// `pub const RULE_SUMMARY: &str = "...";` is the closing paragraph:
/// the rule in one sentence,
/// then what the shadow
/// does and does not do.
/// Why:
/// A reader who sees only one finding still learns every valid remedy and that pnpm was not stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const RULE_SUMMARY = '...';
/// ```
pub const RULE_SUMMARY: &str = "the repository allows `catalog:`, `catalog:<name>` or exactly `workspace:*` in dependencies, \
devDependencies and optionalDependencies, and `*`, `catalog:` or `catalog:<name>` in peerDependencies, and allows \
only package.json manifests. This is advice only: pnpm runs next with its own arguments and its own exit status. \
The warning appears on every run of a command that resolves or changes dependencies and has no switch to turn it \
off; the way to silence it is to fix the manifests named above.";

/// What:
/// `pub fn render_report(root: &Path, report: &ScanReport) -> Option<String>` returns the text to print,
/// or `None`
/// when the scan found nothing to say.
/// Why:
/// A clean workspace stays silent,
/// so the warning means something when it appears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function renderReport(root: string, report: ScanReport): string | undefined
/// ```
pub fn render_report(root: &Path, report: &ScanReport) -> Option<String> {
    let problem_count: usize = report.findings.len() + report.notes.len();
    if problem_count == 0 {
        return None;
    }
    let noun: &str = if problem_count == 1 { "problem" } else { "problems" };

    let mut text: String = String::new();
    text.push_str(&format!(
        "{LINE_PREFIX}{problem_count} {noun} in the package manifests of the pnpm workspace at {}; pnpm runs anyway.\n",
        root.display()
    ));
    for finding in &report.findings {
        text.push_str(&format!("{LINE_PREFIX}{finding}\n"));
    }
    for note in &report.notes {
        text.push_str(&format!("{LINE_PREFIX}{note}\n"));
    }
    text.push_str(&format!("{LINE_PREFIX}{RULE_SUMMARY}\n"));
    return Some(text);
}
