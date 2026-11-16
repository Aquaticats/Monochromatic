//! What:
//!  Whether a Git directory is the repository of the instance whose journal is being written,
//! and the line said when it is not.
//! Why:
//!   A journal belongs to one instance,
//! and an instance is one worktree's Git directory (`service_instance::instance_of`),
//! so a note written for another instance's repository is one the next start removes
//! (`service_claim_watch`'s `scan`).
//! Every path that writes a note asks this one question,
//! and the landing thread asks it before it writes a note of its own,
//! so no two of them can answer it differently and the journal never begins to watch a repository
//!  its service does not serve.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const isForeign = (instance, gitDir) => instance !== undefined && instanceOf(gitDir) !== instance;
//! ```

use crate::service_instance::instance_of;
use std::path::Path;

/// Whether `git_dir` belongs to an instance other than `instance`.
/// Why:
///   A journal belongs to one instance,
/// and a note that names another repository is removed by the watch's `scan` and refused by its `remember`;
/// the landing thread's `note_for_the_next_service` asks the same question,
/// so a note it writes cannot be one the next service throws away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ofAnotherInstance = (instance, gitDir) => instance !== undefined && instanceOf(gitDir) !== instance;
/// ```
pub fn of_another_instance(instance: Option<&str>, git_dir: &Path) -> bool {
    return match instance {
        Some(mine) => instance_of(git_dir) != mine,
        None => false,
    };
}

/// What:
///  Whether `git_dir` is refused for belonging to another instance,
/// said in the log when it is.
/// Why:
///   A journal holds one instance's repositories,
/// so a note written for another one is thrown away by the next start,
/// and a watch that took it would name a repository its service does not serve.
/// Every path of the watch that would write a note,
/// and the keeping of one that may be missing,
/// asks this one question,
/// so none of them can write or watch another instance's repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (refusedForAnotherInstance(instance, gitDir)) return; // said once, and nothing is written or owed
/// ```
pub fn refused_for_another_instance(instance: Option<&str>, git_dir: &Path) -> bool {
    if !of_another_instance(instance, git_dir) {
        return false;
    }
    crate::service_warn!("{} belongs to another instance and is not kept for a restart", git_dir.display());
    return true;
}
