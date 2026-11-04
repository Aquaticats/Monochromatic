//! What:
//!  What the commit service's one landing thread is doing, shared with the threads that talk to waiting clients: the write it is
//!  inside when that write is one the wrapper never ends, the requests of the group it lands, and the withdrawals of requests
//!  queued behind it that can be answered while it is stuck.
//! Why:
//!  The owner, 2026-11-04: "Never cut a branch write (Recommended)" (the option read: "Con: a truly hung write keeps the service
//!  busy, and later commits fail at their own 20-minute ceiling with an error naming it"). With no ceiling on the branch write, a
//!  write that never returns holds the one landing thread, which also answers every withdrawal between two landings
//!  (`service_server.rs`); a client queued behind it would reach its answer ceiling, ask to be withdrawn, hear nothing for the
//!  withdrawal ceiling and end with an error that names nothing. This record lets the thread that reads each connection (a) tell
//!  the client what the service is stuck in, and (b) answer the withdrawal of a request that has not begun to land, which is safe
//!  because the landing thread consults the same record before it begins a group: a request is either in the group in flight
//!  (its withdrawal waits for the landing, as before) or withdrawn here and dropped when its group would begin, never both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const board = new LandingBoard(journal);
//! const dropped = board.beginGroup(ids); landGroup(); board.endGroup();       // the landing thread
//! const answer = board.withdrawEarly(id);                                      // a connection thread
//! ```

use crate::service_journal::landed_commits;
use crate::service_messages::RESTART_COMMAND;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// How many withdrawn identities the board remembers before it forgets them all: a client that withdrew sends nothing more for its
///  request, and the service's own memory of withdrawals has the same bound (`service_server.rs`, `WITHDRAWN_RETAINED`).
const EARLY_RETAINED: usize = 4096;

/// What:
///  The write the landing thread is inside, when the wrapper never ends it.
/// Why:
///   The words for the clients are built from it at the moment they are asked for, so the minutes they name are current.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunningStep = { command: string; detail?: string; since: number };
/// ```
struct RunningStep {
    /// The command, such as `git update-ref --stdin`, and where it runs.
    command: String,
    /// What the landing knows about the work the step does, such as the side ref that holds the commit it makes.
    detail: Option<String>,
    /// When the step began.
    since: Instant,
}

/// What the board holds under its lock.
#[derive(Default)]
struct State {
    /// The write the landing thread is inside, when the wrapper never ends it.
    step: Option<RunningStep>,
    /// The identities of the requests of the group being landed.
    in_flight: HashSet<String>,
    /// The identities withdrawn by a connection thread while the landing thread was stuck; they are dropped when their group begins.
    early_withdrawn: HashSet<String>,
}

/// How a withdrawal asked of a connection thread is answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EarlyAnswer {
    /// The landing thread is not stuck, or the request is in the group it is landing: the withdrawal goes to it as before.
    Queued,
    /// The request had landed: the commit is the answer.
    Landed(String),
    /// The request has not begun to land and never will.
    Withdrawn(String),
}

/// What:
///  The shared record of the landing thread's work.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LandingBoard { beginGroup(ids): string[]; endGroup(): void; describe(): string | undefined; withdrawEarly(id): EarlyAnswer }
/// ```
pub struct LandingBoard {
    /// How long a write runs before it counts as stuck (`wait_ceilings.rs`, `NEVER_ENDED_STEP_SLOW_AFTER`; shorter in a control).
    slow_after: Duration,
    /// The journal the finished batches are read from, for the answer to a withdrawal of a request that already landed.
    journal: PathBuf,
    /// The record.
    state: Mutex<State>,
}

impl LandingBoard {
    /// A board for the service whose journal is `journal`, where a write counts as stuck after `slow_after`.
    pub fn new(journal: PathBuf, slow_after: Duration) -> Arc<Self> {
        return Arc::new(Self { slow_after, journal, state: Mutex::new(State::default()) });
    }

