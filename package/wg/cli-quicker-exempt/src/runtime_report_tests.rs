//! Verifies that a failure after startup reaches every channel,
//! that the person-facing channel is rate limited per distinct failure,
//! and that a failing channel is named.

/// Reporter under test.
use crate::runtime_report::{
    notification_arguments,
    ReportChannel,
    RuntimeReporter,
    JOURNAL_INTERVAL,
    JOURNAL_PRIORITY,
    NOTIFICATION_INTERVAL,
};
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

/// Lines a recording channel received.
type Lines = Rc<RefCell<Vec<String>>>;

/// A reporter with one always channel,
/// a journal-like channel limited to one delivery per minute,
/// and a desktop-like channel limited to one per ten minutes,
/// with their recordings.
fn reporter(fail_desktop: bool) -> (RuntimeReporter, Lines, Lines, Lines) {
    let always = Rc::new(RefCell::new(Vec::new()));
    let journal = Rc::new(RefCell::new(Vec::new()));
    let desktop = Rc::new(RefCell::new(Vec::new()));
    let built = RuntimeReporter::with_channels(
        vec![Box::new(Recording { label: "log file", lines: Rc::clone(&always), fail: false })],
        vec![
            (Box::new(Recording { label: "journal", lines: Rc::clone(&journal), fail: false }), Duration::from_secs(60)),
            (
                Box::new(Recording { label: "desktop notification", lines: Rc::clone(&desktop), fail: fail_desktop }),
                Duration::from_secs(600),
            ),
        ],
    );
    return (built, always, journal, desktop);
}

/// Every report reaches the always channel,
/// and a repeat of the same failure reaches each limited channel once per its own interval.
#[test]
fn a_repeated_failure_is_limited_per_channel_interval() {
    let (mut reporter, always, journal, desktop) = reporter(false);
    let start = Instant::now();
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 12 matched but the move failed (attempt 1): boom", start);
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 3456 matched but the move failed (attempt 2): boom", start + Duration::from_secs(30));
    assert_eq!(always.borrow().len(), 2);
    assert_eq!(journal.borrow().len(), 1, "the same failure within the journal interval");
    assert_eq!(desktop.borrow().len(), 1, "the same failure within the desktop interval");
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 9 matched but the move failed (attempt 3): boom", start + Duration::from_secs(90));
    assert_eq!(journal.borrow().len(), 2, "the journal interval passed");
    assert_eq!(desktop.borrow().len(), 1, "the desktop interval did not");
    reporter.report_failure_at("POLICY NOT IN FORCE: pid 9 matched but the move failed (attempt 4): boom", start + Duration::from_secs(601));
    assert_eq!(desktop.borrow().len(), 2, "the desktop interval passed");
    assert_eq!(always.borrow().len(), 4);
}

/// A failure that repeats every two seconds reaches the journal once a minute,
/// and the next line says how many repeats were held back,
/// so no occurrence is lost from the record.
#[test]
fn held_back_repeats_are_counted_in_the_next_delivery() {
    let (mut reporter, _always, journal, desktop) = reporter(false);
    let start = Instant::now();
    for second in (0..120).step_by(2) {
        reporter.report_failure_at("POLICY NOT IN FORCE: the list of processes could not be read", start + Duration::from_secs(second));
    }
    assert_eq!(journal.borrow().len(), 2, "one line per minute");
    assert!(journal.borrow()[1].contains("(repeated 29 more time(s) since the last report)"), "{:?}", journal.borrow());
    assert!(!journal.borrow()[0].contains("repeated"));
    assert_eq!(desktop.borrow().len(), 1);
}

/// A different failure is delivered at once.
#[test]
fn a_different_failure_is_not_held_back() {
    let (mut reporter, _always, journal, desktop) = reporter(false);
    let start = Instant::now();
    reporter.report_failure_at("POLICY NOT IN FORCE: the process watcher stopped: poll failed", start);
    reporter.report_failure_at("POLICY NOT IN FORCE: the kernel dropped process events", start + Duration::from_secs(1));
    assert_eq!(journal.borrow().len(), 2);
    assert_eq!(desktop.borrow().len(), 2);
}

/// A channel that fails is named in the result.
#[test]
fn a_failing_channel_is_named() {
    let (mut reporter, always, journal, desktop) = reporter(true);
    let failed = reporter.report_failure_at("POLICY NOT IN FORCE: x", Instant::now());
    assert_eq!(failed.len(), 1);
    assert!(failed[0].contains("desktop notification"));
    assert!(failed[0].contains("the service is not running"));
    assert_eq!(always.borrow().len(), 1, "the other channels still received the report");
    assert_eq!(journal.borrow().len(), 1);
    assert_eq!(desktop.borrow().len(), 1);
}

/// The production intervals are a minute for the journal and ten minutes for the desktop.
#[test]
fn the_production_intervals_are_a_minute_and_ten_minutes() {
    assert_eq!(JOURNAL_INTERVAL, Duration::from_secs(60));
    assert_eq!(NOTIFICATION_INTERVAL, Duration::from_secs(600));
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

/// Failures reach the journal at error level,
/// the owner's choice,
/// and not at warning level.
#[test]
fn journal_lines_are_sent_at_error_level() {
    assert_eq!(JOURNAL_PRIORITY, libc::LOG_ERR);
    assert_ne!(JOURNAL_PRIORITY, libc::LOG_WARNING);
}
