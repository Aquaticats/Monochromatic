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
//!  Admission and cancellation are one decision per request, made by creating the file `<id>.decision` with `link(2)`, which fails
//!  when the name exists and which shows the file whole: the landing thread tries to create it with the content `landing` before it
//!  lands the request, a connection thread tries to create it with `withdrawn` before it answers a withdrawal, and exactly one
//!  creation succeeds. The loser reads what the winner wrote. A request that is `landing` is the landing thread's to answer; a
//!  request that is `withdrawn` is dropped whenever a copy of it reaches a group, as often as it comes, until the retention forgets it
//!  (the same bounded memory a normal withdrawal has; only the landing thread forgets, between two groups). A `withdrawn` decision
//!  carries the token of its maker and counts as a cancellation for anyone else only when the file `<id>.<token>.cancelled` exists. A `landing` file is removed when its group is done, after the batch is in the
//!  journal, so a withdrawal that wins afterwards asks the journal again before it answers, and answers `landed` for a request the
//!  journal names. Anything that cannot be established, a file that cannot be made or read, a journal that cannot be read, is never
//!  evidence of a cancellation: the withdrawal is left to the landing thread, and a request that cannot be admitted is answered
//!  with a failure, never dropped silently and never landed on a decision that was not made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const board = new LandingBoard(journal);
//! const admission = board.beginGroup(ids); landGroup(); board.endGroup(ids);    // the landing thread
//! const answer = board.withdrawEarly(id);                                      // a connection thread
//! ```

use crate::service_journal::{DONE_EXTENSION, DONE_RETAINED, OPEN_EXTENSION, parse_intent};
use crate::service_messages::RESTART_COMMAND;
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The directory of the record inside the journal.
const DIRECTORY: &str = "busy";

/// The file that names the write the landing thread is inside.
const STEP_FILE: &str = "step";

/// The extension of the decision file of a request.
const DECISION_EXTENSION: &str = "decision";

/// What the decision file says when the landing thread took the request.
const LANDING: &str = "landing";

/// What the decision file says when a withdrawal took the request: this word, a colon and the token of the withdrawer that made it.
const WITHDRAWN: &str = "withdrawn";

/// The extension of the file that confirms a withdrawal, named `<id>.<token>.cancelled` with the token the decision carries: the
///  withdrawer that won the decision made it after it asked the journal a second time and found no batch naming the request. A
///  decision without its confirmation is provisional: nobody but its creator may rely on it. The token ties the confirmation to
///  that one decision, so a confirmation that outlives its decision (the retention removes the pair in two steps) confirms nothing
///  of the decision that replaces it.
const CANCELLED_EXTENSION: &str = "cancelled";

/// The longest request identity the record keeps a decision for; a longer one, or one with a character outside letters, digits and
///  hyphens, gets no early answer, because its name could not be made into a file name that is only a name.
const LONGEST_ID: usize = 80;

/// How many withdrawals the record remembers: the same as the finished batches the journal remembers, and as the service's own
///  memory of normal withdrawals (`service_server.rs`, `WITHDRAWN_RETAINED`), since a client that withdrew sends nothing more for
///  its request.
const WITHDRAWALS_RETAINED: usize = DONE_RETAINED;

/// The most confirmed withdrawals the record holds at any moment, twice what it retains: only the landing thread forgets (between two
///  groups, so there is one pruner), and while it is stuck in a write that is never ended nothing is forgotten, so past this number
///  the connection threads leave further withdrawals to the landing thread instead of growing the record without a bound. Two connection
///  threads that count together can pass it by the number of connection threads at that moment.
const WITHDRAWALS_CEILING: usize = 2 * WITHDRAWALS_RETAINED;

/// How a withdrawal asked of a connection thread is answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EarlyAnswer {
    /// The landing thread is not stuck, or the request is the landing thread's, or nothing could be established: the withdrawal goes
    ///  to the landing thread as before.
    Queued,
    /// The request had landed: the commit is the answer.
    Landed(String),
    /// The request has not begun to land and never will.
    Withdrawn(String),
}

