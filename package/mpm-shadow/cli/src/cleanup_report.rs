//! What:
//! The text a shadow prints about its post-run cleanup of superseded cached crates.
//! Why:
//! A run that removes many files must say that this is cleanup, after the mpm command, and not list every path
//! unless asked; a failed removal is always said, with its reason.
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

/// A byte count in the largest binary unit that keeps it above one, one decimal place (`12.3 MiB`).
pub fn format_size(_bytes: u64) -> String {
    return String::new();
}

/// The lines to print on standard error for a cleanup.
pub fn cleanup_lines(_report: &PruneReport, _cargo_home: &Path, _verbose: bool) -> Vec<String> {
    return Vec::new();
}
