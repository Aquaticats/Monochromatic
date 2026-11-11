//! What:
//!  What the commit service's one landing thread is doing,
//! shared with the threads that talk to waiting clients:
//! the write it is
//!  inside when that write is one the wrapper never ends,
//! the requests of the group it lands,
//! and the withdrawals of requests
//!  queued behind it that can be answered while it is stuck.
//! Why:
//!  The owner,
//! 2026-11-04:
//! "Never cut a branch write (Recommended)" (the option read:
//! "Con:
//! a truly hung write keeps the service
//!  busy,
//! and later commits fail at their own 20-minute ceiling with an error naming it").
//! With no ceiling on the branch write,
//! a
//!  write that never returns holds the one landing thread,
//! which also answers every withdrawal between two landings
//!  (`service_server.rs`);
//! a client queued behind it would reach its answer ceiling,
//! ask to be withdrawn,
//! hear nothing for the
//!  withdrawal ceiling and end with an error that names nothing.
//! This record lets the thread that reads each connection (a) tell
//!  the client what the service is stuck in,
//! and (b) answer the withdrawal of a request that has not begun to land.
//!  The record holds no lock (the owner:
//! "Use as few locks as possible.";
//! the lock inventory guard,
//!  `source_guard/service_lock_inventory_tests.rs`):
//! one owner per item and exclusive creation,
//! in a directory of the journal.
//!  Admission and cancellation are one decision per request,
//! made by publishing the file `<id>.decision` without replacing (`private_storage`:
//!  `renameat2` with `RENAME_NOREPLACE`,
//! or a hard link,
//! either of which fails when the name exists and shows the file whole;
//! on a
//!  filesystem with neither,
//! a checked rename,
//! which is atomic but not exclusive):
//! the landing thread tries to create it with the content `landing` before it
//!  lands the request,
//! a connection thread tries to create it with `withdrawn` before it answers a withdrawal,
//! and exactly one
//!  creation succeeds.
//! The loser reads what the winner wrote.
//! A request that is `landing` is the landing thread's to answer;
//! a
//!  request that is `withdrawn` is dropped whenever a copy of it reaches a group,
//! as often as it comes,
//! until the retention forgets it
//!  (the same bounded memory a normal withdrawal has;
//! only the landing thread forgets,
//! between two groups).
//! A `withdrawn` decision
//!  carries the token of its maker and counts as a cancellation for anyone else only when the file `<id>.<token>.cancelled` exists.
//! A `landing` file is removed when its group is done,
//! after the batch is in the
//!  journal,
//! so a withdrawal that wins afterwards asks the journal again before it answers,
//! and answers `landed` for a request the
//!  journal names.
//! Anything that cannot be established,
//! a file that cannot be made or read,
//! a journal that cannot be read,
//! is never
//!  evidence of a cancellation:
//! the withdrawal is left to the landing thread,
//! and a request that cannot be admitted is answered
//!  with a failure,
//! never dropped silently and never landed on a decision that was not made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const board = new LandingBoard(journal);
//! const admission = board.beginGroup(ids); landGroup(); board.endGroup(ids);    // the landing thread
//! const answer = board.withdrawEarly(id);                                      // a connection thread
//! ```

pub use crate::service_busy_answers_and_name_checks::{EarlyAnswer, GroupAdmission};
use crate::service_busy_answers_and_name_checks::{CANCELLED_EXTENSION, Decision, Naming, WITHDRAWN, is_confirmation, is_plain, modified_and_path, withdrawal_token};
#[cfg(test)]
use std::cell::RefCell;
use crate::service_journal::{DONE_EXTENSION, DONE_RETAINED, OPEN_EXTENSION, parse_intent};
use crate::service_messages::RESTART_COMMAND;
use crate::private_storage::write_private_file_unsynced;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The directory of the record inside the journal.
const DIRECTORY: &str = "busy";

/// The file that names the write the landing thread is inside.
pub(crate) const STEP_FILE: &str = "step";

/// The extension of the decision file of a request.
const DECISION_EXTENSION: &str = "decision";

/// What the decision file says when the landing thread took the request.
const LANDING: &str = "landing";

/// How many withdrawals the record remembers:
/// the same as the finished batches the journal remembers,
/// and as the service's own
///  memory of normal withdrawals (`service_server.rs`,
/// `WITHDRAWN_RETAINED`),
/// since a client that withdrew sends nothing more for
///  its request.
const WITHDRAWALS_RETAINED: usize = DONE_RETAINED;

