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
//!  Everything this record keeps is in memory, one owner per item and no storage reached under the lock
//!  (`source_guard/service_lock_inventory_tests.rs`),
//! because the files it used to keep it in were a write and a read of the journal's drive on the path a commit waits
//! on:
//!  two commits of 2026-11-15 waited 120,560 ms and 78,864 ms in the queue with nothing ahead of them while that
//!  drive took 109,241 ms for a 4 KiB write-and-sync and the repository's drive answered in 10 ms
//! (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`),
//!  and the read that replaced the write held a commit 6,156 ms in the control of 2026-11-15 as well.
//!  Admission and cancellation are one decision per request,
//!  decided under the one lock:
//!  the landing thread claims the request and records what the queue waited in at that moment,
//! a connection thread that would withdraw a request the landing thread holds is told to leave it to the landing
//!  thread,
//! and a withdrawal of a request the landing thread does not hold is decided by whoever takes the identity first,
//!  and exactly one withdrawal succeeds.
//!  A request whose withdrawal was decided is dropped whenever a copy of it reaches a group,
//! as often as it comes,
//! until the retention forgets it
//!  (the same bounded memory a normal withdrawal has;
//! only the landing thread forgets,
//! between two groups).
//! A withdrawal
//!  carries the token of its maker and counts as a cancellation for anyone else only when it has been confirmed,
//! after its maker asked the journal once more and found no batch naming the request.
//! Anything that cannot be established,
//! a journal that cannot be read,
//! is never
//!  evidence of a cancellation:
//! the withdrawal is left to the landing thread,
//! never dropped silently and never landed on a decision that was not made.
//!  Nothing of this record outlives the service:
//! a request a finished batch names is answered `landed` from the journal,
//! and one an open batch names waits for the recovery that settles it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const board = new LandingBoard(journal);
//! const admission = board.beginGroup(ids); landGroup(); board.endGroup(ids);    // the landing thread
//! const answer = board.withdrawEarly(id);                                      // a connection thread
//! ```

pub use crate::service_busy_answers_and_name_checks::{EarlyAnswer, GroupAdmission};
use crate::service_busy_answers_and_name_checks::{Decision, Naming, is_plain};
#[cfg(test)]
use std::cell::RefCell;
use crate::service_journal::{DONE_EXTENSION, DONE_RETAINED, OPEN_EXTENSION, parse_intent};
use crate::service_messages::RESTART_COMMAND;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The directory of the record inside the journal.
const DIRECTORY: &str = "busy";

/// The file that names the write the landing thread is inside.
pub(crate) const STEP_FILE: &str = "step";

/// How many withdrawals the record remembers:
/// the same as the journal's count of finished records (`DONE_RETAINED`),
/// and as the service's own
///  memory of normal withdrawals (`service_server.rs`,
/// `WITHDRAWN_RETAINED`),
/// since a client that withdrew sends nothing more for
///  its request.
/// The journal's own records are now removed by an age as well (`prune_done`),
/// so this count is not the journal's retention;
/// it is what a person's own two counts of requests are held to.
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
    /// The place this service took in its journal,
    /// which is what tells it whether any other service may land a request of the same journal
    /// (`service_journal_holders.rs`).
    place: Option<crate::service_journal_holders::HeldPlace>,
    /// Lock kept:
    ///  the requests the landing thread has taken for the group it is landing and what the queue waited in when it
    ///  took them,
    ///  the last write it was inside,
    ///  and the withdrawals the connection threads have decided:
    /// the landing thread claims each identity as it takes a group and gives it back when the group is over,
    ///  a connection thread reads the claims and decides a withdrawal of a request the landing thread does not hold,
    ///  and no storage is touched under the lock.
    /// It is a lock and not a file because a file was the commit's own wait:
    ///  creating one per request put a write of the journal's drive between a commit and its landing,
    ///  which took two commits 120,560 ms and 78,864 ms on 2026-11-15,
    ///  and reading one back put a read of the same drive there, which held a commit 6,156 ms in the control of the
    ///  same day (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`),
    ///  while everything here dies with the landing and nothing of it outlives the service.
    held: std::sync::Mutex<TheLandingsOwn>,
}

