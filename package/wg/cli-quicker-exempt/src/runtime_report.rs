//! Delivers failures that happen after startup to a person.
//! The watcher runs detached with its standard error in a log file nobody reads while working,
//! so a failure after startup is delivered three ways:
//! the log file,
//! the system journal at error level,
//! and a desktop notification of the watched user.
//! Each channel has its own repeat limit per failure kind,
//! so a failure that repeats on every scan cannot flood the log,
//! the journal or the desktop,
//! and the next delivery says how many repeats were held back.
//! One channel failing never keeps the others from receiving the report,
//! and no write here can panic.
//! A notification never blocks the watcher:
//! a worker thread sends it,
//! and a later notification of the same kind replaces the earlier one on the desktop.

/// Non-panicking log and escaping.
use crate::log_line::{log_line, sanitize};
/// The desktop notification channel.
use crate::desktop_notification::DesktopChannel;
/// The client run as the watched user.
use crate::user_client::UserClient;
/// Rate-limit bookkeeping.
use std::collections::HashMap;
/// Bounded waits.
use std::time::{Duration, Instant};

/// One log line per distinct cause of a failure kind per this interval,
/// for the first few causes of each kind.
pub const LOG_INTERVAL: Duration = Duration::from_secs(10);

/// One notification per failure kind per this interval.
pub const NOTIFICATION_INTERVAL: Duration = Duration::from_secs(600);

/// One journal line per failure kind per this interval,
/// so a failure that repeats every second or two cannot flood the journal.
pub const JOURNAL_INTERVAL: Duration = Duration::from_secs(60);

/// Failure kinds remembered by one channel before the oldest is forgotten.
pub const MAX_REMEMBERED_KINDS: usize = 64;

/// Distinct normalized causes of one failure kind that one channel delivers per interval.
/// With the bound on the kinds this is the global cap:
/// a channel delivers at most `MAX_REMEMBERED_KINDS` times this many reports per interval,
/// however many names a user chooses.
pub const MAX_CAUSES_PER_KIND: usize = 4;

/// Distinct causes beyond the first of a kind that one channel delivers per interval,
/// over all kinds together.
/// The first cause of every kind is always delivered,
/// so each kind is seen at least once;
/// this bounds what a user who controls names can add on top.
pub const MAX_EXTRA_CAUSES_PER_INTERVAL: usize = 32;

/// Characters of a line that make up its normalized cause.
const MAX_CAUSE_CHARS: usize = 200;

/// The cause of a failure line with its numbers made alike,
/// so the same reason for another process id,
/// count or duration is the same cause,
/// and a different reason or a different name is another one.
/// Only the first `MAX_CAUSE_CHARS` characters count.
pub fn normalized_cause(line: &str) -> String {
    let mut cause = String::new();
    let mut length = 0;
    let mut in_digits = false;
    for character in line.chars() {
        if length >= MAX_CAUSE_CHARS {
            break;
        }
        if character.is_ascii_digit() {
            if !in_digits {
                cause.push('#');
                length += 1;
            }
            in_digits = true;
        } else {
            in_digits = false;
            cause.push(character);
            length += 1;
        }
    }
    return cause;
}

/// Priority of every journal line:
/// error level,
/// as the owner chose for failures after startup.
pub const JOURNAL_PRIORITY: libc::c_int = libc::LOG_ERR;

/// Identifier written before each journal line.
const SYSLOG_IDENT: &std::ffi::CStr = c"wg-quicker-exempt";

/// One way to deliver a line.
pub trait ReportChannel {
    /// Short name used when the channel itself fails.
    fn name(&self) -> &'static str;
    /// Delivers one line of the given failure kind,
    /// or names why it could not.
    fn deliver(&self, kind: &str, line: &str) -> Result<(), String>;
    /// Waits until everything handed to the channel has been sent,
    /// for a process that is about to exit.
    fn finish(&self) {}

    /// Whether what this channel receives reaches the terminal of `up`,
    /// which relays the watcher log after the start.
    /// Such a channel takes every report while the watcher starts,
    /// because a start that hid a failure from `up` would not have said that a policy is not in force.
    fn relayed_to_up(&self) -> bool {
        return false;
    }
}

/// The log file:
/// the watcher's standard error,
/// inside its byte budget.
pub struct LogChannel;

