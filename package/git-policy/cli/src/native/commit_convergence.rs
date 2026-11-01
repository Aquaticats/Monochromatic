//! What:
//!  The commit path's bounded policy convergence over the private commit index:
//!       each changed pass writes its corrections into the index,
//!       then the whole ordered pass restarts until it proposes nothing.
//! Why:
//!  The commit path must end with the corrections the policies settled on,
//!       and must stop when they repeat a state or do not settle.
//!  The loop is the shared `converge`;
//!       this module supplies the commit path's three operations,
//!       as the direct fix does for the worktree.
//!  Exact prior states are compared in memory; the incumbent also streams them to snapshot files,
//!       which only serve the same comparison.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const convergence = await convergeCommitPolicies({ args, gitPath, cwd, workspace, policyOptions, baseRevision, ... });
//! ```

use super::diagnostics::EngineFailureCode;
use super::direct_fix::convergence_pass;
use super::direct_fix_install::InstallChange;
use super::policy_checks::ShippedChecks;
use super::policy_content::Correction;
use super::policy_convergence::{Convergence, FIX_PASS_LIMIT_MESSAGE, FixPasses, PassResult as ConvergencePass, converge};
use super::policy_engine::{StageEnd, StageRequest};
use super::policy_events::PolicyEvent;
use super::policy_pass::{PassResult, run_policy_pass};
use super::path_bytes::path_message_text;
use super::policy_trigger::Trigger;
use super::private_index_stage::{PrivateStage, stage_correction};
use super::repository_facts::RepositoryFacts;
use std::collections::BTreeMap;
use std::ffi::OsString;

/// What:
///  The message of a commit-path loop that repeated an exact prior state.
/// Why:
///   The wording is the incumbent's, so the two paths report the same failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = 'Policy patches repeated an exact prior candidate state.';
/// ```
pub const COMMIT_CYCLE_MESSAGE: &str = "Policy patches repeated an exact prior candidate state.";

/// What:
///  Everything one commit-path convergence reads and writes.
/// Why:
///   The request, the shipped checks and the private index travel together, so no caller can pair them wrongly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvergenceInput = { request; checks; stage };
/// ```
pub struct CommitConvergenceInput<'a, F: RepositoryFacts> {
    /// The pre-forward stage request the passes run under.
    pub request: &'a StageRequest,
    /// The shipped checks whose facts read the private index.
    pub checks: &'a mut ShippedChecks<F>,
    /// The private index and object store the corrections are written to.
    pub stage: PrivateStage<'a>,
}

/// What:
///  The settled or stopped result of one commit-path convergence.
/// Why:
///   The caller needs the outcome, the events that explain it, and the paths the loop corrected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitConvergence = { kind: 'settled' | 'blocked'; pass; changedPaths };
/// ```
pub struct CommitConvergence {
    /// How the bounded loop ended.
    pub convergence: Convergence,
    /// The pass whose events describe the outcome.
    pub pass: PassResult,
    /// Paths whose private-index entry a correction wrote, in first-write order.
    pub changed_paths: Vec<Vec<u8>>,
}

/// What:
///  The commit path's three loop operations over the private index.
/// Why:
///   The loop calls these in order; holding the state here keeps each pass's proposals with its own writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommitPasses { runPass(); apply(); snapshotsEqual(); }
/// ```
struct CommitPasses<'a, F: RepositoryFacts> {
    /// The pre-forward request.
    request: &'a StageRequest,
    /// The shipped checks.
    checks: &'a mut ShippedChecks<F>,
    /// The private index and object store.
    stage: PrivateStage<'a>,
    /// The most recent pass.
    last: Option<PassResult>,
    /// The pass that failed to write its corrections, when that is why the loop stopped.
    blocked: Option<PassResult>,
    /// Exact corrected state after each changed pass, starting from the uncorrected state.
    states: Vec<BTreeMap<Vec<u8>, InstallChange>>,
    /// Paths written so far, in first-write order.
    changed_paths: Vec<Vec<u8>>,
}

impl<'a, F: RepositoryFacts> CommitPasses<'a, F> {
    /// What:
    ///  Start the loop with the uncorrected state stored as state zero.
    /// Why:
    ///   Snapshot zero is the state every later change is compared against.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const visited = [firstSnapshot];
    /// ```
    fn new(input: CommitConvergenceInput<'a, F>) -> Self {
        return CommitPasses {
            request: input.request,
            checks: input.checks,
            stage: input.stage,
            last: None,
            blocked: None,
            states: vec![BTreeMap::new()],
            changed_paths: Vec::new(),
        };
    }
}

