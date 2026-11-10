//! Controls for the destination a refused `git worktree add` left:
//! an empty folder this command made is removed without a word,
//! and a folder that
//!  holds anything stays and is named.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_subvolume::{Reservation, reserve_destination};
use std::path::PathBuf;

/// A destination this command made and nothing wrote into is removed,
/// and the person reads nothing about it.
#[test]
fn an_empty_destination_is_removed_without_a_line() {
    let root: PathBuf = fixture("refused-destination-empty");
    let destination: PathBuf = root.join("topic");
    let Reservation::Mine(made) = reserve_destination(destination.as_path()) else { panic!("the first creation owns its destination") };
    assert_eq!(settle_refused_destination(&made), None, "a removal needs no words");
    assert!(!destination.exists(), "the folder this command made is gone");
    remove(root.as_path());
}

/// A destination this command made that holds anything stays with everything in it,
/// and the line names the path.
#[test]
fn a_destination_that_holds_something_stays_and_is_named() {
    let root: PathBuf = fixture("refused-destination-holding");
    let destination: PathBuf = root.join("topic");
    let Reservation::Mine(made) = reserve_destination(destination.as_path()) else { panic!("the first creation owns its destination") };
    std::fs::write(destination.join("someone-elses-file"), b"theirs").expect("another program's file");
    let line: Option<String> = settle_refused_destination(&made);
    let said: String = line.expect("a folder that stays is named");
    assert!(said.contains("left in place"), "{said}");
    assert!(said.contains(destination.to_str().expect("utf8 destination")), "the line names the path: {said}");
    assert_eq!(std::fs::read(destination.join("someone-elses-file")).expect("the other program's file"), b"theirs");
    remove(root.as_path());
}

/// A directory that replaced the one this command made is neither removed nor named:
/// this command has nothing to say about a folder it did not make.
#[test]
fn a_replacement_directory_is_left_without_a_line() {
    let root: PathBuf = fixture("refused-destination-replaced");
    let destination: PathBuf = root.join("topic");
    let moved: PathBuf = root.join("topic.moved");
    let Reservation::Mine(made) = reserve_destination(destination.as_path()) else { panic!("the first creation owns its destination") };
    std::fs::rename(destination.as_path(), moved.as_path()).expect("move the created directory away");
    std::fs::create_dir(destination.as_path()).expect("another directory at the same name");
    assert_eq!(settle_refused_destination(&made), None);
    assert!(destination.is_dir(), "the replacement is not ours to remove");
    assert!(moved.is_dir(), "neither is the moved one");
    remove(root.as_path());
}
