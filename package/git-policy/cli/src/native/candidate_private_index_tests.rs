//! What:
//!  Controls for the private index copy:
//!  where it is made,
//!  what it holds,
//!  its times,
//!       its permissions,
//!  and that it is removed on every path.
//! Why:
//!  A copy with a fresh time would let a prediction miss a same-second edit,
//!  a copy in
//!      a shared directory would expose staged entries,
//!  and a leftover directory would
//!      accumulate in the Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // { using index = createPrivateIndex(real); expect(read(index.path)).toEqual(read(real)); } expect(exists(dir)).toBe(false);
//! ```
#![cfg(unix)]

/// Import the type under test and its name prefix.
use super::{PRIVATE_INDEX_PREFIX, PrivateIndex};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The entries of a directory,
///  sorted by name.
fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("readable directory") {
        names.push(
            entry
                .expect("directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    return names;
}

/// The failure `create` returned,
///  which the control requires.
fn creation_failure(real_index: &Path) -> CandidateError {
    match PrivateIndex::create(real_index) {
        Ok(created) => panic!("expected a failure, got {created:?}"),
        Err(error) => return error,
    }
}

/// The copy sits in a fresh private directory beside the real index,
///  holds its bytes and
/// times,
///  and disappears with everything in it when dropped.
#[test]
fn the_copy_keeps_bytes_and_times_and_is_removed_on_drop() {
    let root: PathBuf = fixture("private-index-copy");
    let real: PathBuf = root.join("index");
    std::fs::write(&real, b"DIRC index bytes\0\xff").expect("real index");
    let accessed: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    let modified: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_100_000_000);
    std::fs::File::options()
        .write(true)
        .open(&real)
        .expect("open real index")
        .set_times(
            std::fs::FileTimes::new()
                .set_accessed(accessed)
                .set_modified(modified),
        )
        .expect("old times");
    let private: PrivateIndex = PrivateIndex::create(real.as_path()).expect("private index");
    let directory: PathBuf = private.path().parent().expect("directory").to_path_buf();
    assert_eq!(
        directory.parent(),
        Some(root.as_path()),
        "beside the real index"
    );
    let name: String = directory
        .file_name()
        .expect("name")
        .to_string_lossy()
        .into_owned();
    assert!(name.starts_with(PRIVATE_INDEX_PREFIX), "{name}");
    assert_eq!(private.path().file_name(), Some("index".as_ref()));
    // The times are read before the content, because reading may update the access time.
    let copied: std::fs::Metadata = std::fs::metadata(private.path()).expect("copy metadata");
    assert_eq!(copied.modified().expect("modified"), modified);
    assert_eq!(copied.accessed().expect("accessed"), accessed);
    assert_eq!(
        std::fs::read(private.path()).expect("copy"),
        b"DIRC index bytes\0\xff".to_vec()
    );
    let mode: u32 = std::fs::metadata(&directory)
        .expect("directory metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o700, "only the owner may enter");
    // A second copy made right after gets its own directory.
    let second: PrivateIndex = PrivateIndex::create(real.as_path()).expect("second copy");
    assert_ne!(second.path(), private.path());
    // Whatever Git writes into the directory goes with it.
    std::fs::write(directory.join("index.lock"), b"lock").expect("extra file");
    drop(private);
    drop(second);
    assert_eq!(entries(root.as_path()), vec!["index"]);
    remove(root.as_path());
}

/// A repository without an index file gets an empty directory and a private path Git reads as empty.
#[test]
fn a_missing_real_index_needs_no_copy() {
    let root: PathBuf = fixture("private-index-missing");
    let private: PrivateIndex =
        PrivateIndex::create(root.join("index").as_path()).expect("private index");
    assert!(!private.path().exists(), "nothing is copied");
    assert!(private.path().parent().expect("directory").is_dir());
    drop(private);
    assert_eq!(entries(root.as_path()), Vec::<String>::new());
    remove(root.as_path());
}

/// Every step that cannot be done is a private-index failure,
///  and no directory is left behind.
#[test]
fn failures_leave_nothing_behind() {
    let root: PathBuf = fixture("private-index-failures");
    let rootless: CandidateError = creation_failure(Path::new("/"));
    assert_eq!(rootless.failure, CandidateFailure::PrivateIndexUnavailable);
    assert!(
        rootless.message.contains("without a directory"),
        "{}",
        rootless.message
    );
    let no_parent: CandidateError = creation_failure(root.join("missing/index").as_path());
    assert_eq!(no_parent.failure, CandidateFailure::PrivateIndexUnavailable);
    assert!(
        no_parent.message.contains("creating its directory"),
        "{}",
        no_parent.message
    );
    // A real index that is a directory cannot be copied; the fresh directory is removed again.
    std::fs::create_dir(root.join("index")).expect("directory in place of the index");
    let not_a_file: CandidateError = creation_failure(root.join("index").as_path());
    assert_eq!(
        not_a_file.failure,
        CandidateFailure::PrivateIndexUnavailable
    );
    assert!(
        not_a_file.message.contains("copying the index"),
        "{}",
        not_a_file.message
    );
    assert_eq!(entries(root.as_path()), vec!["index"]);
    // An index path that cannot be inspected is a failure, not a missing index.
    std::os::unix::fs::symlink("loop", root.join("loop")).expect("link to itself");
    let looping: CandidateError = creation_failure(root.join("loop").as_path());
    assert_eq!(looping.failure, CandidateFailure::PrivateIndexUnavailable);
    assert!(
        looping.message.contains("copying the index"),
        "{}",
        looping.message
    );
    assert_eq!(entries(root.as_path()), vec!["index", "loop"]);
    remove(root.as_path());
}
