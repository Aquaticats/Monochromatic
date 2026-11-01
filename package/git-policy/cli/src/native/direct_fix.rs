//! What:
//!  `git cli-git fix`:
//!  run policy passes over the selected files,
//!  apply the
//!       corrections they propose in memory until a pass proposes none,
//!  then install the
//!       corrected files into the worktree when that last pass is clean.
//! Why:
//!  The installed wrapper's `direct-fix.ts` and `direct-fix-convergence.ts` decide the
//!      behaviour:
//!  only the last pass is reported,
//!  a fix summary follows it only when a
//!      file changed,
//!  nothing reaches the worktree unless the last pass exits 0,
//!  and a
//!      fix that cycles or keeps changing ends with one engine failure.
//!  The bounded loop
//!      itself is `policy_convergence::converge`;
//!  this module supplies its passes.
//!      Corrections replace whole file contents in memory instead of being staged into
//!      the private index,
//!  so a pass starts no Git process for them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const result = await runDirectFix({ gitGlobalArgs, pathspecs, policyOptions });
//! ```

/// Import the engine-failure codes.
use super::diagnostics::EngineFailureCode;
/// Import the change list a converged fix installs,
///  and the installer.
use super::direct_fix_install::{InstallChange, install_corrections};
/// Import the event form of a pathname.
use super::event_path::EventPath;
/// Import the shipped checks,
///  whose content state carries the corrections.
use super::policy_checks::ShippedChecks;
/// Import one proposed correction.
use super::policy_content::Correction;
/// Import the bounded loop and its interface.
use super::policy_convergence::{
    Convergence, FIX_PASS_LIMIT_MESSAGE, FixPasses, PassResult as ConvergencePass, converge,
};
/// Import the stage request,
///  how a stage ended and the exit code of a pass.
use super::policy_engine::{StageEnd, StageRequest, pass_exit_code};
/// Import the events a fix reports.
use super::policy_events::PolicyEvent;
/// Import one policy pass and its result.
use super::policy_pass::{PassResult, run_policy_pass};
/// Import the lifecycle point of a fix.
use super::policy_trigger::Trigger;
/// Import the facts interface.
use super::repository_facts::RepositoryFacts;
/// `BTreeMap<K, V>` is a map kept sorted by key.
use std::collections::BTreeMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The message of a fix whose corrections returned to an earlier state,
///  as the installed
/// wrapper's direct fix words it.
pub const DIRECT_FIX_CYCLE_MESSAGE: &str =
    "Policy patches entered a repeated candidate-state cycle.";

/// The message of a fix that has no worktree to install into.
pub const NO_WORKTREE_MESSAGE: &str =
    "cli-git fix could not install its corrections: the repository has no worktree.";

/// What:
///  The passes of one direct fix,
///  as the bounded loop sees them.
///  `'a` names how
///       long the borrowed request and checks last.
/// Why:
///   The loop owns the control flow;
///  this owns running a pass,
///  applying its
///       corrections and remembering each changed state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DirectFixPasses implements FixPasses { last?: PassResult; states: Map<string, InstallChange>[] }
/// ```
pub struct DirectFixPasses<'a, F: RepositoryFacts> {
    /// What every pass evaluates.
    request: &'a StageRequest,
    /// The checks,
    ///  kept across passes so candidates and rules are prepared once.
    checks: &'a mut ShippedChecks<F>,
    /// The most recent pass,
    ///  which is the one reported.
    last: Option<PassResult>,
    /// The corrected files of every state:
    ///  the first is the uncorrected one.
    states: Vec<BTreeMap<Vec<u8>, InstallChange>>,
}

/// What:
///  How the loop reads a pass's ending.
/// Why:
///   A proposal asks for another pass;
///  a pass that could not decide blocks the fix;
///       a completed or stopped pass proposes nothing,
///  so the content is stable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const convergencePass = (end) => end === 'proposed' ? 'proposed' : end === 'failed' || isUnavailable(end) ? 'failed' : 'stable';
/// ```
pub fn convergence_pass(end: StageEnd) -> ConvergencePass {
    match end {
        StageEnd::Proposed => return ConvergencePass::Proposed,
        StageEnd::Failed | StageEnd::Unavailable(_) => return ConvergencePass::Failed,
        StageEnd::Completed | StageEnd::Stopped => return ConvergencePass::Stable,
    }
}

