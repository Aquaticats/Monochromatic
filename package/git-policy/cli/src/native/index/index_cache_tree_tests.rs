//! Controls for the cache-tree extension.

use super::*;

/// A valid directory of `count` entries.
fn valid(count: i64, subtrees: Vec<(Vec<u8>, CacheTree)>) -> CacheTree {
    return CacheTree { entry_count: count, id: vec![7; 20], subtrees };
}

/// Git's form reads and writes back unchanged,
/// with an invalid directory carrying no object name.
#[test]
fn the_extension_round_trips() {
    let invalid: CacheTree = CacheTree { entry_count: -1, id: Vec::new(), subtrees: vec![(b"x".to_vec(), valid(1, Vec::new()))] };
    let tree: CacheTree = valid(4, vec![(b"a".to_vec(), valid(2, Vec::new())), (b"b".to_vec(), invalid)]);
    let bytes: Vec<u8> = tree.write();
    assert_eq!(bytes[..9], [0, b'4', b' ', b'2', b'\n', 7, 7, 7, 7]);
    assert_eq!(CacheTree::read(bytes.as_slice(), 20), Some(tree));
}

/// Invalidating `a/b/c` marks the root,
/// `a` and `b` invalid;
/// invalidating a name removes the subtree of that name.
#[test]
fn invalidation_follows_the_path_and_removes_a_named_subtree() {
    let deep: CacheTree = valid(1, vec![(b"c".to_vec(), valid(1, Vec::new()))]);
    let mut tree: CacheTree = valid(3, vec![(b"a".to_vec(), valid(2, vec![(b"b".to_vec(), deep)])), (b"z".to_vec(), valid(1, Vec::new()))]);
    tree.invalidate(b"a/b/file");
    assert_eq!(tree.entry_count, -1);
    let a: &CacheTree = &tree.subtrees[0].1;
    assert_eq!(a.entry_count, -1);
    assert_eq!(a.subtrees[0].1.entry_count, -1);
    assert_eq!(a.subtrees[0].1.subtrees.len(), 1, "the subtree below the last directory stays");
    assert_eq!(tree.subtrees[1].1.entry_count, 1, "a directory off the path stays valid");
    tree.invalidate(b"a");
    assert_eq!(tree.subtrees.len(), 1, "the subtree the path names is removed");
    tree.invalidate(b"missing/deeper");
    assert_eq!(tree.subtrees.len(), 1);
}

/// Unsorted subtrees,
/// trailing bytes and a cut name are left to Git.
#[test]
fn irregular_forms_are_not_read() {
    let tree: CacheTree = valid(2, vec![(b"a".to_vec(), valid(1, Vec::new())), (b"b".to_vec(), valid(1, Vec::new()))]);
    let mut bytes: Vec<u8> = tree.write();
    bytes.push(0);
    assert_eq!(CacheTree::read(bytes.as_slice(), 20), None);
    let unsorted: CacheTree = CacheTree { entry_count: 2, id: vec![7; 20], subtrees: vec![(b"b".to_vec(), valid(1, Vec::new())), (b"a".to_vec(), valid(1, Vec::new()))] };
    assert_eq!(CacheTree::read(unsorted.write().as_slice(), 20), None);
    assert_eq!(CacheTree::read(b"no terminator", 20), None);
}

/// Subtrees are kept shorter name first,
/// then by bytes,
/// as `subtree_name_cmp` orders them;
/// byte order alone is not Git's.
#[test]
fn subtrees_are_ordered_by_length_first() {
    let in_git_order: CacheTree = valid(3, vec![(b"b".to_vec(), valid(1, Vec::new())), (b"aa".to_vec(), valid(1, Vec::new())), (b"ab".to_vec(), valid(1, Vec::new()))]);
    assert_eq!(CacheTree::read(in_git_order.write().as_slice(), 20), Some(in_git_order));
    let byte_order: CacheTree = valid(2, vec![(b"aa".to_vec(), valid(1, Vec::new())), (b"b".to_vec(), valid(1, Vec::new()))]);
    assert_eq!(CacheTree::read(byte_order.write().as_slice(), 20), None);
}
