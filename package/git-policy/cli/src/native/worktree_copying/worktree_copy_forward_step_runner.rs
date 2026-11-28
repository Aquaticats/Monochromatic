//! What:
//!  The runner that starts real Git as a child for each step the copy route runs:
//!  the person's own command with the checkout options it needs,
//!  and the ladder's own steps under their ceilings.
//! Why:
//!  The copy settles only once real Git has returned,
//!  so the wrapper stays alive and starts Git itself;
//!  every step of one command therefore goes through one value,
//!  which decides the parallel-checkout question once and remembers the signal Git died of.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ForwardedGit implements GitRunner {
//!   run(step: GitStep): GitStatus { /* start real Git for this step */ }
//! }
//! ```

use crate::checkout_workers::{
    insert_parallel_checkout,
    is_fallback_checkout,
    parallel_checkout_wanted,
};
use crate::forwarding::{
    ChildOutcome,
    ChildStarted,
    run_real_git_naming,
    run_real_git_until,
};
use crate::hooks_off::StartOwner;
use crate::worktree_command_facts::CopyFacts;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
    GitStep,
    Invocation,
};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_forward_outcome::words_of;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  Decide once whether the checkout gets parallel workers,
/// and say the decision.
/// Why:
///   The first checkout step decides from what the person set,
/// and later steps use that answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parallel ??= parallelCheckoutWanted(context, cwd, arguments); return parallel;
/// ```
fn decided_parallel(
    parallel: &mut Option<bool>,
    invocation: &Invocation,
    arguments: &[OsString],
) -> bool {
    if parallel.is_none() {
        *parallel = Some(parallel_checkout_wanted(
            &invocation.context,
            invocation.effective_cwd.as_path(),
            arguments,
        ));
    }
    return *parallel == Some(true);
}

/// What:
///  The real Git started as a child for each step of the ladder,
/// with the options the step needs.
/// Why:
///   The ladder calls it once per Git step;
/// it carries the invocation,
/// the real Git,
/// the overlay and the parallel-checkout
///  decision that the first checkout step makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ForwardedGit implements GitRunner { parallel?: boolean; run(step) { return runRealGit(realGit, argumentsFor(step), overlayFor(step)); } }
/// ```
pub(super) struct ForwardedGit<'a> {
    /// The variables added to the environment of every step.
    pub(super) overlay: &'a [(OsString, OsString)],
    /// The arguments the person gave.
    pub(super) arguments: &'a [OsString],
    /// What the plan found out about the command.
    pub(super) facts: &'a CopyFacts,
    /// The invocation the ladder runs for.
    pub(super) invocation: &'a Invocation,
    /// The real Git.
    pub(super) real_git: &'a Path,
    /// Whether the checkout may get parallel workers,
    /// decided at the first checkout step from what the person set,
    /// and not again.
    pub(super) parallel: Option<bool>,
    /// The signal the last Git step was ended by,
    /// so a forwarded command's caller sees what Git's caller sees.
    pub(super) signalled: Option<i32>,
}

/// What:
///  The relayed run of one step of the copy route,
///  when the step and its words are a typed network command ssh carries.
/// Why:
///  The copy route starts the person's step itself,
///  so it is the third place a typed network command reaches Git;
///  a step that is not the person's own,
///  or whose remote ssh does not carry,
///  answers nothing and keeps the inherited
///  streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relayedStep(owner, argv, env, step) { return step.kind === 'person' ? relayWhenSshCarries(git, argv, env, owner) : undefined; }
/// ```
impl ForwardedGit<'_> {
    #[cfg(unix)]
    fn relayed_step(
        &self,
        owner: StartOwner,
        arguments: &[OsString],
        overlay: &[(OsString, OsString)],
        step: &GitStep<'_>,
    ) -> Option<std::io::Result<ChildOutcome>> {
        // What: `matches!` asks which step this is: only the person's own step is their command.
        // Why:   The ladder's own steps are the wrapper's work and carry no typed command.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (step.kind !== 'person') return undefined;
        // ```
        if !matches!(step, GitStep::Person) {
            return None;
        }
        // What: `Some(..)` carries the ending as a `ChildOutcome`, the way the other starts of this runner report.
        // Why:   The call site already knows how to report an exit code and a death by a signal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const relayed = relayWhenSshCarries(self.realGit, argv, env, owner);
        // return relayed && ('ran' in relayed ? { exited: relayed.ran } : { signaled: relayed.signalled });
        // ```
        return match crate::typed_network_command_relay::relay_when_ssh_carries(
            self.real_git,
            arguments,
            overlay,
            owner,
        ) {
            Some(Ok(crate::typed_network_command_relay::TypedNetworkCommandEnding::Ran(code))) => {
                Some(Ok(ChildOutcome::Exited(code)))
            }
            Some(Ok(crate::typed_network_command_relay::TypedNetworkCommandEnding::Signalled(
                signal,
            ))) => Some(Ok(ChildOutcome::Signaled(signal))),
            Some(Err(error)) => Some(Err(error)),
            None => None,
        };
    }

    /// What:
    ///  The same for a system with no relay:
    ///  no step is relayed,
    ///  so every step keeps the start it had.
    /// Why:
    ///   Keeps the call site identical on every platform.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// relayedStep(): undefined
    /// ```
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    fn relayed_step(
        &self,
        _owner: StartOwner,
        _arguments: &[OsString],
        _overlay: &[(OsString, OsString)],
        _step: &GitStep<'_>,
    ) -> Option<std::io::Result<ChildOutcome>> {
        return None;
    }
}

