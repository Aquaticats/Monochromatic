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

use crate::iso_time::format_iso_milliseconds;
use crate::standards_review_client::{Gateway, Reply, SCRIPT_QUIET_WINDOW, post_decision};
use crate::standards_review_append::{deliver, names_commit};
use crate::standards_review_entry::{ReviewEntry, render_review, render_skipped};
use crate::standards_review_input::{ChangedPath, LandedCommit, Plan, ReviewInput, build_input, plan_for, shared_log_path};
use crate::standards_review_request::{ENDPOINT, Judged, read_answer, request_body};
use crate::standards_review_text::seconds_with_tenth;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::oneshot;

/// The environment variable that points the log at a disposable fixture.
pub const LOG_VARIABLE: &str = "SHADOW_REVIEW_LOG";

/// The variable that replays a commit other than `HEAD`.
pub const SHA_VARIABLE: &str = "SHADOW_REVIEW_SHA";

/// The variable that asks for the full review route,
/// which this port does not run.
pub const ALLOW_VARIABLE: &str = "SHADOW_REVIEW_ALLOW";

/// The credential variable,
/// and the stop-hook credential used when it is unset.
pub const KEY_VARIABLES: [&str; 2] = crate::service_review_key::KEY_VARIABLES;

/// The failure line written when no credential is set.
pub const NO_KEY_MESSAGE: &str = "no SHADOW_REVIEW_LLMGATEWAY_API_KEY or STOP_REMINDER_LLMGATEWAY_API_KEY in the environment";

/// What:
///  What the run was told:
/// the log,
/// the credential,
/// and where and how long to call.
/// Why:
///   Production passes `ENDPOINT` and `SCRIPT_QUIET_WINDOW`;
/// tests pass a loopback URL and a short quiet window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewOptions = { logPath: string; apiKey?: string; endpoint: string; quietWindow: number };
/// ```
#[derive(Clone, Debug)]
pub struct ReviewOptions {
    /// Where entries are appended.
    pub log_path: PathBuf,
    /// The bearer credential;
    /// `None` writes the no-credential failure entry.
    pub api_key: Option<String>,
    /// The decisions endpoint.
    pub endpoint: String,
    /// How long the gateway call may stay silent before it is ended.
    pub quiet_window: Duration,
    /// The longest the gateway call runs in all,
    /// whatever it shows (`wait_ceilings.rs`,
    /// `REVIEW_CALL_CEILING`).
    pub call_ceiling: Duration,
    /// A PEM authority replacing the embedded roots (tests only).
    pub trusted_pem: Option<Vec<u8>>,
}

/// What:
///  Where the review's options read the variables of the environment.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The service's environment carries no secret:
/// the key comes in the wake message,
/// so the lookup holds the key it was given and
///  answers every other name from the environment.
/// A struct that holds the key implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface EnvironmentVariables { variable(name: string): string | undefined }
/// ```
pub trait EnvironmentVariables {
    /// The value the name has,
    /// or nothing.
    fn variable(&self, name: &str) -> Option<String>;
}

/// What:
///  Where the review reads the patch and the files of a landed commit.
/// Why:
///   The read needs the Git context,
/// the directory,
/// the commit and the subject,
/// and tests fix the answer;
/// a struct that holds
///  them implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface LandedCommitReader { read(files: ChangedPath[]): LandedCommit } // throws a reason
/// ```
pub trait LandedCommitReader {
    /// Read the commit as far as `files` (its code files) are concerned.
    fn read(&self, files: &[ChangedPath]) -> Result<LandedCommit, String>;
}

