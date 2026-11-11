//! Controls for the reset of an index held in memory,
//! including the monitor-on behaviour that no real-Git control can reach
//! (the fixtures never turn fsmonitor on):
//! those cases are worked out from `fsmonitor.c`,
//! `read-cache.c` and `dir.c`.

use super::*;
use crate::index_cache_tree::CacheTree;
use crate::index_fsmonitor::Fsmonitor;
use crate::index_reset_worktree_stat_source::WorktreeStatSource;
use crate::index_stat::{IndexStamp, StatData};
use crate::index_untracked::{Untracked, UntrackedDir};

/// The settings of a repository with no monitor.
const PLAIN: IndexSettings = IndexSettings { monitor: false, untracked: UntrackedMode::Keep, shared_repository: 0, skip_hash: false };

/// The settings of a repository whose fsmonitor is on.
const MONITORED: IndexSettings = IndexSettings { monitor: true, untracked: UntrackedMode::Keep, shared_repository: 0, skip_hash: false };

/// An entry with real-looking stat data.
fn held(path: &str, id: u8, valid: bool) -> FullEntry {
    return FullEntry {
        stat: StatData { mtime_sec: 5, size: 3, ..StatData::default() },
        mode: 0o100_644,
        id: vec![id; 20],
        flags: 0,
        extended: 0,
        path: path.as_bytes().to_vec(),
        monitor_valid: valid,
    };
}

/// A tree entry.
fn wanted(path: &str, id: u8) -> FlatEntry {
    return FlatEntry { path: path.as_bytes().to_vec(), mode: 0o100_644, id: vec![id; 20] };
}

/// An index of `entries` with a cache-tree,
/// an untracked cache listing `d` and,
/// when asked,
/// an fsmonitor extension.
fn index_of(entries: Vec<FullEntry>, monitored: bool) -> FullIndex {
    let directory = |name: &str| -> UntrackedDir {
        return UntrackedDir { name: name.as_bytes().to_vec(), recurse: true, valid: true, check_only: false, stat: StatData::default(), exclude_id: vec![0; 20], untracked: vec![b"u".to_vec()], dirs: Vec::new() };
    };
    let mut top: UntrackedDir = directory("");
    top.dirs.push(directory("d"));
    return FullIndex {
        version: 2,
        entries,
        cache_tree: Some(CacheTree { entry_count: 2, id: vec![9; 20], subtrees: vec![(b"d".to_vec(), CacheTree { entry_count: 1, id: vec![8; 20], subtrees: Vec::new() })] }),
        resolve_undo: None,
        untracked: Some(Untracked {
            ident: Vec::new(),
            info_exclude: (StatData::default(), vec![0; 20]),
            excludes_file: (StatData::default(), vec![0; 20]),
            dir_flags: 0,
            exclude_per_dir: b".gitignore".to_vec(),
            root: Some(top),
        }),
        fsmonitor: monitored.then(|| { return Fsmonitor { token: b"t".to_vec(), dirty: Vec::new() }; }),
    };
}

/// A new entry has zero stat data and is not valid for the monitor;
/// an untouched entry keeps both.
#[test]
fn a_reset_entry_has_no_stat_data_and_the_others_keep_theirs() {
    let mut index: FullIndex = index_of(vec![held("a", 1, true), held("d/x", 2, true)], false);
    reset_paths(&mut index, &[b"a".to_vec()], &[wanted("a", 7)], PLAIN, IndexStamp::NONE, WorktreeStat::None).expect("reset");
    assert_eq!(index.entries[0].stat, StatData::default());
    assert_eq!(index.entries[0].id, vec![7; 20]);
    assert_eq!(index.entries[1].stat.mtime_sec, 5);
    assert!(index.fsmonitor.is_none(), "without a monitor the extension is dropped");
}

/// A caller that proves a file holds the entry's bytes names the stat data the entry gets,
/// and a path it proves nothing for keeps
/// the all-zero value.
#[test]
fn a_recorded_stat_data_source_fills_in_the_entries_it_proves() {
    struct PlainFiles;
    impl WorktreeStatSource for PlainFiles {
        fn recorded(&self, _path: &[u8], _blob: &[u8], mode: u32) -> Option<StatData> {
            return if mode == 0o100_644 { Some(StatData { mtime_sec: 5, size: 7, ..StatData::default() }) } else { None };
        }
    }
    let recorded: StatData = StatData { mtime_sec: 5, size: 7, ..StatData::default() };
    let executable: FlatEntry = FlatEntry { path: b"b".to_vec(), mode: 0o100_755, id: vec![8; 20] };
    let mut index: FullIndex = index_of(vec![held("a", 1, true), held("b", 2, true)], false);
    reset_paths(&mut index, &[b"a".to_vec(), b"b".to_vec()], &[wanted("a", 7), executable], PLAIN, IndexStamp::NONE, WorktreeStat::Recorded(&PlainFiles)).expect("reset");
    assert_eq!(index.entries[0].stat, recorded, "the proven path records the stat data the source names");
    assert_eq!(index.entries[1].stat, StatData::default(), "a path the source proves nothing for keeps the zero value");
}