/// The most confirmed withdrawals the record holds at any moment,
/// twice what it retains:
/// only the landing thread forgets (between two
///  groups,
/// so there is one pruner),
/// and while it is stuck in a write that is never ended nothing is forgotten,
/// so past this number
///  the connection threads leave further withdrawals to the landing thread instead of growing the record without a bound.
/// Two connection
///  threads that count together can pass it by the number of connection threads at that moment.
const WITHDRAWALS_CEILING: usize = 2 * WITHDRAWALS_RETAINED;

/// Milliseconds since 1970,
/// or zero when the clock is before it.
pub(crate) fn now_millis() -> u128 {
    return match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_before_the_epoch) => 0,
    };
}

/// Distinguishes the tokens of withdrawals.
static TEMPORARY_NUMBER: AtomicU64 = AtomicU64::new(0);

/// A token no other withdrawal in any service has,
/// made of letters,
/// digits and hyphens so it can stand in a file name.
fn new_token() -> String {
    return format!("{}-{}-{}", std::process::id(), TEMPORARY_NUMBER.fetch_add(1, Ordering::Relaxed), now_millis());
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
    /// How long a write runs before it counts as stuck (`wait_ceilings.rs`,
    /// `NEVER_ENDED_STEP_SLOW_AFTER`;
    /// shorter in a control).
    slow_after: Duration,
    /// The journal the finished and open batches are read from,
    /// for the answer to a withdrawal of a request that already landed.
    journal: PathBuf,
    /// The directory of the record.
    pub(crate) directory: PathBuf,
}

/// What:
///  One record of this board,
/// read as text,
///  and nothing when the name holds no regular file or the read fails.
/// Why:
///  These reads answer a waiting client's beat (`service_reply.rs`) and the connection that answers a withdrawal,
///  where an open that waited for a writer would hold back the proof of life and no timer could end it.
///  Nothing but this process writes these records,
///  so a pipe or a link at one of their names is another program's,
///  and it names no write or decision of this board.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordText = (path) => { try { return readFileSync(openRegularFile(path), 'utf8'); } catch { return undefined; } };
/// ```
fn record_text(path: &std::path::Path) -> Option<String> {
    let mut file: std::fs::File = crate::private_storage::open_regular_file(path).ok()?;
    let mut text: String = String::new();
    std::io::Read::read_to_string(&mut file, &mut text).ok()?;
    return Some(text);
}

/// A decision file that is not a regular file is a failure of the winner's publish,
/// and never an evidence of a cancellation.
fn decision_read_failure() -> std::io::Error {
    return std::io::Error::other("the decision another program published is not a regular file");
}

impl LandingBoard {
    /// A board for the service whose journal is `journal`,
    /// where a write counts as stuck after `slow_after`.
    /// Whatever an earlier
    ///  service left in the record's directory is removed:
    /// none of it describes a running thread,
    /// and a normal withdrawal's memory
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
    ///   The file is written whole beside its name and then published without replacing it,
    /// so it never shows partly written,
    /// and
    ///  the publish fails when the name exists,
    /// so exactly one side makes the decision (on a filesystem that has neither an exclusive
    ///  rename nor hard links,
    /// the checked rename is made one step to the threads of this service by an in-memory claim,
    ///  `name_held_while_one_thread_publishes_it.rs`,
    /// so the landing thread and a connection thread never both win;
    /// only another
    ///  program could still overwrite the file there,
    /// as the owner was told when he kept that fallback,
    /// 2026-11-07).
    /// Any other failure is an error:
    /// nothing was decided.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { publishWithoutReplacing(temporary, decisionPath); return 'won'; } catch (exists) { return { lost: read(decisionPath) }; }
    /// ```
    fn decide(&self, id: &str, outcome: &str) -> std::io::Result<Decision> {
        let decision: PathBuf = self.decision_path(id);
        // Written whole beside its name and published without replacing (`private_storage`: the exclusive publish of the `tempfile`
        //  crate, the checked rename where the filesystem has neither that nor hard links), removed whichever way it ends.
        return match write_private_file_unsynced(decision.as_path(), outcome.as_bytes()) {
            Ok(()) => Ok(Decision::Won),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(Decision::Lost(record_text(decision.as_path()).ok_or_else(decision_read_failure)?)),
            Err(error) => Err(error),
        };
    }