/// What:
///  The options the script's environment variables give for a repository:
/// the live log under `common_dir` unless
///  `SHADOW_REVIEW_LOG` names another,
/// and the first credential variable that is set,
/// even to an empty text.
/// Why:
///   `??` in the script falls back only for an unset variable,
/// so an empty first key does not fall back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const logPath = process.env.SHADOW_REVIEW_LOG ?? `${commonDir}/monochromatic-git-policy-cli/standards-review-shadow.md`;
/// ```
pub fn options_from_environment(common_dir: &Path, variable: &dyn EnvironmentVariables) -> ReviewOptions {
    let log_path: PathBuf = match variable.variable(LOG_VARIABLE) {
        Some(configured) => PathBuf::from(configured),
        None => shared_log_path(common_dir),
    };
    // The second variable is read only when the first is unset, as `.or_else(...)` did.
    let api_key: Option<String> = match variable.variable(KEY_VARIABLES[0]) {
        Some(key) => Some(key),
        None => variable.variable(KEY_VARIABLES[1]),
    };
    return ReviewOptions { log_path, api_key, endpoint: String::from(ENDPOINT), quiet_window: SCRIPT_QUIET_WINDOW, call_ceiling: crate::wait_ceilings::REVIEW_CALL_CEILING, trusted_pem: None };
}

/// What:
///  Whether the environment asks for the full-review route,
/// which this port leaves out.
/// Why:
///   The caller says so in a notice instead of silently running the cheap route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.SHADOW_REVIEW_ALLOW === '1'
/// ```
pub fn full_review_requested(variable: &dyn Fn(&str) -> Option<String>) -> bool {
    return variable(ALLOW_VARIABLE).as_deref() == Some("1");
}

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
///  The verdict of a gateway step that failed:
/// no top choice,
/// and the message in the outcome line.
/// Why:
///   `ask_gateway` reports every failure (no key,
/// no reply,
/// an unreadable reply) the same way,
/// and a named function
///  stands where a closure that built this verdict stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lunaFailed = (message: string) => ({ top: '', outcome: `Luna failed: ${message}` });
/// ```
fn luna_failed(message: String) -> Judged {
    return Judged { top: String::new(), outcome: format!("Luna failed: {message}") };
}

/// What:
///  The gateway step:
/// the verdict,
/// or the failure message,
/// and the elapsed milliseconds.
/// Why:
///   A missing credential fails before any connection,
/// as in the script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { ...fetch... } catch (error) { lunaOutcome = 'Luna failed: ' + message }
/// ```
fn ask_gateway(input: &ReviewInput, options: &ReviewOptions) -> Judged {
    let key: &str = options.api_key.as_deref().unwrap_or("");
    if key.is_empty() {
        return luna_failed(String::from(NO_KEY_MESSAGE));
    }
    let gateway: Gateway = Gateway { url: options.endpoint.as_str(), api_key: key, quiet_window: options.quiet_window, ceiling: options.call_ceiling, trusted_pem: options.trusted_pem.as_deref() };
    let reply: Result<Reply, String> = post_decision(&gateway, request_body(input.model_input.as_str()).as_str());
    let read: Result<Judged, String> = match reply {
        Ok(answer) => read_answer(answer.status, answer.body.as_str()),
        Err(message) => Err(message),
    };
    return match read {
        Ok(judged) => judged,
        Err(message) => luna_failed(message),
    };
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
            return match deliver(options.log_path.as_path(), sha, render_skipped(sha, subject).as_str()) {
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
    return match deliver(options.log_path.as_path(), sha, render_review(&entry).as_str()) {
        Ok(_) => ReviewOutcome::Reviewed(judged),
        Err(message) => ReviewOutcome::Unwritable(message),
    };
}

/// What:
///  How a contained job ended.
/// Why:
///   The caller words a panic and a ceiling differently:
/// one is a defect,
/// the other is a review that ran as long as the service lets
///  one run;
/// and a thread the system refused is neither,
/// so it carries its own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Contained<T> = { kind: 'finished'; value: T } | { kind: 'panicked' } | { kind: 'no-thread'; sentence: string } | { kind: 'ceiling'; afterMs: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Contained<T> {
    /// The job returned its value.
    Finished(T),
    /// The job panicked.
    Panicked,
    /// The job's thread could not be started,
    /// with the sentence that says what refused it:
    /// nothing ran,
    /// so this is no defect of the job.
    NoThread(String),
    /// The job was still running when the ceiling passed;
    /// the caller stopped waiting and the job's thread is left to end with the
    ///  process,
    /// or when the step it is stuck in returns.
    /// Until then it counts as outstanding (`outstanding_reviews`).
    CeilingReached(Duration),
    /// The job was not started:
    /// an earlier one that the caller stopped waiting for is still running and may hold what this one
    ///  needs (the review log),
    /// so no second thread is added behind it.
    Busy,
}

