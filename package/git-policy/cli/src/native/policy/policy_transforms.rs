//! What:
//!  The fixed argument transforms of a forwarded command,
//!  in their fixed order:
//!       atomic push,
//!  commit-only,
//!  status hints off,
//!  ignored files kept,
//!  then the one force an interactive clean runs with when its menu would offer a nested repository.
//! Why:
//!  These are not policies:
//!  they have no severity and cannot be configured.
//!  They add
//!      `--atomic` to `git push`,
//!  `-o` to `git commit`,
//!  `-c advice.statusHints=false`
//!      before `git status`,
//!  `--no-overwrite-ignore` to a `git checkout` or `git switch`
//!      that changes files,
//!  and `--no-force -f` to an interactive `git clean` whose menu would offer a nested Git repository,
//!  and the commit-only transform can reject a commit.
//!  The
//!      decisions belong to the pure rule cores;
//!  this module runs them in order and
//!      fetches the two repository facts commit-only may ask for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { args, events, complete } = await applyFixedTransforms({ args, rawArgs, sequence });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Each transform is one rule core;
///  the facts interface answers what a core asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { atomicPush } from '../rule/atomic-push.ts';
/// ```
use crate::command_options::OptionError;
use crate::diagnostics::EngineFailureCode;
use crate::policy_events::PolicyEvent;
use crate::repository_facts::RepositoryFacts;
use crate::rule_argument_rewrite::ArgumentRewrite;
use crate::rule_keep_ignored_files::keep_ignored_files;
use crate::rule_clean_interactive_nested_repositories::{CapDecision, keep_nested_repositories_out_of_an_interactive_menu};
use crate::rule_linked_worktree_nested_repository_messages::kept_out_of_an_interactive_menu_notice;
use crate::rule_atomic_push::atomic_push;
use crate::rule_commit_only::{
    CommitOnlyDecision, decide_commit_only, resolve_index_state, resolve_sequencer_state,
    violation_code_text,
};
use crate::rule_status_hints::status_hints_off;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments are forwarded with their exact bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The identifier of the commit-only transform in its events.
///  `&str` is borrowed
///       text baked into the program.
/// Why:
///   A fixed-transform rejection names its transform where a finding names its policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMMIT_ONLY_CORE = 'commit-only';
/// ```
pub const COMMIT_ONLY_CORE: &str = "commit-only";

/// What:
///  What the transform stage produced.
///  A `struct` is a record with named fields;
///       `Vec<T>` is an owned list;
///  `bool` is true or false.
///  `#[derive(...)]` asks the
///       compiler to generate cloning,
///  debug printing and `==`.
/// Why:
///   The caller forwards `arguments` only when no event blocks and `complete` is true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransformResult = { args: string[]; events: PolicyEvent[]; complete: boolean; notices: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformResult {
    /// The arguments to forward;
    ///  the stage's input when a transform rejected or failed.
    pub arguments: Vec<OsString>,
    /// Nothing,
    ///  one rejection,
    ///  or one engine failure.
    pub events: Vec<PolicyEvent>,
    /// False when a transform could not decide.
    pub complete: bool,
    /// What the person reads before Git runs,
    ///  one sentence per thing a transform kept or changed.
    pub notices: Vec<String>,
}

/// What:
///  The result of a transform that could not read a fact.
///  `&[OsString]` borrows the
///       stage's input;
///  `String` is the owned explanation.
/// Why:
///   A transform that cannot decide stops the command with an engine failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return { args, events: [createEngineFailureEvent({ code: 'core-incomplete', message })], complete: false };
/// ```
fn incomplete(arguments: &[OsString], message: String) -> TransformResult {
    return TransformResult {
        // `.to_vec()` copies the borrowed list into an owned one.
        arguments: arguments.to_vec(),
        // `vec![...]` is a macro building an owned list from literal items.
        events: vec![PolicyEvent::EngineFailure {
            code: EngineFailureCode::CoreIncomplete,
            hint: crate::policy_events::EngineFailureHint::OfCode,
            message,
            // `None` is the "absent" case of `Option`.
            trigger: None,
            policy: None,
            path: None,
        }],
        complete: false,
        notices: Vec::<String>::new(),
    };
}

/// What:
///  Ask for the repository fact a commit-only decision still needs,
///  and finish the
///       decision.
///  `&mut dyn RepositoryFacts` lends "any facts provider" for writing;
///       `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   Most commits are decided from their arguments;
///  only a pathless commit asks
///       whether an operation awaits its conclusion or whether the index is dirty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleCommitOnly(decision, args, facts): Promise<CommitOnlyDecision>; // throws
/// ```
fn settle_commit_only(
    decision: CommitOnlyDecision,
    arguments: &[OsString],
    facts: &mut dyn RepositoryFacts,
) -> Result<CommitOnlyDecision, String> {
    // `match` picks one arm per variant; `other` binds every decision that needs no fact.
    match decision {
        CommitOnlyDecision::NeedsSequencerState => {
            // A trailing `?` returns the failure to our caller, or unwraps the state.
            return Ok(resolve_sequencer_state(facts.sequencer_state()?));
        }
        CommitOnlyDecision::NeedsIndexState(pending) => {
            // `&pending` lends the pending insertion read-only.
            return Ok(resolve_index_state(
                arguments,
                &pending,
                facts.index_vs_head()?,
            ));
        }
        other => return Ok(other),
    }
}

