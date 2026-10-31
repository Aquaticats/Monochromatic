//! Verifies which watcher log lines are relayed to the terminal that ran the start.

/// Relay selection under test.
use crate::application_watch_service::failure_lines_since;
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
