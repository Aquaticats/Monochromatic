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

/// What:
///  A start given a bound shorter than the commit's start wait ends within that bound.
/// Why:
///  The review of `567ae37198` (finding 5): a removal or a relink hand-over sets its own two-second bound after the start it
///  was meant to cap, so a slow start spent the commit's ten seconds before the shorter bound existed anywhere.
///  The bound is given to the start here, and a program that never binds spends exactly it.
#[test]
fn a_start_given_a_shorter_bound_ends_within_it() {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join("start-within-shorter");
    let _ = std::fs::remove_dir_all(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("fixture folder");
    let mut socket: std::path::PathBuf = root.join("monochromatic-git-policy-cli");
    std::fs::create_dir_all(socket.as_path()).expect("socket folder");
    socket.push("3f9a.sock");
    let server: crate::service_server_start_by_the_wrapper::ServerToStart = crate::service_server_start_by_the_wrapper::ServerToStart {
        address: crate::service_server::Bind::Path(socket.clone()),
        directory: socket.parent().map(std::path::Path::to_path_buf).expect("the socket folder"),
        instance: String::from("3f9a"),
        git_dir: root.join("repo/.git"),
        journal: root.join("journal"),
        git: std::path::PathBuf::from("/usr/bin/git"),
        // A program that runs and exits without binding, so the whole wait is the bound this control gives.
        program: std::path::PathBuf::from("/bin/true"),
        manager: None,
        named_runtime_dir: None,
        bus_address: None,
    };
    let bound: std::time::Duration = std::time::Duration::from_millis(400);
    let began: std::time::Instant = std::time::Instant::now();
    let attempt: crate::service_server_start_by_the_wrapper::StartAttempt = crate::service_server_start_by_the_wrapper::start_within(&server, bound);
    let took: std::time::Duration = began.elapsed();
    assert!(matches!(attempt, crate::service_server_start_by_the_wrapper::StartAttempt::DidNotGreet(_)), "a program that never binds does not greet");
    assert!(took < bound + std::time::Duration::from_secs(1), "the start spends the bound it was given ({took:?}), not the commit's start wait");
    let _ = std::fs::remove_dir_all(root.as_path());
}