/// What:
///  Run the commit-only transform.
///  `bool` says whether `--no-enforce-only` was given.
/// Why:
///   The hatch skips the transform entirely;
///  its token is already gone from the
///       arguments.
///  A region Git itself refuses is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitOnlyStage(args, escaped, facts): Promise<TransformResult>;
/// ```
fn commit_only_stage(
    arguments: &[OsString],
    escaped: bool,
    facts: &mut dyn RepositoryFacts,
) -> TransformResult {
    let unchanged: TransformResult = TransformResult {
        arguments: arguments.to_vec(),
        // `Vec::<PolicyEvent>::new()` is an empty owned list.
        events: Vec::<PolicyEvent>::new(),
        complete: true,
        notices: Vec::<String>::new(),
    };
    if escaped {
        return unchanged;
    }
    // `&[]` is an empty flag list: wrapper controls were removed before this point.
    let decided: Result<CommitOnlyDecision, OptionError> = decide_commit_only(arguments, &[]);
    // `let Ok(x) = ... else { ... };` unwraps the decision or returns for a refused region.
    let Ok(first) = decided else {
        return unchanged;
    };
    let settled: CommitOnlyDecision = match settle_commit_only(first, arguments, facts) {
        Ok(decision) => decision,
        Err(message) => return incomplete(arguments, message),
    };
    match settled {
        CommitOnlyDecision::Unchanged => return unchanged,
        CommitOnlyDecision::Rewritten(rewritten) => {
            return TransformResult {
                arguments: rewritten,
                events: Vec::<PolicyEvent>::new(),
                complete: true,
                notices: Vec::<String>::new(),
            };
        }
        CommitOnlyDecision::Rejected(violation) => {
            return TransformResult {
                arguments: arguments.to_vec(),
                events: vec![PolicyEvent::CoreFinding {
                    core_id: COMMIT_ONLY_CORE,
                    code: violation_code_text(violation.code),
                    message: violation.message,
                }],
                complete: true,
                notices: Vec::<String>::new(),
            };
        }
        // A finished decision never asks again; one that did is refused, not guessed.
        CommitOnlyDecision::NeedsSequencerState | CommitOnlyDecision::NeedsIndexState(_) => {
            return incomplete(
                arguments,
                String::from(
                    "monochromatic-git-policy-cli: the commit-only transform asked for a second repository fact \
                     and cannot decide.",
                ),
            );
        }
    }
}

/// What:
///  Apply the four transforms in order.
///  `&[OsString]` borrows the arguments,
///  already
///       free of wrapper controls.
/// Why:
///   The order is the incumbent's:
///  atomic push,
///  commit-only,
///  status hints,
///  then ignored files kept,
///  which reads a region Git's own reading already knows and adds one token
///       the caller's own options still override.
///  A rejection or failure of commit-only ends the stage with the stage's input unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function applyFixedTransforms({ args, rawArgs, sequence }): Promise<FixedTransformResult>;
/// ```
pub fn apply_fixed_transforms(
    arguments: &[OsString],
    commit_only_escaped: bool,
    worktree_escaped: bool,
    facts: &mut dyn RepositoryFacts,
) -> TransformResult {
    // `match` unpacks the rewrite; a region Git refuses (`Err`) is left as it is.
    let after_push: Vec<OsString> = match atomic_push(arguments, &[]) {
        Ok(ArgumentRewrite::Rewritten(rewritten)) => rewritten,
        Ok(ArgumentRewrite::Unchanged) | Err(_) => arguments.to_vec(),
    };
    // `.as_slice()` lends the owned list as a borrowed view.
    let commit: TransformResult =
        commit_only_stage(after_push.as_slice(), commit_only_escaped, facts);
    if !commit.complete || !commit.events.is_empty() {
        return TransformResult {
            arguments: arguments.to_vec(),
            events: commit.events,
            complete: commit.complete,
            notices: Vec::<String>::new(),
        };
    }
    let after_status: Vec<OsString> = match status_hints_off(commit.arguments.as_slice()) {
        ArgumentRewrite::Rewritten(rewritten) => rewritten,
        ArgumentRewrite::Unchanged => commit.arguments,
    };
    let after_ignored: Vec<OsString> = match keep_ignored_files(after_status.as_slice()) {
        ArgumentRewrite::Rewritten(rewritten) => rewritten,
        ArgumentRewrite::Unchanged => after_status,
    };
    // An interactive clean cannot be asked what it will delete, so a nested repository it would offer is kept out of the
    //  menu by one force, unless the person asked for the worktree enforcement to be skipped.
    let mut notices: Vec<String> = Vec::<String>::new();
    if worktree_escaped {
        return TransformResult {
            arguments: after_ignored,
            events: Vec::<PolicyEvent>::new(),
            complete: true,
            notices,
        };
    }
    return match keep_nested_repositories_out_of_an_interactive_menu(after_ignored.as_slice(), facts) {
        CapDecision::Unchanged => TransformResult {
            arguments: after_ignored,
            events: Vec::<PolicyEvent>::new(),
            complete: true,
            notices,
        },
        CapDecision::Capped { arguments, repositories } => {
            notices.push(kept_out_of_an_interactive_menu_notice(repositories.as_slice()));
            TransformResult { arguments, events: Vec::<PolicyEvent>::new(), complete: true, notices }
        }
        CapDecision::Stopped { message } => incomplete(after_ignored.as_slice(), message),
    };
}

/// Order,
///  rejection and failure controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_transforms_tests.rs"]
mod tests;
