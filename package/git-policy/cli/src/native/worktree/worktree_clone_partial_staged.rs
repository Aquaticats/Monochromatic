//! What:
//!  The second answer on the paths a partial clone takes:
//! Git's own live listing of the paths the source has staged against
//!  its `HEAD`,
//! and the refusal a path on both that list and the clone list earns.
//! Why:
//!  The split judges the index the source check copied,
//! and the source's index can move before the copies,
//! as
//!  another session's `git add` moves this machine's main checkout;
//! a staged path holds another blob than the tree
//!  names,
//! so a clone of it would put the person's bytes where the commit's belong,
//! and no later step sees it:
//! the
//!  new index records the stat data of the file the clone made,
//! so `git status` trusts it.
//! The listing is a second,
//!  differently built answer for the class where a mistake is worst,
//! and it lives here so
//!  `worktree_clone_partial.rs` stays under the file-size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staged = stagedPaths(context, source);
//! const refusal = aStagedPathAmongTheClones(staged, entries, clones);
//! ```

use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use crate::worktree_clean::StatEntry;
use crate::worktree_snapshot_source::{
    SnapshotSource,
    plain,
};

/// What:
///  The paths the source's index holds another object or mode for than the tree of `HEAD`,
/// asked of Git in the source itself;
/// the error says why the question could not be answered.
/// Why:
///   The split decides which paths are staged from the index it read and the tree it flattened,
/// and that one
///  comparison is the only thing between a staged path and a clone;
/// Git's own listing is a second,
///  differently built answer for the class where a mistake is worst,
/// and the rung refuses when the two disagree
///  rather than clone the person's bytes into a worktree that must hold the commit's.
/// The listing is against the source's `HEAD`,
///  because that is the tree the split and the new index are built from;
/// `read-tree -m -u` carries the difference to a new `HEAD` afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = stagedPaths(context, source); // throws the reason Git could not answer
/// ```
pub(crate) fn staged_paths(
    context: &GitContext,
    source: &SnapshotSource,
) -> Result<Vec<Vec<u8>>, String> {
    let mut request: GitRequest = GitRequest::new(
        source.root.as_path(),
        &[
            "--no-optional-locks",
            "diff-index",
            "--cached",
            "--name-only",
            "-z",
            source.clean.head_commit.as_str(),
        ],
    );
    request.without_prefix = true;
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(format!(
                "`git diff-index --cached --name-only` could not start ({error})"
            ));
        }
    };
    if output.code != Some(0) {
        return Err(format!(
            "`git diff-index --cached --name-only` ended with {:?}",
            output.code
        ));
    }
    return Ok(output
        .stdout
        .split(|byte: &u8| {
            return *byte == 0;
        })
        .filter(|path| {
            return !path.is_empty();
        })
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  The first path among the clones that Git's own listing names as staged,
/// in words,
/// or nothing
///  when the two answers agree.
/// Why:
///   A staged path holds another blob than the tree names,
/// so a clone of it would put the person's bytes where
///  the commit's belong,
/// and no later step sees it:
/// the new index records the stat data of the file the clone
///  made,
/// so `git status` trusts it.
/// An independent second answer,
/// whose wiring the control
///  `a_path_staged_after_the_source_check_is_refused` guards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = aStagedPathAmongTheClones(staged, entries, clones) ?? undefined;
/// ```
pub(crate) fn a_staged_path_among_the_clones(
    staged: &[Vec<u8>],
    entries: &[StatEntry],
    clones: &[usize],
) -> Option<String> {
    for at in clones {
        let Some(entry) = entries.get(*at) else {
            continue;
        };
        if staged.iter().any(|path| {
            return *path == entry.path;
        }) {
            let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
            return Some(format!(
                "{} is staged in the source and would have been cloned",
                plain(shown.as_str())
            ));
        }
    }
    return None;
}
