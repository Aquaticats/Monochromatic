//! What:
//!  The shared clock of every wait notice:
//!  when the first line of a wait is due,
//!  how often it repeats,
//!  and the place in line a queued commit holds.
//! Why:
//!  A commit that queues behind other commits,
//! a lock,
//! a reservation or a child process must say so,
//!  and say it again at a steady interval,
//!  or the caller cannot tell a queued commit from a hung one (issue 1811).
//!  Each wait loop feeds its elapsed time to a `Ticker` and prints a line when the ticker says one is due,
//!  so the decision is a function of elapsed time alone and a test can drive it with chosen durations.
//!  The interval comes from `CLI_GIT_WAIT_NOTICE_SECONDS` at the production entry of each wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ticker = new Ticker(Cadence.fromEnvironment());
//! while (waiting) { if (ticker.due(elapsedMs)) stderr.write(line(elapsedMs)); await wait(pollMs); }
//! ```

/// Positive-seconds parsing shared with the lock wait limits.
use crate::push_deadline_and_patience_settings::positive_seconds;
/// Borrowed OS text of the environment value.
use std::ffi::OsStr;
/// `Write` is the sink a notice goes to.
use std::io::Write;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// Environment variable that sets,
/// in whole seconds,
/// how often a waiting command repeats its notice.
pub const NOTICE_INTERVAL_VARIABLE: &str = "CLI_GIT_WAIT_NOTICE_SECONDS";

/// How often a waiting command repeats its notice when the variable is absent or unusable.
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
pub const DEFAULT_NOTICE_INTERVAL: Duration = Duration::from_secs(10);

/// How long a wait stays silent before its first notice:
///  a landing that holds the lock normally ends within this,
///  so brief contention between healthy commits prints nothing.
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
pub const NOTICE_GRACE: Duration = Duration::from_secs(1);

/// The shortest interval a ticker divides by,
/// so a zero interval cannot divide by zero.
const SHORTEST_INTERVAL: Duration = Duration::from_millis(1);

/// What:
///  When a wait's notices are due:
///  the grace before the first and the interval between the rest.
/// Why:
///   One value travels from the production entry to the wait loop,
///  and tests choose short ones instead of waiting out the defaults.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cadence = { graceMs: number; intervalMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cadence {
    /// Silence before the first notice.
    pub grace: Duration,
    /// Time between two notices.
    pub interval: Duration,
}

impl Cadence {
    /// The cadence for an interval:
    ///  the standard grace,
    /// but never longer than the interval itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { graceMs: Math.min(GRACE_MS, intervalMs), intervalMs }
    /// ```
    pub fn from_interval(interval: Duration) -> Self {
        return Self {
            grace: interval.min(NOTICE_GRACE),
            interval,
        };
    }

    /// The cadence when nothing configures it.
    pub fn standard() -> Self {
        return Self::from_interval(DEFAULT_NOTICE_INTERVAL);
    }

    /// The cadence for a value of the variable;
    ///  a missing,
    /// zero or malformed value falls back to the standard one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Cadence.fromInterval(positiveSeconds(value) ?? DEFAULT_INTERVAL)
    /// ```
    pub fn from_value(value: Option<&OsStr>) -> Self {
        let Some(text) = value else {
            return Self::standard();
        };
        return match positive_seconds(NOTICE_INTERVAL_VARIABLE, text) {
            Some(interval) => Self::from_interval(interval),
            None => Self::standard(),
        };
    }

    /// The cadence this process runs under:
    ///  the variable in its environment,
    /// or the standard one.
    pub fn from_environment() -> Self {
        return Self::from_value(std::env::var_os(NOTICE_INTERVAL_VARIABLE).as_deref());
    }
}

