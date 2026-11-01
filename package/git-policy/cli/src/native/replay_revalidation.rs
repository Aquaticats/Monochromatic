//! What:
//!  The parts of revalidation after a replay that need no policy:
//!       the replayed index and the shadow `HEAD` move the policies see.
//! Why:
//!  Policies belong to the integrator, which owns the policy engine;
//!       these three steps are pure Git work the integrator calls before it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runTransactionGit({ gitPath, cwd, indexPath, args: ['read-tree', mergedTree], objectDirectory });
//! await moveShadowHead({ context, onto });
//! const rerun = !context.noVerify && policyTree !== approvedTree;
//! ```

/// Private directories and the transaction Git runner.
use super::private_storage::create_private_directory;
use super::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
/// `OsString` names the index and object-store variables.
use std::path::{Path, PathBuf};

/// What:
///  The private index of one replayed tree: `replay-<r>/commit.index` under the transaction directory.
/// Why:
///   Policies and `pre-commit` see the replayed tree through this index, not the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(directory, 'commit.index');
/// ```
pub fn replayed_index_path(transaction_directory: &Path, replay: u64) -> PathBuf {
    return transaction_directory
        .join(format!("replay-{replay}"))
        .join("commit.index");
}

/// What:
///  Reads the merged tree into the replay's private index, with the shadow object store reachable.
/// Why:
///   The index is the input to policies and `pre-commit`; objects the merge wrote live in the shadow store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, indexPath, args: ['read-tree', mergedTree], objectDirectory });
/// ```
pub fn read_replayed_index(
    context: &GitContext,
    cwd: &Path,
    object_directory: &Path,
    index_path: &Path,
    merged_tree: &str,
) -> Result<(), GitFailure> {
    if let Some(parent) = index_path.parent() {
        // Subsumption creates the same directory for its base index first.
        if let Err(error) = create_private_directory(parent)
            && error.kind() != std::io::ErrorKind::AlreadyExists
        {
            return Err(GitFailure(format!("creating {} failed: {error}", parent.display())));
        }
    }
    let mut request: GitRequest = GitRequest::new(cwd, &["read-tree", merged_tree]);
    request.index = Some(index_path.to_path_buf());
    request.object_directory = Some(object_directory.to_path_buf());
    run_git_checked(context, &request)?;
    return Ok(());
}

/// What:
///  Moves the shadow's target ref to the replay parent, so policies and hooks see the parent the commit will have.
/// Why:
///   A detached `HEAD` moves itself without dereferencing; a symbolic one moves the branch it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runShadowGit({ args: ['update-ref', ...(detached ? ['--no-deref'] : []), targetRef, onto] });
/// ```
pub fn move_shadow_head(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    target_ref: &str,
    onto: &str,
    detached: bool,
) -> Result<(), GitFailure> {
    let mut arguments: Vec<&str> = vec!["update-ref"];
    if detached {
        arguments.push("--no-deref");
    }
    arguments.push(target_ref);
    arguments.push(onto);
    let request: GitRequest = super::shadow_git::shadow_request(shadow, cwd, &arguments);
    run_git_checked(context, &request)?;
    return Ok(());
}

#[cfg(test)]
#[path = "replay_revalidation_tests.rs"]
mod tests;
