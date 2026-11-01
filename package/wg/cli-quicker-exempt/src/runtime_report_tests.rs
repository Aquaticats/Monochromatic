//! Verifies that a failure after startup reaches every channel,
//! that each channel is limited per failure kind with its own interval,
//! that held-back repeats are counted,
//! that one channel failing never keeps the others from the report,
//! and that the map of remembered kinds stays bounded.

/// Reporter under test.
use crate::desktop_notification::{notification_arguments, parse_notification_id, report_identity, NOTIFICATION_EXPIRY_MS};
/// Reporter under test.
use crate::runtime_report::{
    ReportChannel,
    RuntimeReporter,
    JOURNAL_INTERVAL,
    JOURNAL_PRIORITY,
    held_suffix,
    normalized_cause,
    without_held_suffix,
    JournalChannel,
    LogChannel,
    MAX_CAUSES_PER_KIND,
    MAX_EXTRA_CAUSES_PER_INTERVAL,
    LOG_INTERVAL,
    MAX_REMEMBERED_KINDS,
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
    /// Whether `up` relays what this channel receives.
    relayed: bool,
}

/// The recording channel.
impl ReportChannel for Recording {
    /// The label.
    fn name(&self) -> &'static str {
        return self.label;
    }

    /// The log-like channel is relayed to `up`.
    fn relayed_to_up(&self) -> bool {
        return self.relayed;
    }

    /// Records the line,
    /// then fails when asked to.
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
        (Box::new(Recording { label: "log file", lines: Rc::clone(&log), mode: Mode::Works, relayed: true }), Duration::from_secs(10)),
        (Box::new(Recording { label: "journal", lines: Rc::clone(&journal), mode: Mode::Works, relayed: false }), Duration::from_secs(60)),
        (Box::new(Recording { label: "desktop notification", lines: Rc::clone(&screen), mode: desktop, relayed: false }), Duration::from_secs(600)),
    ]);
    return (built, log, journal, screen);
}

