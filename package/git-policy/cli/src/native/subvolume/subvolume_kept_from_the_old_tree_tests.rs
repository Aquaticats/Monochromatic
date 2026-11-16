//! Controls of what a deletion does with an entry it cannot prove the kept tree holds, and of the record it asks.
//! An entry is proved when the record says it still reads as the migration left it,
//! or when the kept tree holds the same bytes;
//! every other entry is moved into the folder beside the checkout and named by the record there,
//! so the deletion finishes with nothing of a person's gone.
//! The controls were written against `9ef0169599`, where the deletion refused and named the entries instead,
//! and where a branch named `index` and a person's `*.lock` file were taken for this program's own and deleted.

use super::*;
use crate::subvolume_old_tree_record::{OldTreeReading, OldTreeRecord};
use std::ffi::OsString;
use std::path::PathBuf;
use crate::test_support::{fixture, remove};
use std::collections::BTreeMap;

/// A tree about to be deleted and the tree that is kept, beside it, with the layout's folder and record between them.
struct TwoTrees {
    /// The fixture root.
    parent: PathBuf,
    /// The tree that stands where the deleted one was.
    deleted: PathBuf,
    /// The tree the person keeps.
    kept_tree: PathBuf,
    /// The layout of the kept tree's checkout name, which names the folder and the record.
    layout: Layout,
}

impl TwoTrees {
    /// Make the two trees and give the deleted one a file, a folder with a leaf, and a link.
    fn make(name: &str) -> TwoTrees {
        let parent: PathBuf = fixture(name);
        let deleted: PathBuf = parent.join("deleted");
        let checkout: PathBuf = parent.join("checkout");
        let kept_tree: PathBuf = parent.join("kept");
        std::fs::create_dir_all(deleted.join("deep").as_path()).expect("deleted");
        std::fs::create_dir_all(kept_tree.as_path()).expect("kept");
        std::fs::write(deleted.join("file.txt"), b"contents\n").expect("file");
        std::fs::write(deleted.join("deep/leaf"), b"a leaf\n").expect("leaf");
        std::os::unix::fs::symlink("file.txt", deleted.join("link")).expect("link");
        let layout: Layout = crate::subvolume_preflight::layout_of(checkout.as_path()).expect("layout");
        return TwoTrees { parent, deleted, kept_tree, layout };
    }

    /// Copy every entry of the deleted tree into the kept one, so every byte of it stands there.
    fn copy_everything_across(&self) {
        let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
        while let Some(relative) = pending.pop() {
            for item in std::fs::read_dir(self.deleted.join(&relative)).expect("listing").flatten() {
                let child: PathBuf = relative.join(item.file_name());
                let kind = item.file_type().expect("kind");
                if kind.is_dir() {
                    std::fs::create_dir_all(self.kept_tree.join(&child)).expect("directory");
                    pending.push(child);
                } else if kind.is_symlink() {
                    std::os::unix::fs::symlink(std::fs::read_link(self.deleted.join(&child)).expect("target"), self.kept_tree.join(&child)).expect("link");
                } else {
                    std::fs::copy(self.deleted.join(&child), self.kept_tree.join(&child)).expect("file");
                }
            }
        }
    }

    /// The record of what the deleted tree read like at this moment,
    /// as the migration's last pass over it would have written it.
    fn record_now(&self) -> OldTreeRecord {
        let mut readings: BTreeMap<PathBuf, OldTreeReading> = BTreeMap::new();
        let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
        while let Some(relative) = pending.pop() {
            for item in std::fs::read_dir(self.deleted.join(&relative)).expect("listing").flatten() {
                let child: PathBuf = relative.join(item.file_name());
                let metadata: std::fs::Metadata = item.metadata().expect("metadata");
                if metadata.is_dir() {
                    pending.push(child);
                    continue;
                }
                if let Some(reading) = OldTreeReading::of(&metadata) {
                    readings.insert(child, reading);
                }
            }
        }
        return OldTreeRecord::of(readings);
    }

    /// Delete the tree the way the command does, and answer what it could not keep and where it kept the rest.
    fn walk(&self, record: Option<&OldTreeRecord>, accepted: &std::collections::BTreeSet<PathBuf>) -> (Vec<PathBuf>, Vec<(PathBuf, PathBuf)>) {
        let mut kept: KeptFromTheOldTree = KeptFromTheOldTree::open(&self.layout);
        let unproved: Vec<PathBuf> = keep_what_the_kept_tree_does_not_hold(self.deleted.as_path(), self.kept_tree.as_path(), record, accepted, &mut kept)
            .expect("the walk");
        let entries = kept.finish().expect("the record of what was kept");
        return (unproved, entries);
    }
}

