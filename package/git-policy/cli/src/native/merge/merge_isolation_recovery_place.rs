//! What:
//!  Where recovery runs Git:
//!  the real Git and its environment,
//!  the directory every command starts in,
//!  and the repository's common Git directory.
//! Why:
//!  The recovery of an interrupted merge and the settling of a merge whose branch moved
//!  both run Git from the process directory with the invocation's own global options,
//!  whichever worktree the record belongs to,
//!  so both ask for the place this file builds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = RecoveryPlace.settling(git, gitDir, root);
//! ```

/// Real Git and its environment,
/// as the place hands them to every command it runs.
use crate::transaction_git::GitContext;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths;
/// the place owns the three it holds,
/// because it outlives the call that built it.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Where recovery runs Git:
/// the real Git and its environment,
/// the directory every command starts in,
/// and the
///  repository's common Git directory.
/// Why:
///   Recovery of a merge runs from the process directory with the invocation's global options,
/// whichever worktree the
///  record belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryPlace = { gitPath: string; effectiveCwd: string; commonDir: string };
/// ```
#[derive(Clone, Debug)]
pub struct RecoveryPlace {
    /// Real Git and its environment.
    pub git: GitContext,
    /// Working directory of every Git command.
    pub cwd: PathBuf,
    /// The repository's common Git directory.
    pub common_dir: PathBuf,
    /// The Git directory of the worktree whose index recovery writes.
    pub git_dir: PathBuf,
}

/// What:
///  The place recovery runs in when a command that started its own merge has to settle it.
/// Why:
///   The two directories stay apart:
///  the worktree's own Git directory is what recovery writes through,
///  and the repository's common one is where every worktree of it keeps what they share.
///  A linked worktree's own Git directory lives under the main repository's `worktrees` directory,
///  and only its `commondir` file names the repository's own
///  (`foundation/common_directory_of_a_git_directory.rs`),
///  so filling the common field with the worktree's directory would point every reader of it
///  at a directory that goes away with the worktree.
///  The context carries the invocation's common directory;
///  one built without it still knows the worktree's Git directory,
/// whose `commondir` names the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = { git, cwd: root, commonDir: context.commonDir ?? commonDirectoryOf(gitDir), gitDir };
/// ```
impl RecoveryPlace {
    /// The place a settling command needs,
    /// from its context and the two directories.
    pub fn settling(
        git: &GitContext,
        git_dir: &Path,
        root: &Path,
    ) -> RecoveryPlace {
        let common_dir: PathBuf = match git.common_dir.as_ref() {
            Some(found) => found.clone(),
            None => crate::common_directory_of_a_git_directory::common_directory_of(git_dir),
        };
        return RecoveryPlace {
            git: GitContext {
                global_prefix: Vec::new(),
                ..git.clone()
            },
            cwd: root.to_path_buf(),
            common_dir,
            git_dir: git_dir.to_path_buf(),
        };
    }
}
