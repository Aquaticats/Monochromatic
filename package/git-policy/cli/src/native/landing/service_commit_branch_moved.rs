//! What:
//!  What a commit comes to when the service answers that another session moved the branch:
//!  build it again from what this attempt captured,
//!  or say why it cannot be made.
//! Why:
//!  Each kind of commit pays a different price for the move,
//!  and the branch may keep moving while the rebuild runs:
//!  an amend lands as a new commit on the branch as it is now,
//!  a `--fixup=reword:` commit keeps the tree Git wrote,
//!  and a commit that depends on the tip is built again only while the ceiling allows
//!  (`service_commit_rebuild_ceiling.rs`).
//!  Nothing here asks the person a second time,
//!  and nothing waits on the session that moved the branch
//!  (the owner's "Commits should never fail.").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await afterTheBranchMoved({ invocation, ready, nativeInputs, sent, typed, attempt }); // throws when the commit cannot be made
//! ```

use crate::commit_transaction::{CommitInvocation, TransactionStop};
use crate::diagnostic_log::warn;
use crate::service_amend_replayed::{AmendedCommit, amended_fields};
use crate::service_branch_shows::branch_moved;
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_commit_line_reissued::reissued_arguments;
use crate::service_commit_rebuild_ceiling::may_rebuild;
use crate::service_messages::{ClientFailure, render};
use crate::service_prepare::{NativeInputs, Ready};
use crate::service_protocol::CommitRequest;
use crate::service_replay::{Captured, retry_of, retry_of_a_prepared_tree, tree_of_commit};
use crate::transaction_journal::Base;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

/// One attempt of a commit at the moment the service answered `Moved`:
///  the command line,
///  what the preparation captured,
///  and the commit kinds the attempt had settled.
pub(crate) struct MovedBranch<'a> {
    /// The command line being made.
    pub(crate) invocation: &'a CommitInvocation<'a>,
    /// What this attempt prepared,
    ///  whose private index the rebuild captures the change from.
    pub(crate) ready: &'a Ready,
    /// The capture and the layout the rebuild reads.
    pub(crate) native_inputs: &'a NativeInputs,
    /// The request the service answered.
    pub(crate) sent: &'a CommitRequest,
    /// The commit the service builds itself,
    ///  read again when a reworded fixup lands.
    pub(crate) embedded: Option<&'a Embedded>,
    /// The amend this attempt carried,
    ///  which a moved branch lands as a new commit.
    pub(crate) amended: Option<&'a AmendedCommit>,
    /// The message Git typed,
    ///  present only when an editor may have run.
    pub(crate) typed: Option<&'a str>,
    /// This commit's identity,
    ///  which names the file a typed message is kept in.
    pub(crate) id: &'a str,
    /// Which attempt this is,
    ///  counted from zero.
    pub(crate) attempt: usize,
    /// When the first build began,
    ///  which the ceiling on rebuilding counts from.
    pub(crate) first_build_began: Instant,
    /// Whether Git builds this commit rather than the service.
    pub(crate) native: bool,
    /// Whether an editor may have run,
    ///  which decides what the move costs the person.
    pub(crate) editor_possible: bool,
    /// Whether the line is the `--fixup=reword:` commit Git makes from the branch.
    pub(crate) rewords: bool,
}

/// What the next attempt starts from,
///  and what this attempt leaves behind for the person.
pub(crate) struct Rebuild {
    /// The capture the next preparation replays.
    pub(crate) retry: Captured,
    /// The amend's fields,
    ///  when this attempt settled them for a commit that lands as a new one.
    pub(crate) amended: Option<AmendedCommit>,
    /// The file a typed message was kept in,
    ///  when the line is built again from it.
    pub(crate) fixed_message: Option<PathBuf>,
    /// The line a rebuild of a commit Git built is built with.
    pub(crate) reissued_line: Option<Vec<OsString>>,
    /// The commits of this attempt the move made useless,
    ///  whose summaries Git already printed.
    pub(crate) unlanded: Vec<String>,
}

/// The stop a commit ends with once the ceiling on rebuilding is spent.
fn keeps_changing(branch: String, attempt: usize) -> TransactionStop {
    return TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging {
        branch,
        times: attempt + 1,
    }));
}

/// The rebuild that starts from this capture and settles nothing else.
fn only_the_capture(retry: Captured) -> Rebuild {
    return Rebuild {
        retry,
        amended: None,
        fixed_message: None,
        reissued_line: None,
        unlanded: Vec::new(),
    };
}

