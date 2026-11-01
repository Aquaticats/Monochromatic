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
use crate::log_line::{escape_markup, log_line, sanitize};
/// The client run as the watched user.
use crate::user_client::UserClient;
/// Rate-limit bookkeeping.
use std::collections::HashMap;
/// Queue to the notification worker.
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
/// Shared handles of the worker.
use std::sync::Mutex;
/// The notification worker.
use std::thread::JoinHandle;
/// Bounded waits.
use std::time::{Duration, Instant};

/// One log line per failure kind per this interval.
pub const LOG_INTERVAL: Duration = Duration::from_secs(10);

/// One notification per failure kind per this interval.
pub const NOTIFICATION_INTERVAL: Duration = Duration::from_secs(600);

/// One journal line per failure kind per this interval,
/// so a failure that repeats every second or two cannot flood the journal.
pub const JOURNAL_INTERVAL: Duration = Duration::from_secs(60);

/// Failure kinds remembered by one channel before the oldest is forgotten.
pub const MAX_REMEMBERED_KINDS: usize = 64;

/// Notifications that may wait for the worker before new ones are refused.
pub const NOTIFICATION_QUEUE: usize = 8;

/// Milliseconds a notification stays on the desktop,
/// as long as the interval after which the same kind is sent again.
pub const NOTIFICATION_EXPIRY_MS: i32 = 600_000;

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

/// Builds the `busctl` arguments for a desktop notification.
/// `replaces_id` is the notification a new one replaces,
/// zero for none,
/// and `expiry_ms` how long it stays.
pub fn notification_arguments(summary: &str, body: &str, replaces_id: u32, expiry_ms: i32) -> Vec<String> {
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
    ] {
        arguments.push(argument.to_string());
    }
    arguments.push(replaces_id.to_string());
    arguments.push("dialog-warning".to_string());
    arguments.push(summary.to_string());
    arguments.push(escape_markup(&sanitize(body)));
    arguments.push("0".to_string());
    arguments.push("0".to_string());
    arguments.push(expiry_ms.to_string());
    return arguments;
}

/// Reads the notification id out of what `busctl` printed for `Notify`,
/// which looks like `u 42`.
pub fn parse_notification_id(output: &str) -> Option<u32> {
    let mut words = output.split_whitespace();
    if words.next() != Some("u") {
        return None;
    }
    return words.next()?.parse().ok();
}

/// One notification waiting for the worker.
struct Notification {
    /// Failure kind,
    /// which decides what it replaces.
    kind: String,
    /// Text shown.
    line: String,
}

/// A desktop notification to the watched user through the freedesktop notification service,
/// sent by a worker thread so the watcher never waits for the bus.
pub struct DesktopChannel {
    /// Queue to the worker,
    /// absent when the worker could not start or has been told to finish.
    sender: Mutex<Option<SyncSender<Notification>>>,
    /// The worker,
    /// until it is joined.
    worker: Mutex<Option<JoinHandle<()>>>,
}

/// Worker that sends the notifications.
fn notification_worker(receiver: Receiver<Notification>, client: UserClient, uid: u32, gid: u32) {
    let mut shown: HashMap<String, u32> = HashMap::new();
    while let Ok(notification) = receiver.recv() {
        let replaces_id = shown.get(&notification.kind).copied().unwrap_or(0);
        let arguments = notification_arguments("wg-quicker: update traffic policy not in force", &notification.line, replaces_id, NOTIFICATION_EXPIRY_MS);
        match client.run(uid, gid, &arguments) {
            Ok(output) => {
                if let Some(id) = parse_notification_id(&output) {
                    if shown.len() >= MAX_REMEMBERED_KINDS && !shown.contains_key(&notification.kind) {
                        shown.clear();
                    }
                    shown.insert(notification.kind, id);
                }
            }
            Err(error) => {
                log_line(&format!(
                    "wg-quicker-exempt: a failure report channel failed, so a person may not see the report: desktop notification: {}",
                    sanitize(&error.to_string())
                ));
            }
        }
    }
}

