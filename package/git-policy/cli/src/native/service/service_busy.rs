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
//!  the client what the service is stuck in, and (b) answer the withdrawal of a request that has not begun to land.
//!  The record holds no lock (the owner: "Use as few locks as possible."; the lock inventory guard,
//!  `source_guard/service_lock_inventory_tests.rs`): one owner per item and exclusive creation, in a directory of the journal.
//!  The landing thread alone writes the file `step` (a rename of a complete file) and creates `<id>.flight` for each request of
//!  the group it is about to land; a connection thread alone creates `<id>.withdrawn`. Each side creates its own marker first and
//!  then looks for the other's. If both are there, the landing drops the request and the connection thread leaves the withdrawal
//!  to the landing thread, which answers "withdrawn" for a dropped request, so the answer agrees with what happened; if the
//!  connection thread's marker is first and complete the request is dropped when its group begins; if the landing's marker is first
//!  the connection thread removes its own and the withdrawal waits for the landing. A request is withdrawn or landed, never both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const board = new LandingBoard(journal);
//! const dropped = board.beginGroup(ids); landGroup(); board.endGroup(ids);     // the landing thread
//! const answer = board.withdrawEarly(id);                                      // a connection thread
//! ```

use crate::service_journal::{OPEN_EXTENSION, landed_commits, list_intents};
use crate::service_messages::RESTART_COMMAND;
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The directory of the record inside the journal.
const DIRECTORY: &str = "busy";

/// The file that names the write the landing thread is inside.
const STEP_FILE: &str = "step";

/// The longest request identity the record keeps a marker for; a longer one, or one with a character outside letters, digits and
///  hyphens, gets no early answer, because its name could not be made into a file name that is only a name.
const LONGEST_ID: usize = 80;

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

/// Whether `id` can be a file name that is only a name.
fn is_plain(id: &str) -> bool {
    return !id.is_empty() && id.len() <= LONGEST_ID && id.chars().all(|character| { return character.is_ascii_alphanumeric() || character == '-'; });
}

/// Milliseconds since 1970, or zero when the clock is before it.
fn now_millis() -> u128 {
    return SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| { return elapsed.as_millis(); });
}

/// What:
///  The shared record of the landing thread's work.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LandingBoard { beginGroup(ids): string[]; endGroup(ids): void; describe(): string | undefined; withdrawEarly(id): EarlyAnswer }
/// ```
pub struct LandingBoard {
    /// How long a write runs before it counts as stuck (`wait_ceilings.rs`, `NEVER_ENDED_STEP_SLOW_AFTER`; shorter in a control).
    slow_after: Duration,
    /// The journal the finished and open batches are read from, for the answer to a withdrawal of a request that already landed.
    journal: PathBuf,
    /// The directory of the record.
    directory: PathBuf,
}

impl LandingBoard {
    /// A board for the service whose journal is `journal`, where a write counts as stuck after `slow_after`. Whatever an earlier
    ///  service left in the record's directory is removed: none of it describes a running thread.
    pub fn new(journal: PathBuf, slow_after: Duration) -> Arc<Self> {
        let directory: PathBuf = journal.join(DIRECTORY);
        let _removed = std::fs::remove_dir_all(directory.as_path());
        let _made = std::fs::create_dir_all(directory.as_path());
        return Arc::new(Self { slow_after, journal, directory });
    }

    /// The marker file of a request.
    fn marker(&self, id: &str, kind: &str) -> PathBuf {
        return self.directory.join(format!("{id}.{kind}"));
    }

    /// Create a marker, failing when it exists: the exclusive creation that decides who was first.
    fn create_marker(&self, id: &str, kind: &str) -> bool {
        return std::fs::OpenOptions::new().write(true).create_new(true).open(self.marker(id, kind)).is_ok();
    }

    /// What:
    ///  Mark the requests of a group as being landed, except those a connection thread withdrew while the landing thread was
    ///  stuck; the identities of those are returned and are not marked.
    /// Why:
    ///   Each request is marked before its withdrawal marker is looked for, so a request is withdrawn early or lands, never both.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ids.filter((id) => { create(`${id}.flight`); return exists(`${id}.withdrawn`) && (remove(`${id}.flight`, `${id}.withdrawn`), true); })
    /// ```
    pub fn begin_group(&self, ids: &[String]) -> Vec<String> {
        let mut dropped: Vec<String> = Vec::new();
        for id in ids.iter().filter(|id| { return is_plain(id.as_str()); }) {
            let _created = self.create_marker(id.as_str(), "flight");
            if self.marker(id.as_str(), "withdrawn").exists() {
                let _unmarked = std::fs::remove_file(self.marker(id.as_str(), "flight"));
                let _answered = std::fs::remove_file(self.marker(id.as_str(), "withdrawn"));
                dropped.push(id.clone());
            }
        }
        return dropped;
    }

    /// The group is landed (or given up): none of its requests is in flight.
    pub fn end_group(&self, ids: &[String]) {
        for id in ids.iter().filter(|id| { return is_plain(id.as_str()); }) {
            let _removed = std::fs::remove_file(self.marker(id.as_str(), "flight"));
        }
    }

