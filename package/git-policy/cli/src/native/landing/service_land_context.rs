//! What:
//!  What the commit service needs to land a batch:
//! how to run Git,
//! where the journal lives,
//! a test seam and the lock ceiling.
//! Why:
//!  The landing (`service_land.rs`) and the recovery of open batches (`service_land_finish.rs`) both take it;
//! a file that uses
//!  neither lets each use it without using the other in a circle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LandContext = { git: GitContext; journal: string; onObjectsWritten?: () => void; lockCeiling: number };
//! ```

use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};
/// What the service needs to land:
/// how to run Git and where the journal lives.
///
/// Cloned for one batch or one open record,
///  which names the repository its Git must read.
#[derive(Clone)]
pub struct LandContext {
    /// How to run the real Git,
    /// with the bounds of `child_class.rs`.
    pub git: GitContext,
    /// Directory of the journal.
    pub journal: PathBuf,
    /// Test seam:
    /// called after the batch's objects exist and before the branch moves.
    /// Shared rather than moved,
    /// so a clone of this context keeps the one seam.
    pub on_objects_written: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    /// How long a landing waits for a foreign program's `index.lock` before it is withdrawn (`wait_ceilings.rs`).
    pub lock_ceiling: std::time::Duration,
}

/// Methods over the context one landing round works in.
impl LandContext {
    /// What:
    ///  This service context with every Git of one worktree naming that repository.
    /// Why:
    ///   A request and a journal record name the worktree's own Git directory and its work tree,
    ///  and a service inherits none of the caller's options:
    ///  a Git directory beside its work tree,
    ///  or one an environment variable sent elsewhere,
    ///  is read and written only when the landing names it
    ///  (`repository_selection::of_paths`).
    ///  A plain layout is named by nothing,
    ///  so its in-process reads keep answering.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// context.forTheRepository({ gitDir, workTree });
    /// ```
    pub fn for_the_repository(&self, git_dir: &Path, work_tree: &Path) -> LandContext {
        // The selection comes after whatever the service already ran its Git with, so an absolute path it names wins over a
        //  relative one an earlier option wrote.
        let mut prefix: Vec<std::ffi::OsString> = self.git.global_prefix.clone();
        prefix.extend(crate::repository_selection::of_paths(git_dir, work_tree));
        return LandContext {
            git: GitContext { global_prefix: prefix, ..self.git.clone() },
            journal: self.journal.clone(),
            on_objects_written: self.on_objects_written.clone(),
            lock_ceiling: self.lock_ceiling,
        };
    }
}
