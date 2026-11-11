//! Controls for the stat data a landing records for a path whose worktree file provably holds the bytes the
//! commit stores:
//! a file that holds them records its stat data,
//! a file that holds anything else,
//! a
//!  file as new as the index,
//! one that changed while it was read,
//! and one of another kind record nothing.

use super::{ProofSeams, WorktreeStatRequest, read_some, recorded_stat, recorded_stat_seamed, stamp_proves_one_version};
use crate::index_stat::{IndexStamp, StatData};
use crate::worktree_clean_content_attribute_sources::ConversionEnvironment;
use crate::worktree_copy_stage_leaf_copying::{ClockPin, RECENT_CHANGE_NANOS, clock_reads, stamp_nanos};
use std::os::unix::ffi::OsStrExt as _;
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

/// What:
///  A request for `path` in `root`,
///  whose entry records `blob`,
/// against the attributes of the fixture's own Git directory.
/// Why:
///   A control that names a converting attribute writes the fixture's `.gitattributes` and its configuration,
/// and the
///  decision reads the same files Git would,
/// with no system or global file of the machine in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = (stamp, root, path, blob, mode, hashed) => ({ worktreeRoot: root, gitDir: `${root}/.git`, commonDir: `${root}/.git`, environment: fixtureEnvironment, path, blob, mode, format: 'sha1', stamp, hashed });
/// ```
fn request_at_stamp<'a>(stamp: IndexStamp, root: &'a Path, path: &'a [u8], blob: &'a [u8], mode: u32, hashed: &'a StdCell<u64>) -> WorktreeStatRequest<'a> {
    let fixture: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: &'a Path = Box::leak(root.join(".git").into_boxed_path());
    let environment: &'a ConversionEnvironment = Box::leak(Box::new(fixture));
    let request: WorktreeStatRequest<'a> = WorktreeStatRequest { worktree_root: root, git_dir, common_dir: git_dir, environment, path, blob, mode, format: ObjectFormat::Sha1, stamp, hashed, indexed_attributes: &[] };
    return request;
}

/// `recorded` for a landing whose index holds a `.gitattributes` in the directories `indexed`.
fn recorded_with_indexed(indexed: &[Vec<u8>], root: &Path, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData> {
    let hashed: StdCell<u64> = StdCell::new(0);
    let fixture: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: &Path = Box::leak(root.join(".git").into_boxed_path());
    let environment: &ConversionEnvironment = Box::leak(Box::new(fixture));
    let request: WorktreeStatRequest<'_> = WorktreeStatRequest { worktree_root: root, git_dir, common_dir: git_dir, environment, path, blob, mode, format: ObjectFormat::Sha1, stamp: now_stamp(), hashed: &hashed, indexed_attributes: indexed };
    return recorded_stat(&request);
}

/// `recorded` for an index file dated `stamp`.
fn recorded_at(stamp: IndexStamp, root: &Path, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData> {
    let hashed: StdCell<u64> = StdCell::new(0);
    return recorded_stat(&request_at_stamp(stamp, root, path, blob, mode, &hashed));
}

/// What:
///  The clock a control pins so that the attributes file at `path` counts as settled.
/// Why:
///   The proof trusts an attributes file only when its change time is a whole timestamp granularity older than the
///  clock,
/// and a fixture writes its attributes a moment before the control runs:
/// the control states the instant instead of
///  waiting for the clock to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = clockReads(changedNanos(`${root}/.gitattributes`) + RECENT_CHANGE_NANOS);
/// ```
fn settled_clock(path: &Path) -> ClockPin {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("the attributes file");
    return clock_reads(stamp_nanos((metadata.ctime(), metadata.ctime_nsec())) + RECENT_CHANGE_NANOS);
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

/// A symbolic link records nothing:
/// the link itself is rare,
/// its stat data is not needed for a commit to be fast,
/// and a link read across a directory swap cannot be proven to be the one that was examined.
#[test]
fn a_symbolic_link_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-link");
    std::os::unix::fs::symlink("target", root.join("l")).expect("symlink");
    age(root.join("l").as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"target");
    assert_eq!(recorded(root.as_path(), b"l", blob.as_slice(), 0o120_000), None, "a link keeps the zero stat data, so the next commit refreshes it");
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
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    let mut before_open = || {};
    assert_eq!(recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut seam, read: &mut read_some }), None);
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
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &spent);
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

