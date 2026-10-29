//! What:
//!  Waiting for a foreign `index.lock` under the classification rules:
//!  a proven-alive owner
//!       is waited for up to this process's wait limit (`CLI_GIT_LOCK_WAIT_SECONDS`, 30 minutes by default),
//!  a dead or evidence-free owner is retried on Git's backoff
//!       schedule until the unproven budget runs out.
//! Why:
//!  An index writer must not run beside a landing,
//!  and a lock nobody can prove alive must
//!      not block forever.
//!  Nothing here deletes a lock;
//!  the caller decides what the failure means
//!      (`src/index-lock/index-lock-wait.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await waitForIndexLock({ realIndexPath, timeoutMs, consequence, attempt });
//! ```

/// The backstop every wait on a live process is bounded by.
use super::live_lock::{LIVE_WAIT_VARIABLE, live_wait_limit};
/// Foreign-lock evidence and its verdicts.
use super::index_lock_evidence::{
    Evidence, Verdict, classify_index_lock, describe_index_lock_evidence, proven_holder_line,
    proven_holder_progress_line,
};
/// When the notices of a wait are due.
use super::wait_progress::{Cadence, Ticker};
/// Time spans.
use std::time::Duration;

/// Git's `BACKOFF_MAX_MULTIPLIER`:
///  the backoff stops growing here.
pub const BACKOFF_MAX_MULTIPLIER: u64 = 1_000;

/// Git's jitter floor in thousandths of the backoff.
const JITTER_FLOOR_PERMILLE: u64 = 750;

/// Git's jitter span in thousandths of the backoff.
const JITTER_SPAN_PERMILLE: u64 = 500;

/// Thousandths per whole.
const PERMILLE: u64 = 1_000;

/// Longest single poll while a PID file proves the owner alive,
///  so its release is noticed promptly.
pub const PROVEN_BY_PID_FILE_POLL_CAP_MS: u64 = 100;

/// Longest single poll while only an open descriptor proves the owner alive;
///  each poll repeats the
/// open-holder scan,
///  so it polls less often.
pub const PROVEN_BY_DESCRIPTOR_POLL_CAP_MS: u64 = 500;

/// What:
///  Git's backoff position:
///  `step` is Git's `n`,
///  `multiplier` the wait base in milliseconds.
/// Why:
///   `BackoffState` in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BackoffState = { step: number; multiplier: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackoffState {
    /// Git's `n`.
    pub step: u64,
    /// Git's `multiplier`,
    ///  the current backoff in milliseconds.
    pub multiplier: u64,
}

/// Backoff before the first wait.
pub const INITIAL_BACKOFF: BackoffState = BackoffState {
    step: 1,
    multiplier: 1,
};

/// What:
///  One jittered wait from Git's formula `(750 + rand() % 500) * backoff / 1000`,
///  rounded down.
/// Why:
///   Jitter spreads retrying processes apart;
///  `random` is a value in `[0, 1)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.floor(((750 + Math.floor(random * 500)) * state.multiplier) / 1000);
/// ```
pub fn jittered_wait(state: BackoffState, random: f64) -> u64 {
    let span: u64 = (random * (JITTER_SPAN_PERMILLE as f64)).floor() as u64;
    return (JITTER_FLOOR_PERMILLE + span) * state.multiplier / PERMILLE;
}

/// What:
///  The next backoff:
///  `multiplier += 2n + 1`,
///  capped at `BACKOFF_MAX_MULTIPLIER`.
/// Why:
///   Adding `2n + 1` makes the multiplier the square of the step,
///  which is Git's schedule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nextBackoff({ step: 1, multiplier: 1 }); // { step: 2, multiplier: 4 }
/// ```
pub fn next_backoff(state: BackoffState) -> BackoffState {
    let multiplier: u64 = state.multiplier + 2 * state.step + 1;
    if multiplier > BACKOFF_MAX_MULTIPLIER {
        return BackoffState {
            step: state.step,
            multiplier: BACKOFF_MAX_MULTIPLIER,
        };
    }
    return BackoffState {
        step: state.step + 1,
        multiplier,
    };
}

/// What:
///  What one attempt found,
///  or why the wait ended without a result.
/// Why:
///   The caller reports the owner and the consequence;
///  the attempt itself is the lock check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLockWaitError = { kind: 'unproven-owner'; evidence; verdict; timeoutMs; consequence } | { kind: 'read'; reason };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum WaitFailure {
    /// A dead or unproven owner outlasted the unproven budget.
    UnprovenOwner {
        /// Last attempt's evidence.
        evidence: Box<Evidence>,
        /// Last verdict,
        ///  which is never proven alive.
        verdict: Verdict,
        /// Budget that ran out,
        ///  in milliseconds.
        timeout_ms: u64,
        /// What cli-git therefore did not do.
        consequence: String,
    },
    /// A proven-alive owner kept the lock until the wait limit ran out.
    LiveHolderTimedOut {
        /// The lock that stayed in place.
        lock_path: std::path::PathBuf,
        /// The owner's process ID.
        pid: i64,
        /// The owner's command name when readable.
        command: Option<String>,
        /// Milliseconds spent waiting for a proven-alive owner.
        waited_ms: u64,
        /// The limit that ran out,
        ///  in milliseconds.
        limit_ms: u64,
        /// What cli-git therefore did not do.
        consequence: String,
    },
    /// The lock or its evidence could not be read.
    Read(String),
}

