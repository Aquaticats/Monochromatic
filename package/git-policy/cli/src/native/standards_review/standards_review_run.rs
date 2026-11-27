//! What:
//!  The standards-review shadow run,
//! end to end and in-process:
//! classify the landed commit's names,
//! read the commit,
//!  build the capped input,
//! call the gateway,
//! and append the one log entry.
//! Also a runner that contains a panic of any of it.
//! Why:
//!  This replaces the root `mise.toml` task `shadow-review` (its Luna route;
//! the `SHADOW_REVIEW_ALLOW=1` route starts
//!  `pi` and is not ported).
//! The review must never delay or fail a landing,
//! so nothing here panics or returns a failure
//!  the landing would act on.
//! No wall-clock budget covers the run (owner,
//! 2026-11-03:
//! "Stop only on no progress";
//! pick 9 of
//!  `doc/planning/cli-git-shipped-limits-audit.md`,
//! the coordinator's under the owner's delegation and open to the owner's
//!  veto):
//! each step of it ends on no progress by itself,
//! the Git reads under the watch of `child_watch.rs` and the gateway call
//!  under the idle timeouts of `standards_review_client.rs`.
//! The one step with no judge is the local write of the log entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await reviewLanded({ names, sha, subject, read, options, clock });
//! ```

use super::standards_review_run_containment::{
    ABANDONED,
    IDLE,
    RUNNING,
};
/// The runner that contains one review,
/// what it answered,
/// and the one runner the service uses.
pub use super::standards_review_run_containment::{
    Contained,
    Containment,
    RUNNER,
};
/// The one gateway step of a review.
use super::standards_review_run_gateway_step::ask_gateway;
/// What the run was told,
/// and where it reads a landed commit.
pub use super::standards_review_run_options::{
    ALLOW_VARIABLE,
    EnvironmentVariables,
    KEY_VARIABLES,
    LOG_VARIABLE,
    LandedCommitReader,
    NO_KEY_MESSAGE,
    ReviewOptions,
    SHA_VARIABLE,
    full_review_requested,
    options_from_environment,
};
use crate::iso_time::format_iso_milliseconds;
use crate::standards_review_append::{
    deliver,
    names_commit,
};
use crate::standards_review_entry::{
    ReviewEntry,
    render_review,
    render_skipped,
};
use crate::standards_review_input::{
    ChangedPath,
    LandedCommit,
    Plan,
    ReviewInput,
    build_input,
    plan_for,
};
use crate::standards_review_request::Judged;
use crate::standards_review_text::seconds_with_tenth;
/// The two channels the two faces of waiting use.
use std::sync::mpsc;
/// How long an extra wait after a job the caller gave up on may take.
use std::time::Duration;
/// The awaited face's one-shot value.
use tokio::sync::oneshot;

/// What:
///  How a run ended.
/// Why:
///   Callers log it;
/// none of it fails a landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewOutcome = 'no-entry' | 'skipped' | { reviewed: Judged } | { unread: string } | { unwritable: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewOutcome {
    /// No names,
    /// or only the log:
    /// no entry,
    /// as the script exits silently.
    NoEntry,
    /// The no-code-files entry was appended.
    Skipped,
    /// A review entry was appended;
    /// the gateway's verdict,
    /// or its failure in `outcome` with an empty `top`.
    Reviewed(Judged),
    /// The commit could not be read;
    /// no entry.
    Unread(String),
    /// The entry could not be appended.
    Unwritable(String),
    /// The log already names this commit (a repeated item):
    /// nothing was asked and nothing was added.
    AlreadyLogged,
}