/// What:
///  `impl<F: RepositoryFacts> DirectFixPasses<'_, F> { ... }` attaches the constructor
///       and the hand-back of what the loop left.
/// Why:
///   The states start with the uncorrected one,
///  which the cycle check compares against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DirectFixPasses { constructor(request, checks) {} }
/// ```
impl<'a, F: RepositoryFacts> DirectFixPasses<'a, F> {
    /// What:
    ///  Passes over `checks` with no pass run yet and the uncorrected state stored.
    /// Why:
    ///   State number `n` is the state after `n` changes,
    ///  as the loop numbers them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(request: StageRequest, checks: ShippedChecks)
    /// ```
    pub fn new(request: &'a StageRequest, checks: &'a mut ShippedChecks<F>) -> Self {
        return DirectFixPasses {
            request,
            checks,
            last: None,
            states: vec![BTreeMap::new()],
        };
    }
}

/// What:
///  `impl FixPasses for DirectFixPasses<'_, F>` gives the loop its three operations.
/// Why:
///   The loop is tested without Git;
///  these are the real passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DirectFixPasses implements FixPasses {}
/// ```
impl<F: RepositoryFacts> FixPasses for DirectFixPasses<'_, F> {
    /// What:
    ///  Run one whole policy pass on the current content and remember it.
    /// Why:
    ///   Proposals left from an earlier pass were applied already;
    ///  a pass starts
    ///       with none,
    ///  so it is read only from what this pass proposes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async runPass(version: number): Promise<'stable' | 'proposed' | 'failed'>;
    /// ```
    fn run_pass(&mut self, _candidate_version: usize) -> ConvergencePass {
        // What: `let _ = ...` drops proposals no `apply` took.
        // Why:  Only a pass that ends proposing is followed by `apply`; a pass that
        //       proposed and then failed must not carry its proposals into another.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // content.takeProposals();
        // ```
        let _ = self.checks.content.take_proposals();
        let pass: PassResult = run_policy_pass(self.request, self.checks);
        let result: ConvergencePass = convergence_pass(pass.end);
        // `Some(x)` is the "present" case of `Option`.
        self.last = Some(pass);
        return result;
    }

    /// What:
    ///  Apply the last pass's proposals and store the resulting state.
    /// Why:
    ///   Applying in memory cannot fail,
    ///  so the fix is never blocked here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async apply(snapshot: number): Promise<boolean>;
    /// ```
    fn apply(&mut self, _snapshot: usize) -> bool {
        let proposals: Vec<Correction> = self.checks.content.take_proposals();
        self.checks.content.apply(proposals);
        self.states.push(self.checks.content.corrected_state());
        return true;
    }

    /// What:
    ///  Whether two stored states hold the same bytes in every file.
    /// Why:
    ///   Equal corrected maps mean equal content,
    ///  because a file whose bytes return to
    ///       its original is no longer in the map.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async snapshotsEqual(left: number, right: number): Promise<boolean>;
    /// ```
    fn snapshots_equal(&mut self, left: usize, right: usize) -> bool {
        return self.states[left] == self.states[right];
    }
}

/// What:
///  A fix result made of one engine failure of the fix,
///  as the installed wrapper
///       reports a cycle,
///  a pass limit or an installation failure:
///  earlier events are
///       dropped.
///  `&[std::ffi::OsString]` borrows the command's arguments.
/// Why:
///   The person sees why nothing was installed,
///  and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixFailure = (code, message) => ({ events: [engineFailure(code, message, 'direct-fix')], exitCode: 2 });
/// ```
pub fn fix_failure(
    arguments: &[std::ffi::OsString],
    code: EngineFailureCode,
    message: &str,
) -> PassResult {
    return PassResult {
        arguments: arguments.to_vec(),
        events: vec![PolicyEvent::EngineFailure {
            code,
            message: String::from(message),
            trigger: Some(Trigger::DirectFix),
            // `None`: the fix itself failed, not one policy, and no file is named.
            policy: None,
            path: None,
        }],
        end: StageEnd::Failed,
    };
}