    /// The record, whether or not a thread that held it panicked: a poisoned record is still a record of what the thread did.
    fn locked(&self) -> MutexGuard<'_, State> {
        return self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    }

    /// What:
    ///  Mark the requests of a group as being landed, except those a connection thread withdrew while the landing thread was
    ///  stuck; the identities of those are returned and are not marked.
    /// Why:
    ///   The check and the marking happen under one lock, so a request is withdrawn early or lands, never both.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const dropped = ids.filter((id) => earlyWithdrawn.delete(id)); inFlight = ids.filter((id) => !dropped.includes(id));
    /// ```
    pub fn begin_group(&self, ids: &[String]) -> Vec<String> {
        let mut state: MutexGuard<'_, State> = self.locked();
        let dropped: Vec<String> = ids.iter().filter(|id| { return state.early_withdrawn.contains(id.as_str()); }).cloned().collect();
        state.in_flight = ids.iter().filter(|id| { return !dropped.contains(id); }).cloned().collect();
        return dropped;
    }

    /// The group is landed (or given up): none of its requests is in flight.
    pub fn end_group(&self) {
        self.locked().in_flight.clear();
    }

    /// The words that tell a client what the service is stuck in, when the landing thread has been inside a write that is never
    ///  ended for longer than the board's `slow_after`; nothing otherwise.
    pub fn describe(&self) -> Option<String> {
        let state: MutexGuard<'_, State> = self.locked();
        let step: &RunningStep = state.step.as_ref()?;
        let running: Duration = step.since.elapsed();
        if running <= self.slow_after {
            return None;
        }
        let minutes: u64 = running.as_secs() / 60;
        let detail: String = step.detail.as_ref().map_or_else(String::new, |text| { return format!(" ({text})"); });
        return Some(format!(
            "cli-git: the commit service has been inside a write for {minutes} minutes and has not heard back: `{}`{detail}. The service never ends a write in \
             the middle, because that could lose data, so commits queue behind it. If it never returns, look at what that process \
             waits for with `ps` and at the disk, and when no program uses the repository, restart the service with `{RESTART_COMMAND}`.\n",
            step.command
        ));
    }

    /// What:
    ///  Answer a withdrawal on the connection thread when the landing thread is stuck in a write that is never ended and the
    ///  request is not in the group it is landing.
    /// Why:
    ///   The landing thread answers withdrawals between two landings, so behind a stuck write none would be answered. A request
    ///  that has not begun cannot land while the thread is stuck, and the record keeps it from beginning later.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stuck && !inFlight.has(id) ? (landed(id) ?? withdraw(id)) : 'queued'
    /// ```
    pub fn withdraw_early(&self, id: &str) -> EarlyAnswer {
        let Some(words) = self.describe() else {
            return EarlyAnswer::Queued;
        };
        // Read before the lock is taken again: a request that is not in flight cannot land while the thread is stuck.
        let landed: Option<String> = landed_commits(self.journal.as_path()).remove(id);
        let mut state: MutexGuard<'_, State> = self.locked();
        if state.in_flight.contains(id) || state.step.is_none() {
            return EarlyAnswer::Queued;
        }
        if let Some(commit) = landed {
            return EarlyAnswer::Landed(commit);
        }
        if state.early_withdrawn.len() > EARLY_RETAINED {
            state.early_withdrawn.clear();
        }
        state.early_withdrawn.insert(String::from(id));
        return EarlyAnswer::Withdrawn(words);
    }
}

thread_local! {
    /// The board of the service whose landing runs on this thread; no other thread has one.
    static BOARD: RefCell<Option<Arc<LandingBoard>>> = const { RefCell::new(None) };
    /// What the landing knows about the next write that is never ended, for the words that name it.
    static DETAIL: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Make `board` the record of the landing that runs on the calling thread.
pub fn use_on_this_thread(board: Arc<LandingBoard>) {
    BOARD.with(|slot| { *slot.borrow_mut() = Some(board); });
}

/// Clears the detail when the work it describes is over.
pub struct DetailGuard;

impl Drop for DetailGuard {
    fn drop(&mut self) {
        DETAIL.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// Say what the write that follows does, for as long as the guard lives; the words of a stuck write carry it.
pub fn detail(text: String) -> DetailGuard {
    DETAIL.with(|slot| { *slot.borrow_mut() = Some(text); });
    return DetailGuard;
}

/// Clears the running step when the write ends, however it ends.
pub struct StepGuard {
    /// The board the step was recorded on.
    board: Arc<LandingBoard>,
}

impl Drop for StepGuard {
    fn drop(&mut self) {
        self.board.locked().step = None;
    }
}

/// What:
///  Record that this thread is inside a write that is never ended, until the guard is dropped; nothing on a thread that has no
///  board (the wrapper's own commands, the other threads of the service).
/// Why:
///   Only the landing thread holds the service up, so only its steps are recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using step = enterStep(commandText);
/// ```
pub fn enter_step(command: String) -> Option<StepGuard> {
    let board: Arc<LandingBoard> = BOARD.with(|slot| { return slot.borrow().clone(); })?;
    let detail: Option<String> = DETAIL.with(|slot| { return slot.borrow().clone(); });
    board.locked().step = Some(RunningStep { command, detail, since: Instant::now() });
    return Some(StepGuard { board });
}

/// The record's controls stay out of the release executable.
#[cfg(test)]
#[path = "service_busy_tests.rs"]
mod tests;