/// A tree whose bytes every one of them stands in the kept tree is deleted with nothing kept.
#[test]
fn a_tree_whose_bytes_stand_in_the_other_is_deleted_with_nothing_kept() {
    let trees: TwoTrees = TwoTrees::make("kept-proved");
    trees.copy_everything_across();
    let record: OldTreeRecord = trees.record_now();
    let (unproved, kept) = trees.walk(Some(&record), &std::collections::BTreeSet::new());
    assert!(unproved.is_empty(), "nothing is unproved: {unproved:?}");
    assert!(kept.is_empty(), "nothing had to be kept: {kept:?}");
    assert!(!trees.layout.kept_path.exists(), "no folder is made when nothing is kept");
    assert!(!trees.layout.kept_record_path.exists(), "and no record either");
    remove(trees.parent.as_path());
}

/// A file whose bytes the record still describes as the migration left them is deleted,
/// although the kept tree holds other bytes at its path:
/// that is the state of a checkout a person has worked in since the migration.
#[test]
fn an_entry_the_record_still_reads_the_same_is_deleted_although_the_kept_tree_differs() {
    let trees: TwoTrees = TwoTrees::make("kept-unchanged-differs");
    trees.copy_everything_across();
    let record: OldTreeRecord = trees.record_now();
    // The person edits the file in the tree they keep: the bytes there are their own later work.
    std::fs::write(trees.kept_tree.join("file.txt"), b"edited in the checkout\n").expect("the edit");
    let (unproved, kept) = trees.walk(Some(&record), &std::collections::BTreeSet::new());
    assert!(unproved.is_empty(), "nothing is unproved: {unproved:?}");
    assert!(kept.is_empty(), "the person's edit is not the old tree's to keep: {kept:?}");
    remove(trees.parent.as_path());
}

/// Every kind of difference is moved into the folder and named there:
/// a rewrite of the same size,
/// a file the kept tree lacks,
/// a link whose target differs,
/// and an entry written into the deleted tree after the record.
#[test]
fn every_kind_of_difference_is_kept_and_named() {
    let trees: TwoTrees = TwoTrees::make("kept-differences");
    trees.copy_everything_across();
    let record: OldTreeRecord = trees.record_now();
    // A record from before these writes, as the migration's would be for a tree written into afterwards.
    let older: OldTreeRecord = OldTreeRecord::of(BTreeMap::new());
    std::fs::write(trees.deleted.join("file.txt"), b"CONTENTS\n").expect("the same size");
    std::fs::write(trees.deleted.join("added.txt"), b"only here\n").expect("an added file");
    std::fs::remove_file(trees.kept_tree.join("link")).expect("remove the kept link");
    std::os::unix::fs::symlink("elsewhere", trees.kept_tree.join("link")).expect("another target");
    let (unproved, kept) = trees.walk(Some(&older), &std::collections::BTreeSet::new());
    assert!(unproved.is_empty(), "everything was moved, so nothing is unproved: {unproved:?}");
    let mut sources: Vec<String> = kept.iter().map(|(_path, source)| return source.to_string_lossy().into_owned()).collect();
    sources.sort();
    // `deep/leaf` is not among them: its bytes stand in the kept tree, so the proof answers for it without the record.
    assert_eq!(sources, vec![String::from("added.txt"), String::from("file.txt"), String::from("link")], "each difference is named: {kept:?}");
    for (kept_path, source) in &kept {
        if source.as_os_str() == "link" {
            assert_eq!(std::fs::read_link(kept_path.as_path()).expect("a kept link"), PathBuf::from("file.txt"), "the link is kept with the target the deleted tree held");
            continue;
        }
        let bytes: Vec<u8> = std::fs::read(kept_path.as_path()).expect("a kept file");
        assert!(bytes == b"CONTENTS\n" || bytes == b"only here\n", "the bytes kept for {} are the deleted tree's: {kept_path:?}", source.display());
    }
    assert!(trees.layout.kept_record_path.is_file(), "the record that names them is written");
    let named: BTreeMap<OsString, PathBuf> = read_the_record(trees.layout.kept_record_path.as_path()).expect("the record");
    assert_eq!(named.len(), 3, "the record names every entry of the folder: {named:?}");
    assert!(!trees.deleted.join("file.txt").exists(), "the deleted tree no longer holds what was kept");
    // The reading the record was asked about is untouched: this control asked with an empty one.
    let _ = record;
    remove(trees.parent.as_path());
}

