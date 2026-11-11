//! What:
//!  The lines the resident commit service writes to its log (standard error,
//! which the service manager keeps in the journal)
//!  about its own life and about the time each request took:
//! when it started,
//! one line for each request it answered with the
//!  phases of the wait,
//! a retire request,
//! and its exit with the reason and the uptime.
//! Why:
//!  After a slow commit the journal held `ready` and nothing else,
//! so nothing said whether the time went to the queue,
//! to
//!  another program's hold on the index,
//! to the landing or to a service being replaced.
//! These lines cost one clock read per
//!  phase edge and one line per request;
//! they name no person (the commit's message and identities never appear) and are not
//!  shown to the person at the terminal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(`cli-git-service: answered request=${id} queued_ms=${queued} lock_wait_ms=${lockWait} total_ms=${total}`);
//! ```

use crate::service_durable_syncs::{SyncFigure, syncs_held, syncs_waited_count, syncs_word};
use crate::service_index_found::WaitNotes;
use crate::journald_native::{Delivery, MESSAGE_LIMIT, Priority, Sink, StreamWriter, cut, one_line};
use crate::lfs_output_mask::mask_url_secrets_text;
use crate::quiet_ledger::longest_healthy;
use crate::transaction_step_durations::{StepFigure, steps_word};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// The prefix of every line the service writes,
/// which `journalctl` shows after the unit name.
pub const LOG_PREFIX: &str = "monochromatic-git-policy-cli-service: ";

/// When a connection was accepted and how long its request line took to arrive.
#[derive(Clone, Copy, Debug)]
pub struct Arrival {
    /// The moment the connection was accepted.
    pub accepted: Instant,
    /// Time from the acceptance to the whole request line being read;
    /// a client that is slow to send shows up here and not as a
    ///  wait in the queue.
    pub read: Duration,
}

/// What:
///  The phases of one landing the service itself spent time in,
/// summed over the attempts a moving branch made it repeat.
/// Why:
///   The wait for another program's hold on the index,
/// the objects,
/// the journal write,
/// the branch move and the index update are
///  the stretches a slow landing can lose time in;
/// whatever else a landing did is reported as the reads that remain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandTiming = { lockWait: number; lockHolder?: string; objects: number; journal: number; refUpdate: number; indexUpdate: number; attempts: number };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandTiming {
    /// Time to take the index lock,
    /// which is the whole wait for a foreign holder.
    pub lock_wait: Duration,
    /// The holder the evidence named while the lock was waited for,
    /// when it named one.
    pub lock_holder: Option<String>,
    /// What the acquisition noted about its wait:
    /// what the first and the last look at the held lock found,
    /// how many looks there were,
    /// and how the time split between flushing the marker and waiting for the holder.
    pub lock_notes: WaitNotes,
    /// Time to make the commits:
    /// one import for the batch,
    /// or the checks of a commit Git built.
    pub objects: Duration,
    /// Time to write the intent to the journal.
    pub journal: Duration,
    /// Time of the compare-and-swap that moves the branch.
    pub ref_update: Duration,
    /// Time to remove the conclusion state and to build and install the new index.
    pub index_update: Duration,
    /// How long the index lock was held,
    /// from its placement to its release:
    /// the branch move and the index build.
    /// Not a phase of its own (it overlaps `ref_update` and `index_update`),
    /// so it is not part of what `reads` subtracts.
    pub hold: Duration,
    /// Passes over the batch:
    /// one,
    /// and one more each time the branch moved under it.
    pub attempts: usize,
    /// Which of the residual reads spent the time,
    /// named one by one;
    /// their sum is never more than `reads`.
    pub reads_split: ReadSplit,
    /// Every sync this landing made or handed to the background,
    /// in the order it asked for them.
    pub syncs: Vec<SyncFigure>,
}