/// What:
///  The side effects of a wait,
///  injected so tests can drive a fake clock and fake holders.
/// Why:
///   The host implementation reads `/proc`,
///  sleeps and writes standard error;
///       tests script each step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLockWaitEffects = { gather; sleep; random; report; now };
/// ```
pub trait WaitEffects {
    /// Whether the lock currently exists (the attempt).
    fn lock_present(&mut self) -> Result<bool, String>;
    /// This attempt's evidence,
    ///  or nothing when the lock was released meanwhile.
    fn gather(&mut self) -> Result<Option<Evidence>, String>;
    /// Sleeps for `ms` milliseconds.
    fn sleep(&mut self, ms: u64);
    /// A uniform value in `[0, 1)`.
    fn random(&mut self) -> f64;
    /// The current time in milliseconds.
    fn now_ms(&mut self) -> f64;
    /// Writes one line of diagnostics to standard error.
    fn report(&mut self, line: &str);
    /// When this wait repeats its notice:
    ///  the host reads the environment,
    ///  and a fake keeps the standard cadence unless a test sets another.
    fn cadence(&mut self) -> Cadence {
        return Cadence::standard();
    }
}

/// What:
///  The per-wait loop state:
///  backoff,
///  time spent unproven,
///  and whether the holder line was written.
/// Why:
///   `state` in `waitForIndexLock`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = { backoff: INITIAL_BACKOFF, unprovenMs: 0, reported: false };
/// ```
#[derive(Clone, Copy, Debug)]
struct LoopState {
    /// Backoff position.
    backoff: BackoffState,
    /// Milliseconds spent without a proven owner.
    unproven_ms: f64,
    /// Milliseconds spent waiting for a proven-alive owner.
    live_ms: f64,
    /// Whether the holder line was written.
    reported: bool,
    /// When the next repeat of the holder line is due, counted in `live_ms`.
    live_ticker: Ticker,
    /// When the next notice of a wait behind an unproven owner is due, counted in `unproven_ms`.
    unproven_ticker: Ticker,
}

/// What:
///  Wait until the lock is gone,
///  or fail once a dead or unproven owner outlasts `timeout_ms`
///  or a live owner outlasts this process's wait limit.
/// Why:
///   A live holder gets one diagnostic and the process's wait limit;
///  other holders get the budget,
///       which includes the evidence gathering because a `/proc` scan is not free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexLock({ realIndexPath, timeoutMs, consequence, attempt, effects });
/// ```
pub fn wait_for_index_lock(
    effects: &mut dyn WaitEffects,
    timeout_ms: u64,
    consequence: &str,
) -> Result<(), WaitFailure> {
    let limit_ms: u64 = u64::try_from(live_wait_limit().as_millis()).unwrap_or(u64::MAX);
    return wait_for_index_lock_within(effects, timeout_ms, limit_ms, consequence);
}