/// What:
///  The one job a runner has at a time,
/// as the runner knows it:
/// nothing,
/// one the caller waits for,
/// or one the caller stopped waiting
///  for and that still runs.
/// Why:
///   Admission,
/// the caller's give-up and the worker's end are each one atomic change of this state,
/// so two callers cannot both be
///  admitted (sol-2 round 2,
/// P2),
/// and a worker that panics releases it by a guard that runs on unwinding (sol-2 round 2,
/// P1).
/// The
///  production runner is `RUNNER`;
/// a control makes its own,
/// so the controls do not see one another's jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Containment { state: 'idle' | 'running' | 'abandoned' }
/// ```
pub struct Containment {
    /// The generation of the job in the high bits and `IDLE`,
    /// `RUNNING` or `ABANDONED` in the low two.
    /// A change names the generation
    ///  it expects,
    /// so a caller that gave up on job 7 can never change the state of job 8.
    state: std::sync::atomic::AtomicU64,
    /// How long the caller waits for the value of a job that ended as the caller gave up,
    /// when the value does not come:
    /// the short
    ///  extra wait of the owner's "7 s (Recommended)" (`wait_ceilings.rs`,
    /// `SHORT_EXTRA_WAIT`).
    /// A job that released the runner has
    ///  not necessarily sent its value,
    /// so the wait is bounded (sol-1 round 3,
    /// P1 D).
    handoff_wait: Duration,
    /// A control holds this to keep a worker between its release of the runner and its send of the value.
    #[cfg(test)]
    handoff_gate: std::sync::Mutex<()>,
}

/// No job.
const IDLE: u64 = 0;
/// A job the caller waits for.
const RUNNING: u64 = 1;
/// A job the caller stopped waiting for,
/// still running.
const ABANDONED: u64 = 2;
/// The bits of the state that hold the status.
const STATUS: u64 = 3;

/// The runner of the service's reviews.
pub static RUNNER: Containment = Containment::new();

impl Containment {
    /// A runner with no job.
    pub const fn new() -> Self {
        return Self::with_handoff_wait(crate::wait_ceilings::SHORT_EXTRA_WAIT);
    }

    /// A runner whose caller waits `handoff_wait` for the value of a job that ended as the caller gave up.
    pub const fn with_handoff_wait(handoff_wait: Duration) -> Self {
        return Self {
            state: std::sync::atomic::AtomicU64::new(IDLE),
            handoff_wait,
            #[cfg(test)]
            handoff_gate: std::sync::Mutex::new(()),
        };
    }

    /// How many jobs the caller stopped waiting for are still running:
    /// zero or one.
    pub fn outstanding(&self) -> usize {
        return usize::from(self.state.load(std::sync::atomic::Ordering::SeqCst) & STATUS == ABANDONED);
    }

    /// Admit a job when there is none:
    /// the new generation,
    /// or nothing.
    fn admit(&self) -> Option<u64> {
        use std::sync::atomic::Ordering;
        let seen: u64 = self.state.load(Ordering::SeqCst);
        if seen & STATUS != IDLE {
            return None;
        }
        let generation: u64 = (seen >> 2) + 1;
        return match self.state.compare_exchange(seen, (generation << 2) | RUNNING, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => Some(generation),
            Err(_) => None,
        };
    }