impl LandTiming {
    /// What `landing` spent outside the named phases:
    /// the branch reads,
    /// the comparisons with the branch,
    /// the check for removed
    ///  content,
    /// and the rest of what `reads_split` names one by one.
    pub fn reads(&self, landing: Duration) -> Duration {
        return landing.saturating_sub(self.lock_wait + self.objects + self.journal + self.ref_update + self.index_update);
    }
}

/// What:
///  How the residual reads of a landing split,
///  one named stretch each.
/// Why:
///   `reads` was one number for the branch reads,
///  the comparisons with the branch,
///  the check for removed content,
///  the index witness,
///  the conclusion copy,
///  the records a batch owes and the head question under the lock;
///  a landing that spent a second there
///  could not say which one did.
///  Every named stretch is a part of what `reads` subtracts,
///  so the sum of them never exceeds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadSplit = { config: number; branch: number; parents: number; changed: number; content: number; witness: number; conclusion: number; owed: number; follow: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadSplit {
    /// Reading the configuration snapshot and deciding whether the history is plain,
    /// once for the batch,
    /// before it starts.
    pub config: Duration,
    /// Reading the branch's commit and asking whether the worktree still has it checked out,
    /// before the index lock is asked for.
    pub branch: Duration,
    /// Listing the parents of the commit an amend replaces.
    pub parents: Duration,
    /// Comparing the requests with the branch since their base,
    /// one listing per distinct base.
    pub changed: Duration,
    /// Looking for the content of the batch in the object store before anything is imported.
    pub content: Duration,
    /// Recording the index file the landing saw.
    pub witness: Duration,
    /// Keeping the copy of the conclusion state a cleanup removes.
    pub conclusion: Duration,
    /// Taking the index update and the held push a batch owes before its branch moves.
    pub owed: Duration,
    /// Reading the worktree's head again after the branch moved.
    pub follow: Duration,
}

impl ReadSplit {
    /// Every named read added together.
    pub fn total(&self) -> Duration {
        return self.config + self.branch + self.parents + self.changed + self.content + self.witness + self.conclusion + self.owed + self.follow;
    }
}

/// Everything the line for one answered request states.
pub struct Answered<'record> {
    /// The request's identity.
    pub request: &'record str,
    /// The reference the request names.
    pub reference: &'record str,
    /// What became of it:
    /// a word such as `landed`,
    /// `already`,
    /// `retry-prepare`,
    /// `branch-moved` or `failed:<code>`.
    pub outcome: &'record str,
    /// The batch number,
    /// shared by every request landed together.
    pub batch: u64,
    /// How many requests the batch of this branch held.
    pub size: usize,
    /// This request's place among them,
    /// from one.
    pub position: usize,
    /// How many other commits were still waiting for their answer when this landing finished,
    /// which is how many commits queued behind it.
    pub queued_behind: usize,
    /// How the connection and its request line arrived.
    pub arrival: Arrival,
    /// Time from the request being read to its batch starting to land.
    pub queued: Duration,
    /// Time to settle the batches a killed service left open,
    /// which runs before each batch.
    pub settle: Duration,
    /// The phases of the landing.
    pub landing: &'record LandTiming,
    /// Time the landing took,
    /// from its start to its return.
    pub land_total: Duration,
    /// Time from the landing's end to this request being answered.
    pub answer: Duration,
    /// Time from the acceptance of the connection to the answer.
    pub total: Duration,
    /// The longest quiet gap of every Git step and tool the service ran since the previous answered line,
    /// by step name,
    /// taken
    ///  from the ledger (`quiet_ledger.rs`).
    /// The landings of one service share the ledger,
    /// so the first answer of a batch holds
    ///  what the batch's steps showed and the others of that batch hold nothing.
    pub quiet: &'record [(String, Duration)],
    /// Every Git step the service started since the previous answered line,
    /// as program and subcommand with the offset from the
    ///  connection's acceptance and the duration (`transaction_step_durations.rs`).
    /// Taken like `quiet`:
    /// the first answer of a batch
    ///  holds the batch's steps.
    pub steps: &'record [StepFigure],
    /// How many steps came after the list reached its bound.
    pub steps_dropped: usize,
}

