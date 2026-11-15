//! What:
//!  Controls for the record of a landed worktree a sweep kept,
//!  which the next sweep examines again.
//! Why:
//!   A record that a later sweep cannot read leaves a worktree in place for ever,
//!  and one that names another directory would remove something nobody kept,
//!  so both the round trip and the refusal to guess are pinned here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rememberKept(common, '/x/y', 'refs/heads/y', 'process 4242 is working in it');
//! expect(rememberedKept(common)).toEqual([{ worktree: '/x/y', branch: 'refs/heads/y', identity: '1:2 3:4' }]);
//! ```

use super::{KEPT_RETRY_FOLDER, forget_kept, remembered_kept, remember_kept};
use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::test_support::fixture;
use std::path::PathBuf;

#[test]
fn a_kept_worktree_is_remembered_with_its_branch_until_a_later_sweep_reads_it() {
    let common: PathBuf = fixture("kept-retry-round-trip");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "103:4242 103:4343", "process 4242 is working in it");
    let remembered: Vec<super::RememberedKept> = remembered_kept(common.as_path());
    assert_eq!(remembered.len(), 1, "one record for one worktree: {remembered:?}");
    assert_eq!(remembered[0].worktree, worktree, "the record names the worktree");
    assert_eq!(remembered[0].branch, "refs/heads/feature", "and the branch it had checked out");
    assert_eq!(remembered[0].identity, "103:4242 103:4343", "and the two directories the record was written for");
}

#[test]
fn remembering_the_same_worktree_twice_writes_one_record() {
    let common: PathBuf = fixture("kept-retry-twice");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "103:4242 103:4343", "a lock is a promise to keep it");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "103:4242 103:4343", "a bisect is unfinished there");
    let remembered: Vec<super::RememberedKept> = remembered_kept(common.as_path());
    assert_eq!(remembered.len(), 1, "a second sweep of one landing writes one record: {remembered:?}");
    let record: String = std::fs::read_to_string(path_for_reading(common.as_path(), KEPT_RETRY_FOLDER).read_dir().expect("the folder").flatten().next().expect("the record").path()).expect("the record's text");
    assert!(record.contains("a bisect is unfinished there"), "the last reason is the one a person reads: {record}");
}

#[test]
fn forgetting_a_worktree_leaves_no_record_for_it() {
    let common: PathBuf = fixture("kept-retry-forget");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "103:4242 103:4343", "process 4242 is working in it");
    forget_kept(common.as_path(), worktree.as_path());
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "nothing is examined again once the question is answered");
    forget_kept(common.as_path(), worktree.as_path());
}

/// What:
///  A record that names no identity,
///  which is one an earlier build of this shape wrote,
///  names no worktree a sweep may act on.
/// Why:
///  Without the identity nothing tells the worktree the record was written for from one added at the same path,
///  and a sweep that acted on it would remove a worktree nobody asked it to.
#[test]
fn a_record_without_an_identity_names_no_worktree() {
    let common: PathBuf = fixture("kept-retry-no-identity");
    let folder: PathBuf = path_for_reading(common.as_path(), KEPT_RETRY_FOLDER);
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(
        folder.join("0123456789abcdef.txt"),
        b"monochromatic-git-policy-cli kept landed worktree 2\nworktree: /x/y\nbranch: refs/heads/y\n\nprocess 4242 is working in it\n",
    )
    .expect("a record of the shape before the identity");
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "a record with no identity answers for no worktree");
}

#[test]
fn a_file_this_release_did_not_write_names_no_worktree() {
    let common: PathBuf = fixture("kept-retry-foreign");
    let folder: PathBuf = path_for_reading(common.as_path(), KEPT_RETRY_FOLDER);
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("0123456789abcdef.txt"), b"another program's note\n").expect("a file that is not a record");
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "a reader that guessed would remove a folder nobody kept");
}

#[test]
fn a_repository_with_no_such_folder_remembers_nothing() {
    let common: PathBuf = fixture("kept-retry-empty");
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "no folder, no kept worktree");
}

/// What:
///  A named pipe among the records does not hold the reader,
///  and the records that are regular files are read.
/// Why:
///  `remembered_kept` reads every `.txt` under the folder,
///  and a name a process made a named pipe waits inside `open(2)` for a writer that never comes:
///  the person's `git merge` then never returns,
///  which is what a review of this change reproduced with a writerless FIFO (finding 6).
///  Every other record read of this change opens without waiting for a writer
///  (`read_record_here`), and this reader is one of them now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(keptDir + '/stuck.txt'); expect(within(5000, () => rememberedKept(common))).toHaveLength(1);
/// ```
#[test]
fn a_named_pipe_among_the_records_does_not_hold_the_reader() {
    let common: PathBuf = fixture("kept-retry-named-pipe");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "103:4242 103:4343", "a lock is a promise to keep it");
    let directory: PathBuf = path_for_reading(common.as_path(), KEPT_RETRY_FOLDER);
    let pipe: PathBuf = directory.join("stuck.txt");
    let made: std::process::Output = std::process::Command::new("/usr/bin/mkfifo").arg(pipe.as_os_str()).output().expect("mkfifo");
    assert!(made.status.success(), "the named pipe is made: {made:?}");
    let (sender, receiver) = std::sync::mpsc::channel::<usize>();
    let reading_here: PathBuf = common.clone();
    let reader: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let count: usize = remembered_kept(reading_here.as_path()).len();
        let _sent = sender.send(count);
    });
    let counted = receiver.recv_timeout(std::time::Duration::from_secs(5));
    // The read is over before the bound, and the record that is a regular file is the one this asks for.
    assert_eq!(counted, Ok(1), "the records that are files are read and the named pipe is not waited on");
    let _closed = reader.join();
}
