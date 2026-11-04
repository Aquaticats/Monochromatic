//! Controls for the invalidation of the untracked cache along a path.

use super::*;

/// A directory as read from a file: written, valid and listing one file.
fn listed(name: &str, dirs: Vec<UntrackedDir>) -> UntrackedDir {
    return UntrackedDir {
        name: name.as_bytes().to_vec(),
        recurse: true,
        valid: true,
        check_only: false,
        stat: StatData::default(),
        exclude_id: vec![0; 20],
        untracked: vec![b"file".to_vec()],
        dirs,
    };
}

/// A cache over `root` with the given directory flags.
fn cache(root: UntrackedDir, dir_flags: u32) -> Untracked {
    return Untracked {
        ident: b"ident".to_vec(),
        info_exclude: (StatData::default(), vec![0; 20]),
        excludes_file: (StatData::default(), vec![0; 20]),
        dir_flags,
        exclude_per_dir: b".gitignore".to_vec(),
        root: Some(root),
    };
}

/// Only the directory that holds the path is cleared when the cache lists files, not whole untracked directories.
#[test]
fn only_the_directory_of_the_path_is_cleared_without_whole_directory_listing() {
    let mut untracked: Untracked = cache(listed("", vec![listed("a", vec![listed("b", Vec::new())])]), 0);
    untracked.invalidate_path(b"a/b/file");
    let root: &UntrackedDir = untracked.root.as_ref().expect("root");
    assert!(root.valid && root.dirs[0].valid, "ancestors stay valid");
    assert!(!root.dirs[0].dirs[0].valid && root.dirs[0].dirs[0].untracked.is_empty(), "the directory of the path is cleared");
}

/// When untracked directories are listed whole, every directory up to the top is cleared.
#[test]
fn every_ancestor_is_cleared_with_whole_directory_listing() {
    let mut untracked: Untracked = cache(listed("", vec![listed("a", vec![listed("b", Vec::new())])]), SHOW_OTHER_DIRECTORIES);
    untracked.invalidate_path(b"a/b/file");
    let root: &UntrackedDir = untracked.root.as_ref().expect("root");
    assert!(!root.valid && !root.dirs[0].valid && !root.dirs[0].dirs[0].valid);
}

/// A directory the cache did not know is made, in name order, and is not written out.
#[test]
fn an_unknown_directory_is_made_in_order_and_not_written() {
    let mut untracked: Untracked = cache(listed("", vec![listed("a", Vec::new()), listed("c", Vec::new())]), 0);
    untracked.invalidate_path(b"b/file");
    let root: &UntrackedDir = untracked.root.as_ref().expect("root");
    let names: Vec<&[u8]> = root.dirs.iter().map(|dir| { return dir.name.as_slice(); }).collect();
    assert_eq!(names, vec![&b"a"[..], &b"b"[..], &b"c"[..]]);
    assert!(!root.dirs[1].recurse && !root.dirs[1].valid);
    assert!(root.valid, "the top stays valid when directories are not listed whole");
}

/// A path at the top clears the top, and a cache with no directories has nothing to clear.
#[test]
fn a_path_at_the_top_clears_the_top() {
    let mut untracked: Untracked = cache(listed("", vec![listed("a", Vec::new())]), 0);
    untracked.invalidate_path(b"file");
    let root: &UntrackedDir = untracked.root.as_ref().expect("root");
    assert!(!root.valid && root.dirs[0].valid);
    let mut empty: Untracked = cache(listed("", Vec::new()), 0);
    empty.root = None;
    empty.invalidate_path(b"a/b");
    assert!(empty.root.is_none());
}