/// What:
///  One quiet gap as a word:
/// `name=milliseconds` with three decimals.
/// Why:
///  `quiet_steps` joins the words of all the gaps with semicolons.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quietStepWord = ([step, gap]: [string, number]) => `${step}=${(gap / 1000).toFixed(3)}`;
/// ```
fn quiet_step_word(figure: &(String, Duration)) -> String {
    let (step, gap) = figure;
    return format!("{step}={:.3}", gap.as_micros() as f64 / 1000.0);
}

/// The text of the quiet gaps in the answered line:
/// `name=milliseconds` joined by semicolons.
fn quiet_steps(figures: &[(String, Duration)]) -> String {
    let words: Vec<String> = figures.iter().map(quiet_step_word).collect();
    return words.join(";");
}

/// The word for a named holder,
/// in the quoted form.
fn holder_field(who: &String) -> String {
    return format!(" lock_holder={who:?}");
}

/// The words for what the wait for the lock found,
/// empty when the lock was free at the first placement:
/// the placements that found it held,
/// the time spent flushing the marker and the time spent waiting,
/// and the first and the last description of the held lock.
pub fn lock_notes_fields(notes: &WaitNotes) -> String {
    if notes.looks == 0 {
        return format!(" lock_stage_ms={} lock_stage_kind={} lock_place_ms={}", notes.staging.as_millis(), notes.stage_kind, notes.placing.as_millis());
    }
    let first: &str = notes.found.as_deref().unwrap_or("none");
    let last: &str = notes.last.as_deref().unwrap_or("none");
    return format!(
        " lock_stage_ms={} lock_stage_kind={} lock_place_ms={} lock_looks={} lock_found={first:?} lock_last={last:?}",
        notes.staging.as_millis(),
        notes.stage_kind,
        notes.placing.as_millis(),
        notes.looks
    );
}

/// The wait for the index lock that is written down even when no holder was met:
/// the making of the marker or a wait that long is a slow landing's first suspect.
///  Exempt from a citation under `LQM` (class:
/// notice pacing):
/// it times a line the person reads or a log threshold,
/// and
///  nothing waits on it beyond the wait that already exists.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const SLOW_LOCK_STEP: Duration = Duration::from_secs(1);

/// What:
///  The line for the end of a wait for the index lock,
/// or nothing when the lock was free at the first placement and the acquisition was quick.
/// Why:
///   Written when the wait ends,
/// not with the answer,
/// so a landing that fails afterwards still leaves what it waited for.
/// It says the total,
/// how it split between flushing the marker and waiting for a holder,
/// how many times the lock was found held,
/// and what the first and the last look found (`lock_found`,
/// `lock_last`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitLine(seq, total, notes, taken) // 'waited batch=7 result=taken lock_wait_ms=5629 ...' | undefined
/// ```
pub fn wait_line(seq: u64, total: Duration, notes: &WaitNotes, taken: bool) -> Option<String> {
    if notes.looks == 0 && notes.staging < SLOW_LOCK_STEP && total < SLOW_LOCK_STEP {
        return None;
    }
    return Some(format!(
        "waited for the index lock batch={seq} result={} lock_wait_ms={}{}",
        if taken { "taken" } else { "refused" },
        total.as_millis(),
        lock_notes_fields(notes)
    ));
}

/// The key under which `answered` lines written before the index step was named by its role state its duration.
const LEGACY_INDEX_UPDATE_KEY: &str = "sync_ms";

/// What:
///  The duration,
/// in whole milliseconds,
/// that an `answered` line gives for removing the conclusion state and building and
///  installing the new index.
/// Why:
///   The step was first logged as `sync_ms`,
/// which reads as a disk sync although it is the follow-up of the landing that holds
///  the index lock;
/// it is `index_update_ms` now.
/// Lines already in the journal keep the old key,
/// so a reader asks for the new
///  key and falls back to the old one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(/ index_update_ms=(\d+)/.exec(line)?.[1] ?? / sync_ms=(\d+)/.exec(line)?.[1])
/// ```
pub fn index_update_ms_of(line: &str) -> Option<u64> {
    if let Some(found) = digits_after_key(line, "index_update_ms") {
        return Some(found);
    }
    return digits_after_key(line, LEGACY_INDEX_UPDATE_KEY);
}

