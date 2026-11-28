//! What:
//! The one atomic link write every installer shares:
//! the link is made under a temporary name in the same directory and renamed over the final name,
//! so readers see the old link or the new one.
//! Why:
//! The exposure directory is on `PATH` while a sync rewrites it,
//! and the newest-only rule's move of `current` is the same write.
//! The rename is the step a control refuses (`a_control_refuses_the_link_move`),
//! and the failure both arms answer with is built here once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function atomicSymlink(linkPath: string, target: string): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the facts a name records about the install holding the link,
/// which are the facts of the program running now.
use super::super::archive_staging::Owner;
/// Imports `ToolchainError` from the toolchain error module.
use super::super::error::ToolchainError;
/// Imports the refusal a control plants at the rename.
use super::a_control_refuses_the_link_move::refused_link_move;
/// Imports the name a held link carries,
/// which records the install that wrote it,
/// so a later rebuild can tell a link it may remove from one an install still holds.
use super::the_temporary_link_of;

/// What:
/// Creates (or replaces) the symlink `link_path` pointing at `target`:
/// the link is made under a temporary
///           name in the same directory and renamed over the final name,
/// so readers see the old link or the new one.
/// Why:
/// The exposure directory is on `PATH` while a sync rewrites it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function atomicSymlink(linkPath: string, target: string): void
/// ```
pub fn atomic_symlink(
    link_path: &Path,
    target: &Path,
) -> Result<(), ToolchainError> {
    if let Some(parent) = link_path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(ToolchainError::failure(format!(
            "cannot create {}: {error}",
            parent.display()
        )));
    }
    // What: the dotted name beside the link that this install holds it under until the rename.
    // Why: the name records which install is writing, so a second rebuild at the same instant leaves it alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const temporary = theTemporaryLinkOf(linkPath, ownerOfThisProcess());
    // ```
    let temporary: PathBuf = the_temporary_link_of(link_path, Owner::here());
    // Best effort: a stale temporary link from a killed run is replaced; absence is the normal case.
    let _ = fs::remove_file(&temporary);
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(target, &temporary);
    #[cfg(not(unix))]
    let linked = std::fs::hard_link(target, &temporary);
    if let Err(error) = linked {
        return Err(ToolchainError::failure(format!(
            "cannot link {} to {}: {error}",
            temporary.display(),
            target.display()
        )));
    }
    if let Some(error) = refused_link_move() {
        let _ = fs::remove_file(&temporary);
        return Err(cannot_place_the_link(link_path, &error));
    }
    if let Err(error) = fs::rename(&temporary, link_path) {
        let _ = fs::remove_file(&temporary);
        return Err(cannot_place_the_link(link_path, &error));
    }
    return Ok(());
}

/// What:
/// The failure a link write answers when the rename that gives the link its final name is refused.
/// Why:
/// The refusal a control plants and the rename's own failure must read as one sentence,
/// or the control would show a message no run prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cannotPlaceTheLink(linkPath: string, error: Error): ToolchainError {
///   return ToolchainError.failure(`cannot place the link ${linkPath}: ${error}`);
/// }
/// ```
fn cannot_place_the_link(
    link_path: &Path,
    error: &std::io::Error,
) -> ToolchainError {
    return ToolchainError::failure(format!(
        "cannot place the link {}: {error}",
        link_path.display()
    ));
}
