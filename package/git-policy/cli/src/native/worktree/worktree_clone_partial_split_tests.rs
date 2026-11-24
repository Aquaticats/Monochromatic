//! Controls for the split of a source with changes:
//! which paths a clone takes and which Git writes,
//! one per case the design answers,
//! and the states
//!  that end the rung.

use super::*;
use crate::repo_read_index::IndexEntry;
use crate::subvolume_index::{StatFields, StoredStat};
use crate::subvolume_index_files::fields_of;
use crate::worktree_clean::StatEntry;
use crate::worktree_clean_stat_verdicts::ASSUME_VALID;
use crate::worktree_ladder_test_support::tracked;
use std::path::PathBuf;

/// An object name of twenty bytes,
/// one byte per position,
/// so a difference between two names is one number in a failure.
fn object_name(seed: u8) -> Vec<u8> {
    return vec![seed; 20];
}

/// What one fixture file is,
/// in the index and in the tree.
struct Filed {
    /// The path below the fixture root.
    path: &'static str,
    /// The mode the tree and the index record.
    mode: u32,
    /// The object name the tree records.
    tree: u8,
    /// The object name the index records.
    index: u8,
}

/// What:
///  The world of one control:
/// the fixture directory with its files,
/// the index entries the fixture's own files
///  justify and the tree they name.
/// Why:
///   The split judges three things,
/// so a control states all three and takes the stat data from the file
///  itself,
/// which is the one instant an entry proves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type World = { root: string; entries: StatEntry[]; indexed: IndexEntry[]; tree: FlatEntry[]; indexSeconds: number };
/// ```
struct World {
    /// The fixture directory.
    root: PathBuf,
    /// The index's entries with their recorded stat data,
    /// in path order.
    entries: Vec<StatEntry>,
    /// The same entries as the plain reader gives them.
    indexed: Vec<IndexEntry>,
    /// The tree of `HEAD`,
    /// in path order.
    tree: Vec<FlatEntry>,
    /// The whole seconds of the index file's time,
    /// which these controls put after every file.
    index_seconds: u32,
}

/// The recorded stat data of `path` below `root`.
fn recorded(path: &std::path::Path) -> (crate::subvolume_index::StoredStat, StatFields) {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("the fixture's file");
    let fields: StatFields = fields_of(&metadata);
    let stored: StoredStat = StoredStat {
        ctime_seconds: fields.ctime_seconds,
        ctime_nanos: fields.ctime_nanos,
        mtime_seconds: fields.mtime_seconds,
        mtime_nanos: fields.mtime_nanos,
        ino: fields.ino,
        uid: fields.uid,
        gid: fields.gid,
        size: fields.size,
    };
    return (stored, fields);
}

/// A fixture directory with the files of `files`,
/// and the index entries their own stat data justifies,
/// in path order.
fn world(name: &str, files: &[Filed]) -> World {
    let root: PathBuf = crate::test_support::fixture(name);
    let mut sorted: Vec<&Filed> = files.iter().collect();
    sorted.sort_by_key(|filed| { return filed.path; });
    for filed in &sorted {
        if filed.mode == 0o120_000 {
            std::os::unix::fs::symlink("plain.txt", root.join(filed.path)).expect("link");
            crate::test_support::age(root.join(filed.path).as_path());
            continue;
        }
        tracked(root.as_path(), filed.path, filed.path.as_bytes());
    }
    let mut entries: Vec<StatEntry> = Vec::new();
    let mut indexed: Vec<IndexEntry> = Vec::new();
    let mut tree: Vec<FlatEntry> = Vec::new();
    let mut latest: u32 = 0;
    for filed in &sorted {
        let (stored, fields) = recorded(root.join(filed.path).as_path());
        latest = latest.max(fields.mtime_seconds);
        entries.push(StatEntry { path: filed.path.as_bytes().to_vec(), flags: 0, mode: filed.mode, stored });
        indexed.push(IndexEntry { path: filed.path.as_bytes().to_vec(), mode: filed.mode, id: object_name(filed.index), stage: 0, extended: 0 });
        tree.push(FlatEntry { path: filed.path.as_bytes().to_vec(), mode: filed.mode, id: object_name(filed.tree) });
    }
    // A whole second after every file, so no entry of these controls is racy.
    let index_seconds: u32 = latest + 1;
    return World { root: std::fs::canonicalize(root).expect("canonical"), entries, indexed, tree, index_seconds };
}

impl World {
    /// Remove the fixture at the end of a control.
    fn remove(&self) {
        crate::test_support::remove(self.root.as_path());
    }

    /// The split of this world.
    fn split(&self) -> Result<Split, CleanRefusal> {
        return split_the_tree(self.entries.as_slice(), self.indexed.as_slice(), self.tree.as_slice(), self.root.as_path(), self.index_seconds);
    }

