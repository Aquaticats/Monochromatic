//! Controls for reading only plain files.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A plain file is read whole,
/// and one larger than the caller's limit is refused instead of truncated.
#[test]
fn a_plain_file_is_read_whole_and_a_larger_one_is_refused() {
    let parent: PathBuf = fixture("plain-file-bytes");
    let path: PathBuf = parent.join("record");
    std::fs::write(path.as_path(), b"a record\n").expect("record");
    assert_eq!(read_plain_file(path.as_path(), 64).expect("read"), b"a record\n");
    assert_eq!(read_plain_text(path.as_path()).expect("text"), "a record\n");
    std::fs::write(path.as_path(), vec![b'x'; 65]).expect("a longer record");
    let refused: String = read_plain_file(path.as_path(), 64).expect_err("refused").to_string();
    assert!(refused.contains("larger"), "{refused}");
    remove(parent.as_path());
}

/// A named pipe and a directory are each refused at once,
/// and a symbolic link is not followed.
#[test]
fn anything_that_is_not_a_plain_file_is_refused_at_once() {
    let parent: PathBuf = fixture("plain-file-kinds");
    let pipe: PathBuf = parent.join("pipe");
    crate::subvolume_sync_attrs::make_fifo(pipe.as_path(), 0o600).expect("named pipe");
    let read = crate::test_support::finished_within(move || { return read_plain_file(pipe.as_path(), SMALL_FILE_LIMIT).map_err(|error| { return error.to_string(); }); });
    let Some(outcome) = read else { panic!("reading a named pipe did not finish within five seconds") };
    let refused: String = outcome.expect_err("refused");
    assert!(refused.contains("not a plain file"), "{refused}");
    let directory: PathBuf = parent.join("directory");
    std::fs::create_dir(directory.as_path()).expect("directory");
    assert!(read_plain_file(directory.as_path(), SMALL_FILE_LIMIT).expect_err("refused").to_string().contains("not a plain file"));
    let record: PathBuf = parent.join("record");
    std::fs::write(record.as_path(), b"a record\n").expect("record");
    let link: PathBuf = parent.join("link");
    std::os::unix::fs::symlink(record.as_path(), link.as_path()).expect("link");
    assert!(read_plain_file(link.as_path(), SMALL_FILE_LIMIT).is_err(), "a link is not followed");
    assert_eq!(read_plain_file(record.as_path(), SMALL_FILE_LIMIT).expect("the file itself"), b"a record\n");
    remove(parent.as_path());
}

/// Text that is not UTF-8 is refused as not text,
/// and the same bytes are still read as bytes.
#[test]
fn bytes_that_are_not_text_are_refused_as_text() {
    let parent: PathBuf = fixture("plain-file-text");
    let path: PathBuf = parent.join("record");
    std::fs::write(path.as_path(), [0xff_u8, 0xfe]).expect("bytes");
    let refused: String = read_plain_text(path.as_path()).expect_err("not text").to_string();
    assert!(refused.contains("not text"), "{refused}");
    assert_eq!(read_plain_file(path.as_path(), SMALL_FILE_LIMIT).expect("bytes"), [0xff_u8, 0xfe]);
    remove(parent.as_path());
}