/// What:
///  The number that follows ` key=` in a log line,
/// nothing when the key is absent or no digits follow.
/// Why:
///  `index_update_ms_of` reads the current key first and the legacy key after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(new RegExp(` ${key}=(\\d+)`).exec(line)?.[1])
/// ```
fn digits_after_key(line: &str, key: &str) -> Option<u64> {
    let word: String = format!(" {key}=");
    let from: usize = line.find(word.as_str())? + word.len();
    let digits: String = line[from..].chars().take_while(char::is_ascii_digit).collect();
    return digits.parse::<u64>().ok();
}

/// What:
///  One answered request as a line of `key=value` words,
/// without the prefix.
/// Why:
///   Text values are written in Rust's quoted form,
/// which has no raw newline or unescaped quote,
/// so the line stays one line
///  whatever a client sent;
/// the numbers are whole milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `answered request=${JSON.stringify(id)} outcome=${outcome} queued_ms=${queued} ...`
/// ```
pub fn answered_line(answered: &Answered<'_>) -> String {
    let mut holder: String = answered.landing.lock_holder.as_ref().map_or_else(String::new, holder_field);
    holder.push_str(lock_notes_fields(&answered.landing.lock_notes).as_str());
    return format!(
        "answered request={:?} branch={:?} outcome={} batch={} size={} position={} queued_behind={} read_ms={} queued_ms={} settle_ms={} \
         lock_wait_ms={}{holder} lock_hold_ms={} reads_ms={} objects_ms={} journal_ms={} ref_ms={} index_update_ms={} attempts={} landing_ms={} \
         reads_config_ms={} reads_branch_ms={} reads_parents_ms={} reads_changed_ms={} reads_content_ms={} \
         reads_witness_ms={} reads_conclusion_ms={} reads_owed_ms={} reads_follow_ms={} \
         syncs_waited={} syncs_queued={} sync_held_ms={} syncs={:?} \
         answer_ms={} total_ms={} quiet_ms={} quiet_steps={:?} steps={:?} steps_dropped={}",
        answered.request,
        answered.reference,
        answered.outcome,
        answered.batch,
        answered.size,
        answered.position,
        answered.queued_behind,
        answered.arrival.read.as_millis(),
        answered.queued.as_millis(),
        answered.settle.as_millis(),
        answered.landing.lock_wait.as_millis(),
        answered.landing.hold.as_millis(),
        answered.landing.reads(answered.land_total).as_millis(),
        answered.landing.objects.as_millis(),
        answered.landing.journal.as_millis(),
        answered.landing.ref_update.as_millis(),
        answered.landing.index_update.as_millis(),
        answered.landing.attempts,
        answered.land_total.as_millis(),
        answered.landing.reads_split.config.as_millis(),
        answered.landing.reads_split.branch.as_millis(),
        answered.landing.reads_split.parents.as_millis(),
        answered.landing.reads_split.changed.as_millis(),
        answered.landing.reads_split.content.as_millis(),
        answered.landing.reads_split.witness.as_millis(),
        answered.landing.reads_split.conclusion.as_millis(),
        answered.landing.reads_split.owed.as_millis(),
        answered.landing.reads_split.follow.as_millis(),
        syncs_waited_count(answered.landing.syncs.as_slice()),
        answered.landing.syncs.len() - syncs_waited_count(answered.landing.syncs.as_slice()),
        syncs_held(answered.landing.syncs.as_slice()).as_millis(),
        syncs_word(answered.landing.syncs.as_slice()),
        answered.answer.as_millis(),
        answered.total.as_millis(),
        longest_healthy(answered.quiet),
        quiet_steps(answered.quiet),
        steps_word(answered.steps),
        answered.steps_dropped,
    );
}