/// What:
///  Decides,
/// from elapsed time alone,
///  whether a notice is due,
///  and when the next one will be.
/// Why:
///   A wait loop polls at its own pace.
///  The ticker answers "is a line due now" once per due notice,
///  however late the poll comes,
///  and never answers twice for the same interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Ticker { due(elapsedMs: number): boolean; nextInMs(elapsedMs: number): number }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Ticker {
    /// The interval between notices.
    interval: Duration,
    /// When the first notice is due.
    first_at: Duration,
    /// How many notices were already due.
    printed: u32,
}

impl Ticker {
    /// A ticker whose first notice comes after the cadence's grace.
    pub fn new(cadence: Cadence) -> Self {
        return Self {
            interval: cadence.interval.max(SHORTEST_INTERVAL),
            first_at: cadence.grace,
            printed: 0,
        };
    }

    /// A ticker for a wait that printed its first line when it began:
    ///  the next notice comes one interval later.
    pub fn announced_at_start(cadence: Cadence) -> Self {
        return Self {
            interval: cadence.interval.max(SHORTEST_INTERVAL),
            first_at: Duration::ZERO,
            printed: 1,
        };
    }

    /// How many notices are due after `waited`.
    fn due_count(&self, waited: Duration) -> u32 {
        if waited < self.first_at {
            return 0;
        }
        let after_first: u128 = (waited - self.first_at).as_nanos() / self.interval.as_nanos();
        return u32::try_from(after_first.saturating_add(1)).unwrap_or(u32::MAX);
    }

    /// Whether a notice is due now that `waited` has passed;
    ///  true once per due notice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ticker.due(elapsedMs)
    /// ```
    pub fn due(&mut self, waited: Duration) -> bool {
        let count: u32 = self.due_count(waited);
        if count <= self.printed {
            return false;
        }
        self.printed = count;
        return true;
    }

    /// How long after `waited` the next notice is due;
    ///  a watcher sleeps this long.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ticker.nextInMs(elapsedMs)
    /// ```
    pub fn next_in(&self, waited: Duration) -> Duration {
        // The notice after the `printed`th one is due `printed` intervals after the first.
        let due_at: Duration = self.first_at.saturating_add(self.interval.saturating_mul(self.printed));
        return due_at.saturating_sub(waited);
    }
}

/// What:
///  What one notice of a wait for a child process says.
/// Why:
///   A child whose output goes to a file or a pipe is silent while it runs,
///  so the wait names what it waits for,
/// the child's process,
/// how long it has lasted and where it stops the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildWaitFacts = { waitingFor; pid; waitedMs; limitMs; limitNote; repeat };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ChildWaitFacts<'facts> {
    /// What the wait is for,
    /// in words ("monochromatic-lint to check a.md").
    pub waiting_for: &'facts str,
    /// The child's process.
    pub pid: u32,
    /// How long the wait has lasted.
    pub waited: Duration,
    /// How long the child may show no sign of work before it is ended,
    /// when it may be ended.
    pub limit: Duration,
    /// Where the person changes the limit,
    /// spelled to follow "seconds",
    /// or empty when nothing changes it.
    pub limit_note: &'facts str,
    /// Whether an earlier notice of this wait already printed the hint.
    pub repeat: bool,
    /// Whether a child with no sign of work for the limit is ended;
    /// a step that may not be run twice is only waited for.
    pub ends_when_quiet: bool,
    /// How long the command waits for a step it never ends before it stops waiting and leaves it running,
    /// or nothing when it waits for as long as the step lives.
    pub left_running_after: Option<Duration>,
}

