//! Verifies the start notice,
//! the loud failure reports,
//! and the wording of every outcome.

/// The slice name.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Why a process matched.
use crate::update_entry_points::MatchReason;
/// Wording under test.
use crate::update_report::{describe_outcome, policy_failure, policy_notice, Outcome, POLICY_NOT_IN_FORCE_MARKER};
/// Fixture paths.
use std::path::PathBuf;
/// Retry waits.
use std::time::Duration;

/// Makes install roots with the given tool directories.
fn roots_with(name: &str, tools: &[&str]) -> Vec<PathBuf> {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-report-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let root = base.join("toolchain");
    std::fs::create_dir_all(&root).expect("make the root");
    for tool in tools {
        std::fs::create_dir_all(root.join(tool)).expect("make the tool directory");
    }
    return vec![root, base.join("toolchain-global")];
}

/// The start notice names every kind of traffic that leaves,
/// says there is no switch,
/// what the toolchain root rule covers,
/// what can imitate a match,
/// and what is not covered.
#[test]
fn the_start_notice_names_the_traffic_and_the_absence_of_a_switch() {
    let notice = policy_notice(&roots_with("notice", &["node", "uv"]));
    for phrase in [
        "POLICY NOTICE",
        "without any switch",
        "Update entry points",
        "mpm-shadow install, update and outdated",
        "install roots",
        "monochromatic-dev data directory",
        "update-host list",
        "outside the tunnel",
        "every script and dev server run by the toolchain's node",
        "Installed on this machine now: node, uv.",
        "what can imitate it",
        "the executable's file name (mise, mpm-shadow, mpm, node)",
        "any executable named mpm",
        "inside a home directory the user can write",
        "another pid namespace",
        "whole address",
        "shared CDN address",
    ] {
        assert!(notice.contains(phrase), "missing {phrase:?} in {notice}");
    }
}

/// With nothing installed the notice says so instead of listing nothing.
#[test]
fn the_notice_says_when_nothing_is_installed() {
    let notice = policy_notice(&[PathBuf::from("/nonexistent/toolchain")]);
    assert!(notice.contains("Installed on this machine now: nothing yet."), "{notice}");
}

/// A failure report names the policy that is not in force.
#[test]
fn a_failure_report_names_the_policy_not_in_force() {
    let report = policy_failure("the updates slice cannot be created: busctl exited with 1");
    assert!(report.contains(POLICY_NOT_IN_FORCE_MARKER));
    assert!(report.contains("process watch"));
    assert!(report.contains("busctl exited with 1"));
}

/// Quiet outcomes produce no log line,
/// an adoption names the slice and anything left behind,
/// and every problem is a failure.
#[test]
fn only_adoptions_and_problems_are_logged() {
    for quiet in [Outcome::Ignored, Outcome::AlreadyInSlice, Outcome::Deferred] {
        assert_eq!(describe_outcome(&quiet), None);
    }
    let adopted = Outcome::Adopted { pid: 5, reason: MatchReason::MpmShim, count: 2, left_behind: 3 };
    let line = describe_outcome(&adopted).expect("adoption is logged");
    assert!(line.contains(TOOLCHAIN_UPDATES_SLICE));
    assert!(line.contains("pid 5"));
    assert!(line.contains("1 descendant(s)"));
    assert!(line.contains("3 other process(es)"));
    let problems = [
        Outcome::Skipped { pid: 1, why: "outside the subtree".to_string() },
        Outcome::Failed { pid: 2, error: "refused".to_string(), attempt: 2, retry_in: Duration::from_secs(2) },
        Outcome::Unreadable { pid: 3, error: "permission denied".to_string() },
        Outcome::ScanFailed { error: "no procfs".to_string() },
    ];
    for problem in &problems {
        let text = describe_outcome(problem).expect("a problem is reported");
        assert!(text.contains(POLICY_NOT_IN_FORCE_MARKER), "{text}");
    }
    let failed = describe_outcome(&problems[1]).expect("reported");
    assert!(failed.contains("attempt 2"));
    assert!(failed.contains("next attempt in 2s"));
}