/// The log file channel.
impl ReportChannel for LogChannel {
    /// The channel name.
    fn name(&self) -> &'static str {
        return "log file";
    }

    /// `up` relays the loud lines of the log after the start.
    fn relayed_to_up(&self) -> bool {
        return true;
    }

    /// Writes the line to standard error without panicking.
    fn deliver(&self, _kind: &str, line: &str) -> Result<(), String> {
        log_line(line);
        return Ok(());
    }
}

/// The system journal through `syslog(3)`.
pub struct JournalChannel;

/// The journal channel.
impl ReportChannel for JournalChannel {
    /// The channel name.
    fn name(&self) -> &'static str {
        return "journal";
    }

    /// Sends the line at error level under the identifier `wg-quicker-exempt`.
    /// Control characters are escaped,
    /// so untrusted text cannot start a second journal line.
    fn deliver(&self, _kind: &str, line: &str) -> Result<(), String> {
        let text = match std::ffi::CString::new(sanitize(line)) {
            Ok(text) => text,
            Err(error) => return Err(error.to_string()),
        };
        // SAFETY: the identifier is a static C string,
        // the format string is a literal that consumes exactly one string argument,
        // and the argument is a NUL-terminated string alive for the call.
        unsafe {
            libc::openlog(SYSLOG_IDENT.as_ptr(), libc::LOG_PID, libc::LOG_DAEMON);
            libc::syslog(JOURNAL_PRIORITY, c"%s".as_ptr(), text.as_ptr());
        }
        return Ok(());
    }
}

/// A channel with its own repeat limit.
struct Tier {
    /// The channel.
    channel: Box<dyn ReportChannel>,
    /// Minimum time between two deliveries of the same failure kind on this channel.
    interval: Duration,
    /// The window of each failure kind:
    /// when it began,
    /// which distinct causes were delivered in it,
    /// and how many repeats were held back.
    last_delivery: HashMap<String, KindWindow>,
    /// When the current count of extra causes began.
    extra_started: Option<Instant>,
    /// Extra causes delivered since then.
    extra_admitted: usize,
}

/// Counts one extra cause against the global cap of the interval.
/// Returns whether it may be delivered.
fn take_extra_cause(started: &mut Option<Instant>, admitted: &mut usize, interval: Duration, now: Instant) -> bool {
    let expired = match started {
        Some(when) => now.duration_since(*when) >= interval,
        None => true,
    };
    if expired {
        *started = Some(now);
        *admitted = 0;
    }
    if *admitted >= MAX_EXTRA_CAUSES_PER_INTERVAL {
        return false;
    }
    *admitted += 1;
    return true;
}

/// What one channel has seen of one failure kind within its interval.
struct KindWindow {
    /// When the window began.
    started: Instant,
    /// Repeats held back since the last delivery.
    held: u32,
    /// Normalized causes delivered in this window,
    /// at most `MAX_CAUSES_PER_KIND`.
    causes: Vec<String>,
}

/// Limiting for one channel.
impl Tier {
    /// Drops the kinds whose interval is over.
    fn forget_expired(&mut self, now: Instant) {
        let mut kept: HashMap<String, KindWindow> = HashMap::new();
        for (kind, window) in self.last_delivery.drain() {
            if now.duration_since(window.started) < self.interval {
                kept.insert(kind, window);
            }
        }
        self.last_delivery = kept;
    }

    /// Makes room for one more kind:
    /// drops the kinds whose interval is over,
    /// and the oldest one when none is.
    fn make_room(&mut self, now: Instant) {
        self.forget_expired(now);
        if self.last_delivery.len() < MAX_REMEMBERED_KINDS {
            return;
        }
        let mut oldest: Option<(String, Instant)> = None;
        for (kind, window) in &self.last_delivery {
            match &oldest {
                Some((_name, when)) if *when <= window.started => {}
                _ => oldest = Some((kind.clone(), window.started)),
            }
        }
        if let Some((kind, _when)) = oldest {
            self.last_delivery.remove(&kind);
        }
    }

