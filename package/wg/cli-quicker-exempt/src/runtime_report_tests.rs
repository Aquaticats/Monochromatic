//! Verifies that a failure after startup reaches every channel,
//! that each channel is limited per failure kind with its own interval,
//! that held-back repeats are counted,
//! that one channel failing never keeps the others from the report,
//! and that the map of remembered kinds stays bounded.

/// Reporter under test.
use crate::runtime_report::{
    notification_arguments,
    parse_notification_id,
    ReportChannel,
    RuntimeReporter,
    JOURNAL_INTERVAL,
    JOURNAL_PRIORITY,
    LOG_INTERVAL,
    MAX_REMEMBERED_KINDS,
    NOTIFICATION_EXPIRY_MS,
    NOTIFICATION_INTERVAL,
};
/// Shared recording between the test and the channels.
use std::cell::RefCell;
/// Shared recording between the test and the channels.
use std::rc::Rc;
/// Bounded waits.
use std::time::{Duration, Instant};

/// How a recording channel misbehaves.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    /// Delivers.
    Works,
    /// Returns an error.
    Fails,
}

/// Records what it was asked to deliver and optionally fails.
struct Recording {
    /// Channel name.
    label: &'static str,
    /// Lines delivered in order.
    lines: Rc<RefCell<Vec<String>>>,
    /// How delivery behaves.
    mode: Mode,
}

/// The recording channel.
impl ReportChannel for Recording {
    /// The label.
    fn name(&self) -> &'static str {
        return self.label;
    }

    /// Records the line,
    /// then fails or panics when asked to.
    fn deliver(&self, _kind: &str, line: &str) -> Result<(), String> {
        self.lines.borrow_mut().push(line.to_string());
        if self.mode == Mode::Fails {
            return Err("the service is not running".to_string());
        }
        return Ok(());
    }
}

/// Lines a recording channel received.
type Lines = Rc<RefCell<Vec<String>>>;

/// A reporter with a log-like channel limited to one delivery per ten seconds,
/// a journal-like channel limited to one per minute,
/// and a desktop-like channel limited to one per ten minutes,
/// with their recordings.
fn reporter(desktop: Mode) -> (RuntimeReporter, Lines, Lines, Lines) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let journal = Rc::new(RefCell::new(Vec::new()));
    let screen = Rc::new(RefCell::new(Vec::new()));
    let built = RuntimeReporter::with_channels(vec![
        (Box::new(Recording { label: "log file", lines: Rc::clone(&log), mode: Mode::Works }), Duration::from_secs(10)),
        (Box::new(Recording { label: "journal", lines: Rc::clone(&journal), mode: Mode::Works }), Duration::from_secs(60)),
        (Box::new(Recording { label: "desktop notification", lines: Rc::clone(&screen), mode: desktop }), Duration::from_secs(600)),
    ]);
    return (built, log, journal, screen);
}

