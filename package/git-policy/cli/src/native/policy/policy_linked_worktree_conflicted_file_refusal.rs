//! What:
//!  The refusal of a checkout or a restore that would replace the resolution of a conflict,
//!  which exists nowhere else.
//! Why:
//!  A worktree file that holds the person's resolution of a conflict is the only copy of that work,
//!  so the form that would write over it is refused and the refusal names every such file;
//!  it is one whole answer of the check,
//!  asked of every command before the check reads the worktree's state,
//!  and it sits beside the check it answers for,
//!  `policy_linked_worktree.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcome = refuseAFormThatWouldReplaceAConflictedFile({ facts, identity, command, target, seen });
//! ```

/// The conflicted files a `checkout` or `restore` would replace,
///  read before the command is refused.
use crate::conflict_side_guard::{
    SideVerdict,
    judge_conflict_sides,
};
/// The record of every decision.
use crate::discard_audit::{
    AuditOutcome,
    record_audit,
};
/// Which worktree a command targets,
///  for the rule that keeps a main checkout and a linked one apart.
use crate::effective_target::EffectiveTarget;
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{
    GuardedCommand,
    LINKED_WORKTREE_REQUIRED_CODE,
};
/// The rejection text that names every conflicted file the command would replace.
use crate::rule_linked_worktree_messages::conflicted_files_message;
/// The one ending this refusal gives.
use crate::the_answers_a_policy_gives::rejected;
/// Which repository shape the invocation selects.
use crate::worktree_identity::WorktreeIdentity;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Every part of the decision about a form that writes a worktree file the person resolved a conflict in:
///  the facts it is judged on,
///  the form Git will run,
///  and the Git directory its refusal is recorded under.
///  A `struct` is a record with named fields,
///  and `'a` is a lifetime,
///  a promise that the three borrows it holds live as long as this record.
/// Why:
///   The refusal asks two questions of one form and then records one line,
///  so the record keeps its parts named instead of becoming a row of five arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConflictedFileForm = { facts: RepositoryFacts; identity: WorktreeIdentity; command: GuardedCommand; target: EffectiveTarget; seen: OsString[] };
/// ```
pub(crate) struct ConflictedFileForm<'a> {
    /// The facts the form's paths are read through.
    pub(crate) facts: &'a mut dyn RepositoryFacts,
    /// The Git directory whose audit record the refusal is written to.
    pub(crate) identity: &'a WorktreeIdentity,
    /// The guarded form Git will run.
    pub(crate) command: GuardedCommand,
    /// Which worktree the form targets.
    pub(crate) target: EffectiveTarget,
    /// The command as Git will run it,
    /// aliases included.
    pub(crate) seen: &'a [OsString],
}

/// What:
///  The refusal of a form that would replace the resolution of a conflict,
///  or nothing when the form writes over no such file.
///  `Option<PolicyOutcome>` is either one outcome or nothing at all
///  (the siblings a reader might expect are a plain `PolicyOutcome` and `Result`).
/// Why:
///   A checkout or a restore that writes a worktree file would replace the resolution of a conflict,
///  which exists nowhere else,
///  so the command is refused and the refusal names every such file.
///  No copy is kept:
///  the person who means to replace that work passes `--no-enforce-worktree`
///  and the command then runs exactly as real Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseAFormThatWouldReplaceAConflictedFile(form: ConflictedFileForm): PolicyOutcome | undefined;
/// ```
pub(crate) fn refuse_a_form_that_would_replace_a_conflicted_file(
    form: ConflictedFileForm<'_>
) -> Option<PolicyOutcome> {
    // Only a form that writes a worktree path can write over a resolved file.
    if !matches!(
        form.command,
        GuardedCommand::CheckoutPaths | GuardedCommand::RestoreWorktree
    ) {
        // `None`: this form is no question for this refusal, and the check reads the form itself.
        return None;
    }
    // Outside a worktree, and in a cache the environment exempts, no resolution of the person's is written over.
    if !matches!(
        form.target,
        EffectiveTarget::MainWorktree
            | EffectiveTarget::LinkedWorktree
            | EffectiveTarget::Allowlisted
    ) {
        return None;
    }
    // `let Variant(x) = ... else { ... };` unwraps Git's own list of the conflicted paths, or answers nothing.
    let SideVerdict::WouldReplace {
        operation,
        paths,
    } = judge_conflict_sides(&form.facts.transaction_context(), form.seen)
    else {
        return None;
    };
    // The line every refusal leaves, so a person reading the record sees which form was stopped and why.
    record_audit(
        form.identity,
        form.command,
        AuditOutcome::Rejected,
        form.seen,
        None,
    );
    let message: String = conflicted_files_message(
        form.command,
        operation,
        form.target == EffectiveTarget::MainWorktree,
        paths.as_slice(),
    );
    // `Some(...)` constructs the present variant of `Option`, wrapping the one finding that stops the command.
    return Some(rejected(LINKED_WORKTREE_REQUIRED_CODE, message));
}