impl GitRunner for ForwardedGit<'_> {
    fn run(
        &mut self,
        step: &GitStep<'_>,
        child_started: &mut dyn ChildStarted,
    ) -> Result<GitStatus, WorktreeCopyError> {
        let mut child_overlay: Vec<(OsString, OsString)> = self.overlay.to_vec();
        let mut child_arguments: Vec<OsString> = self.arguments.to_vec();
        match step {
            GitStep::Person => {
                // The person's `git worktree add` is the checkout of the new worktree: it gets parallel workers where nothing sets them.
                if self.facts.creates
                    && decided_parallel(&mut self.parallel, self.invocation, self.arguments)
                {
                    insert_parallel_checkout(&mut child_arguments);
                }
            }
            GitStep::WithoutCheckout(at) => {
                // The index was read from the same arguments, so the word before it is `add`.
                if *at == 0 || *at > child_arguments.len() || child_arguments[*at - 1] != "add" {
                    return Err(WorktreeCopyError(String::from(
                        "monochromatic-git-policy-cli: the worktree plan does not match the arguments.",
                    )));
                }
                child_arguments.insert(*at, OsString::from("--no-checkout"));
            }
            GitStep::Other {
                arguments: other,
                environment,
            } => {
                child_arguments = other.to_vec();
                child_overlay.extend(environment.iter().cloned());
                // The ladder's own checkout after a failed snapshot is the same checkout, so it takes the same option.
                if is_fallback_checkout(child_arguments.as_slice())
                    && decided_parallel(&mut self.parallel, self.invocation, self.arguments)
                {
                    insert_parallel_checkout(&mut child_arguments);
                }
            }
        }
        // Only a step the ladder started has a ceiling; the person's own command waits as long as native Git does, whichever
        //  word it names, and a filter or a filesystem that never answers must not hold a ladder step (owner: nothing waits
        //  forever).
        let ladder_started: bool = matches!(step, GitStep::Other { .. });
        // The person's step is the person's command and keeps the person's monitor setting; a ladder step is the wrapper's own.
        let owner: StartOwner = if ladder_started {
            StartOwner::Wrapper
        } else {
            StartOwner::Person
        };
        let ceiling: Option<std::time::Duration> = if ladder_started {
            crate::ladder_step_ceilings::ceiling_of(child_arguments.as_slice())
        } else {
            None
        };
        let ran: std::io::Result<ChildOutcome> = match ceiling {
            Some(longest) => run_real_git_until(
                self.real_git,
                child_arguments.as_slice(),
                child_overlay.as_slice(),
                longest,
                owner,
            ),
            None => match self.relayed_step(
                owner,
                child_arguments.as_slice(),
                child_overlay.as_slice(),
                step,
            ) {
                Some(answered) => answered,
                None => run_real_git_naming(
                    self.real_git,
                    child_arguments.as_slice(),
                    child_overlay.as_slice(),
                    child_started,
                    owner,
                ),
            },
        };
        match ran {
            Ok(ChildOutcome::Exited(code)) => {
                return Ok(GitStatus {
                    code: Some(code),
                });
            }
            Ok(ChildOutcome::Signaled(signal)) => {
                self.signalled = Some(signal);
                return Ok(GitStatus {
                    code: None,
                });
            }
            // A step that was ended at its ceiling did start, so the failure says what happened rather than that it could not start.
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                return Err(WorktreeCopyError(format!(
                    "monochromatic-git-policy-cli: the step `{}` did not finish: {error}",
                    words_of(child_arguments.as_slice())
                )));
            }
            Err(error) => {
                return Err(WorktreeCopyError(format!(
                    "monochromatic-git-policy-cli: cannot start {}: {error}",
                    self.real_git.display()
                )));
            }
        }
    }
}
