//! Controls for the record beside a worktree that names the objects its conversion may leave at the converting name:
//! its bytes are read back,
//! a link planted at a temporary name is not written through,
//! and a named pipe at the record's name is not opened.

use super::*;
use crate::test_support::{fixture, remove};

/// A worktree path in a fixture,
/// with the record's parent directory made.
fn worktree_in(name: &str) -> PathBuf {
    let parent: PathBuf = fixture(name);
    return parent.join("wt");
}

/// What:
///  A symbolic link planted at the writer's temporary name is not written through:
///  the person's file keeps its bytes.
/// Why:
///   A temporary name that includes only the process number can be guessed and replaced by a link to a file of the
///   person's own,
/// and a plain write there truncates that file before the rename.
#[test]
fn a_link_at_the_writers_temporary_name_is_not_written_through() {
    let worktree: PathBuf = worktree_in("leftover-record-link");
    let record: PathBuf = record_path(worktree.as_path()).expect("the record's path");
    let notes: PathBuf = worktree.parent().expect("parent").join("notes.txt");
    std::fs::write(notes.as_path(), b"a person's notes\n").expect("their file");
    let planted: PathBuf = record.with_extension(format!("json.{}", std::process::id()));
    std::os::unix::fs::symlink(notes.as_path(), planted.as_path()).expect("a link at the temporary name");
    let stage = LeftoverStage { original: ObjectId { device: 1, inode: 2, birth: None }, created: None };
    record_leftover_stage(worktree.as_path(), &stage);
    assert_eq!(std::fs::read(notes.as_path()).expect("their file"), b"a person's notes\n", "nothing is written through the link");
    let read: LeftoverStage = leftover_stage_of(worktree.as_path()).expect("the record is written");
    assert_eq!(read.original, stage.original);
    assert_eq!(read.created, None);
    remove(worktree.parent().expect("parent").to_path_buf().as_path());
}

/// What:
///  A named pipe at the record's name is not opened:
///  the record reads as absent,
///  so the stage it might have named is refused rather than waited for.
#[test]
fn a_named_pipe_at_the_records_name_is_not_opened() {
    let worktree: PathBuf = worktree_in("leftover-record-pipe");
    let record: PathBuf = record_path(worktree.as_path()).expect("the record's path");
    std::fs::create_dir_all(worktree.parent().expect("parent")).expect("the worktree's parent");
    let name = std::ffi::CString::new(record.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and outlives the call; mode 0600 is this test's own.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "the named pipe");
    let (sender, receiver) = std::sync::mpsc::channel();
    let scanned: PathBuf = worktree.clone();
    let runner = std::thread::spawn(move || {
        let _sent = sender.send(leftover_stage_of(scanned.as_path()));
    });
    let read: Option<LeftoverStage> = match receiver.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(found) => found,
        Err(_still_waiting) => panic!("the record was opened as a named pipe instead of being refused"),
    };
    runner.join().expect("the read ends");
    assert!(read.is_none(), "a named pipe proves nothing about a stage");
    assert!(record.exists(), "the named pipe itself is left alone");
    remove(worktree.parent().expect("parent").to_path_buf().as_path());
}
