//! What:
//!  Controls for reading the real Git the wrapper records beside its sockets.
//! Why:
//!  The fallback of `real_git.rs` is only as sound as this reading:
//!  a path taken from the wrong line,
//! a relative path,
//!  or a file past the bound would start the wrong program.

#![cfg(unix)]

use super::{read_recorded_git, recorded_git_path, record_file};
use crate::real_git_candidate::MAX_SCRIPT_INSPECTION_BYTES;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// The note the wrapper writes is one line and the path,
///  and a relative or empty line names nothing.
#[test]
fn the_recorded_path_is_the_first_absolute_line() {
    assert_eq!(recorded_git_path("/usr/bin/git\n"), Some(PathBuf::from("/usr/bin/git")));
    assert_eq!(recorded_git_path("  /opt/git/bin/git  \n"), Some(PathBuf::from("/opt/git/bin/git")));
    assert_eq!(recorded_git_path("git\n"), None);
    assert_eq!(recorded_git_path(""), None);
    assert_eq!(recorded_git_path("\n"), None);
}

/// The record is the note beside the sockets.
#[test]
fn the_record_sits_beside_the_sockets() {
    assert_eq!(record_file(Path::new("/run/user/7")), PathBuf::from("/run/user/7/monochromatic-git-policy-cli/real-git"));
}

/// What the wrapper remembers is what a later command reads,
///  and a missing file,
/// a directory and a file past the bound name nothing.
#[test]
fn what_is_remembered_is_read_and_a_file_that_is_not_a_record_names_nothing() {
    let root: PathBuf = fixture("record-read");
    let git: PathBuf = root.join("bin/git");
    std::fs::create_dir_all(git.parent().expect("the folder of the record")).expect("bin");
    std::fs::write(&git, "# a program").expect("a program");
    super::remember(root.as_path(), git.as_path()).expect("the note is written");
    assert_eq!(read_recorded_git(record_file(root.as_path()).as_path()), Some(git.clone()));
    assert_eq!(read_recorded_git(root.join("missing").as_path()), None);
    assert_eq!(read_recorded_git(root.as_path()), None);
    let binary: PathBuf = root.join("binary");
    std::fs::write(&binary, b"/opt/git\xff/git\n").expect("bytes that are not text");
    assert_eq!(read_recorded_git(binary.as_path()), None);
    let long: PathBuf = root.join("long");
    let mut text: String = String::from("/opt/git/bin/git\n");
    text.push_str("#".repeat(MAX_SCRIPT_INSPECTION_BYTES).as_str());
    std::fs::write(&long, text).expect("a file past the bound");
    assert_eq!(read_recorded_git(long.as_path()), None);
    remove(root.as_path());
}

/// What:
///  Two starts that remember different Gits in one runtime directory both succeed,
///  and the record names one of them.
/// Why:
///  The sixth review of issue 6293 (finding 4):
///  every start wrote the same temporary file before its rename,
///  so one start published the other's path while that one's rename failed and was only logged.
/// The temporary file is each writer's own, so neither write can take the other's name away.
#[test]
fn two_starts_that_remember_at_once_both_succeed() {
    let root: PathBuf = fixture("record-race");
    let runtime: PathBuf = root.join("run");
    let first: PathBuf = PathBuf::from("/usr/bin/git");
    let second: PathBuf = PathBuf::from("/opt/another/git");
    for round in 0..8 {
        let round_runtime: PathBuf = runtime.join(format!("round-{round}"));
        let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(2));
        let mut threads: Vec<std::thread::JoinHandle<(PathBuf, std::io::Result<()>)>> = Vec::new();
        for git in [first.clone(), second.clone()] {
            let gate: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&barrier);
            let place: PathBuf = round_runtime.clone();
            threads.push(std::thread::spawn(move || {
                gate.wait();
                return (git.clone(), super::remember(place.as_path(), git.as_path()));
            }));
        }
        for thread in threads {
            let (git, result): (PathBuf, std::io::Result<()>) = thread.join().expect("the remember thread");
            assert!(result.is_ok(), "the start that remembers {} must not lose its write: {result:?}", git.display());
        }
        let recorded: Option<PathBuf> = read_recorded_git(record_file(round_runtime.as_path()).as_path());
        assert!(
            recorded == Some(first.clone()) || recorded == Some(second.clone()),
            "the record names one of the two Gits that were remembered: {recorded:?}"
        );
    }
    remove(root.as_path());
}