/// A name that already equals the tree is not touched,
/// and nothing above it is invalidated.
#[test]
fn an_equal_entry_changes_nothing() {
    let mut index: FullIndex = index_of(vec![held("a", 1, true)], false);
    let before: FullIndex = index.clone();
    reset_paths(&mut index, &[b"a".to_vec()], &[wanted("a", 1)], PLAIN, IndexStamp::NONE, WorktreeStat::None).expect("reset");
    assert_eq!(index, before);
}

/// With the monitor on,
/// the extension is kept,
/// replaced and added entries are dirty,
/// removed ones leave the bitmap,
/// gitlinks
/// are always dirty,
/// and replacing an entry clears the untracked cache of its directory.
#[test]
fn with_the_monitor_on_new_entries_are_dirty_and_replacing_clears_the_untracked_cache() {
    let mut link: FullEntry = held("g", 4, true);
    link.mode = 0o160_000;
    let mut index: FullIndex = index_of(vec![held("a", 1, true), held("d/x", 2, true), held("d/y", 3, true), link], true);
    index.fsmonitor = Some(Fsmonitor { token: b"t".to_vec(), dirty: vec![2] });
    reset_paths(&mut index, &[b"d/x".to_vec(), b"d/y".to_vec(), b"e".to_vec()], &[wanted("d/x", 7), wanted("e", 6)], MONITORED, IndexStamp::NONE, WorktreeStat::None).expect("reset");
    let listed: Vec<(&[u8], bool)> = index.entries.iter().map(|entry| { return (entry.path.as_slice(), entry.monitor_valid); }).collect();
    assert_eq!(listed, vec![(&b"a"[..], true), (&b"d/x"[..], false), (&b"e"[..], false), (&b"g"[..], false)]);
    let top = index.untracked.as_ref().and_then(|cache| { return cache.root.as_ref(); }).expect("kept");
    assert!(!top.dirs[0].valid, "replacing d/x cleared the listing of d");
    assert_eq!(index.cache_tree.as_ref().map(|tree| { return tree.entry_count; }), Some(-1));
}

/// Without the monitor,
/// replacing an entry leaves the untracked cache alone,
/// while adding one clears it.
#[test]
fn without_the_monitor_only_adding_and_removing_clear_the_untracked_cache() {
    let mut index: FullIndex = index_of(vec![held("d/x", 2, true)], false);
    reset_paths(&mut index, &[b"d/x".to_vec()], &[wanted("d/x", 7)], PLAIN, IndexStamp::NONE, WorktreeStat::None).expect("reset");
    assert!(index.untracked.as_ref().and_then(|cache| { return cache.root.as_ref(); }).expect("kept").dirs[0].valid);
    reset_paths(&mut index, &[b"d/n".to_vec()], &[wanted("d/n", 7)], PLAIN, IndexStamp::NONE, WorktreeStat::None).expect("reset");
    assert!(!index.untracked.as_ref().and_then(|cache| { return cache.root.as_ref(); }).expect("kept").dirs[0].valid);
}