    /// The names the split clones.
    fn cloned(&self) -> Vec<String> {
        let split: Split = self.split().expect("the split");
        return split.clones.iter().filter_map(|at| { return self.entries.get(*at).map(|entry| { return String::from_utf8_lossy(entry.path.as_slice()).into_owned(); }); }).collect();
    }

    /// The names the split hands to Git.
    fn written(&self) -> Vec<String> {
        let split: Split = self.split().expect("the split");
        return split.writes.iter().map(|path| { return String::from_utf8_lossy(path.as_slice()).into_owned(); }).collect();
    }

    /// Where `name` sits in the index and in the tree.
    fn position(&self, name: &str) -> usize {
        return self.tree.iter().position(|entry| { return entry.path.as_slice() == name.as_bytes(); }).expect("a tree path of the fixture");
    }
}

/// The three files these controls start from,
/// in path order:
/// one the entry proves,
/// a link,
/// and one whose bytes a control may change.
fn three_files() -> Vec<Filed> {
    return vec![
        Filed { path: "edited.txt", mode: 0o100_644, tree: 3, index: 3 },
        Filed { path: "link", mode: 0o120_000, tree: 2, index: 2 },
        Filed { path: "plain.txt", mode: 0o100_644, tree: 1, index: 1 },
    ];
}

/// A file whose own stat data is its entry's is cloned,
/// and a link is cloned as a link.
#[test]
fn the_files_their_entries_prove_are_cloned() {
    let world: World = world("proven", three_files().as_slice());
    let split: Split = world.split().expect("a split");
    assert_eq!(split.clones, vec![0, 1, 2]);
    assert!(split.writes.is_empty(), "nothing is left to Git");
    world.remove();
}

/// A file whose bytes changed,
/// by a length the entry recorded,
/// is written by Git and never cloned.
#[test]
fn a_changed_file_is_written_not_cloned() {
    let world: World = world("changed", three_files().as_slice());
    std::fs::write(world.root.join("edited.txt"), b"longer than what the entry recorded\n").expect("the edit");
    assert_eq!(world.written(), vec![String::from("edited.txt")]);
    assert_eq!(world.cloned(), vec![String::from("link"), String::from("plain.txt")]);
    world.remove();
}

/// An entry whose file is missing is written by Git:
/// a deletion the source has not staged.
#[test]
fn a_missing_file_is_written() {
    let world: World = world("missing", three_files().as_slice());
    std::fs::remove_file(world.root.join("plain.txt")).expect("the deletion");
    assert_eq!(world.written(), vec![String::from("plain.txt")]);
    world.remove();
}

/// What:
///  Give `path` the modification time `seconds`,
/// so one file is as new as the index and the others are older.
/// Why:
///   Git calls an entry racy when its file is as new as the index file,
/// and a fixture that leaves every
///  file at one instant cannot tell the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setModified(path, seconds);
/// ```
fn set_modified(path: &std::path::Path, seconds: u64) {
    let when: std::time::SystemTime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds);
    let times: std::fs::FileTimes = std::fs::FileTimes::new().set_modified(when).set_accessed(when);
    std::fs::File::open(path).expect("the file whose times are set").set_times(times).expect("the modification time");
}

/// An entry as new as the index file proves nothing,
/// so its file is written even when its bytes are the entry's.
#[test]
fn a_racy_entry_is_written() {
    let world: World = world("racy", three_files().as_slice());
    let at: usize = world.position("plain.txt");
    let later: u64 = u64::from(world.entries[at].stored.mtime_seconds) + 5;
    set_modified(world.root.join("plain.txt").as_path(), later);
    let mut entries: Vec<StatEntry> = world.entries.clone();
    entries[at].stored = recorded(world.root.join("plain.txt").as_path()).0;
    let split: Split = split_the_tree(entries.as_slice(), world.indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), later as u32).expect("a split");
    assert_eq!(split.writes, vec![b"plain.txt".to_vec()], "the entry as new as the index is not cloned");
    assert_eq!(split.clones.len(), 2);
    world.remove();
}

/// An entry whose recorded nanoseconds a file's own time cannot equal proves nothing about its bytes,
/// so the file is written.
#[test]
fn an_entry_without_nanoseconds_is_written() {
    let world: World = world("nanos", three_files().as_slice());
    let at: usize = world.position("plain.txt");
    let mut entries: Vec<StatEntry> = world.entries.clone();
    let nanos: u32 = entries[at].stored.mtime_nanos;
    entries[at].stored.mtime_nanos = if nanos == 0 { 1 } else { 0 };
    let split: Split = split_the_tree(entries.as_slice(), world.indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), world.index_seconds).expect("a split");
    assert_eq!(split.writes, vec![b"plain.txt".to_vec()]);
    world.remove();
}

