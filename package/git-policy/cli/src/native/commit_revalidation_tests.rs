//! Controls for the replay helpers of revalidation: the replay directory and the detached-head decision.

use super::*;
use crate::test_support::{fixture, remove};

/// A replay directory that already exists is accepted; any other failure is reported.
#[test]
fn an_existing_replay_directory_is_accepted_and_another_failure_is_not() {
    let root: PathBuf = fixture("revalidation-directory");
    let directory: PathBuf = root.join("replay-1");
    ensure_replay_directory(directory.as_path()).expect("created");
    assert!(directory.is_dir());
    ensure_replay_directory(directory.as_path()).expect("already there");
    assert!(ensure_replay_directory(root.join("missing/replay-2").as_path()).is_err());
    remove(root.as_path());
}

/// Only a detached `HEAD` is detached.
#[test]
fn only_a_detached_head_is_detached() {
    assert!(is_detached(&SymbolicHead::Detached));
    assert!(!is_detached(&SymbolicHead::Branch(String::from("refs/heads/main"))));
}
