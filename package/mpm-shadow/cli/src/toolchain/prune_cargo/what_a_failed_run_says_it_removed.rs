//! What:
//! The words a failed prune leaves behind:
//! the error it stopped at,
//! restated with everything this run had already removed,
//! and the directory a recursive removal left partly gone.
//! Why:
//! The report is the only place a person learns what a failed run deleted,
//! so a failure after a removal must name what is gone and never say that nothing was deleted.
//! A caller imports from `prune_cargo` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function afterRemoving(error: ToolchainError, removed: string[]): ToolchainError;
//! function partlyRemoved(directory: string, error: Error, removed: string[]): ToolchainError;
//! ```

/// Imports the directory types a removal is held in.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// What:
/// `error`,
/// restated with what this run has already removed from the tree.
/// Why:
/// The report is the only place a person learns what a failed run deleted,
/// so a failure after a removal must name what is gone and never say that nothing was deleted.
pub(crate) fn after_removing(
    error: ToolchainError,
    removed: &[PathBuf],
) -> ToolchainError {
    if removed.is_empty() {
        return ToolchainError {
            kind: error.kind,
            message: format!("{}; nothing was deleted", error.message),
        };
    }
    let message: String = format!(
        "{}; already removed: {}",
        error.message,
        paths_named(removed)
    );
    return ToolchainError {
        kind: error.kind,
        message,
    };
}

/// What:
/// The failure of a recursive removal,
/// which may already have unlinked part of the tree,
/// with what else this run removed.
/// Why:
/// `remove_dir_all` unlinks what it can before it stops,
/// and the directory never reaches the list of removed directories,
/// so the report must name it and say that part of it may be gone,
/// whatever that list holds.
pub(super) fn partly_removed(
    directory: &Path,
    error: &std::io::Error,
    removed: &[PathBuf],
) -> ToolchainError {
    let mut message: String = format!(
        "cannot remove {}: {error}; part of it may be gone",
        directory.display()
    );
    if !removed.is_empty() {
        message.push_str(&format!(", and already removed: {}", paths_named(removed)));
    }
    return ToolchainError::failure(message);
}

/// The removed directories as one list of paths,
/// for a report.
fn paths_named(removed: &[PathBuf]) -> String {
    return removed
        .iter()
        .map(|path| return path.display().to_string())
        .collect::<Vec<String>>()
        .join(", ");
}