    /// What the journal says about `id`:
    /// read whole,
    /// and `Unknown` for anything that cannot be read or understood.
    fn naming(&self, id: &str) -> Naming {
        let Ok(entries) = std::fs::read_dir(self.journal.as_path()) else {
            return Naming::Unknown;
        };
        let mut open: bool = false;
        for entry in entries {
            let Ok(found) = entry else { return Naming::Unknown };
            let path: PathBuf = found.path();
            let extension: Option<&str> = path.extension().and_then(std::ffi::OsStr::to_str);
            if extension != Some(OPEN_EXTENSION) && extension != Some(DONE_EXTENSION) {
                continue;
            }
            // A record that cannot be read or parsed leaves the answer unknown.
            let Some(record) = record_text(path.as_path()) else {
                return Naming::Unknown;
            };
            let Some(intent) = parse_intent(record.as_str()) else {
                return Naming::Unknown;
            };
            let mut found_at: Option<usize> = None;
            for (at, named) in intent.ids.iter().enumerate() {
                if named == id {
                    found_at = Some(at);
                    break;
                }
            }
            let Some(position) = found_at else { continue };
            if extension == Some(DONE_EXTENSION) {
                return match intent.commits.get(position) {
                    Some(commit) => Naming::Landed(commit.clone()),
                    None => Naming::Unknown,
                };
            }
            open = true;
        }
        return if open { Naming::Open } else { Naming::No };
    }

    /// What:
    ///  Take the requests of a group for landing:
    /// decide each as `landing`,
    /// learn which a withdrawal took,
    /// and which could not be
    ///  decided.
    /// Why:
    ///   A request a withdrawal took is dropped,
    /// however many copies of it come;
    /// one that cannot be decided is not landed on a
    ///  decision that was not made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const id of ids) { const d = decide(id, 'landing'); if (d.error) unavailable.push(id); else if (d.lost === 'withdrawn') dropped.push(id); }
    /// ```
    pub fn begin_group(&self, ids: &[String]) -> GroupAdmission {
        let mut admission: GroupAdmission = GroupAdmission::default();
        for id in ids {
            if !is_plain(id.as_str()) {
                continue;
            }
            match self.decide(id.as_str(), LANDING) {
                Ok(Decision::Won) => {}
                Ok(Decision::Lost(content)) if content == LANDING => {}
                Ok(Decision::Lost(content)) if withdrawal_token(content.as_str()).is_some() => admission.dropped.push(id.clone()),
                Ok(Decision::Lost(_)) | Err(_) => admission.unavailable.push(id.clone()),
            }
        }
        return admission;
    }

    /// The group is landed (or given up):
    /// its requests are the landing thread's no more,
    /// and each is in the journal if it landed.
    ///  A tombstone of a withdrawal stays.
    pub fn end_group(&self, ids: &[String]) {
        self.prune_withdrawals();
        for id in ids {
            if !is_plain(id.as_str()) {
                continue;
            }
            if matches!(record_text(self.decision_path(id.as_str()).as_path()), Some(content) if content == LANDING) {
                let _removed = std::fs::remove_file(self.decision_path(id.as_str()));
            }
        }
    }

    /// Whether the request of this identity is in the group the landing thread is landing.
    pub fn is_in_flight(&self, id: &str) -> bool {
        return is_plain(id) && matches!(record_text(self.decision_path(id).as_path()), Some(content) if content == LANDING);
    }

    /// The write the landing thread is inside,
    /// as `(millis when it began, command, detail, millis it has run)`,
    /// when there is one.
    fn running(&self) -> Option<(u128, String, Option<String>, u128)> {
        let text: String = record_text(self.directory.join(STEP_FILE).as_path())?;
        let mut lines = text.lines();
        let began: u128 = lines.next()?.parse().ok()?;
        let command: String = String::from(lines.next()?);
        let detail: Option<String> = match lines.next() {
            Some(line) if !line.is_empty() => Some(String::from(line)),
            _ => None,
        };
        return Some((began, command, detail, now_millis().saturating_sub(began)));
    }

    /// The words that tell a client what the service is stuck in,
    /// when the landing thread has been inside a write that is never
    ///  ended for longer than the board's `slow_after`;
    /// nothing otherwise.
    pub fn describe(&self) -> Option<String> {
        let (_, command, found, running) = self.running()?;
        if running <= self.slow_after.as_millis() {
            return None;
        }
        let minutes: u128 = running / 60_000;
        let detail: String = match found {
            Some(text) => format!(" ({text})"),
            None => String::new(),
        };
        return Some(format!(
            "monochromatic-git-policy-cli: the commit service has been inside a write for {minutes} minutes and has not heard back: `{command}`{detail}. The service never ends a write in \
             the middle, because that could lose data, so commits queue behind it. If it never returns, look at what that process \
             waits for with `ps` and at the disk, and when no program uses the repository, restart the service with `{RESTART_COMMAND}`.\n"
        ));
    }

