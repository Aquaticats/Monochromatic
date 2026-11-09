//! Controls for the stat data a landing records for a path whose worktree file provably holds the bytes the
//! commit stores:
//! a file that holds them records its stat data,
//! a file that holds anything else,
//! a
//!  file as new as the index,
//! one that changed while it was read,
//! and one of another kind record nothing.

use super::{WorktreeStatRequest, recorded_stat};
use crate::index_stat::{IndexStamp, StatData};
use crate::repo_read_layout::ObjectFormat;
use crate::test_support::{age, fixture, git, remove};
use std::cell::Cell as StdCell;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The date an index written now is dated with,
/// to the nanosecond.
fn now_stamp() -> IndexStamp {
    let age: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("after the epoch");
    return IndexStamp { seconds: age.as_secs() as u32, nanoseconds: age.subsec_nanos() };
}

/// The object name real Git gives `content` as a blob in `repo`,
/// so the comparison is against Git's own,
/// not this crate's idea of it.
fn blob_in(repo: &Path, content: &[u8]) -> Vec<u8> {
    let output: std::process::Output = crate::index_reset_file::support::git_with(repo, &[], &["hash-object", "-w", "--stdin"], Some(content));
    assert!(output.status.success(), "git hash-object: {}", String::from_utf8_lossy(&output.stderr));
    let text: String = String::from_utf8(output.stdout).expect("hex");
    return hex_bytes(text.trim_end());
}

/// The bytes a hexadecimal object name spells.
fn hex_bytes(text: &str) -> Vec<u8> {
    let digits: Vec<u8> = text.bytes().map(|byte| { return (byte as char).to_digit(16).expect("a hexadecimal digit") as u8; }).collect();
    let mut bytes: Vec<u8> = Vec::with_capacity(digits.len() / 2);
    for pair in digits.chunks(2) {
        bytes.push((pair[0] << 4) | pair[1]);
    }
    return bytes;
}

/// The answer for `path` in `root`,
/// whose entry records `blob`,
/// for an index written now with nothing hashed for this landing yet.
fn recorded(root: &Path, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData> {
    return recorded_at(now_stamp(), root, path, blob, mode);
}

/// `recorded` for an index file dated `stamp`.
fn recorded_at(stamp: IndexStamp, root: &Path, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData> {
    let hashed: StdCell<u64> = StdCell::new(0);
    return recorded_stat(&WorktreeStatRequest { worktree_root: root, path, blob, mode, format: ObjectFormat::Sha1, stamp, hashed: &hashed });
}

/// A repository fixture with no commit,
/// enough for Git's own object names.
fn fixture_root(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    return root;
}

/// The stat data the file at `path` has,
/// as the index stores it.
fn stat_of_file(path: &Path) -> StatData {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("the file");
    return StatData {
        ctime_sec: metadata.ctime() as u32,
        ctime_nsec: metadata.ctime_nsec() as u32,
        mtime_sec: metadata.mtime() as u32,
        mtime_nsec: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
}

/// A file that holds the entry's bytes records the stat data the file has.
#[test]
fn a_file_that_holds_the_entrys_bytes_records_the_files_stat_data() {
    let root: PathBuf = fixture_root("index-worktree-stat-holds");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    // Aged, so the entry is not racily clean: the index this entry goes into is newer than the file.
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    assert_eq!(recorded(root.as_path(), b"a", blob.as_slice(), 0o100_644), Some(stat_of_file(file.as_path())));
    remove(root.as_path());
}

/// A file that holds other bytes records nothing,
/// however close the size is.
#[test]
fn a_file_that_holds_other_bytes_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-other");
    std::fs::write(root.join("a"), b"a TWO\n").expect("write");
    age(root.join("a").as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a two\n");
    assert_eq!(recorded(root.as_path(), b"a", blob.as_slice(), 0o100_644), None, "the bytes differ though the size does not");
    remove(root.as_path());
}

/// A file as new as the index is racily clean,
/// so its stat data would prove nothing;
/// a file older than the index,
/// by a whole second or by a nanosecond in the same second,
/// records.
#[test]
fn a_file_as_new_as_the_index_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-racy");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    let written: StatData = stat_of_file(file.as_path());
    let at = |stamp: IndexStamp| { return recorded_at(stamp, root.as_path(), b"a", blob.as_slice(), 0o100_644); };
    assert_eq!(at(IndexStamp { seconds: written.mtime_sec, nanoseconds: written.mtime_nsec }), None, "an entry the index is as new as is racily clean");
    assert_eq!(at(IndexStamp { seconds: written.mtime_sec, nanoseconds: written.mtime_nsec.saturating_sub(1) }), None, "an index older than the file trusts nothing");
    assert!(at(IndexStamp { seconds: written.mtime_sec, nanoseconds: written.mtime_nsec.saturating_add(1) }).is_some(), "the same second, a nanosecond later, is an index the file is older than");
    assert!(at(IndexStamp { seconds: written.mtime_sec + 1, nanoseconds: 0 }).is_some(), "one second later the file is older than the index");
    remove(root.as_path());
}

/// A symbolic link records the stat data of the link itself,
/// and its content is the bytes of its target.
#[test]
fn a_symbolic_link_records_the_stat_data_of_the_link() {
    let root: PathBuf = fixture_root("index-worktree-stat-link");
    std::os::unix::fs::symlink("target", root.join("l")).expect("symlink");
    age(root.join("l").as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"target");
    assert_eq!(recorded(root.as_path(), b"l", blob.as_slice(), 0o120_000), Some(stat_of_file(root.join("l").as_path())));
    let elsewhere: Vec<u8> = blob_in(root.as_path(), b"other");
    assert_eq!(recorded(root.as_path(), b"l", elsewhere.as_slice(), 0o120_000), None, "a link to another target is another content");
    remove(root.as_path());
}

/// A file that changed while it was read records nothing:
/// the bytes that were hashed must be the bytes the stat data describes.
#[test]
fn a_file_that_changed_while_it_was_read_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-changed");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    // A writer that replaces the bytes with the same size between the read and the second look at the file's state.
    let hashed: StdCell<u64> = StdCell::new(0);
    let mut writes: usize = 0;
    let written: PathBuf = file.clone();
    let mut seam = || {
        writes += 1;
        std::fs::write(&written, b"a TWO\n").expect("rewrite");
    };
    let request: WorktreeStatRequest<'_> = WorktreeStatRequest { worktree_root: root.as_path(), path: b"a", blob: blob.as_slice(), mode: 0o100_644, format: ObjectFormat::Sha1, stamp: now_stamp(), hashed: &hashed };
    assert_eq!(super::recorded_stat_seamed(&request, &mut seam), None);
    assert_eq!(writes, 1, "the control wrote during the read");
    remove(root.as_path());
}

/// A file larger than the bound records nothing.
#[test]
fn a_file_larger_than_the_content_bound_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-large");
    let file: PathBuf = root.join("a");
    let large: std::fs::File = std::fs::File::create(&file).expect("create");
    large.set_len(crate::worktree_clean_content::CONTENT_LIMIT + 1).expect("sparse growth");
    age(file.as_path());
    let blob: Vec<u8> = vec![7; 20];
    assert_eq!(recorded(root.as_path(), b"a", blob.as_slice(), 0o100_644), None);
    remove(root.as_path());
}

