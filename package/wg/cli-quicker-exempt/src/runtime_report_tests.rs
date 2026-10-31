//! Verifies that a failure after startup reaches every channel,
//! that the person-facing channel is rate limited per distinct failure,
//! and that a failing channel is named.

/// Reporter under test.
use crate::runtime_report::{notification_arguments, ReportChannel, RuntimeReporter};
/// Shared recording between the test and the channels.
use std::cell::RefCell;
/// Shared recording between the test and the channels.
use std::rc::Rc;
/// Bounded waits.
use std::time::{Duration, Instant};

/// Records what it was asked to deliver and optionally fails.
struct Recording {
    /// Channel name.
    label: &'static str,
    /// Lines delivered in order.
    lines: Rc<RefCell<Vec<String>>>,
    /// Whether delivery fails.
    fail: bool,
}

/// The recording channel.
impl ReportChannel for Recording {
    /// The label.
    fn name(&self) -> &'static str {
        return self.label;
    }

    /// Records the line,
    /// then fails when asked to.
    fn deliver(&self, line: &str) -> Result<(), String> {
        self.lines.borrow_mut().push(line.to_string());
        if self.fail {
            return Err("the service is not running".to_string());
        }
        return Ok(());
    }
}

/// A reporter with one always channel and one limited channel,
/// and their recordings.
fn reporter(fail_limited: bool) -> (RuntimeReporter, Rc<RefCell<Vec<String>>>, Rc<RefCell<Vec<String>>>) {
    let always = Rc::new(RefCell::new(Vec::new()));
    let limited = Rc::new(RefCell::new(Vec::new()));
    let built = RuntimeReporter::with_channels(
        vec![Box::new(Recording { label: "journal", lines: Rc::clone(&always), fail: false })],
        vec![Box::new(Recording { label: "desktop notification", lines: Rc::clone(&limited), fail: fail_limited })],
        Duration::from_secs(600),
    );
    return (built, always, limited);
}

/// Every report reaches the always channels,
/// and a repeat of the same failure reaches the limited channel once per interval.
#[test]
fn a_repeated_failure_notifies_once_per_interval() {
    let (mut reporter, always, limited) = reporter(false);
    let start = Instant::now();
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 12 matched but the move failed (attempt 1): boom", start);
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 3456 matched but the move failed (attempt 2): boom", start + Duration::from_secs(60));
    assert_eq!(always.borrow().len(), 2);
    assert_eq!(limited.borrow().len(), 1, "the same failure within the interval");
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 9 matched but the move failed (attempt 3): boom", start + Duration::from_secs(601));
    assert_eq!(limited.borrow().len(), 2, "the interval passed");
    assert_eq!(always.borrow().len(), 3);
}

/// A different failure is delivered at once.
#[test]
fn a_different_failure_is_not_held_back() {
    let (mut reporter, _always, limited) = reporter(false);
    let start = Instant::now();
    reporter.report_failure_at("POLICY NOT IN FORCE: the process watcher stopped: poll failed", start);
    reporter.report_failure_at("POLICY NOT IN FORCE: the kernel dropped process events", start + Duration::from_secs(1));
    assert_eq!(limited.borrow().len(), 2);
}

/// A channel that fails is named in the result.
#[test]
fn a_failing_channel_is_named() {
    let (mut reporter, always, limited) = reporter(true);
    let failed = reporter.report_failure_at("POLICY NOT IN FORCE: x", Instant::now());
    assert_eq!(failed.len(), 1);
    assert!(failed[0].contains("desktop notification"));
    assert!(failed[0].contains("the service is not running"));
    assert_eq!(always.borrow().len(), 1, "the other channels still received the report");
    assert_eq!(limited.borrow().len(), 1);
}

/// The notification call follows the `Notify` signature of the freedesktop notification service.
#[test]
fn the_notification_arguments_follow_the_notify_signature() {
    let arguments = notification_arguments("summary text", "body text");
    assert_eq!(arguments[0..2], ["--user", "call"]);
    assert_eq!(arguments[2], "org.freedesktop.Notifications");
    assert_eq!(arguments[5], "Notify");
    assert_eq!(arguments[6], "susssasa{sv}i");
    assert_eq!(arguments[7..], ["wg-quicker-exempt", "0", "dialog-warning", "summary text", "body text", "0", "0", "0"]);
}
