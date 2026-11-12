//! Controls for the stop of a worktree that has no route to the commit service:
//! an unreachable Git directory says so,
//! and a reachable one names the missing runtime directory.

use super::*;
use crate::test_support::{fixture, remove};

/// A Git directory that cannot be resolved is named with its cause,
/// and the message does not send the person to a service manager.
#[test]
fn an_unreachable_git_directory_is_named_as_the_cause() {
    let root: PathBuf = fixture("route-refusal-unreachable");
    let git_dir: PathBuf = root.join("absent/.git");
    let TransactionStop::Message(text) = refusal_without_route(git_dir.as_path()) else { panic!("a message expected") };
    assert!(text.contains(git_dir.to_string_lossy().as_ref()), "{text}");
    assert!(text.contains("cannot be resolved"), "{text}");
    assert!(text.contains("XDG_RUNTIME_DIR"), "the message names the setting that is missing: {text}");
    remove(root.as_path());
}

/// A Git directory that resolves says the account names no runtime directory:
/// there the missing route is the environment's.
#[test]
fn a_reachable_git_directory_keeps_the_platform_message() {
    let root: PathBuf = fixture("route-refusal-reachable");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    let TransactionStop::Message(text) = refusal_without_route(git_dir.as_path()) else { panic!("a message expected") };
    assert!(text.contains("XDG_RUNTIME_DIR"), "{text}");
    remove(root.as_path());
}
