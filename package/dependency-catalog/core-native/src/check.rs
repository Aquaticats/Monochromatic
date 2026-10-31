//! What:
//! Check mode:
//! which files differ from what the sync would write.
//! Why:
//! CI and `file-enforcer check` must fail on drift without touching a file;
//! this is the same sync with the
//! unchanged files filtered out,
//! so check and write can never disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // check mode: run syncRustCatalog, then report every file whose new text differs from its current text
//! export function checkRustCatalog(args: SyncInput): DriftReport
//! ```

/// Imports `crate::error::Error`.
use crate::error::Error;
/// Imports `crate::sync::{FileResult, SyncInput, SyncOutput, sync}`.
use crate::sync::{FileResult, SyncInput, SyncOutput, sync};

/// What:
/// `pub struct DriftReport { ... }` lists the files that differ from the sync's expected contents.
/// Why:
/// `drifts` holds only drifting files,
/// each with the `changes` that explain it;
/// an empty list means the
/// repository is in sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DriftReport = { drifts: FileResult[]; affectedCrates: string[]; warnings: string[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftReport {
    /// Files whose expected text differs from their actual text:
    /// the catalog first,
    /// then manifests in input order.
    pub drifts: Vec<FileResult>,
    /// Crate directories whose generated `[workspace.dependencies]` table would change.
    pub affected_crates: Vec<String>,
    /// Non-fatal notes from the sync.
    pub warnings: Vec<String>,
}

/// Methods of `DriftReport`.
impl DriftReport {
    /// Whether every file already holds its expected text.
    pub fn is_clean(&self) -> bool {
        return self.drifts.is_empty();
    }
}

/// What:
/// `pub fn check(input: &SyncInput) -> Result<DriftReport, Error>` runs the sync and reports the drift.
/// Why:
/// A catalog or manifest problem that stops the sync is an `Err`,
/// exactly as in write mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkRustCatalog(args: SyncInput): DriftReport // throws like syncRustCatalog
/// ```
pub fn check(input: &SyncInput) -> Result<DriftReport, Error> {
    let output: SyncOutput = sync(input)?;
    let drifts: Vec<FileResult> = output.drift().into_iter().cloned().collect();
    return Ok(DriftReport {
        drifts,
        affected_crates: output.affected_crates,
        warnings: output.warnings,
    });
}
