//! Controls of the record of what the old tree read like when the migration copied it.

use super::*;
use crate::test_support::{fixture, remove};

/// A file to read.
fn a_file(parent: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path: PathBuf = parent.join(name);
    std::fs::write(path.as_path(), bytes).expect("the file");
    return path;
}

/// A reading of a path,
/// as a pass takes one.
fn reading_of(path: &Path) -> OldTreeReading {
    return OldTreeReading::of(&std::fs::symlink_metadata(path).expect("metadata")).expect("a reading of a non-directory");
}

/// What:
///  A record written beside a checkout and read again names every entry with the same reading.
/// Why:
///   The record is the only evidence the deletion has,
///  so a field it loses on the way to the disk is a field it never compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await record.write(path); const back = await OldTreeRecord.read(path);
/// ```
#[test]
fn a_record_reads_back_with_every_field_it_was_written_with() {
    let parent: PathBuf = fixture("record-round-trip");
    let file: PathBuf = a_file(parent.as_path(), "file.txt", b"contents\n");
    let spaced: PathBuf = a_file(parent.as_path(), "a name with spaces.txt", b"spaces\n");
    let link: PathBuf = parent.join("link");
    std::os::unix::fs::symlink("file.txt", link.as_path()).expect("a link");
    let mut readings: BTreeMap<PathBuf, OldTreeReading> = BTreeMap::new();
    readings.insert(PathBuf::from("file.txt"), reading_of(file.as_path()));
    readings.insert(PathBuf::from("link"), reading_of(link.as_path()));
    readings.insert(PathBuf::from("a name with spaces.txt"), reading_of(spaced.as_path()));
    // A directory has no reading of its own: it is judged by its children.
    assert_eq!(OldTreeReading::of(&std::fs::symlink_metadata(parent.as_path()).expect("metadata")), None);
    let record: OldTreeRecord = OldTreeRecord::of(readings.clone());
    let path: PathBuf = parent.join("record.txt");
    record.write(path.as_path()).expect("the record is written");
    let read: OldTreeRecord = OldTreeRecord::read(path.as_path()).expect("readable").expect("there");
    for (relative, from_disk) in [("file.txt", &file), ("link", &link), ("a name with spaces.txt", &spaced)] {
        let now: std::fs::Metadata = std::fs::symlink_metadata(from_disk.as_path()).expect("metadata");
        assert!(read.unchanged(Path::new(relative), &now), "{relative} still reads as it did");
        let written: OldTreeReading = readings.get(Path::new(relative)).copied().expect("a reading");
        assert!(written.still_reads(&now), "{relative} reads as the reading that was written");
    }
    remove(parent.as_path());
}

/// A file with no record beside it is no record,
/// and no failure either.
#[test]
fn a_missing_record_is_none() {
    let parent: PathBuf = fixture("record-missing");
    assert!(OldTreeRecord::read(parent.join("absent.txt").as_path()).expect("readable").is_none());
    remove(parent.as_path());
}

/// What:
///  A write moves the reading,
///  even one that keeps the size and puts the modification time back.
/// Why:
///   The change time is the field no writer can set,
///  and it is what makes this reading a proof rather than a hint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = readingOf(file); writeSameSizeAndTime(file); assert(!record.unchanged(relative, statSync(file)));
/// ```
#[test]
fn a_write_that_keeps_the_size_and_the_time_still_moves_the_reading() {
    let parent: PathBuf = fixture("record-write");
    let file: PathBuf = a_file(parent.as_path(), "file.txt", b"contents\n");
    let before: std::fs::Metadata = std::fs::symlink_metadata(file.as_path()).expect("metadata");
    let mut readings: BTreeMap<PathBuf, OldTreeReading> = BTreeMap::new();
    readings.insert(PathBuf::from("file.txt"), OldTreeReading::of(&before).expect("a reading"));
    let record: OldTreeRecord = OldTreeRecord::of(readings);
    assert!(record.unchanged(Path::new("file.txt"), &before), "the very same reading is unchanged");
    // The same size, and the modification time put back by hand.
    let path: PathBuf = parent.join("file.txt");
    let times: [libc::timespec; 2] =
        [libc::timespec { tv_sec: before.mtime(), tv_nsec: before.mtime_nsec() }, libc::timespec { tv_sec: before.mtime(), tv_nsec: before.mtime_nsec() }];
    std::fs::write(path.as_path(), b"CONTENTS\n").expect("a rewrite of the same size");
    let name = std::ffi::CString::new(std::os::unix::ffi::OsStrExt::as_bytes(path.as_os_str())).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    assert_eq!(unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) }, 0, "put the time back");
    let after: std::fs::Metadata = std::fs::symlink_metadata(file.as_path()).expect("metadata");
    assert_eq!((after.len(), after.mtime(), after.mtime_nsec()), (before.len(), before.mtime(), before.mtime_nsec()), "the size and the time are back");
    assert_eq!(std::fs::read(file.as_path()).expect("the write"), b"CONTENTS\n");
    assert!(!record.unchanged(Path::new("file.txt"), &after), "the change time moved, so the entry was written");
    remove(parent.as_path());
}

/// An entry the record does not name is not unchanged,
/// so a file written into the old tree after the record was taken is kept rather than deleted.
#[test]
fn an_entry_the_record_does_not_name_is_not_unchanged() {
    let parent: PathBuf = fixture("record-unknown");
    let file: PathBuf = a_file(parent.as_path(), "later.txt", b"written later\n");
    let record: OldTreeRecord = OldTreeRecord::of(BTreeMap::new());
    let now: std::fs::Metadata = std::fs::symlink_metadata(file.as_path()).expect("metadata");
    assert!(!record.unchanged(Path::new("later.txt"), &now));
    remove(parent.as_path());
}
