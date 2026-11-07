//! Controls for publishing a staged record at its final name,
//!  each in its own disposable directory.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A staging file for `path` with `content` in it,
/// and the refusal a filesystem with neither `renameat2` with `RENAME_NOREPLACE`
///  nor hard links gives (`s3fs-fuse` answers `ENOTSUP` to the link),
/// as `persist_noclobber` reports it with the file handed back.
#[cfg(unix)]
fn refused_publish(path: &Path, content: &[u8]) -> (PathBuf, Result<std::fs::File, tempfile::PersistError>) {
    let mut staged: tempfile::NamedTempFile = staging_beside(path).expect("staging file");
    staged.as_file_mut().write_all(content).expect("staged content");
    let staged_path: PathBuf = staged.path().to_path_buf();
    let refusal: tempfile::PersistError = tempfile::PersistError { error: std::io::Error::from_raw_os_error(libc::ENOTSUP), file: staged };
    return (staged_path, Err(refusal));
}

/// The owner's answer on a filesystem that has neither (2026-11-07,
/// "Keep the fallback (Recommended)"):
/// the staged file is
///  published by the rename,
/// the staging name is gone,
/// and nothing is left to remove later.
#[cfg(unix)]
#[test]
fn a_filesystem_with_neither_no_replace_rename_nor_links_gets_the_checked_rename() {
    let root: PathBuf = fixture("publish-neither");
    let path: PathBuf = root.join("published");
    let (staged_path, outcome) = refused_publish(path.as_path(), b"whole");
    assert!(staged_path.exists(), "the control: the staged file exists before the publish");
    assert!(!path.exists(), "the control: nothing is published yet");
    settle_exclusive_publish(path.as_path(), outcome).expect("the refusal is answered by the rename");
    assert_eq!(std::fs::read(&path).expect("published content"), b"whole");
    assert!(!staged_path.exists(), "the rename moved the staged file");
    remove(root.as_path());
}

/// The fallback still refuses a taken name with `AlreadyExists`,
/// leaves the record that holds it as it was,
/// and leaves no
///  staging file behind.
#[cfg(unix)]
#[test]
fn the_checked_rename_refuses_a_taken_name_and_leaves_no_staging_file() {
    let root: PathBuf = fixture("publish-neither-taken");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"old").expect("record");
    let (staged_path, outcome) = refused_publish(path.as_path(), b"new");
    let error: std::io::Error = settle_exclusive_publish(path.as_path(), outcome).expect_err("the name is taken");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&path).expect("record"), b"old");
    assert!(!staged_path.exists(), "the refused publish removed its staging file");
    remove(root.as_path());
}

/// A taken name is reported with the operating system's own `EEXIST`,
/// straight from the publish,
/// the record that holds the name
///  is left as it was,
/// and the refused write leaves no staging file.
#[cfg(unix)]
#[test]
fn a_taken_name_is_refused_with_the_operating_systems_error() {
    let root: PathBuf = fixture("publish-taken");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"old").expect("record");
    let error: std::io::Error = write_private_file(path.as_path(), b"new").expect_err("the name is taken");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(error.raw_os_error(), Some(libc::EEXIST), "the publish's own refusal, not a rebuilt one");
    assert_eq!(std::fs::read(&path).expect("record"), b"old");
    let names: Vec<String> = std::fs::read_dir(&root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names, vec![String::from("record")], "no staging file stays");
    remove(root.as_path());
}

/// A publish that works leaves exactly the record,
/// mode `0600`,
/// and no staging file;
/// a failure between the write and the publish
///  leaves neither the record nor a staging file.
#[cfg(unix)]
#[test]
fn a_published_record_is_private_and_a_failed_publish_leaves_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("publish-private");
    let path: PathBuf = root.join("record");
    write_private_file_unsynced(path.as_path(), b"whole").expect("publish");
    assert_eq!(std::fs::metadata(&path).expect("record").permissions().mode() & 0o777, PRIVATE_FILE_MODE);
    let other: PathBuf = root.join("other");
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = Some(Box::new(|| { return Err(std::io::Error::other("the disk filled")); })); });
    let failed: std::io::Result<()> = write_private_file_unsynced(other.as_path(), b"cut");
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = None; });
    failed.expect_err("the injected failure");
    let mut names: Vec<String> = std::fs::read_dir(&root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    assert_eq!(names, vec![String::from("record")], "the failed write left no record and no staging file");
    remove(root.as_path());
}

/// A directory sync that failed in the flush itself stays an error with its own operating-system code;
/// no unprivileged
///  filesystem produces these,
/// so the sync result is fed in.
#[cfg(unix)]
#[test]
fn a_failed_directory_flush_stays_an_error() {
    for code in [libc::EIO, libc::ENOSPC, libc::EDQUOT] {
        let error: std::io::Error = settle_directory_sync(Path::new("/unused"), Err(std::io::Error::from_raw_os_error(code)))
            .expect_err("a failed flush is not skipped");
        assert_eq!(error.raw_os_error(), Some(code));
    }
}

/// A filesystem that has no directory flush counts as synced,
/// whichever of the three codes it answers with,
/// and a
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

/// The owner's rulings of 2026-11-07 ("safe-file-saving shouldn't be something we own at all",
/// and the answer "Keep the
///  fallback (Recommended)" on a filesystem that has neither `renameat2` with `RENAME_NOREPLACE` nor hard links) put the exclusive
///  publish in the `tempfile` crate (`persist_noclobber`);
/// the module keeps only the checked rename for that one case,
/// so it holds
///  no hard link of its own and builds no staging name of its own.
#[test]
fn the_exclusive_publish_is_the_tempfile_crates_and_the_module_links_nothing() {
    let source: &str = include_str!("private_storage.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    assert!(code.contains("persist_noclobber"), "private_storage.rs does not publish through `persist_noclobber`");
    for needle in ["hard_link", "random_uuid", "fn staging_path"] {
        assert!(!code.contains(needle), "private_storage.rs still holds `{needle}`: the publish belongs to the tempfile crate");
    }
}

/// A staging file that cannot be made reports the system's own error,
/// with its code and without the staging path the
///  `tempfile` crate adds,
/// so the callers' texts say exactly what they said when the standard library made the file.
#[cfg(unix)]
#[test]
fn a_staging_file_that_cannot_be_made_reports_the_plain_system_error() {
    let root: PathBuf = fixture("publish-no-directory");
    let path: PathBuf = root.join("absent-directory").join("record");
    let error: std::io::Error = write_private_file(path.as_path(), b"x").expect_err("the directory is missing");
    assert_eq!(error.raw_os_error(), Some(libc::ENOENT));
    assert_eq!(error.to_string(), "No such file or directory (os error 2)");
    remove(root.as_path());
}
