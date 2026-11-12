//! What:
//!  Controls of the push trigger's record:
//! who and where it names,
//! its level,
//! and the masking of its detail.
//! Why:
//!  The file is shared by every worktree of a repository,
//! so a line that names no worktree cannot say whose merge failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(JSON.parse(triggerLine(decision)).worktree).toBe('/work/linked');
//! ```

use super::{Decision, Origin, Outcome, TRIGGER_FILE, level_of, record_decision, trigger_line};
use crate::command_test_support::{fixture, remove};
use crate::service_instance::instance_of;
use std::path::{Path, PathBuf};

/// A decision with the fields the controls do not vary.
fn decision<'a>(origin: Option<Origin<'a>>, outcome: Outcome, reason: &'a str, detail: &'a str) -> Decision<'a> {
    return Decision { common_dir: None, origin, command: "merge", outcome, reason, detail, branch: Some("refs/heads/work") };
}

/// The line as JSON.
fn parsed(line: &str) -> serde_json::Value {
    assert_eq!(line.matches('\n').count(), 1, "one line: {line:?}");
    return serde_json::from_str(line.trim_end()).expect("JSON");
}

/// A line names the worktree,
/// its Git directory and the service instance that Git directory names,
/// so two worktrees' lines
/// in the one shared file tell apart.
#[test]
fn a_line_names_the_worktree_that_made_the_decision() {
    let first: Origin<'_> = Origin { worktree: Path::new("/work/one"), git_dir: Path::new("/repo/.git/worktrees/one") };
    let second: Origin<'_> = Origin { worktree: Path::new("/work/two"), git_dir: Path::new("/repo/.git/worktrees/two") };
    let one = parsed(trigger_line(&decision(Some(first), Outcome::Declined, "command-failed", "Git exited with status 1")).as_str());
    let two = parsed(trigger_line(&decision(Some(second), Outcome::Declined, "command-failed", "Git exited with status 1")).as_str());
    assert_eq!(one["worktree"], "/work/one");
    assert_eq!(one["gitDir"], "/repo/.git/worktrees/one");
    assert_eq!(one["instance"], instance_of(Path::new("/repo/.git/worktrees/one")));
    assert_ne!(one["instance"], two["instance"]);
    assert_eq!(one["branch"], "refs/heads/work");
    assert!(parsed(trigger_line(&decision(None, Outcome::Declined, "x", "")).as_str()).get("worktree").is_none());
}

/// A decision the wrapper could not make,
/// or a hand-over that failed,
/// is `warn`;
/// a decision working as intended is `info`.
#[test]
fn a_decision_the_wrapper_could_not_make_stands_out_by_level() {
    assert_eq!(level_of(Outcome::Declined, "handover-failed"), "warn");
    assert_eq!(level_of(Outcome::Declined, "head-unreadable"), "warn");
    assert_eq!(level_of(Outcome::Declined, "no-route"), "warn");
    assert_eq!(level_of(Outcome::Declined, "tip-unchanged"), "info");
    assert_eq!(level_of(Outcome::Declined, "command-failed"), "info");
    assert_eq!(level_of(Outcome::HandedOver, "handover-failed"), "info", "a hand-over that happened is not a failure");
}

/// The userinfo of a URL in the detail never reaches the file.
#[test]
fn a_url_in_the_detail_is_stored_without_its_credentials() {
    let line: String = trigger_line(&decision(None, Outcome::Declined, "handover-failed", "push to https://user:s3cret@example.invalid/r.git failed"));
    assert!(!line.contains("s3cret") && !line.contains("user:"), "{line}");
    assert!(line.contains("https://<credentials>@example.invalid/<credentials>"), "{line}");
}

/// The record lands in the common Git directory's shared file,
/// one line per decision.
#[test]
fn a_recorded_decision_is_one_line_in_the_shared_file() {
    let root: PathBuf = fixture("trigger-record-file");
    let origin: Origin<'_> = Origin { worktree: root.as_path(), git_dir: root.as_path() };
    let mut recorded: Decision<'_> = decision(Some(origin), Outcome::HandedOver, "ahead-of-upstream", "pushed abc of refs/heads/work");
    recorded.common_dir = Some(root.as_path());
    record_decision(&recorded);
    record_decision(&recorded);
    let text: String = std::fs::read_to_string(root.join(TRIGGER_FILE)).expect("the file");
    assert_eq!(text.lines().count(), 2);
    assert!(text.lines().all(|line| { return serde_json::from_str::<serde_json::Value>(line).is_ok(); }), "{text}");
    remove(root.as_path());
}
