//! What:
//!  The copying of an index into a private one:
//! the bytes,
//! mode and times the copy holds,
//! its independence from the source,
//! and that a filesystem which clones shares the extents instead of reading the data.
//! Why:
//!   A three-megabyte index copied by reading and writing its bytes cost 2,288 ms and 4,813 ms in two traced commits
//! under this host's input and output pressure,
//! which is one `copy_file_range` call an index copy waits for.
//! A clone costs metadata only,
//! and it is acceptable only because the copy is provably equal to the source and stands alone:
//! writing to it must leave the source as it was,
//! which these controls ask.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // copyFile(source, destination, COPYFILE_FICLONE); expect(readFile(destination)).toEqual(readFile(source));
//! ```

/// The copy under control.
use super::copy_index_with_timestamps;
/// Fixture handling shared by the crate's controls.
use crate::test_support::{fixture, remove};
/// The permission bits of the source,
/// which the copy must carry.
use std::os::unix::fs::PermissionsExt;
/// Byte sizes,
/// times and the writes that give a file content.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The modification time the controls give the source,
/// far enough in the past that a copy made now cannot hold it by accident.
const SOURCE_SECONDS: u64 = 1_000_000_000;

/// The permission bits the source is given,
/// which the copy must carry:
/// readable and writable by its owner and its group,
/// and by nobody else.
const SOURCE_MODE: u32 = 0o640;

/// The size of the file the clone control copies.
///
/// A megabyte of anything is enough for the copy to hold data blocks,
/// and small enough that the control costs nothing worth counting.
const CLONE_BYTES: usize = 1024 * 1024;

/// Bytes of a known pattern,
/// so a copy of the wrong length or the wrong content shows.
fn pattern(length: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    for index in 0..length {
        bytes.push(u8::try_from(index % 251).unwrap_or_default());
    }
    return bytes;
}

/// A source file with the pattern,
/// the mode and a modification time in the past.
fn source_file(directory: &Path) -> PathBuf {
    let source: PathBuf = directory.join("index");
    std::fs::write(source.as_path(), pattern(4096).as_slice()).expect("the source");
    std::fs::set_permissions(source.as_path(), std::fs::Permissions::from_mode(SOURCE_MODE)).expect("the mode");
    let times: std::fs::FileTimes = std::fs::FileTimes::new()
        .set_accessed(SystemTime::UNIX_EPOCH + Duration::from_secs(SOURCE_SECONDS))
        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(SOURCE_SECONDS));
    std::fs::File::options()
        .write(true)
        .open(source.as_path())
        .expect("the source opens")
        .set_times(times)
        .expect("the source takes its times");
    return source;
}

/// The copy holds the source's bytes,
/// its mode and its modification time,
/// and writing to it leaves the source as it was:
/// a clone shares storage,
/// never the file's content as either side sees it afterwards.
#[test]
fn a_copied_index_holds_the_source_bytes_mode_and_time_and_stands_alone() {
    let root: PathBuf = fixture("index-copy-equal");
    let source: PathBuf = source_file(root.as_path());
    let destination: PathBuf = root.join("copy");
    copy_index_with_timestamps(source.as_path(), destination.as_path()).expect("the copy");
    let original: Vec<u8> = std::fs::read(source.as_path()).expect("the source reads");
    assert_eq!(std::fs::read(destination.as_path()).expect("the copy reads"), original);
    let copied: std::fs::Metadata = std::fs::metadata(destination.as_path()).expect("the copy's metadata");
    assert_eq!(copied.permissions().mode() & 0o777, SOURCE_MODE, "the copy carries the source's mode");
    assert_eq!(
        copied.modified().expect("the copy's time"),
        SystemTime::UNIX_EPOCH + Duration::from_secs(SOURCE_SECONDS),
        "the copy carries the source's modification time"
    );
    std::fs::write(destination.as_path(), b"written after the copy\n").expect("the copy takes a write");
    assert_eq!(std::fs::read(source.as_path()).expect("the source reads"), original, "the source is untouched by a write to the copy");
    remove(root.as_path());
}

/// The copy asks the filesystem for a clone exactly where the filesystem accepts one,
/// and the answer is the fact the next slow preparation reads instead of guessing:
/// a clone read and wrote no data,
/// a byte copy did.
/// The probe asks the kernel directly on a separate pair of files,
/// so the answer does not depend on the copy under control;
/// where the filesystem cannot clone (tmpfs,
/// overlay) both sides are refusals and the control shows that too.
#[cfg(target_os = "linux")]
#[test]
fn the_copy_clones_exactly_where_the_filesystem_accepts_a_clone() {
    use std::os::unix::io::AsRawFd;
    let root: PathBuf = fixture("index-copy-clone");
    let source: PathBuf = root.join("index");
    let bytes: Vec<u8> = pattern(CLONE_BYTES);
    std::fs::write(source.as_path(), bytes.as_slice()).expect("the source");
    let probe = |name: &str| -> (std::fs::File, std::fs::File) {
        let input: std::fs::File = std::fs::File::open(source.as_path()).expect("the source opens");
        let output: std::fs::File = std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(root.join(name))
            .expect("the probe's destination");
        return (input, output);
    };
    let (probe_input, probe_output) = probe("probe");
    // SAFETY: both descriptors are open for the call, and FICLONE takes the source descriptor as its argument.
    let supported: bool = unsafe { libc::ioctl(probe_output.as_raw_fd(), libc::FICLONE, probe_input.as_raw_fd()) } == 0;
    let destination: PathBuf = root.join("copy");
    let cloned: bool = copy_index_with_timestamps(source.as_path(), destination.as_path()).expect("the copy");
    assert_eq!(cloned, supported, "the copy clones exactly where the filesystem accepts a clone");
    assert_eq!(std::fs::read(destination.as_path()).expect("the copy reads"), bytes, "a clone and a byte copy hold the same bytes");
    remove(root.as_path());
}