/// What the landing thread learns about the requests of a group before it lands them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GroupAdmission {
    /// The requests a withdrawal took: dropped, and answered to nobody, because their clients were told already.
    pub dropped: Vec<String>,
    /// The requests whose admission could not be established: answered with a failure and not landed.
    pub unavailable: Vec<String>,
}

/// What the journal says about a request.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Naming {
    /// A finished batch names the request, with the commit it became.
    Landed(String),
    /// A batch still open names it: its commit may be on the branch.
    Open,
    /// No batch names it.
    No,
    /// The journal could not be read whole, so nothing is known.
    Unknown,
}

/// What a try at the decision came to.
enum Decision {
    /// This side made the decision.
    Won,
    /// The other side made it first, and wrote this.
    Lost(String),
}

/// Whether `id` can be a file name that is only a name.
fn is_plain(id: &str) -> bool {
    return !id.is_empty() && id.len() <= LONGEST_ID && id.chars().all(|character| { return character.is_ascii_alphanumeric() || character == '-'; });
}

/// Milliseconds since 1970, or zero when the clock is before it.
fn now_millis() -> u128 {
    return SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| { return elapsed.as_millis(); });
}

/// A temporary file that is removed when it goes out of scope.
struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(self.0.as_path());
    }
}

/// Distinguishes the temporary files of concurrent decisions, and the tokens of withdrawals.
static TEMPORARY_NUMBER: AtomicU64 = AtomicU64::new(0);

/// A token no other withdrawal in any service has, made of letters, digits and hyphens so it can stand in a file name.
fn new_token() -> String {
    return format!("{}-{}-{}", std::process::id(), TEMPORARY_NUMBER.fetch_add(1, Ordering::Relaxed), now_millis());
}

/// The token a `withdrawn` decision carries, when `content` is one.
fn withdrawal_token(content: &str) -> Option<&str> {
    return content.strip_prefix(WITHDRAWN).and_then(|rest| { return rest.strip_prefix(':'); }).filter(|token| { return !token.is_empty(); });
}