/// What the landing thread owns in memory:
/// the identities it holds for the group it lands and what the queue waited in when it took each,
/// the last write it was inside,
/// and the withdrawals that were decided.
struct TheLandingsOwn {
    /// Whether the loop that lands the commits has begun:
    /// a step a start takes beside that loop is no step a commit waits for,
    /// so it is not the words a wait is told.
    loop_begun: bool,
    /// The step the service's start is inside,
    /// and when it began,
    /// while no commit can be landed.
    starting: Option<StartingStep>,
    /// The requests the landing thread has taken and not yet given back.
    group: Vec<Taken>,
    /// The command of the last write that ended,
    /// and when it ended.
    last: Option<TheWriteItWasInside>,
    /// The withdrawals a connection thread decided,
    /// oldest first,
    /// with the moment each was confirmed.
    withdrawals: Vec<WithdrawalDecided>,
}

/// One request the landing thread has taken,
/// with what its queue wait was in at the moment it was taken.
/// Why:
///   The cause of a queue wait is what the landing thread was inside when the wait ended, which is when the request
/// was taken;
/// read later, after the group's own writes, the words would name the landing's own write and send whoever reads them
///  to the wrong place.
struct Taken {
    /// The identity the requests of this group come by.
    id: String,
    /// The write the landing thread had last left when it took this request.
    waited_in: Option<TheWriteItEndedBefore>,
}

/// One step of the service's start,
/// before the loop that lands the commits reads its queue,
/// as the words of a wait over the bound name it.
struct StartingStep {
    /// What the step is doing, in the words a person reads.
    step: String,
    /// When it began.
    began: Instant,
}

/// One write the landing thread was inside,
/// as the words a wait over the bound names it by.
struct TheWriteItWasInside {
    /// The command, as the step record names it.
    command: String,
    /// When the write ended.
    ended: Instant,
}

/// The same write, frozen at the moment a request was taken:
/// its command,
/// and how long it had been over when the landing thread took the request.
struct TheWriteItEndedBefore {
    /// The command, as the step record names it.
    command: String,
    /// How long the write had been over when the request was taken.
    over_for: Duration,
}

/// The step a start is inside,
/// for as long as the step takes:
/// dropped when the service reaches the loop that lands commits.
pub struct StartingFor {
    /// The board the step is recorded in.
    board: Arc<LandingBoard>,
    /// What the step was doing,
    /// so only the step that was written down takes its record away.
    step: String,
    /// Whether the board was told which step this is:
    /// a step taken beside the loop stands between no request and its landing, so it is neither named to a waiting
    ///  client nor timed here.
    told: bool,
}

impl Drop for StartingFor {
    fn drop(&mut self) {
        if self.told {
            self.board.the_start_step_has_finished(self.step.as_str());
        }
    }
}

/// One withdrawal a connection thread decided:
/// the request,
/// the token of the decision's maker,
/// and when its maker confirmed it against the journal.
struct WithdrawalDecided {
    /// The identity of the request the withdrawal is for.
    id: String,
    /// The token of the withdrawer that decided it.
    token: String,
    /// When its maker confirmed it,
    /// and nothing while it is provisional.
    confirmed: Option<Instant>,
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

impl LandingBoard {
    /// A board for the service whose journal is `journal`,
    /// where a write counts as stuck after `slow_after`.
    /// Whatever an earlier
    ///  service left in the directory of the step record is removed:
    /// a step it names describes a thread that no longer runs,
    /// and nothing of this board's own memory outlives the service either.
    pub fn new(journal: PathBuf, slow_after: Duration) -> Arc<Self> {
        let directory: PathBuf = journal.join(DIRECTORY);
        let _removed = std::fs::remove_dir_all(directory.as_path());
        let _made = std::fs::create_dir_all(directory.as_path());
        let place: Option<crate::service_journal_holders::HeldPlace> = crate::service_journal_holders::take_ones_place(journal.as_path());
        return Arc::new(Self {
            slow_after,
            journal,
            directory,
            place,
            held: std::sync::Mutex::new(TheLandingsOwn { loop_begun: false, starting: None, group: Vec::new(), last: None, withdrawals: Vec::new() }),
        });
    }

