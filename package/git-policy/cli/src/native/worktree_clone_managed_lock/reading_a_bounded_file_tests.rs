//! What:
//!  Controls for the bounded read:
//!  bytes,
//!  absence,
//!  a link,
//!  a directory,
//!  a named pipe,
//!  and a file past the bound.
//! Why:
//!  Every one of these is a state the lock's reader reports differently,
//!  and a reader that followed a link or waited on a pipe would hold a command forever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readABoundedFile(link, 99).kind).toBe('refused');
//! ```

use super::{ARefusedReading, Reading, read_a_bounded_file};
use crate::worktree_clone_managed_lock::test_fixtures::Fixture;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt as _;

/// A path a control may make a named pipe at.
fn a_named_pipe_at(path: &std::path::Path) {
    let name: CString = CString::new(path.as_os_str().as_bytes()).expect("the fixture path holds no NUL");
    // SAFETY: `mkfifo` takes a valid C string and a mode and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "mkfifo: {}", std::io::Error::last_os_error());
}

/// The bytes one file holds read as the bytes they are.
#[test]
fn the_bytes_of_a_regular_file_are_read() {
    let fixture: Fixture = Fixture::new("reading-bytes");
    let path: std::path::PathBuf = fixture.root.join("words");
    std::fs::write(path.as_path(), b"held\n").expect("the file is written");
    assert_eq!(read_a_bounded_file(path.as_path(), 99), Reading::Bytes(Vec::from(b"held\n")));
}

/// A missing name is absence and never a refusal,
/// because a missing lock and a lock the read refuses are different states.
#[test]
fn a_missing_name_is_absence() {
    let fixture: Fixture = Fixture::new("reading-absent");
    assert_eq!(read_a_bounded_file(fixture.root.join("missing").as_path(), 99), Reading::Absent);
}

/// A link is refused rather than followed,
/// and the cause is its own.
#[test]
fn a_link_is_refused() {
    let fixture: Fixture = Fixture::new("reading-link");
    let target: std::path::PathBuf = fixture.root.join("target");
    std::fs::write(target.as_path(), b"held\n").expect("the target is written");
    let link: std::path::PathBuf = fixture.root.join("link");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("the link is made");
    assert_eq!(read_a_bounded_file(link.as_path(), 99), Reading::Refused(ARefusedReading::ALink));
}

/// A directory is refused,
/// and a named pipe is refused without waiting on a writer.
#[test]
fn a_directory_and_a_named_pipe_are_refused() {
    let fixture: Fixture = Fixture::new("reading-directory-and-pipe");
    assert_eq!(read_a_bounded_file(fixture.root.as_path(), 99), Reading::Refused(ARefusedReading::ADirectory));
    let pipe: std::path::PathBuf = fixture.root.join("pipe");
    a_named_pipe_at(pipe.as_path());
    assert_eq!(read_a_bounded_file(pipe.as_path(), 99), Reading::Refused(ARefusedReading::ANamedPipe));
}

/// A file past the bound is refused with both numbers,
/// so a person reads what stands and what the reader allows.
#[test]
fn a_file_past_the_bound_is_refused() {
    let fixture: Fixture = Fixture::new("reading-too-large");
    let path: std::path::PathBuf = fixture.root.join("long");
    std::fs::write(path.as_path(), b"0123456789").expect("the file is written");
    assert_eq!(read_a_bounded_file(path.as_path(), 4), Reading::Refused(ARefusedReading::TooManyBytes { found: 10, bound: 4 }));
}

/// Every cause reads as its own sentence,
/// so a person is told a link from a pipe.
#[test]
fn every_cause_has_its_own_words() {
    let causes: [ARefusedReading; 5] = [
        ARefusedReading::ALink,
        ARefusedReading::ADirectory,
        ARefusedReading::ANamedPipe,
        ARefusedReading::AFileOfAnotherKind,
        ARefusedReading::TooManyBytes { found: 10, bound: 4 },
    ];
    let mut said: Vec<String> = Vec::new();
    for cause in causes {
        let words: String = cause.words();
        assert!(!words.is_empty(), "the cause {cause:?} says nothing");
        assert!(!said.contains(&words), "the cause {cause:?} repeats another cause's words");
        said.push(words);
    }
}