/// The line a service writes as it starts,
/// before `ready`:
/// the release it announces,
/// the protocol,
/// its process and the worktree
///  instance it serves.
pub fn started_line(release: &str, protocol: u32, instance: Option<&str>) -> String {
    return format!("started release={release:?} protocol={protocol} pid={} instance={}", std::process::id(), instance.unwrap_or("any"));
}

/// The line for a retire message:
/// a client of another release found this service and asked it to finish its batch and exit.
pub fn retire_line(release: &str) -> String {
    return format!("retire requested by a client; this service is release {release:?} and exits after its batch");
}

/// Why a service ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
    /// Nothing to do for the whole idle period.
    Idle,
    /// A client asked it to retire.
    Retire,
    /// The queue its connections feed closed,
    /// which only happens when the acceptor thread died.
    Error,
    /// The worktree this service serves is gone,
    /// so it stopped its own socket instead of serving it.
    WorktreeGone,
}

impl ExitReason {
    /// The word written in the line.
    pub fn word(self) -> &'static str {
        return match self {
            ExitReason::Idle => "idle",
            ExitReason::Retire => "retire",
            ExitReason::Error => "error",
            ExitReason::WorktreeGone => "worktree-gone",
        };
    }
}

/// The line for the service's own exit,
/// with the time it ran since it began serving.
pub fn exit_line(reason: ExitReason, uptime: Duration) -> String {
    return format!("exiting reason={} uptime_ms={}", reason.word(), uptime.as_millis());
}

/// The wall clock in whole milliseconds since the Unix epoch,
/// the time a log line states for its own event.
pub fn wall_millis() -> u128 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_before_the_epoch) => 0,
    };
}

/// What:
///  The text of one log line:
/// the prefix,
/// the line,
/// the time of the event as `at_ms=<epoch milliseconds>` at the end,
/// and the newline.
/// Why:
///   The journal stamps a line when it receives it,
/// and under load that can be seconds after the event;
/// the line states its own time,
/// at the end so that the prefix and the first word stay where readers look for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${LOG_PREFIX}${line} at_ms=${Date.now()}\n`
/// ```
pub fn stamped_line(line: &str, at_ms: u128) -> String {
    return format!("{LOG_PREFIX}{line} at_ms={at_ms}\n");
}

/// What:
///  Writes one line to the service's log.
/// Why:
///   Standard error is the service manager's journal;
/// one `write` carries the prefix,
/// the line,
/// its own time and the newline together,
/// and a
///  closed or broken stream is not the service's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(LOG_PREFIX + line + ` at_ms=${Date.now()}`)
/// ```
pub fn log(line: &str) {
    log_entry(Priority::Info, line, &[]);
}

/// What:
///  Whether this process is the resident service,
/// which names its release once at the start (`set_context`).
/// Why:
///   Code both programs run (the auto-push's fetch,
/// the shadow's `init`) records Git's words in the service's log in the
///  service and as a client entry in the wrapper;
/// a control that captures the service's entries counts as the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// contextWasSet
/// ```
pub fn in_service() -> bool {
    #[cfg(test)]
    if capture::active() {
        return true;
    }
    return CONTEXT.get().is_some();
}

/// What the service says of itself in every entry:
/// its release and,
/// when it serves one worktree,
/// its instance.
static CONTEXT: OnceLock<Vec<(&'static str, String)>> = OnceLock::new();

/// Where entries go,
/// decided at the first one.
static SINK: OnceLock<Sink> = OnceLock::new();

/// The sink of this process.
pub fn sink() -> &'static Sink {
    return SINK.get_or_init(Sink::from_environment);
}

