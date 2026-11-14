//! Controls for the stop of a worktree that has no route to the commit service:
//! an unreachable Git directory says so,
//! and a reachable one keeps the platform message.

use super::*;
use crate::test_support::{fixture, remove};

/// A Git directory that cannot be resolved is named with its cause,
/// and the message does not say the machine has no way to start a service.
#[test]
fn an_unreachable_git_directory_is_named_as_the_cause() {
    let root: PathBuf = fixture("route-refusal-unreachable");
    let git_dir: PathBuf = root.join("absent/.git");
    let TransactionStop::Message(text) = refusal_without_route(git_dir.as_path()) else { panic!("a message expected") };
    assert!(text.contains(git_dir.to_string_lossy().as_ref()), "{text}");
    assert!(text.contains("cannot be resolved"), "{text}");
    assert_ne!(text, crate::service_messages::render(&ClientFailure::NoActivation(platform_without_manager())), "the platform message is the wrong cause");
    remove(root.as_path());
}

/// A Git directory that resolves keeps the platform message:
/// there the missing route is the machine's.
#[test]
fn a_reachable_git_directory_keeps_the_platform_message() {
    let root: PathBuf = fixture("route-refusal-reachable");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    let TransactionStop::Client(failure) = refusal_without_route(git_dir.as_path()) else { panic!("the platform failure expected") };
    assert_eq!(failure, ClientFailure::NoActivation(platform_without_manager()));
    remove(root.as_path());
}