    /// Move job `generation` from `from` to `to`;
    /// false when the state is not that job's in that status any more.
    fn change(&self, generation: u64, from: u64, to: u64) -> bool {
        use std::sync::atomic::Ordering;
        return self.state.compare_exchange((generation << 2) | from, (generation << 2) | to, Ordering::SeqCst, Ordering::SeqCst).is_ok();
    }
}

impl Default for Containment {
    fn default() -> Self {
        return Self::new();
    }
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
        let _released: bool = self.runner.change(self.generation, RUNNING, IDLE) || self.runner.change(self.generation, ABANDONED, IDLE);
    }
}

/// How many reviews the service stopped waiting for that are still running.
pub fn outstanding_reviews() -> usize {
    return RUNNER.outstanding();
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
pub fn run_contained<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static, ceiling: Duration) -> Contained<T> {
    return run_contained_in(&RUNNER, job, ceiling);
}

/// `run_contained` on a runner of the caller's own.
pub fn run_contained_in<T: Send + 'static>(runner: &'static Containment, job: impl FnOnce() -> T + Send + 'static, ceiling: Duration) -> Contained<T> {
    // One atomic step admits the job: a review the caller stopped waiting for may still hold the log, and a second one would queue
    // behind it and be abandoned in its turn (sol-2, B); two callers at once cannot both pass.
    let Some(generation) = runner.admit() else {
        return Contained::Busy;
    };
    let (sender, receiver) = mpsc::channel::<T>();
    let worker = std::thread::Builder::new().name(String::from("standards-review")).spawn(move || {
        // Released when the job ends, before its value is sent, so a caller that has the value finds the runner free; and on
        // unwinding, so a job that panics after the caller gave up does not keep the runner closed.
        let release: Release<'static> = Release { runner, generation };
        let value: T = job();
        drop(release);
        #[cfg(test)]
        drop(runner.handoff_gate.lock());
        let _ = sender.send(value);
    });
    if let Err(error) = worker {
        let _released: bool = runner.change(generation, RUNNING, IDLE);
        return Contained::NoThread(crate::scoped_start::refused_thread_sentence("run the review of a commit", &error));
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
    }
}

/// What:
///  `run_contained` awaited by a caller that is itself a task of the runtime.
/// Why:
///   The wait for the job is the runtime's timer here,
/// so a worker that waits holds nothing while the job runs;
/// a caller inside the runtime cannot make the synchronous wait at all,
/// because the job it waits for
///  starts Git,
/// and the synchronous wait is refused from inside the runtime (`lazy_runtime::run_on_runtime`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runContainedAwaiting(job, ceiling)
/// ```
pub async fn run_contained_awaiting<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static, ceiling: Duration) -> Contained<T> {
    return run_contained_in_awaiting(&RUNNER, job, ceiling).await;
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
pub async fn run_contained_in_awaiting<T: Send + 'static>(runner: &'static Containment, job: impl FnOnce() -> T + Send + 'static, ceiling: Duration) -> Contained<T> {
    // The one atomic admission is the synchronous face's, and for the same reason: a job the caller stopped waiting for may still hold
    //  what the next one needs.
    let Some(generation) = runner.admit() else {
        return Contained::Busy;
    };
    let (sender, mut receiver) = oneshot::channel::<T>();
    let worker = std::thread::Builder::new().name(String::from("standards-review")).spawn(move || {
        // Released when the job ends, before its value is sent, so a caller that has the value finds the runner free; and on
        //  unwinding, so a job that panics after the caller gave up does not keep the runner closed.
        let release: Release<'static> = Release { runner, generation };
        let value: T = job();
        drop(release);
        #[cfg(test)]
        drop(runner.handoff_gate.lock());
        // A caller that stopped waiting has no receiver: the value is dropped.
        let _ = sender.send(value);
    });
    if let Err(error) = worker {
        let _released: bool = runner.change(generation, RUNNING, IDLE);
        return Contained::NoThread(crate::scoped_start::refused_thread_sentence("run the review of a commit", &error));
    }
    let waited: Result<T, oneshot::error::RecvError> = match tokio::time::timeout(ceiling, &mut receiver).await {
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
