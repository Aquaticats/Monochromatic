//! What:
//!  The validated private snapshot of one source,
//!  ready to install.
//! Why:
//!  The stage fills it and the journal writes it down before the installation begins,
//!  so the shape a staged copy is handed to the installation and to the journal lives below both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type StagedWorktreeSnapshot = { entries; selectedRoots; sourceRoot; stageContainer; stageRoot };
//! ```

use crate::worktree_copy_manifest::WorktreeCopyEntry;
use std::path::PathBuf;

/// What:
///  The validated private snapshot of one source,
/// ready to install.
/// Why:
///   Installation needs the entries it will create,
/// the roots they came from and the stage to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedWorktreeSnapshot = { entries; selectedRoots; sourceRoot; stageContainer; stageRoot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedSnapshot {
    /// Parent-first manifest that was staged and verified.
    pub entries: Vec<WorktreeCopyEntry>,
    /// Git-selected ignored roots that the manifest walked.
    pub selected_roots: Vec<Vec<u8>>,
    /// Canonical source worktree.
    pub source_root: PathBuf,
    /// Private container holding the payload;
    /// removed after installation.
    pub stage_container: PathBuf,
    /// Payload root mirroring the source's repository paths.
    pub stage_root: PathBuf,
    /// Repository paths the user could not read,
    /// left out of the stage;
    ///  empty for a stage rebuilt from a journal.
    pub unreadable: Vec<Vec<u8>>,
    /// Repository paths no worktree can hold (sockets,
    /// named pipes and device nodes),
    /// left out of the stage;
    ///  empty for a stage rebuilt from a journal.
    pub uncopiable: Vec<Vec<u8>>,
    /// Repository paths that kept changing under the copy and were left in their source;
    ///  empty for a stage rebuilt from a journal.
    pub unsettled: Vec<Vec<u8>>,
}
