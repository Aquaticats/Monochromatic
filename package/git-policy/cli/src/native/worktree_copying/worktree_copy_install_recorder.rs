//! What:
//!  The durable record of an installation in progress:
//! every batch's claims reported before a path is created,
//! every proven creation reported after,
//! and the recorder that keeps nothing for a call that asks for no durability.
//! Why:
//!   Recording each batch before and after it runs bounds the work an interrupted run leaves unrecorded,
//! and the installer reaches the journal only through this face,
//! so it never learns the file format.
//! It lives beside the installation because that file was over the line limit before this change
//! (`worktree_copy_install.rs`,
//! which re-exports it).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface InstallRecorder { recordIntents(paths: Uint8Array[]): void; recordCreations(entries: InstalledPath[]): void }
//! ```

use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_install_entry::InstalledPath;
/// What:
///  The durable record of an installation in progress.
/// Why:
///   The installation reports every claim before it creates a path and every proven creation after,
///  so the journal slice can persist both without this module knowing the file format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordIntents({ state, relativePaths }); recordCreations({ state, entries });
/// ```
pub trait InstallRecorder {
    /// Records the absent paths one batch is about to create,
    /// before any of them exists.
    fn record_intents(
        &mut self,
        relative_paths: &[Vec<u8>],
    ) -> Result<(), WorktreeCopyError>;

    /// Records the identities of the paths one batch created.
    fn record_creations(
        &mut self,
        entries: &[InstalledPath],
    ) -> Result<(), WorktreeCopyError>;
}

/// What:
///  A recorder that writes nothing,
/// for installations whose caller keeps no durable state.
/// Why:
///   Such an installation is not crash-safe;
/// it exists so the installer can be exercised before the journal lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no journal state
/// ```
#[derive(Debug, Default)]
pub struct NoRecord;

impl InstallRecorder for NoRecord {
    /// Accepts the intent without storing it.
    fn record_intents(
        &mut self,
        _relative_paths: &[Vec<u8>],
    ) -> Result<(), WorktreeCopyError> {
        return Ok(());
    }

    /// Accepts the creations without storing them.
    fn record_creations(
        &mut self,
        _entries: &[InstalledPath],
    ) -> Result<(), WorktreeCopyError> {
        return Ok(());
    }
}
