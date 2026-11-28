//! What:
//!  The `git commit` command of the native wrapper:
//! the preparation and landing through the resident service,
//! the settled
//!  events,
//! the post-commit gate and the handoff of the automatic backup.
//! Why:
//!  The wrapper runs the commit instead of forwarding it,
//! prints the pre-forward events only after the commit returned,
//!  then runs the post-commit gate for a landed commit and,
//! when the gate passes,
//! hands the push to the service.
//! A commit
//!  that landed nothing exits with its own status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commit = await serviceCommit({ args, gitPath, policyOptions });
//! renderPolicyEvents(commit.policyResult.events);
//! const post = await runPostCommitLifecycle({ ... }); await handOverPush({ ... });
//! ```

/// The effective invocation directory after every `-C`,
/// kept in a file of its own,
/// re-exported where every caller of it already looks.
pub use crate::commit_command_directory::effective_directory;
/// The directory the commit's own Git runs in.
use crate::commit_command_directory::where_the_commit_runs;
/// The refusal of a form real Git refuses,
/// kept in a file of its own.
use crate::commit_command_form_refusal::refuse_a_form_git_refuses;
/// What the command is handed and how it ends,
/// kept in a file of their own,
/// re-exported where every caller of them already looks.
pub use crate::commit_command_input::{
    CommitCommand,
    CommitEnding,
};
/// The words a stopped command is reported with.
use crate::commit_command_words::{
    report_stop,
    write_stderr,
};
/// Why the commit did not land,
/// by the route that stopped it.
use crate::commit_stop_reason::{
    CommitStopReason,
    reason_of_events,
};
use crate::commit_timing::CommitTiming;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionResult,
};
use crate::diagnostics::{
    EngineFailureCode,
    render_engine_failure,
};
/// The answer a finding carries about what a commit does with the bytes it names.
use crate::event_hints::CommitCorrection;
use crate::git_version::check_git_version;
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::StageRequest;
/// The events a policy pass or the gate reports.
use crate::policy_events::{
    PolicyEvent,
    render_policy_events,
};
use crate::policy_trigger::Trigger;
use crate::post_commit::{
    NO_BLOCKING_CAUSE,
    POST_COMMIT_BLOCKED_EXIT_CODE,
    PostCommitRequest,
    blocked_reason,
    run_post_commit_gate,
};
use crate::repository_facts::RepositoryFacts;
use crate::service_commit::{
    ServiceCommit,
    service_commit,
};
use crate::service_handoff::{
    ServiceHandoff,
    forget_the_push,
    hand_over,
    promote_the_push,
};
use crate::service_messages::render;
/// Standard error as a stream whose whole lines a thread of their own writes,
/// and the writer that counts the time they took.
use crate::standard_error_off_the_calling_thread::{
    StandardErrorHand,
    StandardErrorOffTheCallingThread,
};
use crate::time_spent_outside_the_phases::MeasuredStandardError;
use crate::transaction_git::GitContext;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;
/// `Instant` is a monotonic reading the phases are measured with.
use std::time::Instant;
/// What:
///  Why a commit that reported events did not land,
/// or nothing when the command did not fail.
/// Why:
///   A command that ended without landing and without a non-zero status was not stopped by anything:
/// nothing needed doing,
/// and a reason there would be one nobody can check,
/// which is the signal the record keeps for a real gap (`commit_stop_reason.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopOfAnEnding = (result) => result.exitCode === 0 ? null : reasonOfEvents(result.events);
/// ```
fn stop_of_an_ending(result: &TransactionResult) -> Option<CommitStopReason> {
    if result.exit_code == 0 {
        return None;
    }
    return Some(reason_of_events(result.events.as_slice()));
}

/// What:
///  Whether one event of a commit's own policy pass still belongs in the report of a commit that landed,
///  already filtered to true.
/// Why:
///   A commit whose selection is read-only reads the index before the commit is made,
///  so a finding of that pass says the index holds the bytes and that staging them again is the next step.
///  Once the commit lands that sentence is stale:
///  the bytes are in a commit,
///  and the post-commit gate reports them with the step landed bytes need.
///  Every other event of the pass stands as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const belongsInALandingReport = (event) => !(event.type === 'finding' && event.commitCorrection === 'nothing-corrects-what-the-index-holds');
/// ```
fn belongs_in_a_landing_report(event: &PolicyEvent) -> bool {
    return !matches!(
        event,
        PolicyEvent::Finding(finding)
            if finding.commit_correction == Some(CommitCorrection::NothingCorrectsWhatTheIndexHolds)
    );
}