/// An entry a person accepted with a flag is neither kept nor named: the deletion deletes it.
#[test]
fn an_accepted_entry_is_deleted_rather_than_kept() {
    let trees: TwoTrees = TwoTrees::make("kept-accepted");
    trees.copy_everything_across();
    std::fs::write(trees.deleted.join("file.txt"), b"the old tree's version\n").expect("a write into the deleted tree");
    let mut accepted: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    accepted.insert(PathBuf::from("file.txt"));
    let (unproved, kept) = trees.walk(Some(&OldTreeRecord::of(BTreeMap::new())), &accepted);
    assert!(unproved.is_empty(), "the accepted entry is no unproved one: {unproved:?}");
    assert!(kept.is_empty(), "and nothing was kept for it: {kept:?}");
    remove(trees.parent.as_path());
}

/// This program's own entries are neither kept nor named:
/// the index,
/// its lock,
/// a partial copy,
/// and the bookkeeping its own modules name.
#[test]
fn this_programs_own_entries_are_not_kept() {
    let trees: TwoTrees = TwoTrees::make("kept-own");
    trees.copy_everything_across();
    std::fs::create_dir_all(trees.deleted.join(".git/objects").as_path()).expect("a git directory");
    std::fs::write(trees.deleted.join(".git/index"), b"an index\n").expect("an index");
    std::fs::write(trees.deleted.join(".git/index.lock"), b"a lock\n").expect("a lock");
    std::fs::write(trees.deleted.join(crate::subvolume_sync_pass_state_and_checks::PARTIAL_NAME), b"a partial copy\n").expect("a partial");
    // A branch named `index` and a person's `*.lock` file are the person's, and are kept.
    std::fs::create_dir_all(trees.deleted.join(".git/refs/heads").as_path()).expect("refs");
    std::fs::write(trees.deleted.join(".git/refs/heads/index"), b"a branch\n").expect("the branch");
    std::fs::create_dir_all(trees.deleted.join(".git/hooks").as_path()).expect("hooks");
    std::fs::write(trees.deleted.join(".git/hooks/mine.lock"), b"a person's file\n").expect("the file");
    let (unproved, kept) = trees.walk(Some(&OldTreeRecord::of(BTreeMap::new())), &std::collections::BTreeSet::new());
    assert!(unproved.is_empty(), "{unproved:?}");
    let mut sources: Vec<String> = kept.iter().map(|(_path, source)| return source.to_string_lossy().into_owned()).collect();
    sources.sort();
    assert_eq!(
        sources,
        vec![String::from(".git/hooks/mine.lock"), String::from(".git/refs/heads/index")],
        "the index, its lock and the partial copy are this program's; the branch and the person's file are not: {kept:?}"
    );
    remove(trees.parent.as_path());
}

/// A second walk finds the entries an earlier one kept and names them again,
/// so nothing in the folder is left unexplained.
#[test]
fn a_second_walk_names_what_an_earlier_one_kept() {
    let trees: TwoTrees = TwoTrees::make("kept-twice");
    trees.copy_everything_across();
    std::fs::write(trees.deleted.join("file.txt"), b"the old tree's version\n").expect("a write into the deleted tree");
    let empty: OldTreeRecord = OldTreeRecord::of(BTreeMap::new());
    let (first_unproved, first) = trees.walk(Some(&empty), &std::collections::BTreeSet::new());
    assert!(first_unproved.is_empty(), "{first_unproved:?}");
    assert_eq!(first.len(), 1, "one entry was kept: {first:?}");
    let (second_unproved, second) = trees.walk(Some(&empty), &std::collections::BTreeSet::new());
    assert!(second_unproved.is_empty(), "{second_unproved:?}");
    assert_eq!(second.len(), 1, "the second walk names the entry the first kept: {second:?}");
    assert_eq!(second.first().map(|(_path, source)| return source.clone()), Some(PathBuf::from("file.txt")));
    remove(trees.parent.as_path());
}
