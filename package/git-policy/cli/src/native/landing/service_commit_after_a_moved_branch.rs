//! What:
//!  What a commit does when the service answers that the branch moved under it:
//!  which attempts are built again,
//! what the replay of each one captures,
//! and which of them end the command with the message the person has to read.
//! Why:
//!  Nothing landed on the attempt's tree,
//! so every answer here either prepares the commit again on the branch as it
//!  is now or names why it cannot,
//! and `service_commit.rs` keeps the loop that runs it.
//!  No rule here refuses a commit for another session's commit
//! ("Commits should never fail."),
//! and no attempt waits for one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const again = afterAMovedBranch(moved); // the capture the next attempt replays
//! ```

use crate::commit_transaction::{CommitInvocation, TransactionStop};
use crate::diagnostic_log::warn;
use crate::service_amend_replayed::{AmendedCommit, amended_fields};
use crate::service_branch_shows::branch_moved;
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_commit_rebuild_ceiling::may_rebuild;
use crate::service_messages::{ClientFailure, render};
use crate::service_prepare::{NativeInputs, Ready};
use crate::service_protocol::CommitRequest;
use crate::service_replay::{Captured, retry_of};
use crate::transaction_journal::Base;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

/// What:
///  One attempt that the service sent back because the branch moved,
/// and what the command has carried from the attempts before it.
/// Why:
///  The rebuild rules read what the first preparation found and what every
///  attempt changed,
/// so the state a rule may write is named here instead of being reached
///  through the loop that owns it.
pub(crate) struct MovedBranch<'a> {
    /// The command line the person ran.
    pub invocation: &'a CommitInvocation<'a>,
    /// The preparation this attempt made,
    /// whose reference is the branch the answer is about.
    pub ready: &'a Ready,
    /// The request this attempt sent.
    pub sent: &'a CommitRequest,
    /// The commit the service builds from the command line,
    /// absent while Git builds it instead.
    pub embedded: Option<&'a Embedded>,
    /// What the preparation captured,
    /// which says whether the branch existed at all.
    pub native_inputs: &'a NativeInputs,
    /// The message Git typed into the commit it built,
    /// absent when no editor ran.
    pub typed: Option<&'a str>,
    /// Whether this attempt is the one Git builds the commit for.
    pub native: bool,
    /// Whether an editor may have run,
    /// which decides what a moved branch costs the person.
    pub editor_possible: bool,
    /// Whether the line asks for the one fixup whose commit Git makes from the branch and not from the index.
    pub rewords: bool,
    /// Which attempt this is,
    /// from zero,
    /// which names the retry's scratch files.
    pub attempt: usize,
    /// The first build's start,
    /// from which the ceiling on rebuilding is counted.
    pub first_build_began: Instant,
    /// The identity this commit's timing record and the service's journal entry share.
    pub id: &'a str,
    /// The fields of the commit the wrapper writes itself,
    /// kept from the attempt that prepared them.
    pub amended: &'a mut Option<AmendedCommit>,
    /// The file the person's message is kept in,
    /// made by the attempt that decides to rebuild from it.
    pub fixed_message: &'a mut Option<PathBuf>,
    /// The command line a rebuild reads,
    /// made once from what the preparation captured.
    pub reissued_line: &'a mut Option<Vec<OsString>>,
    /// The commits Git built that the branch moving made useless,
    /// named once the commit that did land is known.
    pub unlanded: &'a mut Vec<String>,
}

/// What:
///  Decides how this attempt is built again on the branch as it is now,
/// and returns what the next attempt replays.
/// Why:
///   An amend whose branch moved is not made as an amend again:
///  its changes become a new commit on the branch as it is now,
///  leaving the commit that moved it as it is (the owner's choice,
///  2026-11-09).
/// The branch can move again while that new commit is being made,
/// and then the same rule applies again:
///  anything else would refuse the person's commit for another session's
///  commit ("Agents should never wait on each other on commits or merges").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const again = afterAMovedBranch(moved); // throws the stop that ends the command
/// ```
pub(crate) fn after_a_moved_branch(input: MovedBranch<'_>) -> Result<Captured, TransactionStop> {
    let MovedBranch {
        invocation,
        ready,
        sent,
        embedded,
        native_inputs,
        typed,
        native,
        editor_possible,
        rewords,
        attempt,
        first_build_began,
        id,
        amended,
        fixed_message,
        reissued_line,
        unlanded,
    } = input;
    let branch: String = ready.reference.short().spoken();
    if amended.is_some() {
        if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
            return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: attempt + 1 })));
        }
        // The replay merges the amended tree onto the new tip by the same rule an ordinary commit's replay uses.
        return retry_of(invocation.context, invocation.cwd, ready, Vec::new());
    }
    // A `--fixup=reword:<commit>` commit is made from the branch and not from the index: Git carries the branch's
    //  own tree and takes the index out of the commit, so the person asked for an empty commit whose message says
    //  which commit it rewrites. The commit the moved branch makes of it keeps that tree and that message, and the
    //  staged change stays where it is; nothing the person chose is lost, and no command is refused for another
    //  session's commit (the owner's "Commits should never fail.").
    if rewords {
        let Some(commit) = sent.commit.as_deref() else { return Err(branch_moved(ready)) };
        if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
            return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: attempt + 1 })));
        }
        // The change this commit stands for is the tree Git wrote into it, not the private index Git ignored.
        let captured: String = crate::service_replay::tree_of_commit(invocation.context, invocation.cwd, commit)?;
        *amended = Some(amended_fields(invocation, sent, embedded)?);
        unlanded.extend(sent.commit.clone());
        return Ok(crate::service_replay::retry_of_a_prepared_tree(ready, Vec::new(), captured));
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
            return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: attempt + 1 })));
        }
        // Git builds it again from the tree this build captured, not from the worktree.
        let again: Captured = retry_of(invocation.context, invocation.cwd, ready, Vec::new())?;
        unlanded.extend(sent.commit.clone());
        return Ok(again);
    }
    if native && editor_possible && let Some(text) = typed {
        // The person typed this message, and everything else the line took from somewhere other than itself is
        //  settled too: while the bound allows, Git builds the commit again on the new tip from the arguments
        //  `reissued_arguments` makes of what the preparation captured, with no editor and no person.
        let file: PathBuf = native_inputs.capture.layout.git_dir.join(format!("monochromatic-git-policy-cli-saved-message-{}", id.get(..8).unwrap_or(id)));
        let prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
        let reissued: Option<Vec<OsString>> = crate::service_commit_line_reissued::reissued_arguments(
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
                        *fixed_message = Some(file);
                        *reissued_line = Some(line);
                        let again: Captured = retry_of(invocation.context, invocation.cwd, ready, Vec::new())?;
                        unlanded.extend(sent.commit.clone());
                        return Ok(again);
                    }
                    let failure: ClientFailure = ClientFailure::BranchMovedMessageSaved { branch, file: file.display().to_string(), amend: sent.amend };
                    return Err(TransactionStop::Message(render(&failure)));
                }
                Err(error) => warn("serviceCommit", format!("saving the typed message failed: {error}").as_str()),
            }
        }
    }
    return Err(branch_moved(ready));
}