impl<F: RepositoryFacts> FixPasses for CommitPasses<'_, F> {
    fn run_pass(&mut self, _candidate_version: usize) -> ConvergencePass {
        // What: `let _ = ...` drops proposals no `apply` took.
        // Why:  Only a pass that ends proposing is followed by `apply`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // content.takeProposals();
        // ```
        let _ = self.checks.content.take_proposals();
        let pass: PassResult = run_policy_pass(self.request, self.checks);
        let result: ConvergencePass = convergence_pass(pass.end);
        self.last = Some(pass);
        return result;
    }

    fn apply(&mut self, _snapshot: usize) -> bool {
        let proposals: Vec<Correction> = self.checks.content.take_proposals();
        let arguments: Vec<OsString> = match &self.last {
            Some(pass) => pass.arguments.clone(),
            None => unreachable!("apply follows a pass that proposed corrections"),
        };
        for correction in &proposals {
            if let Err(message) = stage_correction(&self.stage, correction) {
                self.blocked = Some(patch_conflict_pass(arguments.as_slice(), correction.path.as_slice(), message));
                return false;
            }
            record_path(&mut self.changed_paths, correction);
        }
        self.checks.content.apply(proposals);
        self.states.push(self.checks.content.corrected_state());
        return true;
    }

    fn snapshots_equal(&mut self, left: usize, right: usize) -> bool {
        return self.states[left] == self.states[right];
    }
}

/// What:
///  Add a correction's path to the written list once, keeping first-write order.
/// Why:
///   The summary names each path once, however many passes correct it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changedPaths.add(path);
/// ```
fn record_path(changed: &mut Vec<Vec<u8>>, correction: &Correction) {
    if !changed.contains(&correction.path) {
        changed.push(correction.path.clone());
    }
}

/// What:
///  The pass that stops the commit path because a correction did not write to the private index.
/// Why:
///   The incumbent maps a failed patch application to `patch-conflict`, naming the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// patchApplicationFailure({ path, message });
/// ```
fn patch_conflict_pass(arguments: &[OsString], path: &[u8], message: String) -> PassResult {
    let shown: String = path_message_text(path);
    return PassResult {
        arguments: arguments.to_vec(),
        events: vec![PolicyEvent::EngineFailure {
            code: EngineFailureCode::PatchConflict,
            message: format!("Policy patch for {shown} does not apply to the private index: {message}"),
            trigger: Some(Trigger::PreForward),
            policy: None,
            path: Some(super::event_path::EventPath::from_git_bytes(path)),
        }],
        end: StageEnd::Failed,
    };
}

/// What:
///  A pass that stops the commit path with an engine failure code and message.
/// Why:
///   Cycle and pass-limit failures are engine failures, not policy findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fixCycleFailure({ previous, trigger: 'pre-forward', message });
/// ```
fn engine_failure_pass(arguments: &[OsString], code: EngineFailureCode, message: &str) -> PassResult {
    return PassResult {
        arguments: arguments.to_vec(),
        events: vec![PolicyEvent::EngineFailure {
            code,
            message: String::from(message),
            trigger: Some(Trigger::PreForward),
            policy: None,
            path: None,
        }],
        end: StageEnd::Failed,
    };
}

/// What:
///  Run the bounded commit-path convergence and return the pass that describes its outcome.
/// Why:
///   Settled passes keep their own events; each stop reports its cause as the incumbent does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return await convergeCommitPolicies({ ... });
/// ```
pub fn converge_commit_policies<F: RepositoryFacts>(input: CommitConvergenceInput<'_, F>) -> CommitConvergence {
    input.checks.can_apply_patches = true;
    let mut passes: CommitPasses<'_, F> = CommitPasses::new(input);
    let convergence: Convergence = converge(&mut passes);
    let Some(last) = passes.last.take() else {
        unreachable!("the bounded loop runs at least one pass");
    };
    let pass: PassResult = finish(convergence, last, passes.blocked.take());
    return CommitConvergence {
        convergence,
        pass,
        changed_paths: passes.changed_paths,
    };
}

/// What:
///  Choose the pass that describes a convergence outcome.
/// Why:
///   A blocked loop reports the write that failed when there was one, otherwise the pass that blocked it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// switch (convergence.kind) { case 'blocked': return blocked ?? last; }
/// ```
fn finish(convergence: Convergence, last: PassResult, blocked: Option<PassResult>) -> PassResult {
    match convergence {
        Convergence::Settled { .. } => return last,
        Convergence::Blocked => match blocked {
            Some(found) => return found,
            None => return last,
        },
        Convergence::FixCycle => {
            return engine_failure_pass(last.arguments.as_slice(), EngineFailureCode::FixCycle, COMMIT_CYCLE_MESSAGE);
        }
        Convergence::FixPassLimit => {
            return engine_failure_pass(last.arguments.as_slice(), EngineFailureCode::FixPassLimit, FIX_PASS_LIMIT_MESSAGE);
        }
    }
}

#[cfg(test)]
#[path = "commit_convergence_tests.rs"]
mod tests;