/// What:
///  Decide what a moved branch makes of this attempt.
/// Why:
///  The rebuilder reads the change the service left to this command from its private index,
///  so every answer here names a capture the next attempt replays.
///
/// # Errors
///
/// `TransactionStop::Message` when the ceiling is spent,
///  the amended or reworded commit cannot be read back,
///  or the commit cannot be built again without asking the person.
pub(crate) fn after_the_branch_moved(moved: MovedBranch<'_>) -> Result<Rebuild, TransactionStop> {
    let MovedBranch {
        invocation,
        ready,
        native_inputs,
        sent,
        embedded,
        amended,
        typed,
        id,
        attempt,
        first_build_began,
        native,
        editor_possible,
        rewords,
    } = moved;
    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
    // An amend whose branch moved is not made as an amend again: its changes become a new commit on the branch as
    //  it is now, leaving the commit that moved it as it is (the owner's choice, 2026-11-09).
    //  The branch can move again while that new commit is being made, and then the same rule applies again:
    //  anything else would refuse the person's commit for another session's commit
    //  ("Agents should never wait on each other on commits or merges").
    //  No one waits, and no command is refused for another's commit.
    if amended.is_some() {
        if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
            return Err(keeps_changing(branch, attempt));
        }
        // The replay merges the amended tree onto the new tip by the same rule an ordinary commit's replay uses.
        return Ok(only_the_capture(retry_of(invocation.context, invocation.cwd, ready, Vec::new())?));
    }
    // A `--fixup=reword:<commit>` commit is made from the branch and not from the index: Git carries the branch's
    //  own tree and takes the index out of the commit, so the person asked for an empty commit whose message says
    //  which commit it rewrites. The commit the moved branch makes of it keeps that tree and that message, and the
    //  staged change stays where it is; nothing the person chose is lost, and no command is refused for another
    //  session's commit (the owner's "Commits should never fail.").
    if rewords {
        let Some(commit) = sent.commit.as_deref() else { return Err(branch_moved(ready)) };
        if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
            return Err(keeps_changing(branch, attempt));
        }
        // The change this commit stands for is the tree Git wrote into it, not the private index Git ignored.
        let captured: String = tree_of_commit(invocation.context, invocation.cwd, commit)?;
        return Ok(Rebuild {
            retry: retry_of_a_prepared_tree(ready, Vec::new(), captured),
            amended: Some(amended_fields(invocation, sent, embedded)?),
            fixed_message: None,
            reissued_line: None,
            unlanded: sent.commit.clone().into_iter().collect(),
        });
    }
    // Nobody typed or chose anything, and the commit does not depend on the tip it was made on: Git builds it
    //  again on the new tip (and signs it again), up to the bound.
    //  A commit that concludes a merge, a cherry-pick or a revert depends on the state the worktree holds, not on
    //  the tip: its parents come from `MERGE_HEAD`, `CHERRY_PICK_HEAD` or `REVERT_HEAD`, so the same rebuild makes
    //  it on the branch as it is now, and the preparation refuses when that state is no longer the same.
    let conclusion_may_move: bool = ready.cleanup.is_some();
    // A branch that did not exist when this commit was prepared names no parent to keep: the change it captured is
    //  a whole root tree, which the replay puts on the branch as another session created it
    //  (`service_replay.rs`, `Captured::unborn`).
    let parents_are_fixed: bool = ready.fixed_parents && !matches!(native_inputs.capture.base, Base::Unborn);
    if native && !sent.amend && !editor_possible && !ready.interactive && (!parents_are_fixed || conclusion_may_move) {
        if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
            return Err(keeps_changing(branch, attempt));
        }
        // Git builds it again from the tree this build captured, not from the worktree.
        return Ok(Rebuild {
            retry: retry_of(invocation.context, invocation.cwd, ready, Vec::new())?,
            amended: None,
            fixed_message: None,
            reissued_line: None,
            unlanded: sent.commit.clone().into_iter().collect(),
        });
    }
    if native && editor_possible && let Some(text) = typed {
        // The person typed this message, and everything else the line took from somewhere other than itself is
        //  settled too: while the bound allows, Git builds the commit again on the new tip from the arguments
        //  `reissued_arguments` makes of what the preparation captured, with no editor and no person.
        let file: PathBuf = native_inputs
            .capture
            .layout
            .git_dir
            .join(format!("monochromatic-git-policy-cli-saved-message-{}", id.get(..8).unwrap_or(id)));
        let prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
        let reissued: Option<Vec<OsString>> = reissued_arguments(
            invocation.context,
            invocation.cwd,
            native_inputs.arguments.as_slice(),
            prefix,
            file.as_path(),
            sent.commit.as_deref(),
            ready.interactive,
        )?;
        // A line a rebuild cannot read back (an abbreviated borrowing option the wrapper does not guess at) keeps no
        //  message file: the same command makes that message again, and saving it here would suggest a commit of
        //  another kind.
        if let Some(line) = reissued {
            match std::fs::write(&file, text) {
                Ok(()) => {
                    let repeatable: bool = !sent.amend && (!parents_are_fixed || conclusion_may_move);
                    if repeatable && may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
                        return Ok(Rebuild {
                            retry: retry_of(invocation.context, invocation.cwd, ready, Vec::new())?,
                            amended: None,
                            fixed_message: Some(file),
                            reissued_line: Some(line),
                            unlanded: sent.commit.clone().into_iter().collect(),
                        });
                    }
                    let failure: ClientFailure = ClientFailure::BranchMovedMessageSaved {
                        branch,
                        file: file.display().to_string(),
                        amend: sent.amend,
                    };
                    return Err(TransactionStop::Message(render(&failure)));
                }
                Err(error) => warn("serviceCommit", format!("saving the typed message failed: {error}").as_str()),
            }
        }
    }
    return Err(branch_moved(ready));
}
