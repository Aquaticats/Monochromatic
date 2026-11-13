//! Controls for the one read of a descriptor's own record.

use super::*;
use crate::test_support::{fixture, remove};

/// A process table directory whose descriptor `fd` is a link to `target` and whose record is `record`.
/// Why:
///  A control's `/proc` is a directory of links,
/// and the record of one descriptor is a file beside it.
fn table(root: &Path, fd: &str, target: &str, record: Option<&str>) -> PathBuf {
    let process: PathBuf = root.join("proc").join("4242");
    std::fs::create_dir_all(process.join("fd")).expect("the descriptor directory");
    std::fs::create_dir_all(process.join("fdinfo")).expect("the record directory");
    std::os::unix::fs::symlink(target, process.join("fd").join(fd)).expect("a descriptor of the control's own");
    if let Some(text) = record {
        std::fs::write(process.join("fdinfo").join(fd), text).expect("a record");
    }
    return process;
}

/// A descriptor whose link text is not a path is of no file of a file system,
/// and its record is not opened.
/// Why:
///  The record's own file is a directory here,
/// which no read can open:
/// a scan that opened it would report the refusal.
#[test]
fn a_descriptor_that_is_not_a_file_is_answered_without_opening_its_record() {
    let root: PathBuf = fixture("descriptor-record-anonymous");
    let process: PathBuf = table(root.as_path(), "3", "anon_inode:inotify", None);
    std::fs::create_dir_all(process.join("fdinfo").join("3")).expect("a record no read can open");
    assert_eq!(descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile), Record::NotTheTarget);
    remove(root.as_path());
}

/// A pipe is a file a scan for an anonymous file looks for,
/// and it is read;
/// a kernel anonymous inode is not,
/// and it is not.
/// Why:
///  A step's own output pipe is found by its mount number and inode,
/// so skipping every text that is not a path would lose every holder of a pipe.
#[test]
fn an_anonymous_target_reads_a_pipe_and_skips_a_kernel_anonymous_inode() {
    let root: PathBuf = fixture("descriptor-record-pipe");
    let pipe: PathBuf = table(root.as_path(), "5", "pipe:[42]", Some("pos:\t0\nflags:\t00\nmnt_id:\t9\nino:\t42\n"));
    assert_eq!(
        descriptor_record(pipe.as_path(), std::ffi::OsStr::new("5"), Targeted::AnonymousFile),
        Record::Text(String::from("pos:\t0\nflags:\t00\nmnt_id:\t9\nino:\t42\n"))
    );
    let inotify: PathBuf = table(root.as_path(), "6", "anon_inode:inotify", None);
    std::fs::create_dir_all(inotify.join("fdinfo").join("6")).expect("a record no read can open");
    assert_eq!(descriptor_record(inotify.as_path(), std::ffi::OsStr::new("6"), Targeted::AnonymousFile), Record::NotTheTarget);
    remove(root.as_path());
}

/// A record that fits is taken whole,
/// one of exactly the bound included.
#[test]
fn a_record_that_fits_the_bound_is_taken() {
    let root: PathBuf = fixture("descriptor-record-fits");
    let record: String = record_of_length(RECORD_BYTES_AT_MOST);
    let process: PathBuf = table(root.as_path(), "3", "/a/file", Some(record.as_str()));
    assert_eq!(
        descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile),
        Record::Text(record),
        "a record of exactly the bound is a record that fits"
    );
    remove(root.as_path());
}

/// A record one byte past the bound is not used at all.
/// Why:
///  What a record holds is decided by the process that holds the descriptor,
/// and a scan that took it anyway would hold whatever that process decided.
#[test]
fn a_record_past_the_bound_is_not_used() {
    let root: PathBuf = fixture("descriptor-record-past");
    let record: String = record_of_length(RECORD_BYTES_AT_MOST + 1);
    let process: PathBuf = table(root.as_path(), "3", "/a/file", Some(record.as_str()));
    assert_eq!(descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile), Record::BeyondBound);
    remove(root.as_path());
}

