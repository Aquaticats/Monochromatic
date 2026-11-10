//! What:
//!  What a commit does when the service sends it back because the branch moved,
//!  and what can be built again.
//! Why:
//!  Three attempts can follow a moved branch:
//!  an amend whose changes become a new commit,
//!  an ordinary commit Git builds again on the branch as it is now,
//!  and a commit whose typed message is kept and read back from its file.
//!  The ceiling of the rebuilding is `service_commit_rebuild_ceiling.rs`,
//!  and the loop that uses this decision is `service_commit.rs`,
//!  which is at its line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const after = afterAMovedBranch({ ready, sent, native, editorPossible, amended, attempt, firstBuildBegan });
//! // { rebuild: { fixedMessage?, captured, unlanded? } } | { ended: words } | { moved: true }
//! ```

/// The commit's invocation:
/// its context and the directory its Git runs in.
use crate::commit_transaction::{CommitInvocation, TransactionStop};
/// The length of the global prefix of a command line.
use crate::global_arguments::global_layout;
/// The rebinding of a command line to the file a typed message was kept in.
use crate::service_commit_args::with_fixed_message;
/// The ceiling of the rebuilding,
/// and the notice a rebuild prints.
use crate::service_commit_rebuild_ceiling::may_rebuild;
/// The owner of the file a typed message is kept in.
use crate::service_commit_saved_message::keep_the_typed_message;
/// The client failures a moved branch ends with.
use crate::service_messages::{ClientFailure, render};
/// The prepared attempt and the capture it settled.
use crate::service_prepare::{NativeInputs, Ready};
/// The request the service did not land.
use crate::service_protocol::CommitRequest;
/// What the next preparation replays,
/// and its one caller.
use crate::service_replay::{Captured, retry_of};
/// A path of the committed message's file.
use std::path::PathBuf;
/// When the first build began,
/// which the ceiling is counted from.
use std::time::Instant;

/// What a moved branch is decided with.
pub(crate) struct MovedBranch<'a> {
    /// The invocation the commit runs for.
    pub invocation: &'a CommitInvocation<'a>,
    /// The attempt the service sent back.
    pub ready: &'a Ready,
    /// The capture and the arguments the attempts are built from.
    pub native_inputs: &'a NativeInputs,
    /// The request the service did not land.
    pub sent: &'a CommitRequest,
    /// The message of this attempt,
    /// as Git built it or as the person typed it.
    pub typed: Option<&'a str>,
    /// Whether Git built this attempt rather than the service.
    pub native: bool,
    /// Whether an editor may have run,
    /// which decides what a moved branch costs the person.
    pub editor_possible: bool,
    /// Whether an amend prepared this attempt.
    pub amended: bool,
    /// Which preparation this is,
    /// from zero.
    pub attempt: usize,
    /// When the first build of this commit began.
    pub first_build_began: Instant,
    /// The identity the request was sent with,
    /// which names the file a typed message is kept in.
    pub id: &'a str,
}

/// What a moved branch came to.
pub(crate) enum AfterAMovedBranch {
    /// The commit is built again on the branch as it is now.
    Rebuild {
        /// The file the typed message is now read from,
        /// when this attempt kept one.
        fixed_message: Option<PathBuf>,
        /// What the next preparation replays.
        captured: Captured,
        /// The commit the moved branch made useless,
        /// when Git had built it.
        unlanded: Option<String>,
    },
    /// Nothing is built again,
    /// and the commit ends with these words.
    Ended(TransactionStop),
    /// Nothing is built again,
    /// and the commit ends as a command whose branch moved does.
    Moved,
}