/// A repeat of the same cause,
/// with other numbers,
/// is limited per channel interval.
#[test]
fn a_repeated_kind_is_limited_per_channel_interval() {
    let (mut reporter, log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 12 matched but the move failed (attempt 1): boom", start);
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 3456 matched but the move failed (attempt 2): boom", start + Duration::from_secs(5));
    assert_eq!(log.borrow().len(), 1, "the same cause within the log interval");
    assert_eq!(journal.borrow().len(), 1);
    assert_eq!(desktop.borrow().len(), 1);
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 9 matched but the move failed (attempt 3): boom", start + Duration::from_secs(30));
    assert_eq!(log.borrow().len(), 2, "the log interval passed");
    assert_eq!(journal.borrow().len(), 1, "the journal interval did not");
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 9 matched but the move failed (attempt 3): boom", start + Duration::from_secs(90));
    assert_eq!(journal.borrow().len(), 2, "the journal interval passed");
    assert_eq!(desktop.borrow().len(), 1, "the desktop interval did not");
    reporter.report_failure_at("move failed", "POLICY NOT IN FORCE: pid 9 matched but the move failed (attempt 3): boom", start + Duration::from_secs(601));
    assert_eq!(desktop.borrow().len(), 2, "the desktop interval passed");
}

/// Names a user chose make distinct causes,
/// but only the first few of a kind are delivered per interval,
/// so five hundred distinct names cannot flood a channel;
/// the rest are counted and reported at the next delivery.
#[test]
fn distinct_names_in_the_text_deliver_only_the_first_few_causes_of_a_kind() {
    let (mut reporter, _log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    for index in 0..500 {
        reporter.report_failure_at("move skipped", &format!("POLICY NOT IN FORCE: its cgroup /user.slice/name-{index}x{index} is outside"), start + Duration::from_millis(index));
    }
    assert_eq!(journal.borrow().len(), 1, "the digits are normalized, so five hundred names that differ only in digits are one cause");
    assert_eq!(desktop.borrow().len(), 1);
    for (index, word) in ["alpha", "beta", "gamma", "delta", "epsilon", "zeta"].iter().enumerate() {
        reporter.report_failure_at("move skipped", &format!("POLICY NOT IN FORCE: its cgroup /user.slice/{word} is outside"), start + Duration::from_secs(1) + Duration::from_millis(index as u64));
    }
    assert_eq!(journal.borrow().len(), MAX_CAUSES_PER_KIND, "four distinct causes in all, the fifth and sixth are held back");
    assert_eq!(desktop.borrow().len(), MAX_CAUSES_PER_KIND);
    reporter.report_failure_at("move skipped", "POLICY NOT IN FORCE: its cgroup /user.slice/omega is outside", start + Duration::from_secs(61));
    assert_eq!(journal.borrow().len(), MAX_CAUSES_PER_KIND + 1);
    assert!(journal.borrow()[MAX_CAUSES_PER_KIND].contains("(repeated 502 more time(s) since the last report)"), "{:?}", journal.borrow());
}

/// Three matched processes skipped for three different reasons are three reports on every channel,
/// the case where one line per kind hid two of them.
#[test]
fn distinct_causes_of_one_kind_are_each_reported() {
    let (mut reporter, log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    reporter.report_failure_at("move skipped", "POLICY NOT IN FORCE: pid 11 matched but it runs in a login session scope", start);
    reporter.report_failure_at("move skipped", "POLICY NOT IN FORCE: pid 12 matched but it runs in another pid namespace", start);
    reporter.report_failure_at("move skipped", "POLICY NOT IN FORCE: pid 13 matched but it is the main process of a service", start);
    for channel in [&log, &journal, &desktop] {
        assert_eq!(channel.borrow().len(), 3, "{:?}", channel.borrow());
    }
    reporter.report_failure_at("move skipped", "POLICY NOT IN FORCE: pid 14 matched but it runs in a login session scope", start);
    assert_eq!(journal.borrow().len(), 3, "the same reason for another pid is a repeat");
}

/// Extra causes beyond the first of a kind share a cap over all kinds,
/// and the first cause of every kind is still delivered.
#[test]
fn the_global_cap_bounds_extra_causes_and_keeps_the_first_of_every_kind() {
    let (mut reporter, _log, journal, _desktop) = reporter(Mode::Works);
    let start = Instant::now();
    for kind in 0..20 {
        for cause in ["a", "b", "c", "d"] {
            reporter.report_failure_at(&format!("kind {kind}"), &format!("POLICY NOT IN FORCE: reason {cause}"), start);
        }
    }
    assert_eq!(journal.borrow().len(), 20 + MAX_EXTRA_CAUSES_PER_INTERVAL, "twenty first causes plus the cap on the extra ones");
}

/// While the watcher starts,
/// the channel that `up` relays takes every report,
/// the other channels keep their limits,
/// and after the start the limits apply to all.
#[test]
fn during_startup_the_relayed_channel_takes_every_report_and_the_others_stay_limited() {
    let (mut reporter, log, journal, desktop) = reporter(Mode::Works);
    let start = Instant::now();
    reporter.begin_startup();
    for index in 0..50 {
        let word = format!("name{}", "x".repeat(index));
        reporter.report_failure_at("move skipped", &format!("POLICY NOT IN FORCE: its cgroup /user.slice/{word} is outside"), start);
    }
    assert_eq!(log.borrow().len(), 50, "no startup outcome is dropped from the log that up relays");
    assert_eq!(journal.borrow().len(), MAX_CAUSES_PER_KIND);
    assert_eq!(desktop.borrow().len(), MAX_CAUSES_PER_KIND);
    reporter.end_startup();
    let before = log.borrow().len();
    reporter.report_failure_at("process list unreadable", "POLICY NOT IN FORCE: the list of processes could not be read", start + Duration::from_secs(1));
    reporter.report_failure_at("process list unreadable", "POLICY NOT IN FORCE: the list of processes could not be read", start + Duration::from_secs(2));
    assert_eq!(log.borrow().len(), before + 1, "after the start the log channel is limited again");
}

/// Only the log channel is relayed to `up`.
#[test]
fn only_the_log_channel_is_relayed_to_up() {
    assert!(LogChannel.relayed_to_up());
    assert!(!JournalChannel.relayed_to_up());
}

/// The normalized cause makes numbers alike and keeps names and reasons.
#[test]
fn the_normalized_cause_makes_numbers_alike() {
    assert_eq!(normalized_cause("pid 12 failed (attempt 3) in 30s"), "pid # failed (attempt #) in #s");
    assert_eq!(normalized_cause("pid 12 failed"), normalized_cause("pid 99999 failed"));
    assert_ne!(normalized_cause("pid 12 failed: boom"), normalized_cause("pid 12 failed: other"));
    assert_eq!(normalized_cause(&"x".repeat(1000)).len(), 200);
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

/// The body is sanitized by the builder itself and not by its callers:
/// escape sequences,
/// line breaks,
/// line separators and tag characters become visible escapes before the markup is escaped.
#[test]
fn the_notification_body_is_sanitized_by_the_builder() {
    let arguments = notification_arguments("s", "x\u{1b}[2J\ny\u{2028}z\u{e0041}<b>", 0, 1);
    assert_eq!(arguments[11], "x\\u{001b}[2J\\u{000a}y\\u{2028}z\\u{e0041}&lt;b&gt;");
    assert!(!arguments[11].chars().any(|character| return character.is_control() || character == '\u{2028}'), "{:?}", arguments[11]);
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

/// A repeat that carries the note of held-back repeats has the same identity as the first report of its cause,
/// so a notification replaces its own earlier notification and not a second one beside it.
#[test]
fn a_line_with_the_held_repeats_note_has_the_identity_of_its_cause() {
    let first = report_identity("move skipped", "POLICY NOT IN FORCE: pid 1 runs in a login session scope");
    let repeat = format!("POLICY NOT IN FORCE: pid 2 runs in a login session scope{}", held_suffix(3));
    assert_eq!(first, report_identity("move skipped", &repeat));
    assert_ne!(first, report_identity("move skipped", "POLICY NOT IN FORCE: pid 1 runs in another pid namespace"));
    assert_ne!(first, report_identity("move failed", "POLICY NOT IN FORCE: pid 1 runs in a login session scope"));
    assert_eq!(without_held_suffix("x (repeated 12 more time(s) since the last report)"), "x");
    assert_eq!(without_held_suffix("x (repeated many more time(s) since the last report)"), "x (repeated many more time(s) since the last report)");
    assert_eq!(without_held_suffix("x (repeated  more time(s) since the last report)"), "x (repeated  more time(s) since the last report)");
    assert_eq!(without_held_suffix("x"), "x");
    assert_eq!(held_suffix(3), " (repeated 3 more time(s) since the last report)");
}
