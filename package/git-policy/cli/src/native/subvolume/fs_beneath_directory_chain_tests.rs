//! Controls for the chain of directories a walk keeps open:
//! a deeper path after a sibling keeps the part it shares,
//! a link and a mount are still refused,
//! and a directory on another filesystem is refused there too.

use super::DirectoryChain;
use crate::fs_beneath::{device_of, identity_of, open_directory_beneath, open_root};
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The root of a walk for a fixture directory,
/// by its canonical path.
fn open_directory(path: &Path) -> std::io::Result<File> {
    return open_root(std::fs::canonicalize(path)?.as_path());
}

/// The names of a path's components,
/// for the chain's argument.
fn names(components: &[&str]) -> Vec<OsString> {
    return components.iter().map(OsString::from).collect();
}

/// A path deeper than the one before is opened from the part the two share,
/// and answers the directory it names.
#[test]
fn a_deeper_path_keeps_the_part_it_shares_and_answers_the_same_directory() {
    let root: PathBuf = fixture("chain-deeper");
    std::fs::create_dir_all(root.join("a/b/c")).expect("a/b/c");
    std::fs::create_dir_all(root.join("a/d")).expect("a/d");
    let held: File = open_directory(root.as_path()).expect("root");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("chain");
    let deepest = chain.directory(names(&["a", "b", "c"]).as_slice()).expect("a/b/c");
    let _deepest_identity = identity_of(deepest).expect("identity of a/b/c");
    assert_eq!(chain.held_names(), vec![OsStr::new("a"), OsStr::new("b"), OsStr::new("c")]);
    // A sibling of the deepest directory shares the chain's first name and nothing more.
    let sibling = chain.directory(names(&["a", "d"]).as_slice()).expect("a/d");
    let sibling_identity = identity_of(sibling).expect("identity of a/d");
    assert_eq!(chain.held_names(), vec![OsStr::new("a"), OsStr::new("d")]);
    let direct: File = open_directory_beneath(&held, names(&["a", "d"]).as_slice()).expect("a/d, directly");
    assert_eq!(sibling_identity, identity_of(&direct).expect("identity of the direct a/d"));
    // The same names again open nothing: the chain already holds them.
    let again = chain.directory(names(&["a", "d"]).as_slice()).expect("a/d again");
    assert_eq!(identity_of(again).expect("identity of a/d again"), sibling_identity);
    // The root itself is answered for a path with no components.
    let at_root = chain.directory(&[]).expect("the root");
    assert_eq!(identity_of(at_root).expect("identity of the root"), identity_of(&held).expect("identity of the held root"));
    assert_eq!(chain.held_names().len(), 0);
    remove(root.as_path());
}

/// A link on the way is refused,
/// as it is when the whole path is opened.
#[test]
fn a_link_where_a_directory_is_expected_is_refused() {
    let root: PathBuf = fixture("chain-link");
    std::fs::create_dir_all(root.join("real/inner")).expect("real/inner");
    std::fs::create_dir_all(root.join("elsewhere/inner")).expect("elsewhere/inner");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("real/linked")).expect("link");
    let held: File = open_directory(root.join("real").as_path()).expect("root");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("chain");
    assert!(chain.directory(names(&["inner"]).as_slice()).is_ok(), "positive control: a plain directory");
    assert!(chain.directory(names(&["linked", "inner"]).as_slice()).is_err(), "a link is not a detour");
    remove(root.as_path());
}

/// A directory on another filesystem is refused below a chain,
/// as it is when the whole path is opened.
#[test]
fn a_directory_on_another_device_is_refused() {
    // `/proc` is a filesystem of its own, so entering it from `/` is the refusal a nested subvolume gets, without
    // needing btrfs: the root's device is the one every opened directory is compared against.
    let root: PathBuf = fixture("chain-device");
    std::fs::create_dir_all(root.join("inner")).expect("inner");
    let held: File = open_directory(Path::new("/")).expect("root");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("chain");
    assert!(chain.directory(names(&["proc", "self"]).as_slice()).is_err(), "a directory on another device is not entered");
    let held_fixture: File = open_directory(root.as_path()).expect("fixture root");
    let mut fixture_chain: DirectoryChain<'_> = DirectoryChain::new(&held_fixture).expect("fixture chain");
    assert!(fixture_chain.directory(names(&["inner"]).as_slice()).is_ok(), "positive control: a directory of the root's own filesystem");
    remove(root.as_path());
}

/// A nested btrfs subvolume is refused below a chain,
/// and a plain directory beside it is not.
#[test]
fn btrfs_a_nested_subvolume_is_another_device_in_a_chain() {
    let Some(root) = btrfs_fixture("chain-subvolume") else { return };
    std::fs::create_dir_all(root.join("plain")).expect("plain");
    subvolume_create_at(root.join("nested").as_path()).expect("nested subvolume");
    let held: File = open_directory(root.as_path()).expect("root");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("chain");
    assert!(chain.directory(names(&["plain"]).as_slice()).is_ok(), "positive control: the same device");
    assert!(chain.directory(names(&["nested"]).as_slice()).is_err(), "a subvolume is not crossed");
    assert_eq!(device_of(&held).expect("device"), std::fs::metadata(root.join("plain")).expect("plain").dev());
    remove(root.as_path());
}