/// What:
///  A repository fixture whose `.gitattributes` names `attribute` for `f.txt`,
/// with the clean filter `filter.weird` a script that prepends a `P`.
/// Why:
///   A conversion that is not the identity makes raw bytes and Git's view of the file differ,
/// so a path the attribute names must
///  keep the zero stat data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributes = (root: string, attribute: string) => { writeFile(`${root}/.gitattributes`, `f.txt ${attribute}\n`); writeFile(`${root}/weird.sh`, '#!/bin/sh\nprintf P\ncat\n', { mode: 0o755 }); git(['config', 'filter.weird.clean', `${root}/weird.sh`]); };
/// ```
fn attributes_naming(root: &Path, attribute: &str) {
    std::fs::write(root.join(".gitattributes"), format!("f.txt {attribute}\n")).expect("attributes");
    let filter: PathBuf = root.join("weird.sh");
    std::fs::write(&filter, b"#!/bin/sh\nprintf P\ncat\n").expect("filter");
    std::fs::set_permissions(&filter, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("the filter runs");
    git(root, &["config", "filter.weird.clean", filter.to_str().expect("a path")]);
}

/// A path a converting attribute names records nothing,
/// where the same bytes on a path no attribute names record:
/// the filter,
/// a `working-tree-encoding` and an `ident` each make Git's view of the file differ from its raw bytes.
#[test]
fn a_path_a_converting_attribute_names_records_nothing() {
    for attribute in ["filter=weird", "working-tree-encoding=UTF-16LE", "ident"] {
        let root: PathBuf = fixture_root(format!("index-worktree-stat-{}", attribute.split('=').next().expect("a name")).as_str());
        attributes_naming(root.as_path(), attribute);
        let _settled: ClockPin = settled_clock(root.join(".gitattributes").as_path());
        for name in ["f.txt", "g.txt"] {
            let file: PathBuf = root.join(name);
            std::fs::write(&file, b"a\n").expect("write");
            age(file.as_path());
        }
        let blob: Vec<u8> = blob_in(root.as_path(), b"a\n");
        assert_eq!(recorded(root.as_path(), b"f.txt", blob.as_slice(), 0o100_644), None, "{attribute}: the attribute may change these bytes");
        assert!(recorded(root.as_path(), b"g.txt", blob.as_slice(), 0o100_644).is_some(), "{attribute}: the bytes of a path the attribute does not name record");
        remove(root.as_path());
    }
}

/// A file whose bytes hold a carriage return records nothing,
/// because a line-ending rule Git applies whatever the attributes say may change them;
/// the same bytes without one record.
#[test]
fn a_file_with_a_carriage_return_records_nothing() {
    let root: PathBuf = fixture_root("index-worktree-stat-carriage");
    std::fs::write(root.join("a"), b"a\r\nb\n").expect("write");
    age(root.join("a").as_path());
    std::fs::write(root.join("b"), b"a\nb\n").expect("write");
    age(root.join("b").as_path());
    let carried: Vec<u8> = blob_in(root.as_path(), b"a\r\nb\n");
    let plain: Vec<u8> = blob_in(root.as_path(), b"a\nb\n");
    assert_eq!(recorded(root.as_path(), b"a", carried.as_slice(), 0o100_644), None, "a carriage return may be converted away on the way into the index");
    assert!(recorded(root.as_path(), b"b", plain.as_slice(), 0o100_644).is_some(), "the same bytes without one record");
    remove(root.as_path());
}

/// A read the proof rejects is charged to the landing all the same:
/// the bytes reached the disk and the bound is on the reading,
/// not on the accepting.
#[test]
fn a_read_that_is_rejected_is_still_charged() {
    let root: PathBuf = fixture_root("index-worktree-stat-charged");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    let hashed: StdCell<u64> = StdCell::new(0);
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    let written: PathBuf = file.clone();
    let mut before_open = || {};
    let mut after_read = move || {
        std::fs::write(&written, b"a TWO\n").expect("rewrite");
    };
    assert_eq!(recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut after_read, read: &mut read_some }), None, "the file changed under the read");
    assert!(hashed.get() >= 6, "the six bytes that were read are charged, and the landing has {}", hashed.get());
    remove(root.as_path());
}