/// One landing hashes at most `CONTENT_LIMIT` bytes:
/// a file that would pass the bound records nothing,
/// and nothing more is spent.
#[test]
fn the_bytes_one_landing_hashes_are_bounded() {
    let root: PathBuf = fixture_root("index-worktree-stat-budget");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    let spent: StdCell<u64> = StdCell::new(crate::worktree_clean_content::CONTENT_LIMIT);
    let request: WorktreeStatRequest<'_> = WorktreeStatRequest { worktree_root: root.as_path(), path: b"a", blob: blob.as_slice(), mode: 0o100_644, format: ObjectFormat::Sha1, stamp: now_stamp(), hashed: &spent };
    assert_eq!(recorded_stat(&request), None, "a landing that has spent the bound records nothing more");
    assert_eq!(spent.get(), crate::worktree_clean_content::CONTENT_LIMIT, "and spends nothing");
    remove(root.as_path());
}

/// A path whose kind or executable bit is not the one the entry records records nothing,
/// and so does a missing file and a gitlink.
#[test]
fn a_file_of_another_kind_than_the_entrys_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-kind");
    std::fs::write(root.join("a"), b"a one\n").expect("write");
    std::os::unix::fs::symlink("a", root.join("l")).expect("symlink");
    age(root.join("a").as_path());
    age(root.join("l").as_path());
    let content: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    assert_eq!(recorded(root.as_path(), b"a", content.as_slice(), 0o100_755), None, "the file is not executable");
    assert_eq!(recorded(root.as_path(), b"a", content.as_slice(), 0o120_000), None, "the path is a file, not a link");
    let target: Vec<u8> = blob_in(root.as_path(), b"a");
    assert_eq!(recorded(root.as_path(), b"l", target.as_slice(), 0o100_644), None, "the path is a link, not a file");
    assert_eq!(recorded(root.as_path(), b"l", target.as_slice(), 0o160_000), None, "a gitlink holds no file");
    assert_eq!(recorded(root.as_path(), b"absent", content.as_slice(), 0o100_644), None, "no file, no stat data");
    remove(root.as_path());
}