/// What:
///  Run one commit through the service and its lifecycle.
/// Why:
///   The service is the only writer of a commit,
/// so the command takes no lock and recovers nothing:
/// it prepares the commit,
///  sends it,
/// and reports what the service landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runCommitCommand({ ... });
/// ```
pub fn run_commit_command<F: RepositoryFacts>(
    command: &CommitCommand<'_>,
    checks: &mut ShippedChecks<F>,
) -> CommitEnding {
    // A command line real Git refuses is refused here, before the transaction reads an index or reaches the service, so the
    //  mistyped command changes nothing at all: no staged path, no reference, no record of a commit that never existed.
    if let Some(ending) = refuse_a_form_git_refuses(command) {
        return ending;
    }
    let mut timing: CommitTiming = CommitTiming::start();
    let ending: CommitEnding = run_timed(command, checks, &mut timing);
    // One append to the repository's timing file once everything the person sees is written; the terminal never shows it.
    timing.finish(ending.code);
    // What: A command stopped by a signal ends by it here, after the record, as a shell reports an interrupted native Git.
    // Why:  The record of an interrupted commit is the one thing that outlives the terminal it was stopped from, and the raise
    //       used to happen inside the report, before the record was written.
    if let Some(signal) = ending.signal {
        let _ended = crate::forwarding::end_as_git_ended(signal);
    }
    return ending;
}

/// `run_commit_command`,
/// recording in `timing` how the command's time was spent and how it ended.
fn run_timed<F: RepositoryFacts>(
    command: &CommitCommand<'_>,
    checks: &mut ShippedChecks<F>,
    timing: &mut CommitTiming,
) -> CommitEnding {
    if let Err(reason) = check_git_version(command.context) {
        write_stderr(
            render_engine_failure(0, EngineFailureCode::ContentUnavailable, reason.as_str())
                .as_str(),
        );
        return CommitEnding {
            code: 2,
            signal: None,
        };
    }
    // Every Git of this commit runs where the caller's `-C` chain already reached, with those tokens dropped, so one of
    //  them reaches no start twice (`where_the_commit_runs`).
    let (context, cwd): (GitContext, PathBuf) = where_the_commit_runs(command);
    let invocation: CommitInvocation<'_> = CommitInvocation {
        context: &context,
        environment: command.environment,
        cwd: cwd.as_path(),
        arguments: command.arguments,
        subcommand_index: command.subcommand_index,
        region: command.region,
        config: command.config,
        controls: command.controls,
    };
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: command.config.policies.clone(),
        controls: command.controls.clone(),
        selected: Vec::new(),
    };
    // Every command line lands through the resident service; a machine with none to reach or start ends here with the plain
    // message of its platform.
    let mut served: ServiceCommit = match service_commit(&invocation, checks, &request, timing) {
        Ok(found) => found,
        Err(stop) => {
            // A stop inside the build phase leaves it before its figure is taken, and the record must still state what the build spent.
            timing.close_an_open_build();
            let stopped: CommitEnding = report_stop(stop, timing);
            timing.begin_closing();
            return stopped;
        }
    };
    // A test kills the wrapper here: the commit is on the branch and answered, and everything the command still owes (the
    //  push, the upkeep and the notice of them) is written after this point (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
    //  finding 3). A release build holds no read of `CLI_GIT_TEST_ONLY_SERVICE_FAULT`, so a command that is not a control
    //  reaches nothing, and a build made for a control reaches it only when its variable names this point.
    crate::service_fault::reach("after-answer");
    // What: A commit that landed reports the bytes it holds, not the index the pass read before it.
    // Why:  the pass ran before the commit was made, so a sentence about the index is stale the moment those bytes
    //       are in a commit; the gate below names the landed bytes and the step they need, once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (commit.landed) commit.events = commit.events.filter(belongsInALandingReport);
    // ```
    if served.result.landed_oid.is_some() {
        served.result.events.retain(belongs_in_a_landing_report);
    }
    write_stderr(render_policy_events(0, served.result.events.as_slice()).as_str());
    // Why nothing landed, read before the result is taken apart for the work after a landing.
    let ended: Option<CommitStopReason> = stop_of_an_ending(&served.result);
    let (Some(landed), Some(handoff)) = (served.result.landed_oid, served.handoff) else {
        timing.outcome = "ended";
        timing.stop = ended;
        return CommitEnding {
            code: served.result.exit_code,
            signal: None,
        };
    };
    timing.outcome = "landed";
    let landed_ending: CommitEnding = post_commit(command, &handoff, landed.as_str(), timing);
    // What: Everything the commit still holds is let go inside a phase of its own.
    // Why:  The record's total is read when the record is written, which is after this frame's own values are dropped; without
    //       this edge that stretch is time no phase covers, and on a machine where one file call takes a second it reached
    //       eight and fourteen seconds (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`).
    timing.begin_closing();
    return landed_ending;
}