/// One read takes at most what is left of the bound,
/// counted by the bytes the file hands over rather than by what the landing charges itself:
/// a file grown after its first examination is not read past the room the landing has.
#[test]
fn one_read_takes_at_most_what_is_left_of_the_bound() {
    let root: PathBuf = fixture_root("index-worktree-stat-room");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"abcd").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"abcd");
    let spent: u64 = crate::worktree_clean_content::CONTENT_LIMIT - 4;
    let hashed: StdCell<u64> = StdCell::new(spent);
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    let grown: PathBuf = file.clone();
    let mut before_open = move || {
        std::fs::write(&grown, b"abcdefghijklmnop").expect("grow");
    };
    let mut after_read = || {};
    let handed: &StdCell<u64> = Box::leak(Box::new(StdCell::new(0)));
    let counted: &StdCell<u64> = handed;
    let mut read = move |opened: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        let taken: usize = read_some(opened, buffer)?;
        counted.set(counted.get() + taken as u64);
        return Ok(taken);
    };
    assert_eq!(recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut after_read, read: &mut read }), None, "a file grown past the room is not recorded");
    assert_eq!(handed.get(), 0, "a file whose size is past the room is not read at all");
    assert_eq!(hashed.get(), spent, "and nothing is charged for it");
    remove(root.as_path());
}

/// Every byte the file hands over is charged,
/// counted by the read itself:
/// a read the proof then refuses for a changed file still took its bytes,
/// and one that fails part way took the bytes it got.
#[test]
fn every_byte_the_file_hands_over_is_charged() {
    let root: PathBuf = fixture_root("index-worktree-stat-handed");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    let hashed: StdCell<u64> = StdCell::new(0);
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    let handed: StdCell<u64> = StdCell::new(0);
    let counted: &StdCell<u64> = &handed;
    let mut read = move |opened: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        let taken: usize = read_some(opened, buffer)?;
        counted.set(counted.get() + taken as u64);
        return Ok(taken);
    };
    let written: PathBuf = file.clone();
    let mut before_open = || {};
    let mut after_read = move || {
        std::fs::write(&written, b"a TWO\n").expect("rewrite");
    };
    assert_eq!(recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut after_read, read: &mut read }), None, "the file changed under the read");
    assert_eq!(handed.get(), 6, "the six bytes the file handed over are counted");
    assert_eq!(hashed.get(), handed.get(), "and exactly those are charged");

    // A read that fails after some bytes took those bytes, and they are charged too.
    let partly: StdCell<u64> = StdCell::new(0);
    let failing: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &partly);
    let mut still_open = || {};
    let mut still_read = || {};
    let fails: StdCell<u64> = StdCell::new(0);
    let mut breaks = move |_opened: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        return if fails.get() == 0 {
            fails.set(1);
            buffer[..3].copy_from_slice(b"a o");
            Ok(3)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::Other))
        };
    };
    assert_eq!(recorded_stat_seamed(&failing, &mut ProofSeams { before_open: &mut still_open, after_read: &mut still_read, read: &mut breaks }), None, "a read that fails proves nothing");
    assert_eq!(partly.get(), 3, "the three bytes taken before the failure are charged");
    remove(root.as_path());
}

/// A read the system interrupts for ever ends in the same refusal a hard read error does,
/// after a counted number of tries:
/// nothing waits for an interruption to stop.
#[test]
fn repeatedly_interrupted_reads_end_in_a_refusal() {
    /// How many interruptions the control hands over before it stops the read itself,
    /// so a reader with no bound cannot hang the run.
    const CONTROL_STOPS_AFTER: usize = 10_000;
    let root: PathBuf = fixture_root("index-worktree-stat-interrupted");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    let hashed: StdCell<u64> = StdCell::new(0);
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    let attempts: &StdCell<usize> = Box::leak(Box::new(StdCell::new(0)));
    let mut read = move |_opened: &std::fs::File, _buffer: &mut [u8]| -> std::io::Result<usize> {
        attempts.set(attempts.get() + 1);
        if attempts.get() > CONTROL_STOPS_AFTER {
            return Ok(0);
        }
        return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
    };
    let mut before_open = || {};
    let mut after_read = || {};
    assert_eq!(
        recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut after_read, read: &mut read }),
        None,
        "an interrupted read proves nothing, so the entry keeps the zero stat data"
    );
    assert!(attempts.get() < CONTROL_STOPS_AFTER, "the reader stopped on its own after {} of {CONTROL_STOPS_AFTER} interruptions", attempts.get());
    remove(root.as_path());
}

/// One interruption is not a fact about the file,
/// so the read is tried again and the file records its stat data.
#[test]
fn one_interrupted_read_is_tried_again() {
    let root: PathBuf = fixture_root("index-worktree-stat-one-interruption");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    let attempts: &StdCell<usize> = Box::leak(Box::new(StdCell::new(0)));
    let mut read = move |opened: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        attempts.set(attempts.get() + 1);
        if attempts.get() == 1 {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
        }
        return read_some(opened, buffer);
    };
    let hashed: StdCell<u64> = StdCell::new(0);
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    let mut before_open = || {};
    let mut after_read = || {};
    let recorded: Option<StatData> = recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut after_read, read: &mut read });
    assert!(recorded.is_some(), "an interruption costs nothing, and the file records its stat data");
    // The interruption, the read that takes the file's bytes, and the read that reports the end of the file.
    assert_eq!(attempts.get(), 3, "the interruption is retried once");
    remove(root.as_path());
}