/// A path staged with another blob is written:
/// the index's own bytes never reach the new worktree.
#[test]
fn a_staged_path_is_written_not_cloned() {
    let mut files: Vec<Filed> = three_files();
    files[2].index = 9;
    let world: World = world("staged", files.as_slice());
    assert_eq!(world.written(), vec![String::from("plain.txt")]);
    world.remove();
}

/// A path the index lacks,
/// a deletion staged in the source,
/// is written.
#[test]
fn a_path_the_index_lacks_is_written() {
    let world: World = world("unstaged-deletion", three_files().as_slice());
    let at: usize = world.position("plain.txt");
    let mut entries: Vec<StatEntry> = world.entries.clone();
    let mut indexed: Vec<IndexEntry> = world.indexed.clone();
    entries.remove(at);
    indexed.remove(at);
    let split: Split = split_the_tree(entries.as_slice(), indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), world.index_seconds).expect("a split");
    assert_eq!(split.writes, vec![b"plain.txt".to_vec()]);
    world.remove();
}

/// A path only the index names (a staged addition) reaches nothing:
/// the new worktree's index is built from the tree.
#[test]
fn a_staged_addition_reaches_nothing() {
    let world: World = world("added", three_files().as_slice());
    let mut entries: Vec<StatEntry> = world.entries.clone();
    let mut indexed: Vec<IndexEntry> = world.indexed.clone();
    entries.insert(0, StatEntry { path: b"added.txt".to_vec(), flags: 0, mode: 0o100_644, stored: world.entries[0].stored });
    indexed.insert(0, IndexEntry { path: b"added.txt".to_vec(), mode: 0o100_644, id: object_name(7), stage: 0, extended: 0 });
    let split: Split = split_the_tree(entries.as_slice(), indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), world.index_seconds).expect("a split");
    assert!(split.writes.iter().all(|path| { return path.as_slice() != b"added.txt"; }), "the staged addition is no tree path");
    assert_eq!(split.clones.len(), 3);
    world.remove();
}

/// A submodule in the tree ends the rung with its reason.
#[test]
fn a_submodule_ends_the_split() {
    let mut files: Vec<Filed> = three_files();
    files[2].mode = 0o160_000;
    let world: World = world("submodule", files.as_slice());
    assert_eq!(world.split().expect_err("a refusal").kind, CleanRefusalKind::NotPlain);
    world.remove();
}

/// A skip-worktree or intent-to-add entry ends the rung with its reason.
#[test]
fn an_entry_git_does_not_judge_ends_the_split() {
    let world: World = world("skip-worktree", three_files().as_slice());
    let mut indexed: Vec<IndexEntry> = world.indexed.clone();
    indexed[0].extended = 0x4000;
    assert_eq!(split_the_tree(world.entries.as_slice(), indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), world.index_seconds).expect_err("a refusal").kind, CleanRefusalKind::NotPlain);
    world.remove();
}

/// An unmerged entry ends the rung with its reason.
#[test]
fn an_unmerged_entry_ends_the_split() {
    let world: World = world("unmerged", three_files().as_slice());
    let mut indexed: Vec<IndexEntry> = world.indexed.clone();
    indexed[0].stage = 2;
    assert_eq!(split_the_tree(world.entries.as_slice(), indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), world.index_seconds).expect_err("a refusal").kind, CleanRefusalKind::NotPlain);
    world.remove();
}

/// An assume-unchanged entry ends the rung:
/// Git is told not to judge the file,
/// so no proof here can.
#[test]
fn an_assume_unchanged_entry_ends_the_split() {
    let world: World = world("assume-valid", three_files().as_slice());
    let mut entries: Vec<StatEntry> = world.entries.clone();
    entries[0].flags |= ASSUME_VALID;
    assert_eq!(split_the_tree(entries.as_slice(), world.indexed.as_slice(), world.tree.as_slice(), world.root.as_path(), world.index_seconds).expect_err("a refusal").kind, CleanRefusalKind::NotPlain);
    world.remove();
}

/// A path reached through a link is written rather than cloned:
/// the file the link names is not the one the entry recorded.
#[test]
fn a_path_below_a_link_is_written() {
    let files: Vec<Filed> = vec![
        Filed { path: "plain.txt", mode: 0o100_644, tree: 1, index: 1 },
        Filed { path: "sub/leaf.txt", mode: 0o100_644, tree: 2, index: 2 },
    ];
    let world: World = world("below-a-link", files.as_slice());
    std::fs::rename(world.root.join("sub"), world.root.join("elsewhere")).expect("the move");
    std::os::unix::fs::symlink("elsewhere", world.root.join("sub")).expect("the link");
    assert_eq!(world.written(), vec![String::from("sub/leaf.txt")]);
    world.remove();
}