    /// Delivers the line unless this channel already took enough of the kind within the interval.
    /// The first `MAX_CAUSES_PER_KIND` distinct normalized causes of a kind are delivered in each window,
    /// so three processes skipped for three different reasons are three reports,
    /// and a name chosen by a user can still not make every line look new:
    /// the rest are counted.
    /// A delivery after held-back repeats says how many there were,
    /// so no occurrence is lost from the record.
    fn deliver_limited(&mut self, kind: &str, line: &str, now: Instant) -> Result<(), String> {
        let cause = normalized_cause(line);
        let mut held = 0;
        if let Some(window) = self.last_delivery.get_mut(kind) {
            if now.duration_since(window.started) < self.interval {
                if window.causes.len() < MAX_CAUSES_PER_KIND
                    && !window.causes.contains(&cause)
                    && take_extra_cause(&mut self.extra_started, &mut self.extra_admitted, self.interval, now)
                {
                    window.causes.push(cause);
                    return self.channel.deliver(kind, line);
                }
                window.held += 1;
                return Ok(());
            }
            held = window.held;
        }
        if self.last_delivery.len() >= MAX_REMEMBERED_KINDS && !self.last_delivery.contains_key(kind) {
            self.make_room(now);
        }
        self.last_delivery.insert(kind.to_string(), KindWindow { started: now, held: 0, causes: vec![cause] });
        if held == 0 {
            return self.channel.deliver(kind, line);
        }
        return self.channel.deliver(kind, &format!("{line} (repeated {held} more time(s) since the last report)"));
    }
}

/// The failure reporter of one watcher.
pub struct RuntimeReporter {
    /// Channels with their repeat limits.
    tiers: Vec<Tier>,
    /// Whether the watcher is still starting,
    /// when the channels that `up` relays take every report.
    starting: bool,
}

/// Construction and delivery.
impl RuntimeReporter {
    /// Builds the production reporter for one watched user:
    /// the log file once per kind per ten seconds,
    /// the journal once per kind per minute,
    /// and the desktop once per kind per ten minutes.
    pub fn new(uid: u32, gid: u32) -> RuntimeReporter {
        return RuntimeReporter::with_channels(vec![
            (Box::new(LogChannel), LOG_INTERVAL),
            (Box::new(JournalChannel), JOURNAL_INTERVAL),
            (Box::new(DesktopChannel::new(UserClient::new("busctl"), uid, gid)), NOTIFICATION_INTERVAL),
        ]);
    }

    /// Builds a reporter over the given channels,
    /// each with its interval.
    pub fn with_channels(channels: Vec<(Box<dyn ReportChannel>, Duration)>) -> RuntimeReporter {
        let mut tiers: Vec<Tier> = Vec::new();
        for (channel, interval) in channels {
            tiers.push(Tier { channel, interval, last_delivery: HashMap::new(), extra_started: None, extra_admitted: 0 });
        }
        return RuntimeReporter { tiers, starting: false };
    }

    /// Delivers a failure line of `kind` at `now`.
    /// Each channel receives it unless the same kind reached that channel within its interval.
    /// The kind,
    /// not the text,
    /// decides what counts as a repeat,
    /// so a name chosen by a user cannot make every line look new.
    /// Every channel is tried whatever the others returned.
    /// The result lists the channels that failed.
    pub fn report_failure_at(&mut self, kind: &str, line: &str, now: Instant) -> Vec<String> {
        let mut failed: Vec<String> = Vec::new();
        let starting = self.starting;
        for tier in &mut self.tiers {
            let result = if starting && tier.channel.relayed_to_up() {
                tier.channel.deliver(kind, line)
            } else {
                tier.deliver_limited(kind, line, now)
            };
            if let Err(reason) = result {
                failed.push(format!("{}: {reason}", tier.channel.name()));
            }
        }
        for problem in &failed {
            log_line(&format!(
                "wg-quicker-exempt: a failure report channel failed, so a person may not see the report: {}",
                sanitize(problem)
            ));
        }
        return failed;
    }

    /// Marks the start of the watcher:
    /// until `end_startup`,
    /// the channels that `up` relays take every report without a limit,
    /// so no failure found while starting is missing from the output of `up`.
    pub fn begin_startup(&mut self) {
        self.starting = true;
    }

    /// Ends the start,
    /// and the limits apply to every channel.
    pub fn end_startup(&mut self) {
        self.starting = false;
    }

    /// Waits until every channel has sent what it was handed,
    /// for a process that is about to exit.
    pub fn finish(&self) {
        for tier in &self.tiers {
            tier.channel.finish();
        }
    }

    /// Delivers a failure line of `kind` now.
    pub fn report_failure(&mut self, kind: &str, line: &str) -> Vec<String> {
        return self.report_failure_at(kind, line, Instant::now());
    }
}