/// What:
///  The wait with an explicit limit on the time spent behind a proven-alive owner.
/// Why:
///   The default wait and the tests share one loop; only the limit differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexLock({ realIndexPath, timeoutMs, liveLimitMs, consequence, attempt, effects });
/// ```
pub fn wait_for_index_lock_within(
    effects: &mut dyn WaitEffects,
    timeout_ms: u64,
    live_limit_ms: u64,
    consequence: &str,
) -> Result<(), WaitFailure> {
    let cadence: Cadence = effects.cadence();
    // `mut` lets each iteration advance the loop state.
    let mut state: LoopState = LoopState {
        backoff: INITIAL_BACKOFF,
        unproven_ms: 0.0,
        live_ms: 0.0,
        reported: false,
        live_ticker: Ticker::announced_at_start(cadence),
        unproven_ticker: Ticker::new(cadence),
    };
    loop {
        let iteration_started: f64 = effects.now_ms();
        let present: bool = match effects.lock_present() {
            Ok(found) => found,
            Err(reason) => return Err(WaitFailure::Read(reason)),
        };
        if !present {
            return Ok(());
        }
        let evidence: Evidence = match effects.gather() {
            Ok(Some(found)) => found,
            // The lock went away between the attempt and the evidence read.
            Ok(None) => continue,
            Err(reason) => return Err(WaitFailure::Read(reason)),
        };
        let verdict: Verdict = classify_index_lock(&evidence);
        let random: f64 = effects.random();
        let backoff_ms: u64 = jittered_wait(state.backoff, random);
        state.backoff = next_backoff(state.backoff);
        if let Verdict::ProvenAlive {
            pid,
            by_descriptor,
            command,
        } = &verdict
        {
            if !state.reported {
                let line: String = proven_holder_line(
                    *pid,
                    *by_descriptor,
                    command.as_deref(),
                    evidence.lock_path.as_path(),
                    live_limit_ms / 1_000,
                );
                effects.report(line.as_str());
                state.reported = true;
            }
            let cap: u64 = if *by_descriptor {
                PROVEN_BY_DESCRIPTOR_POLL_CAP_MS
            } else {
                PROVEN_BY_PID_FILE_POLL_CAP_MS
            };
            effects.sleep(backoff_ms.min(cap));
            state.live_ms += effects.now_ms() - iteration_started;
            // A notice that is due goes out before the limit is checked, so a refused wait was announced.
            if state.live_ticker.due(Duration::from_millis(state.live_ms as u64)) {
                let repeat: String = proven_holder_progress_line(
                    *pid,
                    *by_descriptor,
                    command.as_deref(),
                    evidence.lock_path.as_path(),
                    state.live_ms as u64 / 1_000,
                    live_limit_ms / 1_000,
                );
                effects.report(repeat.as_str());
            }
            if state.live_ms >= live_limit_ms as f64 {
                return Err(WaitFailure::LiveHolderTimedOut {
                    lock_path: evidence.lock_path.clone(),
                    pid: *pid,
                    command: command.clone(),
                    waited_ms: state.live_ms as u64,
                    limit_ms: live_limit_ms,
                    consequence: String::from(consequence),
                });
            }
            continue;
        }
        state.unproven_ms += effects.now_ms() - iteration_started;
        // A notice that is due goes out before the budget is checked, so a refused wait was announced.
        if state.unproven_ticker.due(Duration::from_millis(state.unproven_ms as u64)) {
            let notice: String = unproven_progress_line(
                evidence.lock_path.as_path(),
                &verdict,
                state.unproven_ms as u64,
                timeout_ms,
            );
            effects.report(notice.as_str());
        }
        if state.unproven_ms >= timeout_ms as f64 {
            return Err(WaitFailure::UnprovenOwner {
                evidence: Box::new(evidence),
                verdict,
                timeout_ms,
                consequence: String::from(consequence),
            });
        }
        let sleep_started: f64 = effects.now_ms();
        effects.sleep(backoff_ms);
        state.unproven_ms += effects.now_ms() - sleep_started;
    }
}

/// What:
///  The diagnostic text of an unproven-owner failure.
/// Why:
///   The person needs the lock path,
///  the owner state,
///  the budget,
///  what was not done,
///  and the evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new IndexLockUnprovenOwnerError({ evidence, verdict, timeoutMs, consequence }).message
/// ```
pub fn unproven_owner_message(
    evidence: &Evidence,
    verdict: &Verdict,
    timeout_ms: u64,
    consequence: &str,
) -> String {
    let owner: String = match verdict {
        Verdict::Dead(pid) => format!("whose owner PID {pid} no longer runs"),
        _ => String::from("whose owner cli-git could not prove alive"),
    };
    return format!(
        "Another process left {}, {owner}; after {timeout_ms} ms (indexLock.unprovenOwnerTimeoutMs) the lock was still there. cli-git left the lock in place and {consequence}. Evidence: {}. Remove the lock only after confirming no Git process is using this repository.",
        evidence.lock_path.display(),
        describe_index_lock_evidence(evidence),
    );
}

/// What:
///  The notice a wait behind a lock with no proven live owner repeats.
/// Why:
///   The wait retries on Git's backoff schedule and is silent otherwise,
///  so the person learns what is held, why no process is named, how long the wait has lasted and where it gives up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: still waiting for ${lockPath} after ${waitedMs} ms ...`
/// ```
pub fn unproven_progress_line(
    lock_path: &std::path::Path,
    verdict: &Verdict,
    waited_ms: u64,
    timeout_ms: u64,
) -> String {
    let owner: String = match verdict {
        Verdict::Dead(pid) => format!("whose owner PID {pid} no longer runs"),
        _ => String::from("whose owner cli-git cannot prove alive"),
    };
    return format!(
        "cli-git: still waiting for {}, {owner}, after {} seconds; retrying until {timeout_ms} ms \
         (indexLock.unprovenOwnerTimeoutMs) have passed, then stopping and leaving the lock in place.\n",
        lock_path.display(),
        waited_ms / 1_000
    );
}

/// What:
///  The diagnostic text of a live-owner timeout.
/// Why:
///   The person needs the lock path, the owner to look at, the limit and its setting,
///  and what was not done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new IndexLockLiveOwnerTimeoutError({ lockPath, pid, command, waitedMs, limitMs, consequence }).message
/// ```
pub fn live_holder_timeout_message(
    lock_path: &std::path::Path,
    pid: i64,
    command: Option<&str>,
    waited_ms: u64,
    consequence: &str,
) -> String {
    let owner: String = match command {
        Some(name) => format!("PID {pid} ({name})"),
        None => format!("PID {pid}"),
    };
    return format!(
        "Gave up after waiting {} seconds for {owner}, which still holds {}; {LIVE_WAIT_VARIABLE} sets how long cli-git waits. cli-git left the lock in place and {consequence}. Look at that process, or raise the limit.",
        waited_ms / 1_000,
        lock_path.display(),
    );
}

/// Wait controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_wait_tests.rs"]
mod tests;