/// A fifo renamed over the path before the open is not opened for reading:
/// an open that can wait for a writer would hold the landing's index lock for ever.
#[test]
fn a_fifo_swapped_over_the_path_is_not_opened() {
    let root: PathBuf = fixture_root("index-worktree-stat-fifo");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a one\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a one\n");
    // The probe runs on a thread of its own so the control ends even when the open waits for a writer.
    let held_root: &'static std::path::Path = Box::leak(root.clone().into_boxed_path());
    let held_path: &'static [u8] = Box::leak(b"a".to_vec().into_boxed_slice());
    let held_blob: &'static [u8] = Box::leak(blob.into_boxed_slice());
    let swapped: &'static std::path::Path = Box::leak(file.clone().into_boxed_path());
    let mut before_open = move || {
        let _removed = std::fs::remove_file(swapped);
        let name: std::ffi::CString = std::ffi::CString::new(swapped.as_os_str().as_bytes()).expect("no NUL in a fixture path");
        // SAFETY: `name` is a NUL-terminated string that outlives the call.
        let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
        assert_eq!(made, 0, "the control made a fifo at {}", swapped.display());
    };
    let mut after_read = || {};
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let hashed: &'static StdCell<u64> = Box::leak(Box::new(StdCell::new(0)));
        let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), held_root, held_path, held_blob, 0o100_644, hashed);
        let answer: Option<StatData> = recorded_stat_seamed(&request, &mut ProofSeams { before_open: &mut before_open, after_read: &mut after_read, read: &mut read_some });
        let _sent = sender.send(answer);
    });
    let answer: Option<StatData> = receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("the proof must not wait on a fifo for a writer");
    assert_eq!(answer, None, "a fifo is not a file the proof records");
    remove(root.as_path());
}

/// An attributes file the index holds applies where the worktree's copy is gone,
/// so the path keeps the zero stat data even though its raw bytes equal the blob,
/// while the same fixture with no indexed copy records.
#[test]
fn an_attributes_file_only_the_index_holds_refuses_the_record() {
    let root: PathBuf = fixture_root("index-worktree-stat-indexed-attributes");
    std::fs::write(root.join(".gitattributes"), b"a filter=weird\n").expect("attributes");
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a\n");
    std::fs::remove_file(root.join(".gitattributes")).expect("the worktree copy is gone");
    assert_eq!(recorded(root.as_path(), b"a", blob.as_slice(), 0o100_644).unwrap_or_default(), stat_of_file(file.as_path()), "without the index's copy the proof sees no attributes");
    assert_eq!(recorded_with_indexed(&[Vec::new()], root.as_path(), b"a", blob.as_slice(), 0o100_644), None, "the index's copy applies where the worktree's is gone");
    remove(root.as_path());
}

/// What:
///  A repository whose worktree `.gitattributes` is `bytes` long and names no converting attribute,
/// with one aged file that holds a blob.
/// Why:
///   A control for the attribute reads' cost needs an attribute file of a known size that changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributesOf = (root, bytes) => writeFile(`${root}/.gitattributes`, 'a -text\n' + '#'.repeat(bytes - 7) + '\n');
/// ```
fn attributes_of(root: &Path, bytes: usize) {
    let mut text: Vec<u8> = b"a -text\n".to_vec();
    text.extend(std::iter::repeat_n(b'#', bytes.saturating_sub(text.len() + 1)));
    text.push(b'\n');
    std::fs::write(root.join(".gitattributes"), text).expect("attributes");
}

/// An attribute file that does not fit in the room the landing has left refuses the record,
/// while the same file fits and records when the room is whole.
#[test]
fn an_attribute_read_that_does_not_fit_the_room_refuses_the_record() {
    let root: PathBuf = fixture_root("index-worktree-stat-attribute-room");
    attributes_of(root.as_path(), 4096);
    let _settled: ClockPin = settled_clock(root.join(".gitattributes").as_path());
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a\n");
    let spent: StdCell<u64> = StdCell::new(crate::worktree_clean_content::CONTENT_LIMIT - 16);
    let tight: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &spent);
    assert_eq!(recorded_stat(&tight), None, "an attribute file of four kibibytes does not fit in sixteen bytes of room");
    assert_eq!(spent.get(), crate::worktree_clean_content::CONTENT_LIMIT - 16, "and nothing is charged for it");
    let whole: StdCell<u64> = StdCell::new(0);
    let roomy: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &whole);
    assert!(recorded_stat(&roomy).is_some(), "with the whole bound the same file records");
    remove(root.as_path());
}

