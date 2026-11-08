//! What:
//! The text `install` prints for a sync:
//! the line for a tool it locked,
//! the line for a plan change it wrote into the lock,
//! and the whole report.
//! Why:
//! `sync.rs` holds the sync itself and stays under the file-size limit by keeping the wording in a file of its own;
//! the report type stays in `sync.rs`,
//! which uses nothing from here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function formatSyncReport(report: SyncReport): string
//! ```

/// Imports the report the text describes.
use super::sync::SyncReport;
/// Imports the lines about old copies of tools that the report ends with.
use super::sync_old_copies::old_copy_lines_text;

/// The line `install` prints for a plan change it wrote into the lock without a lookup.
pub fn rewritten_line(description: &str) -> String {
    return format!("toolchain install: lock updated from the plan: {description}\n");
}

/// The line `install` prints for a tool it locked itself.
pub fn locked_line(id: &str, version: &str) -> String {
    return format!(
        "toolchain install: locked {id} {version} (the plan was ahead of the lock; resolved alone, no other lock row moved)\n"
    );
}

/// The text `install` prints for a report (the tools it locked first,
/// then warnings,
/// the counts and each failure).
pub fn format_sync_report(report: &SyncReport) -> String {
    let mut text: String = String::new();
    for (id, version) in &report.resolved {
        text.push_str(&locked_line(id, version));
    }
    for description in &report.rewritten {
        text.push_str(&rewritten_line(description));
    }
    for warning in &report.warnings {
        text.push_str(&format!("toolchain install: warning: {warning}\n"));
    }
    text.push_str(&format!(
        "toolchain install: {} installed, {} unchanged, {} failed\n",
        report.installed.len(),
        report.unchanged.len(),
        report.failed.len()
    ));
    for line in &report.reconciled {
        text.push_str(&format!("toolchain install: {line}\n"));
    }
    text.push_str(&old_copy_lines_text(report));
    for line in &report.retired_links {
        text.push_str(&format!("toolchain install: {line}\n"));
    }
    if !report.exported_env.is_empty() {
        text.push_str(&format!("toolchain install: exported to mise: {}\n", report.exported_env.join(", ")));
    }
    for (id, message) in &report.failed {
        text.push_str(&format!("toolchain install: {id} failed: {message}\n"));
    }
    if !report.missing_bins.is_empty() {
        text.push_str(&format!("toolchain install: unresolved bins: {}\n", report.missing_bins.join(", ")));
    }
    return text;
}
