//! What:
//!  Everything the finish pass reads and writes through,
//!  named as values so a control answers each without a pool, a repository or a registration.
//! Why:
//!  The pass reads the plan folder,
//!  asks the compatibility gate before any request,
//!  proves the clone gone through the typed pool reader,
//!  and settles a registration through the route's own settle,
//!  so each is a seam and the release supplies the live answer
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration").
//!  Nothing here decides anything:
//!  the order lives in the pass,
//!  and this file only carries what it decides with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pass = { commonDir, folder, gate, probe, settle };
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::ThePlansFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::RegistrationSettle;
use crate::zfs_compatibility_gate::BeforeRequests;
use std::path::Path;

/// What:
///  The repository, the folder and the three seams one finish pass runs over.
/// Why:
///  Five arguments at each call would hide which one a control changed,
///  and the live wiring of this pass supplies one value for a whole command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FinishPass = { commonDir; folder; gate; probe; settle };
/// ```
pub struct FinishPass<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The one look the pass takes at the destruction plan folder.
    pub folder: &'a dyn ThePlansFolder,
    /// The compatibility gate every process reads before its first raw request.
    pub gate: &'a dyn BeforeRequests,
    /// The reader the absence proof reads the pool through.
    pub probe: &'a dyn DatasetReader,
    /// The Git settle the list's step 11 runs through.
    pub settle: &'a dyn RegistrationSettle,
}