/// What:
///  Review one landed commit:
/// `names` are the changed names the landing knows,
/// `read` fetches the commit's patch and
///  files (called only for a commit with code files),
/// `now_millis` is the clock.
/// Why:
///   One function is the script's whole body,
/// with the reads and the clock as parameters so tests fix both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reviewLanded({ names, sha, subject, read, options, nowMillis }): Promise<ReviewOutcome>;
/// ```
pub fn review_landed(
    names: &[ChangedPath],
    sha: &str,
    subject: &str,
    read: &dyn LandedCommitReader,
    options: &ReviewOptions,
    now_millis: &dyn Fn() -> u64,
) -> ReviewOutcome {
    let code_files: Vec<ChangedPath> = match plan_for(names) {
        Plan::Silent => return ReviewOutcome::NoEntry,
        Plan::NoCode => {
            return match deliver(
                options.log_path.as_path(),
                sha,
                render_skipped(sha, subject).as_str(),
            ) {
                Ok(_) => ReviewOutcome::Skipped,
                Err(message) => ReviewOutcome::Unwritable(message),
            };
        }
        Plan::Review(files) => files,
    };
    if names_commit(options.log_path.as_path(), sha) {
        return ReviewOutcome::AlreadyLogged;
    }
    let commit: LandedCommit = match read.read(code_files.as_slice()) {
        Ok(read_commit) => read_commit,
        Err(message) => return ReviewOutcome::Unread(message),
    };
    let input: ReviewInput = build_input(&commit, code_files.as_slice());
    let started: u64 = now_millis();
    let judged: Judged = ask_gateway(&input, options);
    let finished: u64 = now_millis();
    let entry: ReviewEntry = ReviewEntry {
        date: format_iso_milliseconds(i64::try_from(finished).unwrap_or(i64::MAX)),
        sha: String::from(sha),
        subject: String::from(subject),
        code_files: input.code_file_count,
        diff_lines: input.diff_line_count,
        truncated: input.truncated,
        input_units: input.model_input_units,
        seconds: seconds_with_tenth(finished.saturating_sub(started)),
        top: judged.top.clone(),
        outcome: judged.outcome.clone(),
    };
    return match deliver(
        options.log_path.as_path(),
        sha,
        render_review(&entry).as_str(),
    ) {
        Ok(_) => ReviewOutcome::Reviewed(judged),
        Err(message) => ReviewOutcome::Unwritable(message),
    };
}

/// Releases a runner's job when it goes out of scope,
/// whether the job returned or panicked.
struct Release<'a> {
    /// The runner the job belongs to.
    runner: &'a Containment,
    /// The job.
    generation: u64,
}

impl Drop for Release<'_> {
    fn drop(&mut self) {
        // Whatever status the job has (waited for or abandoned), it ends idle; a state that is another job's is left alone.
        let _released: bool = self.runner.change(self.generation, RUNNING, IDLE)
            || self.runner.change(self.generation, ABANDONED, IDLE);
    }
}

/// What:
///  Run `job` on a helper thread and wait for it,
/// at most `ceiling`.
/// Why:
///   The review shares the service's work thread,
/// so a panic in any step must cost one failed item and never the thread.
/// Each step
///  of the job ends by itself on no progress or at its own ceiling;
/// the ceiling here covers the whole job,
/// including the one step
///  with no judge of its own,
/// the local write of the log entry (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)";
///  `wait_ceilings.rs`,
/// `REVIEW_CEILING`).
/// A thread cannot be stopped from outside,
/// so the caller stops waiting and the thread ends
///  with the process,
/// or when the step it is stuck in returns;
/// meanwhile no other job starts (`Contained::Busy`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([job().catch(() => PANICKED), after(ceilingMs).then(() => CEILING)])
/// ```
pub fn run_contained<T: Send + 'static>(
    job: impl FnOnce() -> T + Send + 'static,
    ceiling: Duration,
) -> Contained<T> {
    return run_contained_in(&RUNNER, job, ceiling);
}