/// What:
///  The post-commit lifecycle of a commit the service landed:
/// the gate,
/// then the handoff of the push and the upkeep.
/// Why:
///   The push runs after the command returns,
/// so the commit never waits for the network;
/// a blocked gate keeps the commit
///  local and exits 2,
/// and a handoff that fails is said plainly without changing the exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = await runGate(); if (gate.blocked) return 2; await handOver(); return 0;
/// ```
fn post_commit(
    command: &CommitCommand<'_>,
    handoff: &ServiceHandoff,
    landed: &str,
    timing: &mut CommitTiming,
) -> CommitEnding {
    let request: PostCommitRequest<'_> = PostCommitRequest {
        real_git: command.context.real_git.as_path(),
        landed_oid: landed,
        arguments: command.arguments,
        global_prefix: command.context.global_prefix.as_slice(),
        overlay: command.context.overlay.as_slice(),
        environment: command.environment,
        config: command.config.policies.clone(),
        controls: command.controls.clone(),
        start_directory: command.start_directory.clone(),
    };
    // What: The notes of the hand-off go to the thread that writes standard error.
    // Why:  The push record outlives the command, and a reader that stopped draining must cost the
    //       hand-off one bound rather than the command itself.
    let mut stderr: MeasuredStandardError<StandardErrorOffTheCallingThread<StandardErrorHand>> =
        MeasuredStandardError::over(StandardErrorOffTheCallingThread::new());
    // The push record is left before the gate runs, so a wrapper killed while the gate runs loses no push; the gate takes it back
    //  when it blocks, which keeps a blocked commit local, and the hand-off below wakes the record the gate allowed.

    let gating: Instant = Instant::now();
    let gate = run_post_commit_gate(&request);
    timing.gate += gating.elapsed();
    write_stderr(render_policy_events(0, gate.events.as_slice()).as_str());
    if gate.blocked {
        // The verdict keeps the gate's own words: the next push of this branch stops before this commit, and every command says
        //  why until the commit is off the branch's way out. A verdict that could not be kept is said here, with what it means.
        let said: String = blocked_reason(gate.events.as_slice());
        // What: Taking the push's record back is part of the hand-over, so its time is the hand-off phase's.
        // Why:  It writes the gate's refusal beside the record, and on this machine one such write can take a second; no other
        //       phase names that time, and the record must account for its own total.
        let taking_back: Instant = Instant::now();
        // The gate's refusal takes the push's record back, and the service that held the values of the command that asked for it is
        //  told they can be given up: nothing will run that record.
        let unkept: Option<String> = forget_the_push(
            handoff,
            landed,
            if said.is_empty() {
                NO_BLOCKING_CAUSE
            } else {
                said.as_str()
            },
            &crate::held_push_release_wakes_the_service::AskTheService {
                context: command.context,
            },
        );
        timing.handoff += taking_back.elapsed();
        if let Some(kept) = unkept {
            write_stderr(format!("{kept}\n").as_str());
        }
        return CommitEnding {
            code: POST_COMMIT_BLOCKED_EXIT_CODE,
            signal: None,
        };
    }
    // What: The push the gate approved becomes runnable inside the hand-off phase.
    // Why:  Promoting it renames the record the service held and writes the verdict that approved it; those are the hand-over's
    //       own file work, and the one wait of a commit that used to be named by no phase.
    let promoting: Instant = Instant::now();
    let promoted = promote_the_push(handoff, landed);
    timing.handoff += promoting.elapsed();
    if let Err(failure) = promoted {
        write_stderr(format!("{}\n", render(&failure)).as_str());
    }
    // A test kills the wrapper here: the push record is runnable and nothing has been woken for it yet, so the service finds it
    //  only through the note the landing wrote (`service_claim_watch.rs`), and the person only through the line the next command
    //  reads from the record (`held_push_notices`). A release build holds no read of `CLI_GIT_TEST_ONLY_SERVICE_FAULT`, so only
    //  a build made for a control reaches this point with its variable naming it.
    crate::service_fault::reach(crate::service_fault::AFTER_PROMOTE);
    let handing_over: Instant = Instant::now();
    let handed = hand_over(handoff, landed, &mut stderr);
    timing.handoff += handing_over.elapsed();
    if let Err(failure) = handed {
        write_stderr(format!("{}\n", render(&failure)).as_str());
    }
    return CommitEnding {
        code: 0,
        signal: None,
    };
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_command_tests.rs"]
mod tests;