/// The attribute files a landing reads are charged to its content bound,
/// so the bytes it reads under the index lock are the bytes it accounts for.
#[test]
fn the_attribute_files_a_landing_reads_are_charged() {
    let root: PathBuf = fixture_root("index-worktree-stat-attribute-charged");
    attributes_of(root.as_path(), 4096);
    let _settled: ClockPin = settled_clock(root.join(".gitattributes").as_path());
    let file: PathBuf = root.join("a");
    std::fs::write(&file, b"a\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a\n");
    let hashed: StdCell<u64> = StdCell::new(0);
    let request: WorktreeStatRequest<'_> = request_at_stamp(now_stamp(), root.as_path(), b"a", blob.as_slice(), 0o100_644, &hashed);
    assert!(recorded_stat(&request).is_some(), "the file records");
    assert!(hashed.get() >= 4096 + 2, "the attribute file and the file itself are charged, and the landing has {}", hashed.get());
    remove(root.as_path());
}

/// A nested path with an indexed root `.gitattributes` is judged
/// without a panic:
/// the handle of the file's own directory is borrowed then,
/// and replacing the cache under that borrow panics.
/// The worktree's copy is the one Git applies here,
/// so the file records.
#[test]
fn a_nested_path_with_an_indexed_root_attributes_file_records_rather_than_panicking() {
    let root: PathBuf = fixture_root("index-worktree-stat-nested-attributes");
    std::fs::write(root.join(".gitattributes"), b"a -text\n").expect("attributes");
    std::fs::create_dir_all(root.join("d")).expect("directory");
    let file: PathBuf = root.join("d/a");
    std::fs::write(&file, b"a\n").expect("write");
    age(file.as_path());
    let _settled: ClockPin = settled_clock(root.join(".gitattributes").as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a\n");
    assert_eq!(
        recorded_with_indexed(&[Vec::new()], root.as_path(), b"d/a", blob.as_slice(), 0o100_644).unwrap_or_default(),
        stat_of_file(file.as_path()),
        "the worktree's copy is the one Git applies, so the nested file records"
    );
    remove(root.as_path());
}

/// The same for an indexed `.gitattributes` of a strict ancestor directory,
/// which replaces the cached handle the same way.
#[test]
fn a_nested_path_with_an_indexed_ancestor_attributes_file_records_rather_than_panicking() {
    let root: PathBuf = fixture_root("index-worktree-stat-ancestor-attributes");
    std::fs::create_dir_all(root.join("d/e")).expect("directory");
    std::fs::write(root.join("d/.gitattributes"), b"a -text\n").expect("attributes");
    let _settled: ClockPin = settled_clock(root.join("d/.gitattributes").as_path());
    let file: PathBuf = root.join("d/e/a");
    std::fs::write(&file, b"a\n").expect("write");
    age(file.as_path());
    let blob: Vec<u8> = blob_in(root.as_path(), b"a\n");
    assert_eq!(
        recorded_with_indexed(&[b"d".to_vec()], root.as_path(), b"d/e/a", blob.as_slice(), 0o100_644).unwrap_or_default(),
        stat_of_file(file.as_path()),
        "the ancestor's worktree copy is the one Git applies, so the nested file records"
    );
    remove(root.as_path());
}

/// What:
///  A change time with no nanoseconds proves nothing,
///  so no state it reports can be one version of a file.
/// Why:
///  A file system that stores whole seconds reports a change time with none,
/// and a same-size rewrite whose modification time is put back inside that second leaves every field the proof
///  compares unchanged,
/// so the entry would record stat data over an edit that stays hidden.
/// This host's file systems store nanoseconds,
/// so the rule is checked at the moment it reads,
/// and the controls above show a file whose change time does carry nanoseconds still records
/// (a cross-vendor review's seventh finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(stampProvesOneVersion([1700000000, 0])).toBe(false); expect(stampProvesOneVersion([1700000000, 1])).toBe(true);
/// ```
#[test]
fn a_change_time_without_nanoseconds_proves_nothing() {
    assert!(
        !stamp_proves_one_version((1_700_000_000, 0)),
        "a whole-second change time cannot tell two rewrites in one second apart"
    );
    assert!(stamp_proves_one_version((1_700_000_000, 1)), "one nanosecond is enough to tell them apart");
}
