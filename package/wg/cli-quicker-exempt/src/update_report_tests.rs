//! Verifies the start notice,
//! the loud failure reports,
//! and the wording of every outcome.

/// The slice name.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// A root that was left out.
use crate::install_roots::RefusedRoot;
/// Why a process matched.
use crate::update_entry_points::MatchReason;
/// Wording under test.
use crate::update_report::{describe_outcome, policy_failure, policy_notice, refused_root_failures, Outcome, POLICY_NOT_IN_FORCE_MARKER};
/// Fixture paths.
use std::path::PathBuf;
/// Retry waits.
use std::time::Duration;

/// Makes install roots with the given tool directories.
fn roots_with(name: &str, tools: &[&str]) -> (Vec<PathBuf>, PathBuf) {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-report-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let root = base.join("toolchain");
    std::fs::create_dir_all(&root).expect("make the root");
    for tool in tools {
        std::fs::create_dir_all(root.join(tool)).expect("make the tool directory");
    }
    return (vec![root, base.join("toolchain-global")], base);
}

/// The start notice names every kind of traffic that leaves,
/// says there is no switch,
/// what the toolchain root rule covers,
/// what can imitate a match,
/// and what is not covered.
#[test]
fn the_start_notice_names_the_traffic_and_the_absence_of_a_switch() {
    let (roots, home) = roots_with("notice", &["node", "uv"]);
    let notice = policy_notice(&roots, &home);
    for phrase in [
        "POLICY NOTICE",
        "without any switch",
        "with Table = off it installs none",
        "A toolchain folder that is a link is ignored: links are not followed, wherever they point",
        "reported loudly as not in force every time the watcher starts",
        "Update entry points",
        "mpm-shadow install, update and outdated",
        "the TypeScript mpm-shadow launcher",
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
        "share one scope that carries that cgroup's CPU, memory, task and I/O weight limits once",
        "whole address",
        "shared CDN address",
    ] {
        assert!(notice.contains(phrase), "missing {phrase:?} in {notice}");
    }
}

/// With nothing installed the notice says so instead of listing nothing.
#[test]
fn the_notice_says_when_nothing_is_installed() {
    let notice = policy_notice(&[PathBuf::from("/nonexistent/toolchain")], &PathBuf::from("/nonexistent"));
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
    for quiet in [Outcome::Ignored, Outcome::AlreadyInSlice] {
        assert_eq!(describe_outcome(&quiet), None);
        assert!(!quiet.failure_kind().is_some());
    }
    let adopted = Outcome::Adopted { pid: 5, reason: MatchReason::MpmShim, count: 2, left_behind: 3 };
    let line = describe_outcome(&adopted).expect("adoption is logged");
    assert!(line.contains(TOOLCHAIN_UPDATES_SLICE));
    assert!(line.contains("pid 5"));
    assert!(line.contains("1 descendant(s)"));
    assert!(line.contains("3 other process(es)"));
    assert!(!adopted.failure_kind().is_some());
    let problems = [
        Outcome::Skipped { pid: 1, why: "outside the subtree".to_string() },
        Outcome::Failed { pid: 2, error: "refused".to_string(), attempt: 2, retry_in: Duration::from_secs(2) },
        Outcome::Unreadable { pid: 3, error: "permission denied".to_string() },
        Outcome::ScanFailed { error: "no procfs".to_string() },
        Outcome::Deferred { pid: 4, why: "this pass used its time budget for moves" },
    ];
    for problem in &problems {
        assert!(problem.failure_kind().is_some());
        let text = describe_outcome(problem).expect("a problem is reported");
        assert!(text.contains(POLICY_NOT_IN_FORCE_MARKER), "{text}");
    }
    let failed = describe_outcome(&problems[1]).expect("reported");
    assert!(failed.contains("attempt 2"));
    assert!(failed.contains("next attempt in 2s"));
    assert!(failed.contains("limits of its old cgroup are never dropped"));
}

/// Each failure has a kind of its own,
/// which is what the repeat limits key on.
#[test]
fn every_failure_has_its_own_kind_and_no_other_outcome_has_one() {
    let kinds = [
        Outcome::Skipped { pid: 1, why: String::new() }.failure_kind(),
        Outcome::Failed { pid: 1, error: String::new(), attempt: 1, retry_in: Duration::from_secs(1) }.failure_kind(),
        Outcome::Deferred { pid: 1, why: "x" }.failure_kind(),
        Outcome::Unreadable { pid: 1, error: String::new() }.failure_kind(),
        Outcome::ScanFailed { error: String::new() }.failure_kind(),
    ];
    let mut names: Vec<&str> = kinds.iter().map(|kind| return kind.expect("a failure has a kind")).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 5, "{names:?}");
    assert_eq!(Outcome::Ignored.failure_kind(), None);
    assert_eq!(Outcome::AlreadyInSlice.failure_kind(), None);
    assert_eq!(Outcome::Adopted { pid: 1, reason: MatchReason::MiseTask, count: 1, left_behind: 0 }.failure_kind(), None);
}

