//! What:
//!  The commit's own index,
//!  built the one of two ways a preparation needs it:
//!  from the change a command captured when the service sent the commit back,
//!  merged onto the branch as it is now,
//!  or from the working copy the command line selects,
//!  staged into a private index.
//!  Also the notice a worktree whose index a landed commit did not finish is given.
//! Why:
//!   Neither way writes the live index,
//!  so a landing that did not finish its own index update cannot make this commit record the
//!  landed paths as their old content,
//!  and the replay never reads the worktree again.
//!  Both ways answer with what became of the commit,
//!  because a conflict,
//!  a refusal a command line wrote,
//!  and a commit that landed meanwhile are endings the caller hands the person unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const built = theCommitIndex(invocation, setup, indexSelection); // { wordMergeEvents } | { ended: Prepared }
//! ```

use super::service_prepare_selection_refusals::{
    named_ignored_ending,
    pathspec_matching_nothing_ending,
};
use super::service_prepare_what_it_reads::Preparation;
use crate::commit_index::{
    IndexSelection,
    initialize_commit_index,
    initialize_retry_index,
};
use crate::commit_selection_staging::SelectionStagingFailure;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionStop,
    failed,
};
use crate::merge_second_pass::Merged as MergedFile;
use crate::policy_events::PolicyEvent;
use crate::preparation_notice::deliver_notice;
use crate::service_prepare_ready_shapes::{
    Prepared,
    ReplayConflict,
};
use crate::service_replay::{
    MergeSite,
    Merged,
    Retry,
    first_landed,
    merge_captured,
    word_merge_notice,
};
use crate::service_replay_conclusion::the_same_conclusion;
use crate::transaction_git::GitContext;
use std::path::Path;

/// What:
///  What building the commit's own index came to.
/// Why:
///   The caller either goes on with the parts a word pass merged,
/// or hands the person an ending the index build already decided,
///  and one answer says which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexBuilt = { wordMergeEvents: PolicyEvent | undefined } | { ended: Prepared };
/// ```
pub(crate) enum IndexBuilt {
    /// The commit's own index is ready,
    /// and the parts a word pass merged when a retry combined the change with the branch.
    Ready {
        /// What the word passes merged,
        /// told with the commit and sent to the journal.
        word_merge_events: Option<PolicyEvent>,
    },
    /// The commit ends here,
    /// with the ending the index build found.
    Ended(Prepared),
}