/// Names the release and instance every later entry carries as `CLI_GIT_RELEASE` and `CLI_GIT_INSTANCE`;
/// the first call wins.
pub fn set_context(release: &str, instance: Option<&str>) {
    let mut fields: Vec<(&'static str, String)> = vec![("CLI_GIT_RELEASE", release.to_string())];
    if let Some(name) = instance {
        fields.push(("CLI_GIT_INSTANCE", name.to_string()));
    }
    let _first = CONTEXT.set(fields);
}

/// What:
///  Writes one entry of a priority,
/// with fields a reader can filter the journal on,
/// and gives where it went.
/// Why:
///   The userinfo of every URL is masked (a push or a review reports what Git and a server said),
/// the line on standard error
///  is one line,
///  then it states its own time (`stamped_line`);
/// a native entry also carries the time as `CLI_GIT_AT_MS`,
/// the priority
///  and the context.
/// Nothing here waits:
/// see `journald_native`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// logEntry('warning', 'cannot listen', [['CLI_GIT_REQUEST_ID', id]])
/// ```
pub fn log_entry(priority: Priority, line: &str, fields: &[(&str, String)]) -> Delivery {
    let at_ms: u128 = wall_millis();
    // Masked before it is cut: a cut that falls just before the `@` of `scheme://user:secret@host` would leave a password
    //  that no longer looks like userinfo.
    let masked: String = mask_url_secrets_text(line);
    let mut all: Vec<(&str, String)> = Vec::new();
    if let Some(context) = CONTEXT.get() {
        for (name, value) in context.iter() {
            all.push((*name, value.clone()));
        }
    }
    all.push(("CLI_GIT_AT_MS", at_ms.to_string()));
    for (name, value) in fields {
        all.push((*name, mask_url_secrets_text(value.as_str())));
    }
    // A native entry carries the text as written (a newline in the length-prefixed form); the line on standard error is one line.
    let message: String = format!("{} at_ms={at_ms}", cut(masked.as_str(), MESSAGE_LIMIT));
    #[cfg(test)]
    if capture::record(priority, message.as_str(), all.as_slice()) {
        return Delivery::Stderr;
    }
    let text: String = stamped_line(one_line(masked.as_str(), MESSAGE_LIMIT).as_str(), at_ms);
    // On the journal's stream a line that finds no room is dropped, never waited for.
    if sink().on_stream() {
        return sink().deliver(priority, message.as_str(), all.as_slice(), text.as_str(), &mut StreamWriter::stderr());
    }
    return sink().deliver(priority, message.as_str(), all.as_slice(), text.as_str(), &mut std::io::stderr());
}

/// What:
///  Writes one formatted line at the `warning` priority,
/// with its time.
/// Why:
///   Something did not go as intended and the service went on;
/// `journalctl --priority=warning` shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceWarn(`the review of ${oid} was skipped: ${reason}`)
/// ```
#[macro_export]
macro_rules! service_warn {
    ($($argument:tt)*) => {
        {
            let _delivered = $crate::service_timing::log_entry($crate::journald_native::Priority::Warning, format!($($argument)*).as_str(), &[]);
        }
    };
}

/// What:
///  Writes one formatted line at the `err` priority,
/// with its time.
/// Why:
///   Something the service needed failed;
/// `journalctl --priority=err` shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceError(`cannot listen: ${error}`)
/// ```
#[macro_export]
macro_rules! service_error {
    ($($argument:tt)*) => {
        {
            let _delivered = $crate::service_timing::log_entry($crate::journald_native::Priority::Error, format!($($argument)*).as_str(), &[]);
        }
    };
}

/// What:
///  Writes one formatted line to the service's log,
/// with its time.
/// Why:
///   Every line the service writes states its own time (`stamped_line`);
/// the macro is the one way to write a line with arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceLog(`cannot take the index lock ${path}: ${error}`)
/// ```
#[macro_export]
macro_rules! service_log {
    ($($argument:tt)*) => {
        $crate::service_timing::log(format!($($argument)*).as_str())
    };
}

/// What a control sees of the entries the code under test writes,
/// on the thread that runs it.
#[cfg(test)]
#[path = "service_timing_capture_test_support.rs"]
pub mod capture;

/// The lines,
/// their fields,
/// and the escaping.
#[cfg(test)]
#[path = "service_timing_tests.rs"]
mod tests;