/// A record that cannot be opened is the kernel's own refusal,
/// and not a bound's.
#[test]
fn a_record_that_cannot_be_opened_is_reported() {
    let root: PathBuf = fixture("descriptor-record-unreadable");
    let process: PathBuf = table(root.as_path(), "3", "/a/file", None);
    match descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile) {
        Record::Unreadable { kind, reason } => assert_eq!(kind, std::io::ErrorKind::NotFound, "{reason}"),
        other => panic!("a record that is not there is unreadable: {other:?}"),
    }
    remove(root.as_path());
}

/// A link whose text cannot be produced is a descriptor whose kind is unknown,
/// which a caller reads as a file that may be held.
/// Why:
///  `readlink(2)` of `/proc/<pid>/fd/<n>` answers `EINVAL` when the entry is no link and `EACCES` when the kernel refuses
/// the scan that entry,
///  and only the second of those is the "could not look at this process" case the callers tolerated before this gate.
#[test]
fn a_link_that_cannot_be_read_is_a_kind_unknown() {
    let root: PathBuf = fixture("descriptor-record-no-link");
    let process: PathBuf = table(root.as_path(), "3", "/a/file", None);
    std::fs::remove_file(process.join("fd").join("3")).expect("the link");
    std::fs::write(process.join("fd").join("3"), b"not a link").expect("an entry that is no link");
    match descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile) {
        Record::KindUnknown { kind, reason } => assert_eq!(kind, std::io::ErrorKind::InvalidInput, "{reason}"),
        other => panic!("an entry that is no link has an unknown kind: {other:?}"),
    }
    remove(root.as_path());
}

/// A descriptor that closed during the walk is gone,
/// and the race is no evidence about the file.
#[test]
fn a_descriptor_that_closed_is_gone() {
    let root: PathBuf = fixture("descriptor-record-gone");
    let process: PathBuf = table(root.as_path(), "3", "/a/file", None);
    std::fs::remove_file(process.join("fd").join("3")).expect("the descriptor's link, as the holder's own close takes it");
    assert_eq!(descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile), Record::Gone);
    remove(root.as_path());
}

/// A descriptor the kernel refuses this scan is a process it could not look at,
/// which is the case the callers tolerated before the gate and not a file that may be held.
#[test]
fn a_link_the_kernel_refuses_is_a_record_unreadable() {
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("descriptor-record-refused");
    let process: PathBuf = table(root.as_path(), "3", "/a/file", None);
    std::fs::set_permissions(process.join("fd"), std::fs::Permissions::from_mode(0o000)).expect("hide the descriptor directory");
    let found: Record = descriptor_record(process.as_path(), std::ffi::OsStr::new("3"), Targeted::RealFile);
    std::fs::set_permissions(process.join("fd"), std::fs::Permissions::from_mode(0o700)).expect("reveal the descriptor directory");
    match found {
        Record::Unreadable { kind, reason } => assert_eq!(kind, std::io::ErrorKind::PermissionDenied, "{reason}"),
        other => panic!("a descriptor the kernel refuses is a record this scan could not read: {other:?}"),
    }
    remove(root.as_path());
}

/// The number of the last `key:` line of a record is the one a scan takes.
/// Why:
///  A record holds one `ino:` and one `mnt_id:` line today,
/// and the parse of a record has always taken the last of a key it saw.
#[test]
fn the_last_line_of_a_key_names_the_number() {
    let first: u64 = 7;
    let second: u64 = 11;
    let text: String = format!("pos:\t0\nmnt_id:\t{first}\nino:\t{second}\nmnt_id:\t{second}\nino:\t{first}\n");
    assert_eq!(inode_in(text.as_str()), Some(first));
    assert_eq!(mount_id_in(text.as_str()), Some(second));
    assert_eq!(inode_in("pos:\t0\n"), None, "a record with no `ino:` line names no inode");
}

/// A record of exactly `length` bytes that still holds an `ino:` line.
fn record_of_length(length: usize) -> String {
    let head: String = String::from("ino:\t1\n");
    let filler: String = String::from("x".repeat(length - head.len() - 1));
    let text: String = format!("{head}{filler}\n");
    assert_eq!(text.len(), length, "the control's own record is as long as it says");
    return text;
}