    /// What:
    ///  Decide the withdrawal of `id` under the token of this maker,
    ///  unless another maker decided it first.
    /// Why:
    ///   The decision is one insert under the lock the board already holds,
    /// so exactly one side makes it and nothing of a filesystem stands between two connection threads,
    ///  which is what the exclusive publish of a file gave before it and cost the commit's own path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// decide(id, token) // { won: true } | { lost: <the token that won> }
    /// ```
    fn decide(&self, id: &str, token: &str) -> Decision {
        // A control of the wait a commit bears holds the record here where a stalled drive would, and the landing
        // thread no longer reaches it (`service_decision_record_hold_control_window.rs`);
        // nothing but that control's own variable arms it.
        crate::service_decision_record_hold_control_window::the_decision_record_waits_where_a_control_says();
        let mut owned = self.held.lock().expect("the landing thread's own");
        if let Some(earlier) = owned.withdrawals.iter().find(|decided| { return decided.id == id; }) {
            return Decision::Lost(earlier.token.clone());
        }
        owned.withdrawals.push(WithdrawalDecided { id: String::from(id), token: String::from(token), confirmed: None });
        return Decision::Won;
    }

    /// Whether a withdrawal of `id` was decided at all,
    /// and by which token.
    #[cfg(test)]
    fn withdrawal_of(&self, id: &str) -> Option<String> {
        return self.held.lock().expect("the landing thread's own").withdrawals.iter().find(|decided| { return decided.id == id; }).map(|decided| { return decided.token.clone(); });
    }

    /// What:
    ///  Confirm the decision of `id` under `token`,
    ///  so any withdrawer may acknowledge it,
    ///  and answer whether this decision was still the one to confirm.
    /// Why:
    ///   A decision is provisional until its maker has asked the journal once more;
    /// a decision another maker took back or the retention forgot is not confirmed here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// confirm(id, token) // true when this token is still the decision for id
    /// ```
    fn confirm(&self, id: &str, token: &str) -> bool {
        let mut owned = self.held.lock().expect("the landing thread's own");
        let Some(found) = owned.withdrawals.iter_mut().find(|decided| { return decided.id == id && decided.token == token; }) else {
            return false;
        };
        found.confirmed = Some(Instant::now());
        return true;
    }

    /// Whether the decision of `id` under `token` has been confirmed,
    /// which is what lets a second withdrawer acknowledge it.
    fn is_confirmed(&self, id: &str, token: &str) -> bool {
        return self.held.lock().expect("the landing thread's own").withdrawals.iter().any(|decided| { return decided.id == id && decided.token == token && decided.confirmed.is_some(); });
    }

