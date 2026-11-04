//! Controls for publishing a staged record at its final name,
//!  each in its own disposable directory.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A hard link cannot name a directory (Linux and macOS answer `EPERM`), which is the one refusal of the link that
///  an unprivileged test can cause on any filesystem: the publisher then falls back to the rename, and the directory
///  and what it holds appear at the final name.
#[cfg(unix)]
#[test]
fn a_refused_link_falls_back_to_the_rename() {
    let root: PathBuf = fixture("publish-link-refused");
    let staged: PathBuf = root.join("staged");
    let path: PathBuf = root.join("published");
    std::fs::create_dir(&staged).expect("staged directory");
    std::fs::write(staged.join("inner"), b"whole").expect("staged content");
    let probe: PathBuf = root.join("probe");
    let refusal: std::io::Error = std::fs::hard_link(&staged, &probe).expect_err("the control: a directory cannot be linked");
    assert_ne!(refusal.kind(), std::io::ErrorKind::AlreadyExists, "the control: the refusal is not a taken name");
    assert!(!probe.exists(), "the control: the refused link created nothing");
    publish_exclusive(staged.as_path(), path.as_path()).expect("a refused link is published by rename");
    assert_eq!(std::fs::read(path.join("inner")).expect("published content"), b"whole");
    assert!(!staged.exists(), "the rename moved the staged entry");
    remove(root.as_path());
}

/// A taken name is reported with the operating system's own `EEXIST`, straight from the link, and the record that
///  holds the name is left as it was.
#[cfg(unix)]
#[test]
fn a_taken_name_is_refused_with_the_operating_systems_error() {
    let root: PathBuf = fixture("publish-taken");
    let staged: PathBuf = root.join("staged");
    let path: PathBuf = root.join("record");
    std::fs::write(&staged, b"new").expect("staged file");
    std::fs::write(&path, b"old").expect("record");
    let error: std::io::Error = publish_exclusive(staged.as_path(), path.as_path()).expect_err("the name is taken");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(error.raw_os_error(), Some(libc::EEXIST), "the link's own refusal, not a rebuilt one");
    assert_eq!(std::fs::read(&path).expect("record"), b"old");
    assert_eq!(std::fs::read(&staged).expect("staged"), b"new");
    remove(root.as_path());
}

/// A staged file that is gone makes the link fail with `ENOENT`, not a taken name: the publisher must then check
///  the final name before it renames, and still answer `AlreadyExists` when the name is taken.
#[cfg(unix)]
#[test]
fn a_link_that_fails_for_another_reason_still_checks_the_taken_name() {
    let root: PathBuf = fixture("publish-staged-missing");
    let staged: PathBuf = root.join("staged");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"old").expect("record");
    let control: std::io::Error = std::fs::hard_link(&staged, &path).expect_err("the control: the link fails");
    assert_eq!(control.kind(), std::io::ErrorKind::NotFound, "the control: the source is checked before the name");
    let error: std::io::Error = publish_exclusive(staged.as_path(), path.as_path()).expect_err("the name is taken");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&path).expect("record"), b"old");
    remove(root.as_path());
}

/// A directory sync that failed in the flush itself stays an error with its own operating-system code; no unprivileged
///  filesystem produces these, so the sync result is fed in.
#[cfg(unix)]
#[test]
fn a_failed_directory_flush_stays_an_error() {
    for code in [libc::EIO, libc::ENOSPC, libc::EDQUOT] {
        let error: std::io::Error = settle_directory_sync(Path::new("/unused"), Err(std::io::Error::from_raw_os_error(code)))
            .expect_err("a failed flush is not skipped");
        assert_eq!(error.raw_os_error(), Some(code));
    }
}

/// A filesystem that has no directory flush counts as synced, whichever of the three codes it answers with, and a
///  sync that worked stays a success.
#[cfg(unix)]
#[test]
fn a_filesystem_without_directory_sync_counts_as_synced() {
    for code in [libc::EINVAL, libc::ENOTSUP, libc::ENOSYS] {
        settle_directory_sync(Path::new("/unused"), Err(std::io::Error::from_raw_os_error(code)))
            .expect("an unsupported directory sync is skipped");
    }
    settle_directory_sync(Path::new("/unused"), Ok(())).expect("a sync that worked");
}

/// A staging file that is already gone (the rename moved it) needs no diagnostic, a success needs none, and any other
///  removal failure is reported with the staging path.
#[test]
fn only_a_real_removal_failure_is_reported() {
    let staged: &Path = Path::new("/records/.record.partial");
    assert_eq!(staging_removal_report(staged, Ok(())), None);
    assert_eq!(staging_removal_report(staged, Err(std::io::Error::from(std::io::ErrorKind::NotFound))), None);
    let report: String = staging_removal_report(staged, Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)))
        .expect("a permission failure is reported");
    assert!(report.contains("/records/.record.partial"), "the report names the staging file: {report}");
}
