//! What:
//! The two places where `install` meets the cleanup of old copies of this repository's tools:
//! running it after the installs,
//! and printing the lines it produced.
//! Why:
//! Kept apart from `sync.rs` so the installer stays about installing,
//! and so the order is one readable fact:
//! the old copies are judged only after the managed ones and this tree's own links exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync-old-copies.ts
//! export function retireAfterInstall(ctx: Context, report: SyncReport): void
//! export function oldCopyLinesText(report: SyncReport): string
//! ```

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `retire_old_copies` from `super::old_copy_apply`.
use super::old_copy_apply::retire_old_copies;
/// Imports `SyncReport` from `super::sync`.
use super::sync::SyncReport;

/// What:
/// Runs the cleanup of old copies and folds its result into the report:
/// its lines,
/// its loud warnings for copies that stay,
/// and its failures (which make `install` exit 1).
/// Why:
/// A tool whose install failed in this run is passed in `report.failed`,
/// so its old copy is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireAfterInstall(ctx: Context, report: SyncReport): void
/// ```
pub fn retire_after_install(ctx: &Context, report: &mut SyncReport) {
    let retired = retire_old_copies(ctx, &report.failed);
    report.old_copies = retired.lines;
    report.warnings.extend(retired.warnings);
    report.failed.extend(retired.failed);
}

/// The lines `install` prints for the cleanup of old copies,
/// one per action,
/// in the order they happened.
pub fn old_copy_lines_text(report: &SyncReport) -> String {
    let mut text: String = String::new();
    for line in &report.old_copies {
        text.push_str(&format!("toolchain install: {line}\n"));
    }
    return text;
}
