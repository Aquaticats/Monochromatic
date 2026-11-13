//! Controls of the start a commit makes: a worktree with no route costs nothing,
//! and one whose folder cannot be made private fails with the words a person can act on.

use super::*;

/// A commit in a worktree with no route is not a start.
#[test]
fn a_worktree_with_no_route_starts_nothing() {
    let mut timing = std::time::Duration::ZERO;
    assert!(!ensure_a_server(None, false, &mut timing).expect("no route is not a failure"), "nothing was provisioned");
    assert_eq!(timing, std::time::Duration::ZERO, "nothing is timed");
}

/// A command that already made the service exist asks nothing more.
#[test]
fn a_service_already_made_is_not_made_again() {
    let mut timing = std::time::Duration::ZERO;
    let host = ServiceHost {
        runtime_dir: std::path::PathBuf::from("/nonexistent-runtime"),
        state_dir: std::path::PathBuf::from("/nonexistent-state"),
        service_binary: std::path::PathBuf::from("/nonexistent-program"),
        git: std::path::PathBuf::from("/usr/bin/git"),
        instance: String::from("3f9a"),
        git_dir: std::path::PathBuf::from("/nonexistent/.git"),
        named_runtime_dir: None,
        bus_address: None,
    };
    assert!(ensure_a_server(Some(&host), true, &mut timing).expect("an already provisioned command starts nothing"));
    assert_eq!(timing, std::time::Duration::ZERO);
}