/// What:
///  The result of a settled fix:
///  the last pass as it is,
///  or that pass followed by a
///       fix summary once its corrections were installed.
///  `Option<&Path>` is the
///       worktree's top level or nothing.
/// Why:
///   Only a last pass that exits 0 installs anything,
///  and the summary is reported only
///       after a file changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settledFix(pass, changedPasses, checks, root): Promise<PassResult>;
/// ```
pub fn settled_fix<F: RepositoryFacts>(
    pass: PassResult,
    changed_passes: usize,
    checks: &ShippedChecks<F>,
    root: Option<&Path>,
) -> PassResult {
    if pass_exit_code(pass.events.as_slice(), pass.end) != 0 {
        return pass;
    }
    let changes: Vec<InstallChange> = checks.content.install_changes();
    if changes.is_empty() {
        return pass;
    }
    let (Some(top_level), Some(real_index)) = (root, checks.content.real_index()) else {
        return fix_failure(
            pass.arguments.as_slice(),
            EngineFailureCode::TransactionFailed,
            NO_WORKTREE_MESSAGE,
        );
    };
    if let Err(message) = install_corrections(top_level, real_index, changes.as_slice()) {
        return fix_failure(
            pass.arguments.as_slice(),
            EngineFailureCode::TransactionFailed,
            message.as_str(),
        );
    }
    let mut changed_paths: Vec<EventPath> = Vec::new();
    for change in &changes {
        changed_paths.push(EventPath::from_git_bytes(change.path.as_slice()));
    }
    let mut events: Vec<PolicyEvent> = pass.events;
    events.push(PolicyEvent::FixSummary {
        trigger: Trigger::DirectFix,
        // `as u64` widens the count to the event's number type.
        passes: changed_passes as u64,
        changed_paths,
    });
    return PassResult {
        arguments: pass.arguments,
        events,
        end: pass.end,
    };
}

/// What:
///  What a fix reports once the loop ended:
///  `convergence` says how,
///  `pass` is the
///       last pass run.
/// Why:
///   A settled fix may install;
///  a blocked fix reports the pass that could not decide;
///       a cycle and a pass limit report one engine failure each,
///  as the installed
///       wrapper's `fixCycleFailure` and `fixPassLimitFailure` do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishFix(convergence, pass, checks, root): Promise<PassResult>;
/// ```
pub fn finish_fix<F: RepositoryFacts>(
    convergence: Convergence,
    pass: PassResult,
    checks: &ShippedChecks<F>,
    root: Option<&Path>,
) -> PassResult {
    match convergence {
        Convergence::Settled { changed_passes } => {
            return settled_fix(pass, changed_passes, checks, root);
        }
        Convergence::Blocked => return pass,
        Convergence::FixCycle => {
            return fix_failure(
                pass.arguments.as_slice(),
                EngineFailureCode::FixCycle,
                DIRECT_FIX_CYCLE_MESSAGE,
            );
        }
        Convergence::FixPassLimit => {
            return fix_failure(
                pass.arguments.as_slice(),
                EngineFailureCode::FixPassLimit,
                FIX_PASS_LIMIT_MESSAGE,
            );
        }
    }
}

/// What:
///  Run a whole direct fix and return what it reports.
///  `root` is the worktree's top
///       level,
///  where corrected files are installed.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runDirectFix(request, checks, root): Promise<PassResult>;
/// ```
pub fn run_direct_fix<F: RepositoryFacts>(
    request: &StageRequest,
    checks: &mut ShippedChecks<F>,
    root: Option<&Path>,
) -> PassResult {
    let mut passes: DirectFixPasses<'_, F> = DirectFixPasses::new(request, checks);
    let convergence: Convergence = converge(&mut passes);
    let Some(pass) = passes.last else {
        unreachable!("the bounded loop runs at least one pass");
    };
    return finish_fix(convergence, pass, checks, root);
}

/// Pass,
///  settlement and installation-boundary controls stay out of the release executable.
#[cfg(test)]
#[path = "direct_fix_tests.rs"]
mod tests;
