//! Verifies the log budget,
//! the non-panicking write,
//! and the escaping of untrusted text for each destination.

/// Budget and escaping under test.
use crate::log_line::{escape_markup, sanitize, write_unchecked, Admission, LogBudget};
/// Fixed clock.
use std::time::{Duration, Instant};

/// Lines inside the budget are written,
/// the first line past it says so,
/// and later lines are dropped until the window ends.
#[test]
fn a_budget_writes_then_says_so_once_then_drops() {
    let mut budget = LogBudget::new(100, Duration::from_secs(60), 10_000);
    let start = Instant::now();
    assert_eq!(budget.admit(60, start), Admission::Write { dropped_before: 0 });
    assert_eq!(budget.admit(40, start + Duration::from_secs(1)), Admission::Write { dropped_before: 0 });
    assert_eq!(budget.admit(1, start + Duration::from_secs(2)), Admission::BudgetReached);
    assert_eq!(budget.admit(1, start + Duration::from_secs(3)), Admission::Drop);
    assert_eq!(budget.admit(1, start + Duration::from_secs(59)), Admission::Drop);
}

/// A new window starts empty and reports how many lines the last one dropped.
#[test]
fn a_new_window_reports_what_the_last_one_dropped() {
    let mut budget = LogBudget::new(10, Duration::from_secs(60), 10_000);
    let start = Instant::now();
    assert_eq!(budget.admit(10, start), Admission::Write { dropped_before: 0 });
    assert_eq!(budget.admit(5, start), Admission::BudgetReached);
    assert_eq!(budget.admit(5, start), Admission::Drop);
    assert_eq!(budget.admit(5, start), Admission::Drop);
    assert_eq!(budget.admit(5, start + Duration::from_secs(61)), Admission::Write { dropped_before: 3 });
    assert_eq!(budget.admit(5, start + Duration::from_secs(62)), Admission::Write { dropped_before: 0 });
}

/// The total over the life of the watcher is bounded too,
/// however many windows pass.
#[test]
fn the_total_over_the_life_of_the_watcher_is_bounded() {
    let mut budget = LogBudget::new(100, Duration::from_secs(60), 150);
    let start = Instant::now();
    assert_eq!(budget.admit(100, start), Admission::Write { dropped_before: 0 });
    assert_eq!(budget.admit(50, start + Duration::from_secs(61)), Admission::Write { dropped_before: 0 });
    assert_eq!(budget.admit(1, start + Duration::from_secs(122)), Admission::BudgetReached);
    assert_eq!(budget.admit(1, start + Duration::from_secs(183)), Admission::Drop);
}

/// A line larger than the whole budget is never written,
/// and the sum cannot overflow.
#[test]
fn an_oversized_line_is_refused_without_overflow() {
    let mut budget = LogBudget::new(10, Duration::from_secs(60), 10_000);
    assert_eq!(budget.admit(u64::MAX, Instant::now()), Admission::BudgetReached);
}

/// Writing to a standard error that fails does not panic.
/// The test points descriptor 2 at `/dev/full`,
/// where every write fails with `ENOSPC`,
/// in a child process so the test runner keeps its own descriptor.
#[test]
fn a_failed_write_does_not_panic() {
    let status = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("exec 2>/dev/full; exec \"$0\" --exact log_line_tests::the_child_writes_to_a_full_device --ignored --nocapture")
        .arg(std::env::current_exe().expect("this test binary"))
        .stdout(std::process::Stdio::null())
        .status()
        .expect("start the child");
    assert!(status.success(), "the child panicked or failed: {status}");
}

/// Runs only inside the child of the previous test,
/// where standard error is a full device.
#[test]
#[ignore = "runs inside a child with standard error on a full device"]
fn the_child_writes_to_a_full_device() {
    write_unchecked("this write fails and must not panic");
    crate::log_line::log_line("neither must this one");
}

/// Control characters,
/// escape sequences,
/// line breaks and direction marks become visible escapes,
/// and ordinary text is untouched.
#[test]
fn untrusted_text_is_escaped_for_logs_journals_and_terminals() {
    assert_eq!(sanitize("plain text, ünïcode, /user.slice/a-b.scope"), "plain text, ünïcode, /user.slice/a-b.scope");
    assert_eq!(sanitize("a\nb"), "a\\u{000a}b");
    assert_eq!(sanitize("\u{1b}[2J"), "\\u{001b}[2J");
    assert_eq!(sanitize("x\u{7f}y\u{9b}z"), "x\\u{007f}y\\u{009b}z");
    assert_eq!(sanitize("evil\u{202e}gpj.exe"), "evil\\u{202e}gpj.exe");
    assert_eq!(sanitize("a\0b"), "a\\u{0000}b");
    assert!(!sanitize("\u{1b}]0;title\u{7}").contains('\u{1b}'));
}

/// Markup syntax characters become entities for the notification body.
#[test]
fn notification_markup_is_escaped() {
    assert_eq!(escape_markup("<b>x</b> & 'y' \"z\""), "&lt;b&gt;x&lt;/b&gt; &amp; &apos;y&apos; &quot;z&quot;");
    assert_eq!(escape_markup("plain"), "plain");
}