    /// Whether the request of this identity is in the group the landing thread is landing.
    pub fn is_in_flight(&self, id: &str) -> bool {
        return is_plain(id) && self.marker(id, "flight").exists();
    }

    /// The write the landing thread is inside, as `(command, detail, millis since it began)`, when there is one.
    fn running(&self) -> Option<(String, Option<String>, u128)> {
        let text: String = std::fs::read_to_string(self.directory.join(STEP_FILE)).ok()?;
        let mut lines = text.lines();
        let began: u128 = lines.next()?.parse().ok()?;
        let command: String = String::from(lines.next()?);
        let detail: Option<String> = lines.next().filter(|line| { return !line.is_empty(); }).map(String::from);
        return Some((command, detail, now_millis().saturating_sub(began)));
    }

    /// The words that tell a client what the service is stuck in, when the landing thread has been inside a write that is never
    ///  ended for longer than the board's `slow_after`; nothing otherwise.
    pub fn describe(&self) -> Option<String> {
        let (command, found, running) = self.running()?;
        if running <= self.slow_after.as_millis() {
            return None;
        }
        let minutes: u128 = running / 60_000;
        let detail: String = found.map_or_else(String::new, |text| { return format!(" ({text})"); });
        return Some(format!(
            "cli-git: the commit service has been inside a write for {minutes} minutes and has not heard back: `{command}`{detail}. The service never ends a write in \
             the middle, because that could lose data, so commits queue behind it. If it never returns, look at what that process \
             waits for with `ps` and at the disk, and when no program uses the repository, restart the service with `{RESTART_COMMAND}`.\n"
        ));
    }

    /// What:
    ///  Answer a withdrawal on the connection thread when the landing thread is stuck in a write that is never ended and the
    ///  request is not in the group it is landing.
    /// Why:
    ///   The landing thread answers withdrawals between two landings, so behind a stuck write none would be answered. A request
    ///  that has not begun cannot land while the thread is stuck, and the markers keep it from beginning later.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stuck && !named(openBatches, id) ? (landed(id) ?? (create(`${id}.withdrawn`), exists(`${id}.flight`) ? 'queued' : withdraw(id))) : 'queued'
    /// ```
    pub fn withdraw_early(&self, id: &str) -> EarlyAnswer {
        if !is_plain(id) {
            return EarlyAnswer::Queued;
        }
        let Some(words) = self.describe() else {
            return EarlyAnswer::Queued;
        };
        if let Some(commit) = landed_commits(self.journal.as_path()).remove(id) {
            return EarlyAnswer::Landed(commit);
        }
        // A batch still open may hold the commit on the branch already (its index update did not install, and a restart finishes
        // it), and only the landing thread knows which: it answers "already" when the write returns, so the withdrawal waits for it.
        let named_by_open_batch: bool = list_intents(self.journal.as_path(), OPEN_EXTENSION)
            .iter()
            .any(|(_, intent)| { return intent.as_ref().is_some_and(|open| { return open.ids.iter().any(|named| { return named == id; }); }); });
        if named_by_open_batch || self.marker(id, "flight").exists() {
            return EarlyAnswer::Queued;
        }
        // Marked first, then the landing's marker is looked for again: a landing that marked the request in between sees this
        // marker or is seen by it, and in both orders the request is dropped or the withdrawal is left to the landing thread.
        let _marked = self.create_marker(id, "withdrawn");
        if self.marker(id, "flight").exists() {
            let _unmarked = std::fs::remove_file(self.marker(id, "withdrawn"));
            return EarlyAnswer::Queued;
        }
        return EarlyAnswer::Withdrawn(words);
    }
}

thread_local! {
    /// The board of the service whose landing runs on this thread; no other thread has one.
    static BOARD: RefCell<Option<Arc<LandingBoard>>> = const { RefCell::new(None) };
    /// What the landing knows about the next write that is never ended, for the words that name it.
    static DETAIL: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Whether the calling thread is the commit service's landing thread, the one thread that waits for a write it cannot end.
pub fn on_landing_thread() -> bool {
    return BOARD.with(|slot| { return slot.borrow().is_some(); });
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
    /// The file that names the step.
    file: PathBuf,
}

impl Drop for StepGuard {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(self.file.as_path());
    }
}

/// Write `text` to `path` as a complete file or not at all: a rename of a file written beside it.
fn publish(path: &Path, text: &str) -> bool {
    let beside: PathBuf = path.with_extension("tmp");
    let written: bool = std::fs::File::create(beside.as_path()).and_then(|mut file| { return file.write_all(text.as_bytes()); }).is_ok();
    return written && std::fs::rename(beside.as_path(), path).is_ok();
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
    let file: PathBuf = board.directory.join(STEP_FILE);
    let one_line = |text: &str| -> String { return text.split_whitespace().collect::<Vec<&str>>().join(" "); };
    let text: String = format!("{}\n{}\n{}\n", now_millis(), one_line(command.as_str()), detail.as_deref().map_or_else(String::new, one_line));
    // A step that cannot be recorded is still run; the clients just are not told about it.
    let _recorded = publish(file.as_path(), text.as_str());
    return Some(StepGuard { file });
}

/// The record's controls stay out of the release executable.
#[cfg(test)]
#[path = "service_busy_tests.rs"]
mod tests;