/// What:
///  Builds the index this commit is made from,
/// either out of the change the first preparation captured or out of the working copy.
/// Why:
///   A commit the service sent back is never prepared again from the worktree,
///  so the tree it lands is the change the person already chose;
///  and a commit staged here can end before any policy reads it,
///  which the caller must hand back exactly as Git answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCommitIndex(invocation, setup, indexSelection, retry, first): IndexBuilt
/// ```
pub(crate) fn the_commit_index(
    invocation: &CommitInvocation<'_>,
    setup: &Preparation,
    index_selection: &IndexSelection<'_>,
    retry: Option<&Retry<'_>>,
    first: bool,
) -> Result<IndexBuilt, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let capture = &setup.capture;
    let indexes = &setup.indexes;
    let scratch = &setup.scratch;
    let base = setup.base.as_str();
    // The worktree Git answered for this invocation, which the selection's relative pathspecs are read from, whichever way
    //  the caller named it (`pathspec_base`).
    let worktree_root: &Path = capture.layout.repository_root.as_path();
    // The parts the word-level second pass merged when a retry combined the change with the branch, told with the commit.
    let mut word_merge_events: Option<PolicyEvent> = None;
    // The file the commit copies its index from, as it was before the copy: a live index that is another file later has entries
    //  the copy does not, and those are what the commit must not record.
    let index_witness: crate::service_land_start::IndexWitness =
        crate::service_land_start::witness_of(capture.layout.real_index_path.as_path())
            .unwrap_or(crate::service_land_start::IndexWitness::Absent);
    if let Some(again) = retry {
        // A conclusion is made again only over the state it was prepared over: another session that concluded or
        //  abandoned the merge, the cherry-pick or the revert meanwhile leaves nothing this commit could conclude, and an
        //  ordinary commit of the same tree is not what the person asked for (`Captured::conclusion`).
        if let Some(expected) = again.captured.conclusion.as_deref() {
            the_same_conclusion(
                context,
                cwd,
                capture,
                expected,
                setup.reference.short().spoken().as_str(),
            )?;
        }
        // The change this command captured when it started, on the branch as it is now; the worktree is not read.
        let site: MergeSite<'_> = MergeSite {
            context,
            cwd,
            common_dir: capture.layout.common_dir.as_path(),
            object_directory: indexes.object_directory.as_path(),
            scratch: scratch.0.as_path(),
        };
        let (tree, combined): (String, Vec<MergedFile>) =
            match merge_captured(&site, again.captured, base, again.attempt)? {
                Merged::Conflict(paths) => {
                    let landed = first_landed(
                        context,
                        cwd,
                        (!again.captured.unborn).then_some(again.captured.base.as_str()),
                        base,
                        paths.as_slice(),
                    )?;
                    // What: `IndexBuilt::Ended(Prepared::Conflict(..))` is this commit ending as a conflict, with where each path still landed.
                    // Why: the caller hands the person the same ending it would have built here, and builds no index for it.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // return { ended: { kind: 'conflict', paths, landed, gitDir } };
                    // ```
                    return Ok(IndexBuilt::Ended(Prepared::Conflict(Box::new(
                        ReplayConflict {
                            paths,
                            landed,
                            git_dir: capture.layout.git_dir.clone(),
                        },
                    ))));
                }
                Merged::Tree(tree) => (tree, Vec::new()),
                Merged::Combined {
                    tree,
                    files,
                } => (tree, files),
            };
        // Built from a copy of the real index, so the entries the tree shares with it keep their stat data and the scratch commit
        //  does not hash the whole work tree again (`commit_index.rs`, `initialize_retry_index`).
        initialize_retry_index(
            context,
            cwd,
            capture.layout.real_index_path.as_path(),
            indexes,
            tree.as_str(),
        )?;
        // Every part the word pass merged is told to the person with the commit and sent to the journal.
        word_merge_events = word_merge_notice(combined.as_slice());
    } else {
        // A path an ignore rule matches is refused before the private staging runs, from the elements the command names,
        //  so the refusal is the command line's and never read from the words of a Git message (issue 6278).
        if let Some(ending) = named_ignored_ending(
            context,
            cwd,
            worktree_root,
            capture.layout.real_index_path.as_path(),
            index_selection,
        )? {
            // What: `IndexBuilt::Ended(ending)` is the refusal the selection already came to, carried to the caller unchanged.
            // Why: an ignored path the command names is the command line's fault, and the words that say so are Git's.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { ended };
            // ```
            return Ok(IndexBuilt::Ended(Prepared::Ended(ending)));
        }
        if let Err(stop) = initialize_commit_index(
            context,
            cwd,
            worktree_root,
            capture.layout.real_index_path.as_path(),
            indexes,
            index_selection,
        ) {
            // Git refused a pathspec the caller wrote for matching nothing: that is the caller's input, not the engine's, so it
            // ends as the finding that names the token. Every other failure of the staging stays an engine failure (issue 6283).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { ended: pathspecMatchingNothing(refused) } // or throw the staging failure
            // ```
            return match stop {
                SelectionStagingFailure::Git(git) => Err(failed(git.0)),
                SelectionStagingFailure::PathspecMatchesNothing(refused) => Ok(IndexBuilt::Ended(
                    Prepared::Ended(pathspec_matching_nothing_ending(refused.as_slice())),
                )),
            };
        }
    }
    // A commit that landed but did not finish its index update leaves this worktree's index describing the branch before that
    //  landing, so a plain commit of this index would record the landed commit's own paths as their old content, and a path it added
    //  as a staged deletion. Which entries are the landing's own cannot be decided from what they hold, so nothing here writes the
    //  live index: this commit's own index is set to the head for the paths whose entry disagrees with it, and the person is told
    //  (`owed_index_repair.rs`, `doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`). An explicit-path commit names
    //  its own selection, so only the notice is given for it.
    let (excluded, unreadable_note): (Vec<String>, Option<String>) =
        crate::owed_index_update::exclude_or_stop(
            context,
            capture,
            indexes,
            setup.selection.mode,
            &index_witness,
        )?;
    if first && let Some(note) = unreadable_note.as_ref() {
        // A notice is worth less than a commit that returns: what cannot be said within the ceiling is left unsaid.
        let _delivered = deliver_notice(format!("{note}\n"));
    }
    if first && let Some(notice) = crate::owed_index_update::excluded_notice(excluded.as_slice()) {
        let _delivered = deliver_notice(notice);
    }
    // What: `IndexBuilt::Ready` is the index being ready, with the word passes' events when a replay merged parts of the change.
    // Why: the caller goes on to the policies with exactly what this build produced, and nothing is read again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { wordMergeEvents };
    // ```
    return Ok(IndexBuilt::Ready {
        word_merge_events,
    });
}