/// What:
///  Decide what a commit does once the service reports that the branch moved under it.
/// Why:
///   The person's message is theirs and is kept,
///  a commit nobody typed or chose anything for is built again up to the ceiling,
///  and an amend becomes a new commit while the replaced one stays as it is
///  (the owner's choice,
///  2026-11-09).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterAMovedBranch(moved)
/// ```
pub(crate) fn after_a_moved_branch(moved: MovedBranch<'_>) -> Result<AfterAMovedBranch, TransactionStop> {
    let MovedBranch { invocation, ready, native_inputs, sent, typed, native, editor_possible, amended, attempt, first_build_began, id } = moved;
    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
    // An amend whose branch moved is not made as an amend again: its changes become a new commit on the branch as
    //  it is now, leaving the commit that moved it as it is. No one waits, and no command is refused for another's commit.
    if sent.amend && amended {
        rebuild_within_the_ceiling(branch.as_str(), attempt, first_build_began)?;
        // The replay merges the amended tree onto the new tip by the same rule an ordinary commit's replay uses.
        return Ok(AfterAMovedBranch::Rebuild {
            fixed_message: None,
            captured: retry_of(invocation.context, invocation.cwd, ready, Vec::new())?,
            unlanded: None,
        });
    }
    // Nobody typed or chose anything, and the commit does not depend on the tip it was made on: Git builds it
    //  again on the new tip (and signs it again), up to the bound.
    //  A commit that concludes a merge, a cherry-pick or a revert depends on the state the worktree holds, not on
    //  the tip: its parents come from `MERGE_HEAD`, `CHERRY_PICK_HEAD` or `REVERT_HEAD`, so the same rebuild makes
    //  it on the branch as it is now, and the preparation refuses when that state is no longer the same.
    let conclusion_may_move: bool = ready.cleanup.is_some();
    if native && !sent.amend && !editor_possible && !ready.interactive && (!ready.fixed_parents || conclusion_may_move) {
        rebuild_within_the_ceiling(branch.as_str(), attempt, first_build_began)?;
        // Git builds it again from the tree this build captured, not from the worktree.
        return Ok(AfterAMovedBranch::Rebuild {
            fixed_message: None,
            captured: retry_of(invocation.context, invocation.cwd, ready, Vec::new())?,
            unlanded: sent.commit.clone(),
        });
    }
    if native && editor_possible && let Some(text) = typed {
        // The person typed this message: keep it where they can reuse it.
        let Some(file): Option<PathBuf> = keep_the_typed_message(native_inputs.capture.layout.git_dir.as_path(), id, Some(text)) else {
            return Ok(AfterAMovedBranch::Moved);
        };
        // After the editor the message is fixed: while the bound allows, Git builds the commit again on the new
        //  tip from `-F <file>`, with no editor and no person, unless the line borrows its author or its parents.
        let prefix: usize = global_layout(native_inputs.arguments.as_slice()).prefix_len;
        let repeatable: bool = !sent.amend
            && !ready.interactive
            && (!ready.fixed_parents || conclusion_may_move)
            && with_fixed_message(native_inputs.arguments.as_slice(), prefix, file.as_path()).is_some();
        if repeatable && may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
            return Ok(AfterAMovedBranch::Rebuild {
                fixed_message: Some(file),
                captured: retry_of(invocation.context, invocation.cwd, ready, Vec::new())?,
                unlanded: sent.commit.clone(),
            });
        }
        let failure: ClientFailure = ClientFailure::BranchMovedMessageSaved { branch, file: file.display().to_string(), amend: sent.amend };
        return Ok(AfterAMovedBranch::Ended(TransactionStop::Message(render(&failure))));
    }
    return Ok(AfterAMovedBranch::Moved);
}

/// What:
///  Refuse to build the commit again once the ceiling has passed,
///  naming the branch that keeps moving and how many builds were made.
/// Why:
///   Both rebuilds past a moved branch pass the same ceiling,
///  and a commit that the ceiling ends says so in the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!mayRebuild(branch, attempt + 1, firstBuildBegan)) throw branchKeepsChanging(branch, attempt + 1);
/// ```
fn rebuild_within_the_ceiling(branch: &str, attempt: usize, first_build_began: Instant) -> Result<(), TransactionStop> {
    if may_rebuild(branch, attempt + 1, first_build_began) {
        return Ok(());
    }
    return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch: branch.to_string(), times: attempt + 1 })));
}
