//! What:
//! The text a shadow prints about its post-run cleanup of superseded cached crates.
//! Why:
//! A run that removes many files must say that this is cleanup,
//! after the mpm command,
//! and not list every path
//! unless asked;
//! a failed removal is always said,
//! with its reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function cleanupLines(report: PruneReport, cargoHome: string, verbose: boolean): string[]
//! ```

/// Imports the owned and borrowed path types.
use std::path::Path;

/// Imports the report of a prune.
use crate::prune::PruneReport;

/// The environment variable that asks for one line per removed path.
pub const VERBOSE_ENV: &str = "MPM_SHADOW_VERBOSE";

/// The units of `format_size`,
/// smallest first;
/// each is 1024 times the one before.
const SIZE_UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

/// How many of the smaller unit make one of the next.
const UNIT_STEP: f64 = 1024.0;

/// What:
/// A byte count in the largest binary unit that keeps it at or above one,
/// with one decimal place (`12.3 MiB`);
/// plain bytes below one KiB.
/// Why:
/// The summary line gives a reader a sense of how much was removed without a long number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function formatSize(bytes: number): string
/// ```
pub fn format_size(bytes: u64) -> String {
    if bytes < UNIT_STEP as u64 {
        return format!("{bytes} B");
    }
    let mut value: f64 = bytes as f64;
    let mut unit: usize = 0;
    while value >= UNIT_STEP && unit + 1 < SIZE_UNITS.len() {
        value /= UNIT_STEP;
        unit += 1;
    }
    return format!("{value:.1} {}", SIZE_UNITS[unit]);
}

/// What:
/// The lines to print on standard error for a cleanup:
/// nothing when it removed and failed nothing;
/// otherwise one summary line that says it is post-run cleanup after the mpm command,
/// then (only when `verbose`) one `deleted stale` line per removed path,
/// then (always) one line per path that could not be removed,
/// with the system's reason.
/// Why:
/// A run that removes many paths must not bury the mpm command's own output,
/// and must not read as part of the command;
/// a failure is never hidden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function cleanupLines(report: PruneReport, cargoHome: string, verbose: boolean): string[]
/// ```
pub fn cleanup_lines(report: &PruneReport, cargo_home: &Path, verbose: bool) -> Vec<String> {
    if report.deleted.is_empty() && report.failed.is_empty() {
        return Vec::new();
    }
    let noun: &str = if report.deleted.len() == 1 { "copy" } else { "copies" };
    let mut summary: String = format!(
        "mpm-shadow: post-run cleanup after the mpm command (not part of it): removed {} superseded cached {noun} of this repository's crates ({}) under {}",
        report.deleted.len(),
        format_size(report.bytes),
        cargo_home.join("registry").display()
    );
    if !report.failed.is_empty() {
        summary.push_str(&format!("; {} could not be removed (listed below)", report.failed.len()));
    }
    if !verbose && !report.deleted.is_empty() {
        summary.push_str(&format!("; set {VERBOSE_ENV}=1 to list every path"));
    }
    let mut lines: Vec<String> = vec![summary];
    if verbose {
        for path in &report.deleted {
            lines.push(format!("mpm-shadow: deleted stale {}", path.display()));
        }
    }
    for (path, reason) in &report.failed {
        lines.push(format!("mpm-shadow: could not remove {}: {reason}", path.display()));
    }
    return lines;
}
