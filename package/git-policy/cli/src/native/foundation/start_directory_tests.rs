//! Controls for the directory a command starts in:
//!  the caller's own,
//!  the process's,
//!  and none.

use super::*;
use crate::test_support::{
    fixture,
    remove,
};

/// A directory a caller names is the directory every reader gets,
///  and it names no reason there is none.
#[test]
fn a_named_directory_is_the_answer() {
    let root: PathBuf = fixture("start-directory-named");
    let start: StartDirectory = StartDirectory::at(root.as_path());
    assert_eq!(start.path(), Some(root.as_path()));
    assert_eq!(start.reason(), None);
    remove(root.as_path());
}

/// The process's own directory is the one this control was started in,
///  and a read of it never answers nothing on a running process.
#[test]
fn the_process_directory_is_this_control_directory() {
    let read: StartDirectory = StartDirectory::read_from_the_process();
    assert!(
        read.path().is_some(),
        "a running process has a current directory"
    );
    assert_eq!(read.reason(), None);
    assert_eq!(
        read.path(),
        Some(
            std::env::current_dir()
                .expect("the directory this control runs in")
                .as_path()
        )
    );
}

/// Nothing named reads as no directory,
///  and the reason says what a person may act on.
#[test]
fn no_directory_names_its_reason() {
    let unknown: StartDirectory = StartDirectory::unknown();
    assert_eq!(unknown.path(), None);
    assert!(
        unknown
            .reason()
            .is_some_and(|reason| return reason.contains("named no directory")),
        "the reason names the caller's own answer: {:?}",
        unknown.reason()
    );
}
