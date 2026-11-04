//! Controls for the file operations recovery shares.

use super::*;
use crate::test_support::{fixture, remove};

/// Existence probes tell missing from present and report other failures.
#[cfg(unix)]
#[test]
fn existence_probes_report_failures() {
    let root: PathBuf = fixture("recovery-files-exists");
    let file: PathBuf = root.join("file");
    std::fs::write(&file, b"x").expect("file");
    assert!(recovery_path_exists(file.as_path()).expect("probe"));
    assert!(!recovery_path_exists(root.join("missing").as_path()).expect("probe"));
    let under_file: PathBuf = file.join("child");
    assert!(
        recovery_path_exists(under_file.as_path())
            .expect_err("not a directory")
            .0
            .starts_with("probing ")
    );
    assert!(path_present(under_file.as_path()).is_err());
    let dangling: PathBuf = root.join("dangling");
    std::os::unix::fs::symlink(root.join("missing"), &dangling).expect("symlink");
    // A dangling link is present for `lstat` and absent for `access`.
    assert!(path_present(dangling.as_path()).expect("lstat"));
    assert!(!recovery_path_exists(dangling.as_path()).expect("access"));
    assert!(!path_present(root.join("missing").as_path()).expect("lstat"));
    remove(root.as_path());
}

/// An optional read gives the bytes, nothing for a missing file, and reports any other failure.
#[cfg(unix)]
#[test]
fn an_optional_read_tells_missing_from_unreadable() {
    let root: PathBuf = fixture("recovery-files-optional");
    let file: PathBuf = root.join("entry");
    std::fs::write(&file, b"bytes").expect("file");
    assert_eq!(read_optional(file.as_path()).expect("read"), Some(b"bytes".to_vec()));
    assert_eq!(read_optional(root.join("missing").as_path()).expect("read"), None);
    assert!(read_optional(file.join("child").as_path()).expect_err("not a directory").0.starts_with("reading "));
    remove(root.as_path());
}

/// Removal treats a missing file as removed and reports any other failure.
#[cfg(unix)]
#[test]
fn removal_of_a_missing_file_is_a_success() {
    let root: PathBuf = fixture("recovery-files-remove");
    let file: PathBuf = root.join("entry");
    std::fs::write(&file, b"x").expect("file");
    remove_file_if_present(file.as_path()).expect("remove");
    assert!(!file.exists());
    remove_file_if_present(file.as_path()).expect("a missing file is removed");
    assert!(remove_file_if_present(root.as_path()).expect_err("a directory is not a file").0.starts_with("removing "));
    remove(root.as_path());
}

/// Git's PID file sits beside the lock.
#[test]
fn pid_files_sit_beside_the_lock() {
    assert_eq!(
        lock_pid_path(Path::new("/repo/.git/index")),
        PathBuf::from("/repo/.git/index~pid.lock")
    );
}
