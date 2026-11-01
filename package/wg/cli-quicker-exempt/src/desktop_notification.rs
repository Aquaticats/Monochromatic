//! The desktop notification channel of the failure reporter.
//! A notification goes to the watched user through the freedesktop notification service,
//! sent with `busctl` as that user by a worker thread,
//! so the watcher never waits for the bus.
//! A later notification of the same failure kind and cause replaces the earlier one,
//! and every notification expires after the interval.

/// Non-panicking log and escaping.
use crate::log_line::{escape_markup, log_line, sanitize};
/// The channel trait,
/// the cause normalizer
/// and the bounds shared with the reporter.
use crate::runtime_report::{normalized_cause, ReportChannel, MAX_CAUSES_PER_KIND, MAX_REMEMBERED_KINDS};
/// The client run as the watched user.
use crate::user_client::UserClient;
/// Replacement ids of the notifications shown.
use std::collections::HashMap;
/// Queue to the notification worker.
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
/// Shared handles of the worker.
use std::sync::Mutex;
/// The notification worker.
use std::thread::JoinHandle;

/// Notifications that may wait for the worker before new ones are refused.
pub const NOTIFICATION_QUEUE: usize = 8;

/// Milliseconds a notification stays on the desktop,
/// as long as the interval after which the same kind is sent again.
pub const NOTIFICATION_EXPIRY_MS: i32 = 600_000;

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

/// What a notification replaces:
/// the failure kind and the normalized cause,
/// so two distinct causes of one kind are two notifications,
/// and a repeat of one cause replaces its earlier notification.
fn report_identity(kind: &str, line: &str) -> String {
    return format!("{kind}\u{0}{}", normalized_cause(line));
}

/// One notification waiting for the worker.
struct Notification {
    /// What it replaces,
    /// see `report_identity`.
    identity: String,
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
        let replaces_id = shown.get(&notification.identity).copied().unwrap_or(0);
        let arguments = notification_arguments("wg-quicker: update traffic policy not in force", &notification.line, replaces_id, NOTIFICATION_EXPIRY_MS);
        match client.run(uid, gid, &arguments) {
            Ok(output) => {
                if let Some(id) = parse_notification_id(&output) {
                    if shown.len() >= MAX_REMEMBERED_KINDS * MAX_CAUSES_PER_KIND && !shown.contains_key(&notification.identity) {
                        shown.clear();
                    }
                    shown.insert(notification.identity, id);
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
        return match sender.try_send(Notification { identity: report_identity(kind, line), line: line.to_string() }) {
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
