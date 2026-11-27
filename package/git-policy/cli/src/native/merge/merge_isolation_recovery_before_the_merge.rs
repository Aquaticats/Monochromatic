//! What:
//!  The merges a merge that died left behind,
//!  finished or discarded before this merge reads the checkout.
//! Why:
//!   A merge killed between Git's checkout update and the branch move leaves the checkout half done,
//! and the checks that follow would read it as another session's staged work.
//!  The step belongs to `merge_isolation.rs`,
//! whose round calls it before it reads anything,
//! and it is a file of its own so the round's file stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recoverDeadMerges({ input, gitDir, root });
//! ```

/// The failure rendering for state that cannot be read.
use crate::diagnostics::{
    EngineFailureCode,
    render_engine_failure_with_hint,
};
/// Where recovery runs,
/// and the recovery of the records a dead process left.
use crate::merge_isolation_recovery::{
    RecoveryPlace,
    recover_interrupted_merges,
};
/// The invocation the merge was asked for,
/// and the ending an early stop of it takes.
use crate::merge_isolation_run_types::{
    Bail,
    IsolatedMergeInput,
    refused,
};
/// Real Git.
use crate::transaction_git::GitContext;
/// The repository shape.
use crate::worktree_identity::WorktreeIdentity;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Recover merges that died,
/// before this merge reads the checkout.
/// Why:
///   A merge killed between Git's checkout update and the branch move leaves the checkout half done,
/// and the
///  checks that follow would read it as another session's staged work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInterruptedMerges({ place, gitDir, stderr });
/// ```
pub(crate) fn recover_dead_merges(
    input: &IsolatedMergeInput<'_>,
    git_dir: &Path,
    root: &Path,
) -> Result<(), Bail> {
    let common_dir: PathBuf = match input.identity {
        WorktreeIdentity::MainWorktree {
            common_dir,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            common_dir,
            ..
        } => common_dir.clone(),
        _ => return Ok(()),
    };
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext {
            real_git: input.real_git.to_path_buf(),
            overlay: input.overlay.to_vec(),
            global_prefix: Vec::new(),
            common_dir: Some(common_dir.clone()),
        },
        cwd: root.to_path_buf(),
        common_dir,
        git_dir: git_dir.to_path_buf(),
    };
    // The notices of what recovery did reach the command's standard error: the branch may have moved onto a merge commit
    //  this command finished rather than the one it is about to compute (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`, finding 1).
    match recover_interrupted_merges(&place, git_dir, &mut std::io::stderr()) {
        Ok(_) => return Ok(()),
        Err(error) => {
            return Err(refused(
                2,
                render_engine_failure_with_hint(
                    0,
                    EngineFailureCode::ContentUnavailable,
                    error.0.as_str(),
                    None,
                ),
            ));
        }
    }
}
