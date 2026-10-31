//! What:
//!  Write paired sparse trees of actual and expected content for the drifted files.
//! Why:
//!  `check --expected-dir <dir>` lets `diff -ru <dir>/actual <dir>/expected`,
//! `git diff --no-index` or any diff viewer
//!  render the drift,
//! so file-enforcer implements no diff of its own.
//! Removals and mode changes have no content to
//!  show,
//! so they are events only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function writeExpectedTrees(dir: string, root: string, drifts: Drift[]): Array<[string, string]>
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Drift`,
/// `DriftReason` from `crate::drift`.
use crate::drift::{Drift, DriftReason};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// What:
///  Check the directory is absent or empty,
/// and disjoint from the worktree.
/// Why:
///  Writing diffs into the worktree would create the very files the tool then reports as drift,
/// and a non-empty
///  directory could hold earlier output that would mislead the diff.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function validateExpectedDir(dir: string, root: string): void
/// ```
pub fn validate_expected_dir(dir: &Path, root: &Path) -> Result<PathBuf, Diagnostic> {
    let mut absolute_dir = dir.to_path_buf();
    if absolute_dir.is_relative() {
        match std::env::current_dir() {
            Ok(current) => absolute_dir = current.join(dir),
            Err(error) => {
                let message = format!("cannot resolve --expected-dir {}: {error}", dir.display());
                return Err(Diagnostic::new("expected-dir-invalid", &message));
            }
        }
    }
    let canonical_root = fs::canonicalize(root).unwrap_or_else(|_| return root.to_path_buf());
    let mut probe = absolute_dir.clone();
    let mut suffix = PathBuf::new();
    while !probe.exists() {
        match (probe.parent(), probe.file_name()) {
            (Some(parent), Some(name)) => {
                suffix = Path::new(name).join(&suffix);
                probe = parent.to_path_buf();
            }
            _ => break,
        }
    }
    let canonical_dir = fs::canonicalize(&probe).unwrap_or(probe).join(suffix);
    if canonical_dir.starts_with(&canonical_root) || canonical_root.starts_with(&canonical_dir) {
        let message = format!(
            "--expected-dir {} overlaps the worktree {}; pick a directory outside it",
            dir.display(),
            root.display()
        );
        return Err(Diagnostic::new("expected-dir-invalid", &message));
    }
    if let Ok(mut entries) = fs::read_dir(&canonical_dir)
        && entries.next().is_some()
    {
        let message = format!("--expected-dir {} is not empty; pick an absent or empty directory", dir.display());
        return Err(Diagnostic::new("expected-dir-invalid", &message));
    }
    return Ok(canonical_dir);
}

/// What:
///  Write `actual/<path>` and `expected/<path>` for every drift that has content to show;
/// return the pairs written.
///  `Vec<(String, String)>` pairs the drifted path with the directory the trees live in.
/// Why:
///  A missing file has no actual side,
/// so only the expected side exists and diff tools show an addition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeExpectedTrees(dir: string, drifts: Drift[]): string[]
/// ```
pub fn write_expected_trees(dir: &Path, drifts: &[Drift]) -> Result<Vec<String>, Diagnostic> {
    let mut written: Vec<String> = Vec::new();
    for drift in drifts {
        let has_content = matches!(drift.reason, DriftReason::Content | DriftReason::Missing | DriftReason::ReplaceLink);
        if !has_content {
            continue;
        }
        for (side, bytes) in [("actual", &drift.actual), ("expected", &drift.expected)] {
            let Some(content) = bytes else { continue };
            let target = dir.join(side).join(&drift.path);
            if let Some(parent) = target.parent()
                && let Err(error) = fs::create_dir_all(parent)
            {
                let message = format!("cannot create {}: {error}", parent.display());
                return Err(Diagnostic::new("io-failure", &message));
            }
            if let Err(error) = fs::write(&target, content) {
                let message = format!("cannot write {}: {error}", target.display());
                return Err(Diagnostic::new("io-failure", &message));
            }
        }
        written.push(drift.path.clone());
    }
    return Ok(written);
}
