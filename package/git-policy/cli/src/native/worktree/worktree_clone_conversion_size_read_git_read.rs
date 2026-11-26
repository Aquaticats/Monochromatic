//! What:
//!  The early read's one Git run:
//!  the request `git ls-tree -r -l -z` makes,
//!  the variable that keeps a partial clone from fetching the blobs such a run would need,
//!  and the parse of the sizes it prints.
//! Why:
//!   Both rungs decide the conversion fence with these sizes,
//!  and this run is the one place they are read without Git writing anything;
//!  the variable is on every read with no test of the source,
//!  because a test that read the configuration could miss a promisor remote whose flags arrive through an
//!  included file,
//!  a worktree config or the environment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sizes = sizesOfTheCommitWhereNothingIsFetched(context, root, commit, child); // throws a failed read's sentence
//! ```

use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::worktree_clone_conversion_git_reads::{
    BlobSizes,
    a_git_run_that_did_not_answer,
    parse_sizes,
    unreadable_sizes,
};
#[cfg(test)]
use crate::worktree_clone_conversion_size_read_seams::{
    die_where_a_control_asked,
    hold_the_read_where_a_control_asked,
    hold_the_sizes_after_the_wait_where_a_control_asked,
};
use std::ffi::OsString;
use std::path::Path;

/// The variable that stops Git from fetching the objects a partial clone is missing.
/// Why:
///   `git ls-tree -l` needs every blob's size.
/// On a partial clone,
/// a missing blob is fetched from the promisor remote to answer it,
///  which wrote packs into a source the check was about to refuse
///  (the gate review's High 1 of 2026-11-22).
/// The variable is set on every early read,
/// with no test of the source:
///  the decision says the read never fetches
///  (`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
/// its section "The overlap,
/// answered 2026-11-22"),
///  and a test that read the config could miss a promisor remote whose flags arrive through an included file,
/// a worktree config or the environment
///  (the integration recheck's High 1 of 2026-11-22).
/// The variable is honoured by the May 2024 maintenance releases
///  (`v2.39.4`,
/// `v2.40.2`,
/// `v2.41.1`,
/// `v2.42.2`,
/// `v2.43.4`,
/// `v2.44.1`;
/// the commit `7b70e9efb`),
/// by `v2.45.0` and later,
/// and by Ubuntu 24.04's `2.43.0-1ubuntu7.1` and later.
/// On a missing blob the maintenance releases and Ubuntu's package exit 128
///  ("lazy fetching disabled ... could not fetch ... from promisor remote"),
///  so the rung declines to Git's checkout;
/// `v2.45.0` and later print `BAD` in the size column and exit 0,
/// which `parse_sizes` skips.
/// A path whose size cannot be listed therefore only ever goes to Git's writing.
/// A Git older than the fix that does not carry it ignores the variable and can fetch,
///  which is a recorded limit,
/// not something this read can avoid.
/// The option `--no-lazy-fetch` is deliberately not used:
///  it is Git 2.45.0's,
/// and an older Git refuses it (exit 129),
///  which sent a plain source from the snapshot to Git's whole checkout
///  (the integration recheck's Medium 3 of 2026-11-22).
const NO_LAZY_FETCH: &str = "GIT_NO_LAZY_FETCH";

/// What:
///  The one request the early read makes:
/// `git ls-tree -r -l -z` of `commit`,
///  with `GIT_NO_LAZY_FETCH=1` on every read,
///  which leaves the process-wide failed-step note alone,
///  and whose child ends with the thread that starts it.
/// Why:
///   Both of the last facts are conditions of starting before the source check:
/// the add's own steps may fail while this read runs,
///  so the read must not take their note (`GitRequest::beside_another_step`),
///  and an add that ends with the read still running must leave no child (`end_with_the_starting_thread`).
/// The variable is on every read and is the only switch:
///  the decision says the read never fetches,
/// and the option `--no-lazy-fetch` is left out because an older Git refuses it
///  (see `NO_LAZY_FETCH`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theEarlyReadRequest = (root, commit) => ({ cwd: root, args: ['ls-tree', '-r', '-l', '-z', commit], env: { GIT_NO_LAZY_FETCH: '1' }, besideAnotherStep: true, endWithTheStartingThread: true });
/// ```
pub(crate) fn the_early_read_request(
    root: &Path,
    commit: &str,
) -> GitRequest {
    let arguments: Vec<OsString> = vec![
        OsString::from("ls-tree"),
        OsString::from("-r"),
        OsString::from("-l"),
        OsString::from("-z"),
        OsString::from(commit),
    ];
    let environment: Vec<(OsString, OsString)> =
        vec![(OsString::from(NO_LAZY_FETCH), OsString::from("1"))];
    return GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        environment,
        ..GitRequest::default()
    }
    .beside_another_step()
    .ends_with_the_starting_thread();
}

/// What:
///  One `git ls-tree -r -l -z` of `commit`,
/// run where a partial clone does not fetch.
/// Why:
///   This is the early read's own run;
/// its output is parsed here with the same `parse_sizes` the fence's serial read uses,
///  on this thread and off the adding thread,
///  and a run that fails says what the fence's own read says.
/// The runner clears the published child's cell with a compare-exchange once the child is gone,
///  before this function parses anything;
/// the abandon can still signal an id no process has in the window that clear leaves,
///  and the `abandon` doc says which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sizesOfTheCommitWhereNothingIsFetched(context, root, commit): BlobSizes;
/// ```
pub(crate) fn sizes_of_the_commit_where_nothing_is_fetched(
    context: &GitContext,
    root: &Path,
    commit: &str,
    child: &std::sync::Arc<std::sync::atomic::AtomicI32>,
) -> Result<BlobSizes, String> {
    #[cfg(test)]
    hold_the_read_where_a_control_asked();
    #[cfg(test)]
    die_where_a_control_asked();
    let output: GitOutput = run_git(
        context,
        &the_early_read_request(root, commit).publishing_its_child(child.clone()),
    )
    .map_err(|error| return a_git_run_that_did_not_answer(&error))?;
    if output.code != Some(0) {
        return Err(unreadable_sizes(&output));
    }
    // This seam sits after the wait and before this thread parses anything: an abandon here must find the cell
    //  clear, so a control can show that the parse starts only after the runner let go of the child.
    #[cfg(test)]
    hold_the_sizes_after_the_wait_where_a_control_asked();
    return Ok(parse_sizes(output.stdout.as_slice()));
}
