//! Controls of the proof that a tree's bytes stand in another.

use super::*;
use crate::subvolume_sync_pass_state_and_checks::PARTIAL_NAME;
use crate::test_support::{fixture, remove};

/// A source tree and a copy of it beside it.
fn two_trees(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let source: PathBuf = parent.join("source");
    let destination: PathBuf = parent.join("destination");
    std::fs::create_dir_all(source.join("deep").as_path()).expect("source");
    std::fs::create_dir_all(&destination).expect("destination");
    std::fs::write(source.join("file.txt"), b"contents\n").expect("file");
    std::fs::write(source.join("deep/leaf"), b"a leaf\n").expect("leaf");
    std::os::unix::fs::symlink("file.txt", source.join("link")).expect("link");
    return (parent, source, destination);
}

/// Copy every entry of the source into the destination, without attributes and without the pass.
fn copy_all(source: &Path, destination: &Path) {
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        for item in std::fs::read_dir(source.join(&relative)).expect("listing").flatten() {
            let child: PathBuf = relative.join(item.file_name());
            let kind = item.file_type().expect("kind");
            if kind.is_dir() {
                std::fs::create_dir_all(destination.join(&child)).expect("directory");
                pending.push(child);
            } else if kind.is_symlink() {
                std::os::unix::fs::symlink(std::fs::read_link(source.join(&child)).expect("target"), destination.join(&child)).expect("link");
            } else {
                std::fs::copy(source.join(&child), destination.join(&child)).expect("file");
            }
        }
    }
}

/// A copy of every byte of the source proves the tree.
#[test]
fn a_tree_whose_bytes_stand_in_the_other_is_proved() {
    let (parent, source, destination) = two_trees("proof-proved");
    copy_all(source.as_path(), destination.as_path());
    assert_eq!(entries_not_in(source.as_path(), destination.as_path(), &BTreeSet::new()).expect("proof"), Vec::<PathBuf>::new());
    remove(parent.as_path());
}

/// One changed byte of the same size,
/// one file the other tree lacks,
/// one link with another target,
/// and one entry of another kind are each named.
#[test]
fn every_kind_of_difference_is_named() {
    let (parent, source, destination) = two_trees("proof-differences");
    copy_all(source.as_path(), destination.as_path());
    // The same size, so only the bytes tell; and a modification time that says nothing.
    std::fs::write(source.join("file.txt"), b"CONTENTS\n").expect("a rewrite of the same size");
    std::fs::write(source.join("added.txt"), b"only here\n").expect("an added file");
    std::fs::remove_file(source.join("deep/leaf")).expect("a file the copy lacks");
    std::fs::remove_file(destination.join("link")).expect("remove the link");
    std::os::unix::fs::symlink("deep", destination.join("link")).expect("another target");
    let mut unproved: Vec<PathBuf> = entries_not_in(source.as_path(), destination.as_path(), &BTreeSet::new()).expect("proof");
    unproved.sort();
    let mut wanted: Vec<PathBuf> = vec![PathBuf::from("file.txt"), PathBuf::from("added.txt"), PathBuf::from("link")];
    wanted.sort();
    assert_eq!(unproved, wanted);
    remove(parent.as_path());
}

/// The Git index and this program's own bookkeeping are not a person's bytes and are not proved.
#[test]
fn the_git_index_and_the_programs_own_bookkeeping_are_not_proved() {
    let (parent, source, destination) = two_trees("proof-ours");
    copy_all(source.as_path(), destination.as_path());
    std::fs::create_dir_all(source.join(".git/cli-git-lock-holders").as_path()).expect("git directory");
    std::fs::create_dir_all(destination.join(".git").as_path()).expect("git directory");
    std::fs::write(source.join(".git/index"), b"an index the migration rewrote\n").expect("index");
    let identity: String = crate::lock_holder_records::identity_of_birth("linux:4242");
    std::fs::write(source.join(".git/cli-git-lock-holders/4242.0f3a"), crate::lock_holder_records::record_text(&[(crate::lock_holder_records::SELF_KIND, 4242, identity)])).expect("record");
    let unproved: Vec<PathBuf> = entries_not_in(source.as_path(), destination.as_path(), &BTreeSet::new()).expect("proof");
    assert_eq!(unproved, Vec::<PathBuf>::new(), "neither the index nor a lock-holder record is proved");
    remove(parent.as_path());
}

/// A path a person accepted is not named,
/// and the partial copy this program writes for itself is not proved.
#[test]
fn an_accepted_path_and_the_partial_copy_are_not_named() {
    let (parent, source, destination) = two_trees("proof-accepted");
    copy_all(source.as_path(), destination.as_path());
    std::fs::write(source.join("accepted.txt"), b"the person said it may go\n").expect("accepted");
    std::fs::write(source.join(PARTIAL_NAME), b"half a copy\n").expect("partial");
    let mut accepted: BTreeSet<PathBuf> = BTreeSet::new();
    accepted.insert(PathBuf::from("accepted.txt"));
    assert_eq!(entries_not_in(source.as_path(), destination.as_path(), &accepted).expect("proof"), Vec::<PathBuf>::new());
    remove(parent.as_path());
}

/// A file whose copy is a pipe is compared without waiting on it.
#[test]
fn a_copy_that_is_a_pipe_is_not_waited_on() {
    let (parent, source, destination) = two_trees("proof-pipe");
    copy_all(source.as_path(), destination.as_path());
    std::fs::remove_file(destination.join("file.txt")).expect("remove the copy");
    let made = std::process::Command::new("/usr/bin/mkfifo").arg(destination.join("file.txt")).status().expect("mkfifo");
    assert!(made.success());
    assert_eq!(entries_not_in(source.as_path(), destination.as_path(), &BTreeSet::new()).expect("proof"), vec![PathBuf::from("file.txt")]);
    remove(parent.as_path());
}