/// A repeat of the same kind is limited per channel interval,
/// whatever the text says.
#[test]
fn a_repeated_kind_is_limited_per_channel_interval() {
    let (mut reporter, log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 12 matched but the move failed (attempt 1): boom", start);
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 3456 matched but the move failed (attempt 2): other", start + Duration::from_secs(5));
    assert_eq!(log.borrow().len(), 1, "the same kind within the log interval");
    assert_eq!(journal.borrow().len(), 1);
    assert_eq!(desktop.borrow().len(), 1);
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 9 matched", start + Duration::from_secs(30));
    assert_eq!(log.borrow().len(), 2, "the log interval passed");
    assert_eq!(journal.borrow().len(), 1, "the journal interval did not");
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 9 matched", start + Duration::from_secs(90));
    assert_eq!(journal.borrow().len(), 2, "the journal interval passed");
    assert_eq!(desktop.borrow().len(), 1, "the desktop interval did not");
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 9 matched", start + Duration::from_secs(601));
    assert_eq!(desktop.borrow().len(), 2, "the desktop interval passed");
}

/// A name chosen by a user in the text cannot make a repeat look new,
/// because the kind and not the text decides.
#[test]
fn distinct_names_in_the_text_do_not_defeat_the_limit() {
    let (mut reporter, _log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    for index in 0..500 {
        reporter.report_failure_at("move skipped", &format!("POLICY NOT IN FORCE: its cgroup /user.slice/name-{index}x{index} is outside"), start + Duration::from_millis(index));
    }
    assert_eq!(journal.borrow().len(), 1);
    assert_eq!(desktop.borrow().len(), 1);
}

/// A failure that repeats every two seconds reaches the journal once a minute,
/// and the next line says how many repeats were held back,
/// so no occurrence is lost from the record.
#[test]
fn held_back_repeats_are_counted_in_the_next_delivery() {
    let (mut reporter, _log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    for second in (0..120).step_by(2) {
        reporter.report_failure_at("process list unreadable", "POLICY NOT IN FORCE: the list of processes could not be read", start + Duration::from_secs(second));
    }
    assert_eq!(journal.borrow().len(), 2, "one line per minute");
    assert!(journal.borrow()[1].contains("(repeated 29 more time(s) since the last report)"), "{:?}", journal.borrow());
    assert!(!journal.borrow()[0].contains("repeated"));
    assert_eq!(desktop.borrow().len(), 1);
}

/// A different kind is delivered at once.
#[test]
fn a_different_kind_is_not_held_back() {
    let (mut reporter, _log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    reporter.report_failure_at("watcher stopped", "POLICY NOT IN FORCE: the process watcher stopped: poll failed", start);
    reporter.report_failure_at("process events dropped", "POLICY NOT IN FORCE: the kernel dropped process events", start + Duration::from_secs(1));
    assert_eq!(journal.borrow().len(), 2);
    assert_eq!(desktop.borrow().len(), 2);
}

/// A channel that fails is named in the result,
/// and the others still receive the report.
#[test]
fn a_failing_channel_is_named_and_the_others_still_receive_the_report() {
    let (mut reporter, log, journal, desktop) = reporter(Mode::Fails);
    let failed = reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: x", Instant::now());
    assert_eq!(failed.len(), 1);
    assert!(failed[0].contains("desktop notification"));
    assert!(failed[0].contains("the service is not running"));
    assert_eq!(log.borrow().len(), 1);
    assert_eq!(journal.borrow().len(), 1);
    assert_eq!(desktop.borrow().len(), 1);
}

/// More kinds than the bound are remembered one at a time:
/// the map never grows past the bound,
/// and the oldest kind is forgotten first.
#[test]
fn the_remembered_kinds_are_bounded_and_the_oldest_is_evicted() {
    let (mut reporter, _log, journal, _desktop) = reporter(Mode::Works);
    let start = Instant::now();
    for index in 0..(MAX_REMEMBERED_KINDS + 10) {
        reporter.report_failure_at(&format!("kind {index}"), "POLICY NOT IN FORCE: x", start + Duration::from_millis(index as u64));
    }
    assert_eq!(journal.borrow().len(), MAX_REMEMBERED_KINDS + 10);
    // The oldest kinds were evicted, so they are delivered again at once; the newest are still held back.
    reporter.report_failure_at("kind 0", "POLICY NOT IN FORCE: x", start + Duration::from_secs(1));
    assert_eq!(journal.borrow().len(), MAX_REMEMBERED_KINDS + 11);
    reporter.report_failure_at(&format!("kind {}", MAX_REMEMBERED_KINDS + 9), "POLICY NOT IN FORCE: x", start + Duration::from_secs(1));
    assert_eq!(journal.borrow().len(), MAX_REMEMBERED_KINDS + 11);
}

/// The production intervals are ten seconds for the log,
/// a minute for the journal and ten minutes for the desktop.
#[test]
fn the_production_intervals_are_ten_seconds_a_minute_and_ten_minutes() {
    assert_eq!(LOG_INTERVAL, Duration::from_secs(10));
    assert_eq!(JOURNAL_INTERVAL, Duration::from_secs(60));
    assert_eq!(NOTIFICATION_INTERVAL, Duration::from_secs(600));
    assert_eq!(NOTIFICATION_EXPIRY_MS, 600_000);
}

/// The notification call follows the `Notify` signature of the freedesktop notification service,
/// replaces the earlier notification of the kind by id,
/// expires,
/// and escapes the markup of its body.
#[test]
fn the_notification_arguments_follow_the_notify_signature() {
    let arguments = notification_arguments("summary text", "body <b>text</b> & more", 42, 600_000);
    assert_eq!(arguments[0..2], ["--user", "call"]);
    assert_eq!(arguments[2], "org.freedesktop.Notifications");
    assert_eq!(arguments[5], "Notify");
    assert_eq!(arguments[6], "susssasa{sv}i");
    assert_eq!(
        arguments[7..],
        ["wg-quicker-exempt", "42", "dialog-warning", "summary text", "body &lt;b&gt;text&lt;/b&gt; &amp; more", "0", "0", "600000"]
    );
}

/// The id of the notification is read from what `busctl` prints.
#[test]
fn the_notification_id_is_read_from_the_client_output() {
    assert_eq!(parse_notification_id("u 42\n"), Some(42));
    assert_eq!(parse_notification_id("u 0"), Some(0));
    assert_eq!(parse_notification_id("s \"x\""), None);
    assert_eq!(parse_notification_id(""), None);
    assert_eq!(parse_notification_id("u many"), None);
}

/// Failures reach the journal at error level,
/// the owner's choice,
/// and not at warning level.
#[test]
fn journal_lines_are_sent_at_error_level() {
    assert_eq!(JOURNAL_PRIORITY, libc::LOG_ERR);
    assert_ne!(JOURNAL_PRIORITY, libc::LOG_WARNING);
}
