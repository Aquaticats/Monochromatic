//! What:
//!  The standards-review shadow run, end to end and in-process: classify the landed commit's names, read the commit,
//!  build the capped input, call the gateway, and append the one log entry. Also a runner that contains a panic of any of it.
//! Why:
//!  This replaces the root `mise.toml` task `shadow-review` (its Luna route; the `SHADOW_REVIEW_ALLOW=1` route starts
//!  `pi` and is not ported). The review must never delay or fail a landing, so nothing here panics or returns a failure
//!  the landing would act on. No wall-clock budget covers the run (owner, 2026-11-03: "Stop only on no progress"; pick 9 of
//!  `doc/planning/cli-git-shipped-limits-audit.md`, the coordinator's under the owner's delegation and open to the owner's
//!  veto): each step of it ends on no progress by itself, the Git reads under the watch of `child_watch.rs` and the gateway call
//!  under the idle timeouts of `standards_review_client.rs`. The one step with no judge is the local write of the log entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await reviewLanded({ names, sha, subject, read, options, clock });
//! ```

use crate::iso_time::format_iso_milliseconds;
use crate::standards_review_client::{Gateway, Reply, SCRIPT_QUIET_WINDOW, post_decision};
use crate::standards_review_append::{deliver, names_commit};
use crate::standards_review_entry::{ReviewEntry, render_review, render_skipped};
use crate::standards_review_input::{LandedCommit, Plan, ReviewInput, build_input, plan_for, shared_log_path};
use crate::standards_review_request::{ENDPOINT, Judged, read_answer, request_body};
use crate::standards_review_text::seconds_with_tenth;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// The environment variable that points the log at a disposable fixture.
pub const LOG_VARIABLE: &str = "SHADOW_REVIEW_LOG";

/// The variable that replays a commit other than `HEAD`.
pub const SHA_VARIABLE: &str = "SHADOW_REVIEW_SHA";

/// The variable that asks for the full review route, which this port does not run.
pub const ALLOW_VARIABLE: &str = "SHADOW_REVIEW_ALLOW";

/// The credential variable, and the stop-hook credential used when it is unset.
pub const KEY_VARIABLES: [&str; 2] = crate::service_review_key::KEY_VARIABLES;

/// The failure line written when no credential is set.
pub const NO_KEY_MESSAGE: &str = "no SHADOW_REVIEW_LLMGATEWAY_API_KEY or STOP_REMINDER_LLMGATEWAY_API_KEY in the environment";

/// What:
///  What the run was told: the log, the credential, and where and how long to call.
/// Why:
///   Production passes `ENDPOINT` and `SCRIPT_QUIET_WINDOW`; tests pass a loopback URL and a short quiet window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewOptions = { logPath: string; apiKey?: string; endpoint: string; quietWindow: number };
/// ```
#[derive(Clone, Debug)]
pub struct ReviewOptions {
    /// Where entries are appended.
    pub log_path: PathBuf,
    /// The bearer credential; `None` writes the no-credential failure entry.
    pub api_key: Option<String>,
    /// The decisions endpoint.
    pub endpoint: String,
    /// How long the gateway call may stay silent before it is ended.
    pub quiet_window: Duration,
    /// A PEM authority replacing the embedded roots (tests only).
    pub trusted_pem: Option<Vec<u8>>,
}

/// What:
///  The options the script's environment variables give for a repository: the live log under `common_dir` unless
///  `SHADOW_REVIEW_LOG` names another, and the first credential variable that is set, even to an empty text.
/// Why:
///   `??` in the script falls back only for an unset variable, so an empty first key does not fall back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const logPath = process.env.SHADOW_REVIEW_LOG ?? `${commonDir}/cli-git/standards-review-shadow.md`;
/// ```
pub fn options_from_environment(common_dir: &Path, variable: &dyn Fn(&str) -> Option<String>) -> ReviewOptions {
    let log_path: PathBuf = variable(LOG_VARIABLE).map_or_else(|| return shared_log_path(common_dir), PathBuf::from);
    let api_key: Option<String> = variable(KEY_VARIABLES[0]).or_else(|| return variable(KEY_VARIABLES[1]));
    return ReviewOptions { log_path, api_key, endpoint: String::from(ENDPOINT), quiet_window: SCRIPT_QUIET_WINDOW, trusted_pem: None };
}

/// What:
///  Whether the environment asks for the full-review route, which this port leaves out.
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
///   Callers log it; none of it fails a landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewOutcome = 'no-entry' | 'skipped' | { reviewed: Judged } | { unread: string } | { unwritable: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewOutcome {
    /// No names, or only the log: no entry, as the script exits silently.
    NoEntry,
    /// The no-code-files entry was appended.
    Skipped,
    /// A review entry was appended; the gateway's verdict, or its failure in `outcome` with an empty `top`.
    Reviewed(Judged),
    /// The commit could not be read; no entry.
    Unread(String),
    /// The entry could not be appended.
    Unwritable(String),
    /// The log already names this commit (a repeated item): nothing was asked and nothing was added.
    AlreadyLogged,
}

/// What:
///  The gateway step: the verdict, or the failure message, and the elapsed milliseconds.
/// Why:
///   A missing credential fails before any connection, as in the script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { ...fetch... } catch (error) { lunaOutcome = 'Luna failed: ' + message }
/// ```
fn ask_gateway(input: &ReviewInput, options: &ReviewOptions) -> Judged {
    let failed = |message: String| return Judged { top: String::new(), outcome: format!("Luna failed: {message}") };
    let key: &str = options.api_key.as_deref().unwrap_or("");
    if key.is_empty() {
        return failed(String::from(NO_KEY_MESSAGE));
    }
    let gateway: Gateway = Gateway { url: options.endpoint.as_str(), api_key: key, quiet_window: options.quiet_window, trusted_pem: options.trusted_pem.as_deref() };
    let reply: Result<Reply, String> = post_decision(&gateway, request_body(input.model_input.as_str()).as_str());
    return match reply.and_then(|answer| return read_answer(answer.status, answer.body.as_str())) {
        Ok(judged) => judged,
        Err(message) => failed(message),
    };
}

/// What:
///  Review one landed commit: `names` are the changed names the landing knows, `read` fetches the commit's patch and
///  files (called only for a commit with code files), `now_millis` is the clock.
/// Why:
///   One function is the script's whole body, with the reads and the clock as parameters so tests fix both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reviewLanded({ names, sha, subject, read, options, nowMillis }): Promise<ReviewOutcome>;
/// ```
pub fn review_landed(
    names: &[String],
    sha: &str,
    subject: &str,
    read: &dyn Fn(&[String]) -> Result<LandedCommit, String>,
    options: &ReviewOptions,
    now_millis: &dyn Fn() -> u64,
) -> ReviewOutcome {
    let code_files: Vec<String> = match plan_for(names) {
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
    let commit: LandedCommit = match read(code_files.as_slice()) {
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
///  Run `job` on a helper thread and wait for it; `None` means the job panicked.
/// Why:
///   The review shares the service's work thread, so a panic in any step must cost one failed item and never the thread. No
///  time limit applies: each step of the job ends on no progress by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await job().catch(() => undefined)
/// ```
pub fn run_contained<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (sender, receiver) = mpsc::channel::<T>();
    let worker = std::thread::Builder::new().name(String::from("standards-review")).spawn(move || {
        let _ = sender.send(job());
    });
    if worker.is_err() {
        return None;
    }
    return receiver.recv().ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_run_tests.rs"]
mod tests;