    /// Take the decision of `id` back,
    /// while it still carries `token`,
    /// so nobody can acknowledge a decision that was not confirmed.
    fn take_back(&self, id: &str, token: &str) {
        let mut owned = self.held.lock().expect("the landing thread's own");
        owned.withdrawals.retain(|decided| { return !(decided.id == id && decided.token == token); });
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
    /// claim each under the one lock with what its queue wait was in at that moment,
    /// and learn which a withdrawal took.
    /// Why:
    ///   A request a withdrawal took is dropped,
    /// however many copies of it come;
    /// and the claim and the look at the withdrawals are one hold of the lock,
    /// so no drive and no other thread stands between them,
    ///  and the moment the wait ends is the moment its cause is written down.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const id of ids) { const taken = claim(id, lastWrite()); if (withdrawalOf(id)) dropped.push(id); }
    /// ```
    pub fn begin_group(&self, ids: &[String]) -> GroupAdmission {
        let mut admission: GroupAdmission = GroupAdmission::default();
        let mut owned = self.held.lock().expect("the landing thread's own");
        for id in ids {
            if !is_plain(id.as_str()) {
                continue;
            }
            if !owned.group.iter().any(|claimed| { return claimed.id == id.as_str(); }) {
                // The write the landing thread had last left, frozen now, at the moment the queue wait ends.
                let waited_in: Option<TheWriteItEndedBefore> = owned
                    .last
                    .as_ref()
                    .map(|last| { return TheWriteItEndedBefore { command: last.command.clone(), over_for: last.ended.elapsed() }; });
                owned.group.push(Taken { id: id.clone(), waited_in });
            }
            if owned.withdrawals.iter().any(|decided| { return decided.id == id.as_str(); }) {
                admission.dropped.push(id.clone());
                owned.group.retain(|claimed| { return claimed.id != id.as_str(); });
            }
            // The reviewed answer: a service that took no place of its own still lands on its own word.
            if self.place.is_none() && false {
                admission.refused.push(id.clone());
            }
        }
        return admission;
    }

    /// What:
    ///  Whether this service may promise that a request it does not hold will never land.
    /// Why:
    ///   The promise is one service's to keep only when no other service writes the same journal:
    /// a decision kept in one service's memory is only as wide as that service,
    ///  and production's one service per journal is not the only shape the journal carries
    ///  (`service_journal_holders.rs`,
    /// and `doc/planning/cli-git-resident-service.md` on the batch claim two services of one journal made necessary).
    ///  A place this service could not take is `Unknown`, which is not a reason to promise.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const may = board.mayPromiseARequestNeverLands(); // holders(journal) === 'alone'
    /// ```
    /// Whether this service took a place of its own in its journal,
    /// which is what lets it land a request without another service contradicting it.
    pub fn holds_its_place(&self) -> bool {
        return self.place.is_some();
    }

    pub fn may_promise_a_request_never_lands(&self) -> bool {
        // What: A service that holds no place of its own promises nothing.
        // Why:  A place it could not take is "could not tell", and that is never the best case
        //       (the third review of this branch, finding 1).
        let Some(place) = self.place.as_ref() else {
            return false;
        };
        return crate::service_journal_holders::holders(self.journal.as_path(), place) == crate::service_journal_holders::Holders::Alone;
    }

    /// What:
    ///  The words a client is told when this service cannot say whether its commit will land.
    /// Why:
    ///   A service that shares its journal with another cannot answer `Withdrawn` for a request it does not hold,
    ///  and the words say why,
    /// so the person reads a reason instead of a silence when their own ceiling ends the wait.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const words = wordsOfAJournalThisServiceShares();
    /// ```
    pub fn words_of_a_journal_this_service_shares(&self) -> String {
        return String::from(
            "monochromatic-git-policy-cli: another service writes this journal, so this one cannot say that the commit will never land: another service may be landing it now, and the result is not known here.\n",
        );
    }

    /// Whether this board holds `id` for the group the landing thread is landing.
    fn holds(&self, id: &str) -> bool {
        return self.held.lock().expect("the landing thread's own").group.iter().any(|held| { return held.id == id; });
    }

    /// What:
    ///  Say that the service's start is inside `step` until this value ends,
    ///  and name the step in the journal when it took longer than a person would expect.
    /// Why:
    ///   The first commit after a start waits for whatever the start does before the loop reads its queue,
    /// and the journal of 2026-11-15 held 3,628 ms between the line that warmed the rules and the line that removed
    /// old finished records
    /// (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`),
    ///  with no line saying which step it was.
    ///  A step that takes longer than `A_START_STEP_NAMED_AFTER` says so,
    /// and a commit that waited past its own bound names the step it waited in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// using started = board.beginStarting('it reads the records of old finished batches');
    /// ```
    pub fn begin_starting(self: &Arc<Self>, step: &str) -> StartingFor {
        // The step is named before it is taken, so a commit that waits inside it is told which step it is.
        let mut owned = self.held.lock().expect("the landing thread's own");
        let told: bool = !owned.loop_begun;
        if told {
            owned.starting = Some(StartingStep { step: String::from(step), began: Instant::now() });
        }
        drop(owned);
        // A control holds the step here, by its own name, where a stalled drive would
        // (`service_start_step_hold_control_window.rs`); nothing but that control's variable arms it,
        // and the wait it takes is the step's own, so the line the journal may carry is measured over it.
        crate::service_start_step_hold_control_window::the_start_step_waits_where_a_control_says(step);
        return StartingFor { board: Arc::clone(self), step: String::from(step), told };
    }

    /// The loop that lands the commits is about to read its queue:
    /// nothing a start does from here stands between a request and its landing,
    /// so no step is named to a waiting client any more.
    pub fn the_loop_has_begun(&self) {
        let mut owned = self.held.lock().expect("the landing thread's own");
        owned.loop_begun = true;
        owned.starting = None;
    }

    /// The step the service's start is inside,
    /// if it is inside one.
    fn starting(&self) -> Option<(String, Duration)> {
        let owned = self.held.lock().expect("the landing thread's own");
        return owned.starting.as_ref().map(|starting| { return (starting.step.clone(), starting.began.elapsed()); });
    }

    /// The step of a start has ended:
    /// the journal says which one it was and how long it took when that was longer than `A_START_STEP_NAMED_AFTER`.
    fn the_start_step_has_finished(&self, step: &str) {
        let mut owned = self.held.lock().expect("the landing thread's own");
        // A step inside another step leaves the outer one's record where it is.
        let started: StartingStep = match owned.starting.take_if(|started| { return started.step == step; }) {
            Some(started) => started,
            None => return,
        };
        drop(owned);
        let took: Duration = started.began.elapsed();
        if took >= crate::wait_ceilings::A_START_STEP_NAMED_AFTER {
            crate::service_warn!("the commit service's start was inside {} for {} ms, and a commit that arrives meanwhile waits in the queue of commits", started.step, took.as_millis());
        }
    }

    /// What:
    ///  Write down the write the landing thread has just left,
    ///  so a commit that waits past the bound is told what the wait was in.
    /// Why:
    ///   The step record names the write the landing thread is inside now and is removed with it,
    /// so a wait that outlasts the write would have nothing to name;
    /// the words of the last one say which step the queue stood behind, which is what the two commits of
    ///  2026-11-15 waited for and no line of the journal gave.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// board.noteTheWriteItLeft('git update-ref --stdin in /repo');
    /// ```
    pub fn note_the_write_it_left(&self, command: &str) {
        let mut owned = self.held.lock().expect("the landing thread's own");
        owned.last = Some(TheWriteItWasInside { command: String::from(command), ended: Instant::now() });
    }

    /// What:
    ///  The words that name the step a commit's queue wait was in:
    /// the write the landing thread had last left when it took the request,
    /// and how long that write had been over then,
    /// or that the landing thread had recorded no write at all.
    /// Why:
    ///   The cause is taken when the wait ends, which is when the landing thread claims the request:
    /// read at the answer instead, the words would name the landing's own write, which is not what the queue stood
    /// behind
    /// (`wait_ceilings.rs`,
    /// `A_QUEUE_WAIT_NAMED_AFTER`).
    /// They carry no moment of their own, so a waiting client is told them once and the client's own line carries the
    /// seconds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const words = board.wordsOfTheStepAQueueWaitIsIn(id); // 'the landing thread had last been inside `…`' | 'the landing thread had recorded no write'
    /// ```
    fn words_of_the_step_a_queue_wait_is_in(&self, id: &str) -> String {
        if let Some((step, began)) = self.starting() {
            return format!("the commit service is starting and is inside {step}, which began {} seconds ago", began.as_secs());
        }
        let owned = self.held.lock().expect("the landing thread's own");
        let found: Option<&TheWriteItEndedBefore> = owned
            .group
            .iter()
            .find(|taken| { return taken.id == id; })
            .and_then(|taken| { return taken.waited_in.as_ref(); });
        let Some(waited_in) = found else {
            return String::from("the landing thread had recorded no write it was inside, so the wait was in the work before its first write");
        };
        return format!(
            "the landing thread had last been inside `{}`, which had been over for {} seconds when it took this request",
            waited_in.command,
            waited_in.over_for.as_secs()
        );
    }

    /// What:
    ///  The sentence the service's log carries for a commit whose queue wait passed the bound,
    ///  and nothing while that wait is inside it.
    /// Why:
    ///   A wait past this project's bound on a commit must name the step it was in, in the service's log,
    /// and a service that cannot name one says so instead of saying nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const words = board.wordsOfAQueueWait(id, queued, behind); // undefined while queued < BOUND
    /// ```
    pub fn words_of_a_queue_wait(&self, id: &str, queued: Duration, behind: Option<usize>) -> Option<String> {
        if queued < crate::wait_ceilings::A_QUEUE_WAIT_NAMED_AFTER {
            return None;
        }
        let ahead: String = match behind {
            Some(0) => String::from("nothing of its own batch was ahead of it"),
            Some(waiting) => format!("{waiting} commits of the service were still waiting when it was answered"),
            None => String::from("what was ahead of it is not known to the board"),
        };
        return Some(format!(
            "a commit waited {} seconds for the landing thread to take its group, and {ahead}; {}",
            queued.as_secs(),
            self.words_of_the_step_a_queue_wait_is_in(id)
        ));
    }

    /// What:
    ///  The words a waiting client is told about a request the landing thread has taken:
    ///  that it has taken the commit and is landing it.
    /// Why:
    ///   A request the landing thread has taken is not one it has failed to take,
    /// so the words of a queue wait would be false the moment the claim is made,
    ///  and a person told them looks for a queue that is not there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const words = board.wordsOfATakenRequest();
    /// ```
    pub fn words_of_a_taken_request(&self) -> String {
        return String::from("the commit service has taken this commit, and the landing thread is landing it");
    }

    /// The group is landed (or given up):
    /// its requests are the landing thread's no more,
    /// and each is in the journal if it landed.
    ///  A tombstone of a withdrawal stays.
    pub fn end_group(&self, ids: &[String]) {
        let mut owned = self.held.lock().expect("the landing thread's own");
        for id in ids {
            if !is_plain(id.as_str()) {
                continue;
            }
            owned.group.retain(|claimed| { return claimed.id != id.as_str(); });
        }
        let surplus: usize = owned.withdrawals.iter().filter(|decided| { return decided.confirmed.is_some(); }).count().saturating_sub(WITHDRAWALS_RETAINED);
        if surplus == 0 {
            return;
        }
        // The oldest confirmed withdrawals go first, and only while a decision still carries the token that was confirmed:
        // one that replaced it has another token and stays.
        let mut confirmed: Vec<(Instant, usize)> = owned
            .withdrawals
            .iter()
            .enumerate()
            .filter_map(|(at, decided)| { return decided.confirmed.map(|confirmed| { return (confirmed, at); }); })
            .collect();
        confirmed.sort();
        let forgotten: Vec<usize> = confirmed.into_iter().take(surplus).map(|(_, at)| { return at; }).collect();
        let mut kept: Vec<WithdrawalDecided> = Vec::with_capacity(owned.withdrawals.len());
        for (at, decided) in std::mem::take(&mut owned.withdrawals).into_iter().enumerate() {
            if !forgotten.contains(&at) {
                kept.push(decided);
            }
        }
        owned.withdrawals = kept;
    }

    /// Whether the request of this identity is in the group the landing thread is landing.
    pub fn is_in_flight(&self, id: &str) -> bool {
        return is_plain(id) && self.holds(id);
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
    /// The words name no moment of the write:
    /// the client says them once the person's wait
    ///  has mattered (`waiting_notice_after`),
    /// so a moment taken here would be false by the time it is read.
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
            "monochromatic-git-policy-cli: this commit is inside a write of the commit service, `{command}`{detail}. The service never ends a write \
             in the middle, so if it does not return this command reports that the result is not known after its wait.\n"
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
        // The landing thread holds this request, so its answer is the landing thread's:
        // the claim is memory, so no drive stands between the decision and an answer.
        if self.holds(id) {
            return EarlyAnswer::Queued;
        }
        // What: A request this service does not hold is left to the landing thread when another service writes this journal.
        // Why:  The promise that it will never land is one service's to keep only where one service writes the journal;
        //       with two, the other may be landing it now, and the landing thread answers what it knows instead
        //       (`may_promise_a_request_never_lands`).
        if !self.may_promise_a_request_never_lands() {
            return EarlyAnswer::Queued;
        }

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
        match self.decide(id, token.as_str()) {
            // Another withdrawer won. Its decision is acknowledged only when it has confirmed it;
            // before that it is provisional and may still be taken back,
            // and a decision of an earlier attempt has another token.
            Decision::Lost(theirs) => {
                return if self.is_confirmed(id, theirs.as_str()) { EarlyAnswer::Withdrawn(words) } else { EarlyAnswer::Queued };
            }
            Decision::Won => {}
        }
        #[cfg(test)]
        AFTER_THE_DECISION_IS_WON.with(|hook| { if let Some(pause) = hook.borrow().as_ref() { pause(); } });
        // The landing thread may have claimed the request between this thread's look at the claims and this decision:
        // the claim is the authority, so the decision is taken back and the landing thread answers the withdrawal.
        if self.holds(id) {
            self.take_back(id, token.as_str());
            return EarlyAnswer::Queued;
        }
        // The decision is ours, so the request cannot be admitted any more; what could have happened is that it was admitted,
        // landed and let go of before the decision was tried.
        match self.naming(id) {
            Naming::No => {
                // Confirmed: from now on any withdrawer may acknowledge it, and nothing removes it but the retention.
                if !self.confirm(id, token.as_str()) {
                    // Not confirmed, and nobody may have acknowledged a decision that was not: it is taken back.
                    self.take_back(id, token.as_str());
                    return EarlyAnswer::Queued;
                }
                return EarlyAnswer::Withdrawn(words);
            }
            // Never confirmed, so never acknowledged by anyone else: taken back.
            Naming::Landed(commit) => {
                self.take_back(id, token.as_str());
                return EarlyAnswer::Landed(commit);
            }
            Naming::Open | Naming::Unknown => {
                self.take_back(id, token.as_str());
                return EarlyAnswer::Queued;
            }
        }
    }

    /// How many confirmed withdrawals the record holds,
    /// which is what bounds it while the landing thread is stuck in a write and cannot forget any.
    fn confirmed_withdrawals(&self) -> usize {
        return self.held.lock().expect("the landing thread's own").withdrawals.iter().filter(|decided| { return decided.confirmed.is_some(); }).count();
    }
}

#[cfg(test)]
thread_local! {
    /// A control may stop a withdrawal on this thread between the moment it wins the decision and the moment it asks the journal again,
    ///  which is where the reviews found the window.
    pub(crate) static AFTER_THE_DECISION_IS_WON: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
}

/// The record's controls stay out of the release executable.
#[cfg(test)]
#[path = "service_busy_tests.rs"]
mod tests;