/// `run_contained` on a runner of the caller's own.
pub fn run_contained_in<T: Send + 'static>(
    runner: &'static Containment,
    job: impl FnOnce() -> T + Send + 'static,
    ceiling: Duration,
) -> Contained<T> {
    // One atomic step admits the job: a review the caller stopped waiting for may still hold the log, and a second one would queue
    // behind it and be abandoned in its turn (sol-2, B); two callers at once cannot both pass.
    let Some(generation) = runner.admit() else {
        return Contained::Busy;
    };
    let (sender, receiver) = mpsc::channel::<T>();
    let worker = std::thread::Builder::new()
        .name(String::from("standards-review"))
        .spawn(move || {
            // Released when the job ends, before its value is sent, so a caller that has the value finds the runner free; and on
            // unwinding, so a job that panics after the caller gave up does not keep the runner closed.
            let release: Release<'static> = Release {
                runner,
                generation,
            };
            let value: T = job();
            drop(release);
            #[cfg(test)]
            drop(runner.handoff_gate.lock());
            let _ = sender.send(value);
        });
    if let Err(error) = worker {
        let _released: bool = runner.change(generation, RUNNING, IDLE);
        return Contained::NoThread(crate::scoped_start::refused_thread_sentence(
            "run the review of a commit",
            &error,
        ));
    }
    return match receiver.recv_timeout(ceiling) {
        Ok(value) => Contained::Finished(value),
        // The thread ended without a value: it panicked, and its guard released the runner.
        Err(mpsc::RecvTimeoutError::Disconnected) => Contained::Panicked,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if runner.change(generation, RUNNING, ABANDONED) {
                Contained::CeilingReached(ceiling)
            } else {
                // The job released the runner between the timeout and now. Its value is in the channel, or on its way, or it panicked; the
                //  release is not proof the value was sent, so the wait is bounded, and a value that does not come is a ceiling.
                match receiver.recv_timeout(runner.handoff_wait) {
                    Ok(value) => Contained::Finished(value),
                    Err(mpsc::RecvTimeoutError::Disconnected) => Contained::Panicked,
                    Err(mpsc::RecvTimeoutError::Timeout) => Contained::CeilingReached(ceiling),
                }
            }
        }
    };
}

/// What:
///  `run_contained` on a runner of the caller's own,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The admit,
/// the thread,
/// the release before the send and the two bounded waits are the synchronous face's;
/// what differs is
///  only where the wait happens,
/// so the two cannot answer differently.
/// A value that arrives after the ceiling is dropped with the sender that comes back to nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let waited = await timeout(ceiling, value); if (timed out) { if (markAbandoned(generation)) return CEILING; waited = await timeout(handoffWait, value) } return answer(waited)
/// ```
pub async fn run_contained_in_awaiting<T: Send + 'static>(
    runner: &'static Containment,
    job: impl FnOnce() -> T + Send + 'static,
    ceiling: Duration,
) -> Contained<T> {
    // The one atomic admission is the synchronous face's, and for the same reason: a job the caller stopped waiting for may still hold
    //  what the next one needs.
    let Some(generation) = runner.admit() else {
        return Contained::Busy;
    };
    let (sender, mut receiver) = oneshot::channel::<T>();
    let worker = std::thread::Builder::new()
        .name(String::from("standards-review"))
        .spawn(move || {
            // Released when the job ends, before its value is sent, so a caller that has the value finds the runner free; and on
            //  unwinding, so a job that panics after the caller gave up does not keep the runner closed.
            let release: Release<'static> = Release {
                runner,
                generation,
            };
            let value: T = job();
            drop(release);
            #[cfg(test)]
            drop(runner.handoff_gate.lock());
            // A caller that stopped waiting has no receiver: the value is dropped.
            let _ = sender.send(value);
        });
    if let Err(error) = worker {
        let _released: bool = runner.change(generation, RUNNING, IDLE);
        return Contained::NoThread(crate::scoped_start::refused_thread_sentence(
            "run the review of a commit",
            &error,
        ));
    }
    let waited: Result<T, oneshot::error::RecvError> =
        match tokio::time::timeout(ceiling, &mut receiver).await {
            Ok(answer) => answer,
            Err(_elapsed) => {
                if runner.change(generation, RUNNING, ABANDONED) {
                    return Contained::CeilingReached(ceiling);
                }
                // The job released the runner between the ceiling and now. Its value is on its way or it panicked, so the wait is bounded
                //  again and a value that does not come is a ceiling.
                match tokio::time::timeout(runner.handoff_wait, &mut receiver).await {
                    Ok(answer) => answer,
                    Err(_late) => return Contained::CeilingReached(ceiling),
                }
            }
        };
    return match waited {
        Ok(value) => Contained::Finished(value),
        // The thread ended without a value: it panicked, and its guard released the runner.
        Err(_closed) => Contained::Panicked,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_run_tests.rs"]
mod tests;

/// Controls of what may leave the machine when this run builds a review,
/// against real Git and the loopback mock;
/// never in the release executable.
#[cfg(test)]
#[path = "standards_review_run_request_provenance_tests.rs"]
mod request_provenance_tests;
