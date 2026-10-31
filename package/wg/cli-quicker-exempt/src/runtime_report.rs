//! Delivers failures that happen after startup to a person.
//! The watcher runs detached with its standard error in a log file nobody reads while working,
//! so a failure after startup is delivered three ways:
//! the log file,
//! the system journal,
//! and a desktop notification of the watched user.
//! The notification is the channel that reaches a person who is not reading logs.
//! It is limited to one per distinct failure per interval,
//! so a failure that repeats on every scan cannot flood the desktop.
//! A channel that fails is itself named in the log,
//! and no channel failure changes routing.

/// The client run as the watched user.
use crate::user_client::UserClient;
/// Rate-limit bookkeeping.
use std::collections::HashMap;
/// Bounded waits.
use std::time::{Duration, Instant};

/// One notification per distinct failure per this interval.
pub const NOTIFICATION_INTERVAL: Duration = Duration::from_secs(600);

/// Distinct failures remembered for the rate limit before expired ones are dropped.
const MAX_REMEMBERED_FAILURES: usize = 256;

/// Identifier written before each journal line.
const SYSLOG_IDENT: &std::ffi::CStr = c"wg-quicker-exempt";

/// One way to deliver a line.
pub trait ReportChannel {
    /// Short name used when the channel itself fails.
    fn name(&self) -> &'static str;
    /// Delivers one line,
    /// or names why it could not.
    fn deliver(&self, line: &str) -> Result<(), String>;
}

/// The log file:
/// the watcher's standard error.
pub struct LogChannel;

/// The log file channel.
impl ReportChannel for LogChannel {
    /// The channel name.
    fn name(&self) -> &'static str {
        return "log file";
    }

    /// Writes the line to standard error.
    fn deliver(&self, line: &str) -> Result<(), String> {
        eprintln!("{line}");
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

    /// Sends the line at warning level under the identifier `wg-quicker-exempt`.
    fn deliver(&self, line: &str) -> Result<(), String> {
        let text = match std::ffi::CString::new(line.replace('\0', " ")) {
            Ok(text) => text,
            Err(error) => return Err(error.to_string()),
        };
        // SAFETY: the identifier is a static C string,
        // the format string is a literal that consumes exactly one string argument,
        // and the argument is a NUL-terminated string alive for the call.
        unsafe {
            libc::openlog(SYSLOG_IDENT.as_ptr(), libc::LOG_PID, libc::LOG_DAEMON);
            libc::syslog(libc::LOG_WARNING, c"%s".as_ptr(), text.as_ptr());
        }
        return Ok(());
    }
}

/// Builds the `busctl` arguments for a desktop notification.
pub fn notification_arguments(summary: &str, body: &str) -> Vec<String> {
    let mut arguments: Vec<String> = Vec::new();
    for argument in [
        "--user",
        "call",
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
        "susssasa{sv}i",
        "wg-quicker-exempt",
        "0",
        "dialog-warning",
        summary,
        body,
        "0",
        "0",
        "0",
    ] {
        arguments.push(argument.to_string());
    }
    return arguments;
}

/// A desktop notification to the watched user through the freedesktop notification service.
pub struct DesktopChannel {
    /// The client run as that user.
    pub client: UserClient,
    /// User who sees the notification.
    pub uid: u32,
    /// Group id used when dropping privilege.
    pub gid: u32,
}

/// The notification channel.
impl ReportChannel for DesktopChannel {
    /// The channel name.
    fn name(&self) -> &'static str {
        return "desktop notification";
    }

    /// Sends the line as a notification that stays until dismissed.
    fn deliver(&self, line: &str) -> Result<(), String> {
        let arguments = notification_arguments("wg-quicker: update traffic policy not in force", line);
        return match self.client.run(self.uid, self.gid, &arguments) {
            Ok(()) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
}

/// The identity of a failure for the rate limit:
/// the line without its digits,
/// so pids and durations do not make a repeat look new.
fn failure_key(line: &str) -> String {
    let mut key = String::new();
    for character in line.chars() {
        if !character.is_ascii_digit() {
            key.push(character);
        }
    }
    return key;
}

/// The failure reporter of one watcher.
pub struct RuntimeReporter {
    /// Channels that receive every failure.
    always: Vec<Box<dyn ReportChannel>>,
    /// Channels limited to one delivery per distinct failure per interval.
    limited: Vec<Box<dyn ReportChannel>>,
    /// Minimum time between two deliveries of the same failure on the limited channels.
    interval: Duration,
    /// When each distinct failure last reached the limited channels.
    last_delivery: HashMap<String, Instant>,
}

/// Construction and delivery.
impl RuntimeReporter {
    /// Builds the production reporter for one watched user.
    pub fn new(uid: u32, gid: u32) -> RuntimeReporter {
        return RuntimeReporter::with_channels(
            vec![Box::new(LogChannel), Box::new(JournalChannel)],
            vec![Box::new(DesktopChannel { client: UserClient::new("busctl"), uid, gid })],
            NOTIFICATION_INTERVAL,
        );
    }

    /// Builds a reporter over the given channels.
    pub fn with_channels(always: Vec<Box<dyn ReportChannel>>, limited: Vec<Box<dyn ReportChannel>>, interval: Duration) -> RuntimeReporter {
        return RuntimeReporter {
            always,
            limited,
            interval,
            last_delivery: HashMap::new(),
        };
    }

    /// Drops the remembered failures whose interval is over.
    fn forget_expired(&mut self, now: Instant) {
        let mut kept: HashMap<String, Instant> = HashMap::new();
        for (text, last) in self.last_delivery.drain() {
            if now.duration_since(last) < self.interval {
                kept.insert(text, last);
            }
        }
        self.last_delivery = kept;
    }

    /// Delivers a failure line at `now`.
    /// Every always channel receives it.
    /// The limited channels receive it unless the same failure reached them within the interval.
    /// Two lines are the same failure when they differ only in digits,
    /// such as pids and durations.
    /// The result lists the channels that failed.
    pub fn report_failure_at(&mut self, line: &str, now: Instant) -> Vec<String> {
        let mut failed: Vec<String> = Vec::new();
        for channel in &self.always {
            if let Err(reason) = channel.deliver(line) {
                failed.push(format!("{}: {reason}", channel.name()));
            }
        }
        let key = failure_key(line);
        let due = match self.last_delivery.get(&key) {
            Some(last) => now.duration_since(*last) >= self.interval,
            None => true,
        };
        if due {
            if self.last_delivery.len() >= MAX_REMEMBERED_FAILURES {
                self.forget_expired(now);
            }
            self.last_delivery.insert(key, now);
            for channel in &self.limited {
                if let Err(reason) = channel.deliver(line) {
                    failed.push(format!("{}: {reason}", channel.name()));
                }
            }
        }
        for problem in &failed {
            eprintln!("wg-quicker-exempt: a failure report channel failed, so a person may not see the report above: {problem}");
        }
        return failed;
    }

    /// Delivers a failure line now.
    pub fn report_failure(&mut self, line: &str) -> Vec<String> {
        return self.report_failure_at(line, Instant::now());
    }
}
