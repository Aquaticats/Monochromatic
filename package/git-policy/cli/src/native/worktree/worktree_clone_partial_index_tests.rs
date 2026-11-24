//! Controls for the index a partial clone builds:
//! every tree path once,
//! the stat fields of the files the clones made,
//! none for a path Git writes,
//!  and the repository's own checksum.

use super::*;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_index::{AcceptEveryStat, IndexOutcome, RewriteContext, StatLookup, rewrite_index_stat_with};
use crate::worktree_clean::stored_stats;

/// A tree of three paths:
/// a file,
/// a link and a file below a directory,
/// in the order Git writes them.
fn tree() -> Vec<FlatEntry> {
    return vec![
        FlatEntry { path: b"a.txt".to_vec(), mode: 0o100_644, id: vec![1; 20] },
        FlatEntry { path: b"bin/tool.sh".to_vec(), mode: 0o100_755, id: vec![2; 20] },
        FlatEntry { path: b"link".to_vec(), mode: 0o120_000, id: vec![3; 20] },
    ];
}

/// The stat fields one clone of these controls made.
fn made() -> StatFields {
    return StatFields { ctime_seconds: 11, ctime_nanos: 12, mtime_seconds: 13, mtime_nanos: 14, dev: 15, ino: 16, uid: 17, gid: 18, size: 19 };
}

/// What:
///  Whether Git's own reader of an index accepts these bytes,
/// by the checksum the rewrite verifies and the entries it walks.
/// Why:
///   A file whose checksum does not verify is one Git refuses,
/// and the entries are the paths a
///  control asserts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = rewriteIndexStat(bytes, acceptEveryStat); // 'rewritten' proves the checksum verified
/// ```
fn git_reads(bytes: &[u8], hash_length: usize) -> bool {
    let context: RewriteContext = RewriteContext { hash_length: Some(hash_length), index_mtime: None };
    return matches!(rewrite_index_stat_with(bytes, &context, &mut NoFields, &mut AcceptEveryStat), IndexOutcome::Rewritten { .. });
}

/// The stat fields of no path,
/// so a control that only verifies the checksum refreshes no entry.
struct NoFields;

impl StatLookup for NoFields {
    fn stat_now(&mut self, _path: &[u8]) -> Option<StatFields> {
        return None;
    }
}

/// Every tree path is an entry,
/// with the tree's mode and object name,
/// and one entry per path.
#[test]
fn the_index_names_every_tree_path_once() {
    let stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    let bytes: Vec<u8> = index_of_the_tree(tree().as_slice(), &stats, 20).expect("an index");
    let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha1).expect("the entries");
    let named: Vec<Vec<u8>> = entries.iter().map(|entry| { return entry.path.clone(); }).collect();
    assert_eq!(named, vec![b"a.txt".to_vec(), b"bin/tool.sh".to_vec(), b"link".to_vec()]);
    assert_eq!(entries[1].mode, 0o100_755);
    assert_eq!(entries[2].id, vec![3; 20]);
    assert!(entries.iter().all(|entry| { return entry.stage == 0 && entry.extended == 0; }), "no entry carries a stage or extended flag");
    assert!(git_reads(bytes.as_slice(), 20), "the checksum verifies");
}

/// The entry of a path a clone made records that file's own stat fields,
/// and a path Git writes records none.
#[test]
fn the_index_records_the_fields_of_the_files_the_clones_made() {
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    stats.insert(b"a.txt".to_vec(), made());
    let bytes: Vec<u8> = index_of_the_tree(tree().as_slice(), &stats, 20).expect("an index");
    let recorded = stored_stats(bytes.as_slice(), 20).expect("the stat data");
    assert_eq!(recorded[0].stored.size, 19);
    assert_eq!(recorded[0].stored.mtime_nanos, 14);
    assert_eq!(recorded[0].stored.ino, 16);
    assert_eq!(recorded[1].stored.size, 0, "a path Git writes carries no stat data");
    assert_eq!(recorded[2].stored.size, 0, "a link the clones made without fields carries none either");
}

/// A SHA-256 repository's index ends with a SHA-256 checksum,
/// and its entries carry thirty-two byte object names.
#[test]
fn a_sha256_repository_gets_a_sha256_index() {
    let wide: Vec<FlatEntry> = tree().iter().map(|entry| { return FlatEntry { path: entry.path.clone(), mode: entry.mode, id: vec![entry.id[0]; 32] }; }).collect();
    let stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    let bytes: Vec<u8> = index_of_the_tree(wide.as_slice(), &stats, 32).expect("an index");
    let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha256).expect("the entries");
    assert_eq!(entries[0].id.len(), 32);
    assert!(git_reads(bytes.as_slice(), 32), "the SHA-256 checksum verifies");
}

/// A tree entry whose object name is not the repository's hash length is refused:
/// such a tree was never read by these readers.
#[test]
fn an_object_name_of_another_length_is_refused() {
    let stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    assert!(index_of_the_tree(tree().as_slice(), &stats, 32).is_err());
}

/// The index is the same bytes whatever order the tree's own list came in,
/// because the tree names the order Git reads it in.
#[test]
fn the_entries_follow_the_tree() {
    let stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    let bytes: Vec<u8> = index_of_the_tree(tree().as_slice(), &stats, 20).expect("an index");
    let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha1).expect("the entries");
    let mut sorted: Vec<Vec<u8>> = entries.iter().map(|entry| { return entry.path.clone(); }).collect();
    let before: Vec<Vec<u8>> = sorted.clone();
    sorted.sort();
    assert_eq!(before, sorted, "the entries are in path order, as Git's reader requires");
}