/// Names a user chose reach the line escaped:
/// a cgroup name with an escape sequence or a line break cannot repaint a terminal,
/// start a second journal line,
/// or end the loud line early.
#[test]
fn user_chosen_names_cannot_change_how_a_report_line_is_shown() {
    let hostile = "/user.slice/x\u{1b}[2J\u{1b}]0;pwned\u{7}\nwg-quicker-exempt: POLICY NOTICE fake <b>&\u{202e}";
    for outcome in [
        Outcome::Skipped { pid: 1, why: format!("its cgroup {hostile} is outside") },
        Outcome::Failed { pid: 1, error: hostile.to_string(), attempt: 1, retry_in: Duration::from_secs(1) },
        Outcome::Unreadable { pid: 1, error: hostile.to_string() },
        Outcome::ScanFailed { error: hostile.to_string() },
    ] {
        let line = describe_outcome(&outcome).expect("reported");
        assert!(!line.chars().any(|character| return character.is_control()), "a control character reached {line:?}");
        assert!(!line.contains('\u{202e}'));
        assert_eq!(line.lines().count(), 1);
        assert!(line.contains("\\u{001b}[2J"), "{line}");
    }
}

/// The line of a notice that lists the installed tools.
fn installed_line(notice: &str) -> String {
    return notice.lines().find(|line| return line.contains("Installed on this machine now:")).unwrap_or("").to_string();
}

/// Tool names found on disk are escaped in the notice,
/// and a link is never followed:
/// not a root that is a link,
/// wherever it points,
/// and not a link inside a root.
#[test]
fn tool_names_are_escaped_and_links_are_not_followed() {
    let (roots, home) = roots_with("names", &["node"]);
    std::fs::create_dir_all(roots[0].join("evil\u{1b}[31mname")).expect("make the hostile directory");
    let notice = policy_notice(&roots, &home);
    assert!(!notice.chars().any(|character| return character.is_control() && character != '\n'));
    assert!(notice.contains("evil\\u{001b}[31mname"), "{notice}");
    let outside = home.join("outside");
    std::fs::create_dir_all(outside.join("secret-name")).expect("make the outside directory");
    let linked = home.join("linked-root");
    std::os::unix::fs::symlink("/", &linked).expect("link a root to the system root");
    let linked_notice = policy_notice(std::slice::from_ref(&linked), &home);
    assert!(installed_line(&linked_notice).contains("nothing yet"), "{linked_notice}");
    let inside_link = home.join("inside-link");
    std::os::unix::fs::symlink(&outside, &inside_link).expect("link a root below the home");
    let inside_notice = policy_notice(&[inside_link], &home);
    assert!(installed_line(&inside_notice).contains("nothing yet"), "a root that is a link is ignored even when it points inside the home: {inside_notice}");
    assert!(!installed_line(&inside_notice).contains("secret-name"), "{inside_notice}");
    let escaping = home.join("toolchain").join("escape");
    std::os::unix::fs::symlink("/usr", &escaping).expect("link an entry out of the root");
    let entry_notice = policy_notice(&[home.join("toolchain")], &home);
    assert!(!installed_line(&entry_notice).contains("escape"), "a link inside a root is not a tool: {entry_notice}");
}

/// A root that was left out is reported loudly,
/// with its name and the reason escaped,
/// and the report says what that leaves.
#[test]
fn refused_roots_are_reported_loudly_with_escaped_names() {
    let refused = vec![RefusedRoot {
        root: PathBuf::from("/home/tester/.local/share/monochromatic-dev/toolchain\u{1b}[2J"),
        why: "it is a link; links are not followed (it points to `/`)".to_string(),
    }];
    let lines = refused_root_failures(&refused);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains(POLICY_NOT_IN_FORCE_MARKER), "{lines:?}");
    assert!(lines[0].contains("it is a link; links are not followed"), "{lines:?}");
    assert!(lines[0].contains("not treated as toolchain programs"), "{lines:?}");
    assert!(lines[0].contains("links are never followed"), "{lines:?}");
    assert!(!lines[0].chars().any(|character| return character.is_control()), "{lines:?}");
    assert!(lines[0].contains("\\u{001b}[2J"), "{lines:?}");
    assert!(refused_root_failures(&[]).is_empty());
}