/// What:
///  The shared record of the landing thread's work.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LandingBoard { beginGroup(ids): GroupAdmission; endGroup(ids): void; describe(): string | undefined; withdrawEarly(id): EarlyAnswer }
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
    ///  service left in the record's directory is removed: none of it describes a running thread, and a normal withdrawal's memory
    ///  does not outlive the service either.
    pub fn new(journal: PathBuf, slow_after: Duration) -> Arc<Self> {
        let directory: PathBuf = journal.join(DIRECTORY);
        let _removed = std::fs::remove_dir_all(directory.as_path());
        let _made = std::fs::create_dir_all(directory.as_path());
        return Arc::new(Self { slow_after, journal, directory });
    }

    /// The decision file of a request.
    pub(crate) fn decision_path(&self, id: &str) -> PathBuf {
        return self.directory.join(format!("{id}.{DECISION_EXTENSION}"));
    }

    /// The file that confirms the withdrawal of a request that carries `token`.
    fn cancelled_path(&self, id: &str, token: &str) -> PathBuf {
        return self.directory.join(format!("{id}.{token}.{CANCELLED_EXTENSION}"));
    }

    /// What:
    ///  Try to make the decision of `id` with the content `outcome`.
    /// Why:
    ///   The file is written whole beside its name and then linked to it, so it never shows partly written, and `link(2)` fails when
    ///  the name exists, so exactly one side makes the decision. Any other failure is an error: nothing was decided.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { link(temporary, decisionPath); return 'won'; } catch (exists) { return { lost: read(decisionPath) }; }
    /// ```
    fn decide(&self, id: &str, outcome: &str) -> std::io::Result<Decision> {
        // Removed whichever way this function leaves, a failed write included (a leaked file is an inode the recovery needs).
        let beside: TemporaryFile = TemporaryFile(self.directory.join(format!("{id}.{}.{}.tmp", std::process::id(), TEMPORARY_NUMBER.fetch_add(1, Ordering::Relaxed))));
        std::fs::File::create(beside.0.as_path()).and_then(|mut file| { return file.write_all(outcome.as_bytes()); })?;
        #[cfg(test)]
        AFTER_THE_TEMPORARY_IS_WRITTEN.with(|hook| { return hook.borrow().as_ref().map_or(Ok(()), |fail| { return fail(); }); })?;
        let linked: std::io::Result<()> = std::fs::hard_link(beside.0.as_path(), self.decision_path(id));
        return match linked {
            Ok(()) => Ok(Decision::Won),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(Decision::Lost(std::fs::read_to_string(self.decision_path(id))?)),
            Err(error) => Err(error),
        };
    }

    /// What the journal says about `id`: read whole, and `Unknown` for anything that cannot be read or understood.
    fn naming(&self, id: &str) -> Naming {
        let Ok(entries) = std::fs::read_dir(self.journal.as_path()) else {
            return Naming::Unknown;
        };
        let mut open: bool = false;
        for entry in entries {
            let Ok(found) = entry else { return Naming::Unknown };
            let path: PathBuf = found.path();
            let extension: Option<&str> = path.extension().and_then(|text| { return text.to_str(); });
            if extension != Some(OPEN_EXTENSION) && extension != Some(DONE_EXTENSION) {
                continue;
            }
            let Some(intent) = std::fs::read_to_string(path.as_path()).ok().and_then(|text| { return parse_intent(text.as_str()); }) else {
                return Naming::Unknown;
            };
            let Some(position) = intent.ids.iter().position(|named| { return named == id; }) else { continue };
            if extension == Some(DONE_EXTENSION) {
                return intent.commits.get(position).map_or(Naming::Unknown, |commit| { return Naming::Landed(commit.clone()); });
            }
            open = true;
        }
        return if open { Naming::Open } else { Naming::No };
    }

    /// What:
    ///  Take the requests of a group for landing: decide each as `landing`, learn which a withdrawal took, and which could not be
    ///  decided.
    /// Why:
    ///   A request a withdrawal took is dropped, however many copies of it come; one that cannot be decided is not landed on a
    ///  decision that was not made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const id of ids) { const d = decide(id, 'landing'); if (d.error) unavailable.push(id); else if (d.lost === 'withdrawn') dropped.push(id); }
    /// ```
    pub fn begin_group(&self, ids: &[String]) -> GroupAdmission {
        let mut admission: GroupAdmission = GroupAdmission::default();
        for id in ids.iter().filter(|id| { return is_plain(id.as_str()); }) {
            match self.decide(id.as_str(), LANDING) {
                Ok(Decision::Won) => {}
                Ok(Decision::Lost(content)) if content == LANDING => {}
                Ok(Decision::Lost(content)) if withdrawal_token(content.as_str()).is_some() => admission.dropped.push(id.clone()),
                Ok(Decision::Lost(_)) | Err(_) => admission.unavailable.push(id.clone()),
            }
        }
        return admission;
    }

    /// The group is landed (or given up): its requests are the landing thread's no more, and each is in the journal if it landed.
    ///  A tombstone of a withdrawal stays.
    pub fn end_group(&self, ids: &[String]) {
        self.prune_withdrawals();
        for id in ids.iter().filter(|id| { return is_plain(id.as_str()); }) {
            if std::fs::read_to_string(self.decision_path(id.as_str())).is_ok_and(|content| { return content == LANDING; }) {
                let _removed = std::fs::remove_file(self.decision_path(id.as_str()));
            }
        }
    }

    /// Whether the request of this identity is in the group the landing thread is landing.
    pub fn is_in_flight(&self, id: &str) -> bool {
        return is_plain(id) && std::fs::read_to_string(self.decision_path(id)).is_ok_and(|content| { return content == LANDING; });
    }

    /// The write the landing thread is inside, as `(millis when it began, command, detail, millis it has run)`, when there is one.
    fn running(&self) -> Option<(u128, String, Option<String>, u128)> {
        let text: String = std::fs::read_to_string(self.directory.join(STEP_FILE)).ok()?;
        let mut lines = text.lines();
        let began: u128 = lines.next()?.parse().ok()?;
        let command: String = String::from(lines.next()?);
        let detail: Option<String> = lines.next().filter(|line| { return !line.is_empty(); }).map(String::from);
        return Some((began, command, detail, now_millis().saturating_sub(began)));
    }

    /// The words that tell a client what the service is stuck in, when the landing thread has been inside a write that is never
    ///  ended for longer than the board's `slow_after`; nothing otherwise.
    pub fn describe(&self) -> Option<String> {
        let (_, command, found, running) = self.running()?;
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
    ///  The notice for the client whose own request is in the group being landed while the landing thread is inside a write that is
    ///  never ended: a key that changes when the notice should be sent again, and the words.
    /// Why:
    ///   The client must learn that its commit may be on the branch when the write starts, not a minute later (owner, 2026-11-04,
    ///  "Short extra wait"); the words that follow a minute's stuck write are the longer ones.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// inFlight(id) && running ? { key: [began, minutes], text } : undefined
    /// ```
    pub fn inside_notice(&self, id: &str) -> Option<(String, String)> {
        let (began, command, found, running) = self.running()?;
        if !self.is_in_flight(id) {
            return None;
        }
        if let Some(stuck) = self.describe() {
            return Some((format!("{began}:{}", running / 60_000), stuck));
        }
        let detail: String = found.map_or_else(String::new, |text| { return format!(" ({text})"); });
        return Some((format!("{began}:start"), format!(
            "cli-git: this commit is inside a write of the commit service, `{command}`{detail}, begun {} seconds ago. The service never ends a write \
             in the middle, so if it does not return this command reports that the result is not known after its wait.\n",
            running / 1000
        )));
    }

    /// What:
    ///  Answer a withdrawal on the connection thread when the landing thread is stuck in a write that is never ended and the
    ///  request is not the landing thread's.
    /// Why:
    ///   The landing thread answers withdrawals between two landings, so behind a stuck write none would be answered. A request
    ///  that has not begun cannot land while the thread is stuck, and the decision keeps it from beginning later. The journal is
    ///  asked before the decision is tried and again after it is won, because a request that landed and was let go of between the
    ///  two looks like a request nobody took.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stuck && journal(id) === 'no' && won(decide(id, 'withdrawn')) && journal(id) === 'no' ? 'withdrawn' : (landed ?? 'queued')
    /// ```
    pub fn withdraw_early(&self, id: &str) -> EarlyAnswer {
        if !is_plain(id) {
            return EarlyAnswer::Queued;
        }
        let Some(words) = self.describe() else {
            return EarlyAnswer::Queued;
        };
        match self.naming(id) {
            Naming::Landed(commit) => return EarlyAnswer::Landed(commit),
            Naming::Open | Naming::Unknown => return EarlyAnswer::Queued,
            Naming::No => {}
        }
        // The landing thread forgets old withdrawals between two groups, and a stuck write is one group: the record is bounded here.
        if self.confirmed_withdrawals() >= WITHDRAWALS_CEILING {
            return EarlyAnswer::Queued;
        }
        let token: String = new_token();
        match self.decide(id, format!("{WITHDRAWN}:{token}").as_str()) {
            Err(_) => return EarlyAnswer::Queued,
            // Another withdrawer won. Its decision is acknowledged only when it has confirmed it, with the confirmation that carries
            // the token of that decision; before that it is provisional, and may still be taken back (the journal may name the
            // request when the winner asks again). A confirmation of an earlier decision of this request has another token.
            Ok(Decision::Lost(content)) => {
                let confirmed: bool = withdrawal_token(content.as_str()).is_some_and(|theirs| { return self.cancelled_path(id, theirs).exists(); });
                return if confirmed { EarlyAnswer::Withdrawn(words) } else { EarlyAnswer::Queued };
            }
            Ok(Decision::Won) => {}
        }
        #[cfg(test)]
        AFTER_THE_DECISION_IS_WON.with(|hook| { if let Some(pause) = hook.borrow().as_ref() { pause(); } });
        // The decision is ours, so the request cannot be admitted any more; what could have happened is that it was admitted,
        // landed and let go of before the decision was tried.
        match self.naming(id) {
            Naming::No => {
                // Confirmed: from now on any withdrawer may acknowledge it, and nothing removes it but the retention.
                if std::fs::File::create_new(self.cancelled_path(id, token.as_str())).is_err() {
                    // Not confirmed, and nobody may have acknowledged a decision that was not: it is taken back.
                    let _undone = std::fs::remove_file(self.decision_path(id));
                    return EarlyAnswer::Queued;
                }
                return EarlyAnswer::Withdrawn(words);
            }
            // Never confirmed, so never acknowledged by anyone else: taken back.
            Naming::Landed(commit) => {
                let _undone = std::fs::remove_file(self.decision_path(id));
                return EarlyAnswer::Landed(commit);
            }
            Naming::Open | Naming::Unknown => {
                let _undone = std::fs::remove_file(self.decision_path(id));
                return EarlyAnswer::Queued;
            }
        }
    }

    /// How many confirmed withdrawals the record holds; the ceiling is reached when the directory cannot be listed, so that a record that
    ///  cannot be counted is not grown.
    fn confirmed_withdrawals(&self) -> usize {
        let Ok(entries) = std::fs::read_dir(self.directory.as_path()) else { return usize::MAX };
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().and_then(|text| { return text.to_str(); }) == Some(CANCELLED_EXTENSION); })
            .count();
    }

    /// What:
    ///  Forget the oldest confirmed withdrawals once more than `WITHDRAWALS_RETAINED` are remembered.
    /// Why:
    ///   Only the landing thread calls this (from `end_group`), so there is one pruner and no two work from the same listing. A
    ///  decision is removed only while it still carries the token of the confirmation being removed, and before that
    ///  confirmation: a decision that replaced it has another token and stays, and a confirmation left standing without its decision
    ///  (a pause between the two removals) confirms nothing of any later decision, which has a token of its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const [id, token] of oldestConfirmations(surplus)) { if (read(decision(id)) === `withdrawn:${token}`) rm(decision(id)); rm(confirmation(id, token)); }
    /// ```
    fn prune_withdrawals(&self) {
        let Ok(entries) = std::fs::read_dir(self.directory.as_path()) else { return };
        let mut remembered: Vec<(SystemTime, PathBuf)> = entries
            .flatten()
            .filter(|entry| { return entry.path().extension().and_then(|text| { return text.to_str(); }) == Some(CANCELLED_EXTENSION); })
            .filter_map(|entry| { return Some((entry.metadata().ok()?.modified().ok()?, entry.path())); })
            .collect();
        if remembered.len() <= WITHDRAWALS_RETAINED {
            return;
        }
        remembered.sort();
        let surplus: usize = remembered.len() - WITHDRAWALS_RETAINED;
        #[cfg(test)]
        AFTER_THE_LISTING.with(|hook| { if let Some(pause) = hook.borrow().as_ref() { pause(); } });
        for (_, confirmation) in remembered.into_iter().take(surplus) {
            let Some(name) = confirmation.file_stem().and_then(|text| { return text.to_str(); }) else { continue };
            let Some((id, token)) = name.split_once('.') else { continue };
            let decision: PathBuf = self.decision_path(id);
            if std::fs::read_to_string(decision.as_path()).is_ok_and(|content| { return withdrawal_token(content.as_str()) == Some(token); }) {
                let _forgotten = std::fs::remove_file(decision);
            }
            let _also = std::fs::remove_file(confirmation);
        }
    }
}

#[cfg(test)]
thread_local! {
    /// A control may fail a decision on this thread after its temporary file was made and written, which is where the real write fails.
    pub(crate) static AFTER_THE_TEMPORARY_IS_WRITTEN: RefCell<Option<Box<dyn Fn() -> std::io::Result<()>>>> = const { RefCell::new(None) };
    /// A control may stop the retention on this thread between its listing and its removals, where another decision can replace one.
    pub(crate) static AFTER_THE_LISTING: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
    /// A control may stop a withdrawal on this thread between the moment it wins the decision and the moment it asks the journal again,
    ///  which is where the reviews found the window.
    pub(crate) static AFTER_THE_DECISION_IS_WON: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
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