/// Construction and delivery of notifications.
impl DesktopChannel {
    /// Starts the worker for the given user.
    pub fn new(client: UserClient, uid: u32, gid: u32) -> DesktopChannel {
        let (sender, receiver) = sync_channel(NOTIFICATION_QUEUE);
        let started = std::thread::Builder::new()
            .name("wg-quicker-notify".to_string())
            .spawn(move || notification_worker(receiver, client, uid, gid));
        return match started {
            Ok(handle) => DesktopChannel {
                sender: Mutex::new(Some(sender)),
                worker: Mutex::new(Some(handle)),
            },
            Err(_error) => DesktopChannel {
                sender: Mutex::new(None),
                worker: Mutex::new(None),
            },
        };
    }
}

/// The notification channel.
impl ReportChannel for DesktopChannel {
    /// The channel name.
    fn name(&self) -> &'static str {
        return "desktop notification";
    }

    /// Queues the line for the worker,
    /// and refuses it when the queue is full or the worker is gone.
    fn deliver(&self, kind: &str, line: &str) -> Result<(), String> {
        let Ok(guard) = self.sender.lock() else {
            return Err("the notification queue is unavailable".to_string());
        };
        let Some(sender) = guard.as_ref() else {
            return Err("the notification worker is not running".to_string());
        };
        return match sender.try_send(Notification { kind: kind.to_string(), line: line.to_string() }) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_dropped)) => Err("the notification queue is full".to_string()),
            Err(TrySendError::Disconnected(_dropped)) => Err("the notification worker stopped".to_string()),
        };
    }

    /// Closes the queue and waits for the worker to send what is left,
    /// which takes at most the client timeout for each notification still queued.
    fn finish(&self) {
        if let Ok(mut sender) = self.sender.lock() {
            sender.take();
        }
        let worker = match self.worker.lock() {
            Ok(mut guard) => guard.take(),
            Err(_poisoned) => None,
        };
        if let Some(running) = worker {
            let _joined = running.join();
        }
    }
}

/// A channel with its own repeat limit.
struct Tier {
    /// The channel.
    channel: Box<dyn ReportChannel>,
    /// Minimum time between two deliveries of the same failure kind on this channel.
    interval: Duration,
    /// When each failure kind last reached the channel,
    /// and how many repeats were held back since.
    last_delivery: HashMap<String, (Instant, u32)>,
}

/// Limiting for one channel.
impl Tier {
    /// Drops the kinds whose interval is over.
    fn forget_expired(&mut self, now: Instant) {
        let mut kept: HashMap<String, (Instant, u32)> = HashMap::new();
        for (kind, (last, held)) in self.last_delivery.drain() {
            if now.duration_since(last) < self.interval {
                kept.insert(kind, (last, held));
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
        for (kind, (last, _held)) in &self.last_delivery {
            match &oldest {
                Some((_name, when)) if *when <= *last => {}
                _ => oldest = Some((kind.clone(), *last)),
            }
        }
        if let Some((kind, _when)) = oldest {
            self.last_delivery.remove(&kind);
        }
    }

    /// Delivers the line unless the same kind reached this channel within the interval.
    /// A delivery after held-back repeats says how many there were,
    /// so no occurrence is lost from the record.
    fn deliver_limited(&mut self, kind: &str, line: &str, now: Instant) -> Result<(), String> {
        let held = match self.last_delivery.get_mut(kind) {
            Some((last, held)) if now.duration_since(*last) < self.interval => {
                *held += 1;
                return Ok(());
            }
            Some((_last, held)) => *held,
            None => 0,
        };
        if self.last_delivery.len() >= MAX_REMEMBERED_KINDS && !self.last_delivery.contains_key(kind) {
            self.make_room(now);
        }
        self.last_delivery.insert(kind.to_string(), (now, 0));
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
            tiers.push(Tier { channel, interval, last_delivery: HashMap::new() });
        }
        return RuntimeReporter { tiers };
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
        for tier in &mut self.tiers {
            if let Err(reason) = tier.deliver_limited(kind, line, now) {
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