    /// What:
    ///  The notice for the client whose own request is in the group being landed while the landing thread is inside a write that is
    ///  never ended:
    /// a key that changes when the notice should be sent again,
    /// and the words.
    /// Why:
    ///   The client must learn that its commit may be on the branch when the write starts,
    /// not a minute later (owner,
    /// 2026-11-04,
    ///  "Short extra wait");
    /// the words that follow a minute's stuck write are the longer ones.
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
        let detail: String = match found {
            Some(text) => format!(" ({text})"),
            None => String::new(),
        };
        return Some((format!("{began}:start"), format!(
            "monochromatic-git-policy-cli: this commit is inside a write of the commit service, `{command}`{detail}, begun {} seconds ago. The service never ends a write \
             in the middle, so if it does not return this command reports that the result is not known after its wait.\n",
            running / 1000
        )));
    }

    /// What:
    ///  Answer a withdrawal on the connection thread when the landing thread is stuck in a write that is never ended and the
    ///  request is not the landing thread's.
    /// Why:
    ///   The landing thread answers withdrawals between two landings,
    /// so behind a stuck write none would be answered.
    /// A request
    ///  that has not begun cannot land while the thread is stuck,
    /// and the decision keeps it from beginning later.
    /// The journal is
    ///  asked before the decision is tried and again after it is won,
    /// because a request that landed and was let go of between the
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
                let confirmed: bool = match withdrawal_token(content.as_str()) {
                    Some(theirs) => self.cancelled_path(id, theirs).exists(),
                    None => false,
                };
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

    /// How many confirmed withdrawals the record holds;
    /// the ceiling is reached when the directory cannot be listed,
    /// so that a record that
    ///  cannot be counted is not grown.
    fn confirmed_withdrawals(&self) -> usize {
        let Ok(entries) = std::fs::read_dir(self.directory.as_path()) else { return usize::MAX };
        return entries
            .flatten()
            .filter(is_confirmation)
            .count();
    }

    /// What:
    ///  Forget the oldest confirmed withdrawals once more than `WITHDRAWALS_RETAINED` are remembered.
    /// Why:
    ///   Only the landing thread calls this (from `end_group`),
    /// so there is one pruner and no two work from the same listing.
    /// A
    ///  decision is removed only while it still carries the token of the confirmation being removed,
    /// and before that
    ///  confirmation:
    /// a decision that replaced it has another token and stays,
    /// and a confirmation left standing without its decision
    ///  (a pause between the two removals) confirms nothing of any later decision,
    /// which has a token of its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const [id, token] of oldestConfirmations(surplus)) { if (read(decision(id)) === `withdrawn:${token}`) rm(decision(id)); rm(confirmation(id, token)); }
    /// ```
    fn prune_withdrawals(&self) {
        let Ok(entries) = std::fs::read_dir(self.directory.as_path()) else { return };
        let mut remembered: Vec<(SystemTime, PathBuf)> = entries
            .flatten()
            .filter(is_confirmation)
            .filter_map(modified_and_path)
            .collect();
        if remembered.len() <= WITHDRAWALS_RETAINED {
            return;
        }
        remembered.sort();
        let surplus: usize = remembered.len() - WITHDRAWALS_RETAINED;
        #[cfg(test)]
        AFTER_THE_LISTING.with(|hook| { if let Some(pause) = hook.borrow().as_ref() { pause(); } });
        for (_, confirmation) in remembered.into_iter().take(surplus) {
            let Some(name) = confirmation.file_stem().and_then(std::ffi::OsStr::to_str) else { continue };
            let Some((id, token)) = name.split_once('.') else { continue };
            let decision: PathBuf = self.decision_path(id);
            let names_this_token: bool = match record_text(decision.as_path()) {
                Some(content) => withdrawal_token(content.as_str()) == Some(token),
                None => false,
            };
            if names_this_token {
                let _forgotten = std::fs::remove_file(decision);
            }
            let _also = std::fs::remove_file(confirmation);
        }
    }
}

#[cfg(test)]
thread_local! {
    /// A control may stop the retention on this thread between its listing and its removals,
    /// where another decision can replace one.
    pub(crate) static AFTER_THE_LISTING: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
    /// A control may stop a withdrawal on this thread between the moment it wins the decision and the moment it asks the journal again,
    ///  which is where the reviews found the window.
    pub(crate) static AFTER_THE_DECISION_IS_WON: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
}

/// The record's controls stay out of the release executable.
#[cfg(test)]
#[path = "service_busy_tests.rs"]
mod tests;