/// What:
///  The notice of a wait for a child process.
/// Why:
///   The first notice of a wait adds what to do if the child is stuck,
/// and the repeats stay short;
///  a wait that has a bound of its own says that bound instead of the quiet window,
/// because the window is not what ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: still waiting for ${waitingFor} (process ${pid}) after ${waited} seconds; ${ending}`
/// ```
pub fn child_wait_line(facts: &ChildWaitFacts<'_>) -> String {
    let hint: &str = if facts.repeat {
        ""
    } else {
        " Hint: it is still running; if it looks stuck, check it with `ps` and report it."
    };
    // A child is ended for being quiet and never for being slow, and a step that may not be run twice is never ended.
    if !facts.ends_when_quiet {
        // A step the wrapper never ends that this command stops waiting for has a bound of its own: saying it is waited for as
        //  long as it lives would tell the person there is no bound where there is one (2026-11-11, the release gate of
        //  0.6.1-alpha.11: a notice said so of a write whose wait ended at 127 seconds).
        if let Some(after) = facts.left_running_after {
            return format!(
                "monochromatic-git-policy-cli: still waiting for {} (process {}) after {} seconds; the wrapper never ends it, and this \
                 command stops waiting for it after {} seconds and leaves it running, so whether it finished is not known then.{hint}",
                facts.waiting_for,
                facts.pid,
                facts.waited.as_secs(),
                after.as_secs()
            );
        }
        return format!(
            "monochromatic-git-policy-cli: still waiting for {} (process {}) after {} seconds; it is a step that may not be run twice, so it is \
             waited for as long as it lives{}.{hint}",
            facts.waiting_for,
            facts.pid,
            facts.waited.as_secs(),
            facts.limit_note
        );
    }
    return format!(
        "monochromatic-git-policy-cli: still waiting for {} (process {}) after {} seconds; it is ended only after {} seconds with no sign of work{}.{hint}",
        facts.waiting_for,
        facts.pid,
        facts.waited.as_secs(),
        facts.limit.as_secs(),
        facts.limit_note
    );
}

/// What:
///  What the caller needs to build the line of a notice that is due.
/// Why:
///   The line is built by the caller,
/// from its own variables,
/// only when a notice is due,
/// so a poll that is not due costs a
///  clock read and nothing else,
/// and no closure has to carry the caller's variables into the reporter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DueNotice = { waitedMs: number; repeat: boolean };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct DueNotice {
    /// How long the wait has lasted.
    pub waited: Duration,
    /// Whether an earlier notice of this wait was printed.
    pub repeat: bool,
}

/// What:
///  Prints a wait's notices to a sink at the pace of its ticker.
/// Why:
///   Wait loops that poll (reservations,
/// child processes) share one printing rule:
///  the caller builds the line only when `notice_due` says it is due,
///  so a poll that is not due costs a clock read and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reporter = new WaitReporter(cadence, stderr);
/// const due = reporter.noticeDue(); if (due) reporter.printNotice(line(due.waitedMs, due.repeat));
/// ```
pub struct WaitReporter<'sink> {
    /// When the wait began.
    started: Instant,
    /// Decides when a line is due.
    ticker: Ticker,
    /// Where the lines go.
    sink: &'sink mut dyn Write,
    /// How many lines were printed.
    printed: u32,
}

impl<'sink> WaitReporter<'sink> {
    /// A reporter whose clock starts now and whose first notice follows the cadence's grace.
    pub fn start(cadence: Cadence, sink: &'sink mut dyn Write) -> Self {
        return Self {
            started: Instant::now(),
            ticker: Ticker::new(cadence),
            sink,
            printed: 0,
        };
    }

    /// Say whether a notice is due now,
    /// and how long the wait has lasted and whether an earlier notice of this wait was
    ///  printed,
    /// which is what the caller needs to build the line.
    ///  A due notice is answered once;
    /// the caller then hands the line to `print_notice`.
    pub fn notice_due(&mut self) -> Option<DueNotice> {
        let waited: Duration = self.started.elapsed();
        if !self.ticker.due(waited) {
            return None;
        }
        return Some(DueNotice { waited, repeat: self.printed > 0 });
    }

    /// Print the line the caller built for the notice that `notice_due` reported,
    /// and count it.
    pub fn print_notice(&mut self, text: &str) {
        self.printed += 1;
        // Terminal output error: a notice is advisory; a failed write must not stop the wait.
        let _written = writeln!(self.sink, "{text}");
    }
}

/// Cadence,
/// ticker and line controls stay out of the release executable.
#[cfg(test)]
#[path = "wait_progress_tests.rs"]
mod tests;
