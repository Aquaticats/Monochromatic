//! Verifies which watcher log lines are relayed to the terminal that ran the start.

/// Relay selection under test.
use crate::startup_relay::{failure_lines_since, relayed_text};
/// The marker of a notice about the log itself.
use crate::log_line::LOG_NOTICE_MARKER;
/// The marker the producer of loud reports uses.
use crate::update_report::policy_failure;

/// Only loud lines written after the start began are relayed.
#[test]
fn only_new_loud_lines_are_relayed() {
    let old = format!("{}\n", policy_failure("an earlier run failed"));
    let notice = "wg-quicker-exempt: POLICY NOTICE. By the repository's policy this traffic leaves outside the tunnel\n";
    let fresh = format!("{notice}{}\nmoved pid 5\n", policy_failure("the updates slice cannot be created"));
    let log = format!("{old}{fresh}");
    let offset = u64::try_from(old.len()).expect("fits");
    let lines = failure_lines_since(&log, offset);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("the updates slice cannot be created"));
    assert_eq!(failure_lines_since(&log, 0).len(), 2);
    assert!(failure_lines_since(&log, u64::try_from(log.len()).expect("fits")).is_empty());
    assert!(failure_lines_since(&log, u64::MAX).is_empty());
    assert!(failure_lines_since("", 0).is_empty());
}

/// A notice that the log dropped lines is relayed next to the loud reports,
/// because a dropped line may have been a report.
#[test]
fn log_limit_notices_are_relayed_with_the_loud_reports() {
    let log = format!(
        "{}\nwg-quicker-exempt: {LOG_NOTICE_MARKER}: 12 log line(s) were dropped because the log budget was reached\nmoved pid 5\n",
        policy_failure("a process was skipped")
    );
    let lines = failure_lines_since(&log, 0);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[1].contains("were dropped"), "{lines:?}");
}

/// A start with more findings than the terminal shows prints the first ones,
/// escaped,
/// and says how many are left in the log;
/// a start with fewer prints them all and no count.
#[test]
fn a_relay_of_many_lines_is_bounded_and_says_how_many_are_in_the_log() {
    let lines: Vec<String> = (0..250).map(|index| return policy_failure(&format!("finding number {index}"))).collect();
    let printed = relayed_text(&lines, "/run/wg-quicker-exempt/watchers/x/log");
    assert_eq!(printed.len(), 201);
    assert!(printed[0].contains("finding number 0"));
    assert!(printed[199].contains("finding number 199"));
    assert!(printed[200].contains("50 more loud line(s)"), "{}", printed[200]);
    assert!(printed[200].contains("/run/wg-quicker-exempt/watchers/x/log"));
    let few = relayed_text(&lines[..3], "/log");
    assert_eq!(few.len(), 3);
    let hostile = vec!["wg-quicker-exempt: POLICY NOT IN FORCE: name \u{1b}[2J".to_string()];
    assert!(!relayed_text(&hostile, "/log")[0].contains('\u{1b}'));
    assert!(relayed_text(&[], "/log").is_empty());
}