/// Forms the update leaves to Git return `None`:
/// a monitor with no extension,
/// an untracked cache Git would add,
/// a bitmap
/// longer than the entries,
/// a file where the index has a directory,
/// a name Git refuses,
/// and a racily clean entry.
#[test]
fn forms_that_depend_on_more_than_the_tree_and_the_index_are_declined() {
    let names: Vec<Vec<u8>> = vec![b"a".to_vec()];
    let tree: Vec<FlatEntry> = vec![wanted("a", 7)];
    assert!(reset_paths(&mut index_of(vec![held("a", 1, true)], false), &names, &tree, MONITORED, IndexStamp::NONE, WorktreeStat::None).is_none(), "monitor without extension");
    let mut long: FullIndex = index_of(vec![held("a", 1, true)], true);
    long.fsmonitor = Some(Fsmonitor { token: Vec::new(), dirty: vec![3] });
    assert!(reset_paths(&mut long, &names, &tree, MONITORED, IndexStamp::NONE, WorktreeStat::None).is_none(), "bitmap past the entries");
    let mut absent: FullIndex = index_of(vec![held("a", 1, true)], false);
    absent.untracked = None;
    let adding: IndexSettings = IndexSettings { monitor: false, untracked: UntrackedMode::Write, shared_repository: 0, skip_hash: false };
    assert!(reset_paths(&mut absent, &names, &tree, adding, IndexStamp::NONE, WorktreeStat::None).is_none(), "untracked cache to add");
    let mut directory: FullIndex = index_of(vec![held("d/x", 1, true)], false);
    assert!(reset_paths(&mut directory, &[b"d".to_vec()], &[wanted("d", 7)], PLAIN, IndexStamp::NONE, WorktreeStat::None).is_none(), "file over directory");
    for bad in [".git/hooks/x", "a/../b", "a//b", ".GIT", "x/.git ", "git~1"] {
        let mut refused: FullIndex = index_of(vec![held("z", 1, true)], false);
        assert!(reset_paths(&mut refused, &[bad.as_bytes().to_vec()], &[wanted(bad, 7)], PLAIN, IndexStamp::NONE, WorktreeStat::None).is_none(), "{bad}");
    }
    let mut racy: FullIndex = index_of(vec![held("a", 1, true), held("b", 2, true)], false);
    assert!(reset_paths(&mut racy, &names, &tree, PLAIN, IndexStamp { seconds: 5, nanoseconds: 0 }, WorktreeStat::None).is_none(), "an entry recorded at the index file's second");
    assert!(reset_paths(&mut racy, &names, &tree, PLAIN, IndexStamp { seconds: 6, nanoseconds: 0 }, WorktreeStat::None).is_some(), "one second later it is not racy");
}

/// Executable bits are kept,
/// other file modes become plain files,
/// and a directory mode in a tree becomes a gitlink as
/// `create_ce_mode` makes it.
#[test]
fn modes_are_made_as_create_ce_mode_makes_them() {
    assert_eq!(index_mode(0o100_755), 0o100_755);
    assert_eq!(index_mode(0o100_664), 0o100_644);
    assert_eq!(index_mode(0o100_775), 0o100_755);
    assert_eq!(index_mode(0o120_000), 0o120_000);
    assert_eq!(index_mode(0o160_000), 0o160_000);
    assert_eq!(index_mode(0o040_000), 0o160_000);
}

/// Names Git refuses under `core.protectNTFS` and `core.protectHFS`,
/// and their aliases,
/// are declined:
/// the stream and
/// trailing-character spellings of `.git` and of a link named `.gitmodules`,
/// the short names,
/// the backslash spellings and the
/// ignorable characters of a case-folding disk.
#[test]
fn names_that_git_refuses_under_the_protection_settings_are_declined() {
    let link: u32 = 0o120_000;
    let refused: [(&str, u32); 14] = [
        (".gitmodules::$DATA", link),
        (".gitmodules . .", link),
        (".gitmodules ", link),
        ("GITMOD~1", link),
        ("gitmod~1 ", link),
        (".git::$INDEX_ALLOCATION", 0o100_644),
        ("dir/.git ", 0o100_644),
        ("dir\\.git\\config", 0o100_644),
        ("a\\b", 0o100_644),
        ("GIT~1", 0o100_644),
        ("dir/.g\u{200c}it", 0o100_644),
        (".git\u{200d}", 0o100_644),
        ("x/.\u{206a}git/y", 0o100_644),
        (".GitModules", link),
    ];
    for (name, mode) in refused {
        let mut index: FullIndex = index_of(vec![held("z", 1, true)], false);
        let entry: FlatEntry = FlatEntry { path: name.as_bytes().to_vec(), mode, id: vec![7; 20] };
        assert!(reset_paths(&mut index, &[name.as_bytes().to_vec()], &[entry], PLAIN, IndexStamp::NONE, WorktreeStat::None).is_none(), "{name:?}");
    }
}

/// Names that look like those but are not,
/// and ordinary names,
/// are taken.
#[test]
fn ordinary_names_are_taken() {
    for name in [".gitignore", ".gitattributes", ".github/workflows/ci.yml", "a.b", "name with space", "ünïcode/файл", ".gitmodules", "dir/.config", "gitmodules", "git-hooks/pre", "a:b"] {
        let mut index: FullIndex = index_of(vec![held("z", 1, true)], false);
        let entry: FlatEntry = FlatEntry { path: name.as_bytes().to_vec(), mode: 0o100_644, id: vec![7; 20] };
        let done: Option<()> = reset_paths(&mut index, &[name.as_bytes().to_vec()], &[entry], PLAIN, IndexStamp::NONE, WorktreeStat::None);
        // A colon is declined with the stream spellings; every other name here is plain.
        assert_eq!(done.is_some(), name != "a:b", "{name:?}");
    }
}
