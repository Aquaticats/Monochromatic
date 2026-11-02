//! What:
//!  The lines the resident commit service writes to its log (standard error, which the service manager keeps in the journal)
//!  about its own life and about the time each request took: when it started, one line for each request it answered with the
//!  phases of the wait, a retire request, and its exit with the reason and the uptime.
//! Why:
//!  After a slow commit the journal held `ready` and nothing else, so nothing said whether the time went to the queue, to
//!  another program's hold on the index, to the landing or to a service being replaced. These lines cost one clock read per
//!  phase edge and one line per request; they name no person (the commit's message and identities never appear) and are not
//!  shown to the person at the terminal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(`cli-git-service: answered request=${id} queued_ms=${queued} lock_wait_ms=${lockWait} total_ms=${total}`);
//! ```

use std::io::Write;
use std::time::{Duration, Instant};

/// The prefix of every line the service writes, which `journalctl` shows after the unit name.
pub const LOG_PREFIX: &str = "cli-git-service: ";

/// When a connection was accepted and how long its request line took to arrive.
#[derive(Clone, Copy, Debug)]
pub struct Arrival {
    /// The moment the connection was accepted.
    pub accepted: Instant,
    /// Time from the acceptance to the whole request line being read; a client that is slow to send shows up here and not as a
    ///  wait in the queue.
    pub read: Duration,
}

/// What:
///  The phases of one landing the service itself spent time in, summed over the attempts a moving branch made it repeat.
/// Why:
///   The wait for another program's hold on the index, the objects, the journal write, the branch move and the index update are
///  the stretches a slow landing can lose time in; whatever else a landing did is reported as the reads that remain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandTiming = { lockWait: number; lockHolder?: string; objects: number; journal: number; refUpdate: number; sync: number; attempts: number };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandTiming {
    /// Time to take the index lock, which is the whole wait for a foreign holder.
    pub lock_wait: Duration,
    /// The holder the evidence named while the lock was waited for, when it named one.
    pub lock_holder: Option<String>,
    /// Time to make the commits: one import for the batch, or the checks of a commit Git built.
    pub objects: Duration,
    /// Time to write the intent to the journal.
    pub journal: Duration,
    /// Time of the compare-and-swap that moves the branch.
    pub ref_update: Duration,
    /// Time to remove the conclusion state and to build and install the new index.
    pub sync: Duration,
    /// Passes over the batch: one, and one more each time the branch moved under it.
    pub attempts: usize,
}

impl LandTiming {
    /// What `landing` spent outside the named phases: the branch reads, the comparisons with the branch, the check for removed
    ///  content.
    pub fn reads(&self, landing: Duration) -> Duration {
        return landing.saturating_sub(self.lock_wait + self.objects + self.journal + self.ref_update + self.sync);
    }
}

/// Everything the line for one answered request states.
pub struct Answered<'record> {
    /// The request's identity.
    pub request: &'record str,
    /// The reference the request names.
    pub reference: &'record str,
    /// What became of it: a word such as `landed`, `already`, `retry-prepare`, `branch-moved` or `failed:<code>`.
    pub outcome: &'record str,
    /// The batch number, shared by every request landed together.
    pub batch: u64,
    /// How many requests the batch of this branch held.
    pub size: usize,
    /// This request's place among them, from one.
    pub position: usize,
    /// How the connection and its request line arrived.
    pub arrival: Arrival,
    /// Time from the request being read to its batch starting to land.
    pub queued: Duration,
    /// Time to settle the batches a killed service left open, which runs before each batch.
    pub settle: Duration,
    /// The phases of the landing.
    pub landing: &'record LandTiming,
    /// Time the landing took, from its start to its return.
    pub land_total: Duration,
    /// Time from the landing's end to this request being answered.
    pub answer: Duration,
    /// Time from the acceptance of the connection to the answer.
    pub total: Duration,
}

/// The word for a named holder, in the quoted form.
fn holder_field(who: &String) -> String {
    return format!(" lock_holder={who:?}");
}

/// What:
///  One answered request as a line of `key=value` words, without the prefix.
/// Why:
///   Text values are written in Rust's quoted form, which has no raw newline or unescaped quote, so the line stays one line
///  whatever a client sent; the numbers are whole milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `answered request=${JSON.stringify(id)} outcome=${outcome} queued_ms=${queued} ...`
/// ```
pub fn answered_line(answered: &Answered<'_>) -> String {
    let holder: String = answered.landing.lock_holder.as_ref().map_or_else(String::new, holder_field);
    return format!(
        "answered request={:?} branch={:?} outcome={} batch={} size={} position={} read_ms={} queued_ms={} settle_ms={} \
         lock_wait_ms={}{holder} reads_ms={} objects_ms={} journal_ms={} ref_ms={} sync_ms={} attempts={} landing_ms={} \
         answer_ms={} total_ms={}",
        answered.request,
        answered.reference,
        answered.outcome,
        answered.batch,
        answered.size,
        answered.position,
        answered.arrival.read.as_millis(),
        answered.queued.as_millis(),
        answered.settle.as_millis(),
        answered.landing.lock_wait.as_millis(),
        answered.landing.reads(answered.land_total).as_millis(),
        answered.landing.objects.as_millis(),
        answered.landing.journal.as_millis(),
        answered.landing.ref_update.as_millis(),
        answered.landing.sync.as_millis(),
        answered.landing.attempts,
        answered.land_total.as_millis(),
        answered.answer.as_millis(),
        answered.total.as_millis(),
    );
}

/// The line a service writes as it starts, before `ready`: the release it announces, the protocol, its process and the worktree
///  instance it serves.
pub fn started_line(release: &str, protocol: u32, instance: Option<&str>) -> String {
    return format!("started release={release:?} protocol={protocol} pid={} instance={}", std::process::id(), instance.unwrap_or("any"));
}

/// The line for a retire message: a client of another release found this service and asked it to finish its batch and exit.
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
    /// The queue its connections feed closed, which only happens when the acceptor thread died.
    Error,
}

impl ExitReason {
    /// The word written in the line.
    pub fn word(self) -> &'static str {
        return match self {
            ExitReason::Idle => "idle",
            ExitReason::Retire => "retire",
            ExitReason::Error => "error",
        };
    }
}

/// The line for the service's own exit, with the time it ran since it began serving.
pub fn exit_line(reason: ExitReason, uptime: Duration) -> String {
    return format!("exiting reason={} uptime_ms={}", reason.word(), uptime.as_millis());
}

/// What:
///  Writes one line to the service's log.
/// Why:
///   Standard error is the service manager's journal; one `write` carries the prefix, the line and the newline together, and a
///  closed or broken stream is not the service's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(LOG_PREFIX + line)
/// ```
pub fn log(line: &str) {
    let mut text: String = format!("{LOG_PREFIX}{line}");
    text.push('\n');
    // The result is dropped on purpose: `eprintln!` panics when the stream breaks (a journal restarting), and a log line may
    //  never take the landing that wrote it down with it.
    let _written = std::io::stderr().write_all(text.as_bytes());
}

/// The lines, their fields, and the escaping.
#[cfg(test)]
#[path = "service_timing_tests.rs"]
mod tests;
