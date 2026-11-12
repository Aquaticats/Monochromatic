//! What:
//!  The two Git steps the preparation starts itself,
//! with the face a caller on a thread asks and the face a task of the runtime asks:
//!  the staging of every unmerged file into the commit's own index,
//! and the tree of the branch the commit was prepared on.
//! Why:
//!  The preparation refuses a commit of an unmerged index ("Warn and finish the merge",
//! owner,
//!  2026-11-03),
//! so each unmerged file is staged into the private index exactly as `git add` stages it,
//! and a private tree equal to the base's is the
//!  nothing-to-commit answer that gives Git's own words.
//! Each step is named here rather than inside the preparation's body
//!  (`service_prepare.rs`) so that the awaited face has one place to live and the body keeps its refusals in their
//!  order,
//! and so that a task of the runtime never calls the synchronous face:
//! that face enters the runtime to wait for its child,
//! which a task may not do (`transaction_git_awaiting.rs`).
//! Both faces ask one request and read the answer through `checked_from`,
//!  so a refusal keeps Git's words and the status `TransactionStop::Message` gives whichever face ran it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await stageTheUnmergedFiles(git, cwd, indexes, files); const tree = await baseTreeOf(git, cwd, base);
//! ```

use crate::commit_transaction::TransactionStop;
use crate::private_index_request::{CaptureIndexes, indexed_request};
use crate::service_prepare_unmerged_staging::unmerged_staging_request;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use std::path::Path;

/// What:
///  The staging start that puts the working copy of every file still unmerged into the commit's own index,
/// and nothing at all when no file is unmerged.
/// Why:
///   Git refuses a commit of an index with unmerged paths (`builtin/commit.c:348`,
/// `refresh_cache_or_die`),
/// and the owner's rule
///  is "Warn and finish the merge" (2026-11-03),
/// so each unmerged file is staged into the private index exactly as `git add`
///  stages it,
/// and the commit that follows is the one Git would have built for a resolved merge.
/// The step is named
///  here so that a task of the runtime asks an awaited face of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGitChecked({ index, objects, args: ['add', '--all', '--', ...files] }) // nothing when files is empty
/// ```
pub(crate) fn stage_the_unmerged_files(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, files: &[Vec<u8>]) -> Result<(), TransactionStop> {
    if files.is_empty() {
        return Ok(());
    }
    run_git_checked(context, &unmerged_staging_request(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), files))?;
    return Ok(());
}

/// What:
///  `stage_the_unmerged_files` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot call the synchronous face,
/// which enters the runtime to wait for its child
///  (`transaction_git_awaiting.rs`),
/// and the staging is a step of the preparation once that preparation is a task.
/// The question and
///  the failure are the synchronous face's (`checked_from`),
/// so a refused staging says the same words and ends the command the
///  same way whichever face ran it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stageTheUnmergedFilesAwaiting(git, cwd, indexes, files)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the preparation's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn stage_the_unmerged_files_awaiting(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    files: &[Vec<u8>],
) -> Result<(), TransactionStop> {
    if files.is_empty() {
        return Ok(());
    }
    run_git_checked_awaiting(context, &unmerged_staging_request(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), files)).await?;
    return Ok(());
}

/// What:
///  The tree the branch the commit was prepared on holds,
/// as the command line's base names it.
/// Why:
///   A commit whose private tree is the base tree has nothing to record,
/// and the comparison needs the base's own tree
///  whatever object format the repository uses,
/// so Git is asked instead of recomputing it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['rev-parse', '--verify', `${base}^{tree}`]).trim()
/// ```
pub(crate) fn base_tree_of(context: &GitContext, cwd: &Path, base: &str) -> Result<String, TransactionStop> {
    let output = run_git_checked(context, &base_tree_request(context, cwd, base))?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  The question `base_tree_of` asks Git.
/// Why:
///  Both faces ask the one question,
/// so neither can compare the intended tree with another base than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = indexedRequest({ cwd, args: ['rev-parse', '--verify', `${base}^{tree}`] }); request.withoutPrefix = true;
/// ```
fn base_tree_request(context: &GitContext, cwd: &Path, base: &str) -> GitRequest {
    let mut request: GitRequest = indexed_request(context, cwd, &["rev-parse", "--verify", format!("{base}^{{tree}}").as_str()], None, None);
    request.without_prefix = true;
    return request;
}

/// What:
///  `base_tree_of` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot call the synchronous face,
/// which enters the runtime to wait for its child
///  (`transaction_git_awaiting.rs`),
/// and the comparison is a step of the preparation once that preparation is a task.
/// The
///  question and the failure are the synchronous face's (`checked_from`),
/// so the two cannot compare against another tree or
///  word a refusal differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await baseTreeOfAwaiting(git, cwd, base)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the preparation's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn base_tree_of_awaiting(context: &GitContext, cwd: &Path, base: &str) -> Result<String, TransactionStop> {
    let output = run_git_checked_awaiting(context, &base_tree_request(context, cwd, base)).await?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// Controls for the awaited faces of the preparation's own Git steps stay out of the release executable.
#[cfg(test)]
#[path = "service_prepare_awaiting_tests.rs"]
mod awaiting_tests;
