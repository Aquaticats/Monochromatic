//! Controls for the chain of directories a walk keeps open:
//! a deeper path after a sibling keeps the part it shares,
//! a path too deep to keep is walked without holding one descriptor per component,
//! a link and a mount are still refused,
//! and a directory on another filesystem is refused there too.

use super::{DirectoryChain, MOST_DIRECTORIES_HELD};
use crate::fs_beneath::{device_of, identity_of, open_directory_beneath, open_root};
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, open_descriptors, remove, restore_limit, set_soft_limit};
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

/// The identity of a directory an answer names,
/// as its encoded text,
/// so an assertion compares text and the borrow of the chain ends with the answer.
///
/// Why:
/// an identity is what tells two directories apart,
/// and encoded text needs no borrow to be compared.
fn identity_text(found: &File) -> String {
    return identity_of(found).expect("an identity").encode();
}

/// A directory `depth` deep below `root`,
/// one character per component,
/// and the names that lead to it.
///
/// Why:
/// this is the plain shape of a deeply nested checkout:
/// one-letter components keep the whole pathname below `PATH_MAX` at a depth no descriptor limit allows.
fn deeply_nested(root: &Path, depth: usize) -> Vec<OsString> {
    let mut deepest: PathBuf = root.to_path_buf();
    let mut component_names: Vec<OsString> = Vec::with_capacity(depth);
    for _ in 0..depth {
        let next: PathBuf = deepest.join("d");
        std::fs::create_dir(next.as_path()).expect("a directory of the deep fixture");
        deepest = next;
        // The same one-letter name, the next level down.
        component_names.push(OsString::from("d"));
    }
    return component_names;
}

/// A path deeper than a chain may keep every ancestor of is walked one descriptor at a time,
/// so a valid depth is answered rather than refused at the process's descriptor limit.
#[test]
fn a_path_deeper_than_a_chain_may_keep_is_walked_one_descriptor_at_a_time() {
    // Deeper than the 1024 descriptors a service commonly allows, and short enough that 1100 one-character
    //  components still fit the 4096-byte pathname limit.
    const DEPTH: usize = 1100;
    // The room the walk gets beyond the descriptors this process already holds: a walk that keeps one
    //  directory open at a time needs two at its peak, the one it holds and the one it is opening.
    const ROOM: u64 = 3;
    let root: PathBuf = fixture("chain-deep");
    let component_names: Vec<OsString> = deeply_nested(root.as_path(), DEPTH);
    let held: File = open_directory(root.as_path()).expect("the root of the deep fixture");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("a chain");
    let before = set_soft_limit(u64::try_from(open_descriptors()).expect("a count of descriptors") + ROOM);
    // The walk under test, with only `ROOM` descriptors left free; it answers the identity of the directory so the
    //  borrow of the chain ends here.
    let walked = chain.directory(component_names.as_slice()).map(identity_text);
    restore_limit(before);
    let from_the_chain: String = walked.expect("a path deeper than the chain may keep is still answered");
    let direct: File = open_directory_beneath(&held, component_names.as_slice()).expect("the same path, one directory at a time");
    assert_eq!(from_the_chain, identity_text(&direct), "the chain answers the directory the names lead to");
    remove(root.as_path());
}

/// A path past the bound is still answered when a path at the bound filled the chain first,
/// and the descriptor limit leaves room for the walk alone.
///
/// Why:
/// the directories the chain keeps were opened for the path before;
/// held through the walk of a deeper path,
/// they take the descriptors its root's clone and its first directory need,
/// and a directory that is there is refused as unreachable.
#[test]
fn a_path_past_the_bound_is_answered_after_a_path_at_the_bound_filled_the_chain() {
    // The room the run gets beyond the descriptors the process holds before a path fills the chain: filling it takes
    //  one descriptor per kept directory, and the walk past the bound then needs two, its root's clone and the
    //  directory it is opening.
    let room: u64 = u64::try_from(MOST_DIRECTORIES_HELD + 1).expect("a count of descriptors");
    let root: PathBuf = fixture("chain-deep-after-filled");
    // One directory past the bound, so the leading names are a path at the bound and all of them a path past it.
    let component_names: Vec<OsString> = deeply_nested(root.as_path(), MOST_DIRECTORIES_HELD + 1);
    let at_bound: &[OsString] = &component_names[..MOST_DIRECTORIES_HELD];
    let held: File = open_directory(root.as_path()).expect("the root of the fixture");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("a chain");
    // The limit is set before the chain holds anything, as a service's limit is: the kept directories are what leaves
    //  one descriptor free when the deeper path is asked for.
    let before = set_soft_limit(u64::try_from(open_descriptors()).expect("a count of descriptors") + room);
    let at_bound_identity = chain.directory(at_bound).map(identity_text);
    assert_eq!(chain.held_names().len(), MOST_DIRECTORIES_HELD, "the path at the bound is kept, which is what fills the chain");
    // The walk under test: it answers the identity of the directory so the borrow of the chain ends here.
    let walked = chain.directory(component_names.as_slice()).map(identity_text);
    restore_limit(before);
    let kept: String = at_bound_identity.expect("a path at the bound is answered under its own descriptor room");
    let from_the_chain: String = walked.expect("a path past the bound is answered after a path at the bound filled the chain");
    let direct: File = open_directory_beneath(&held, component_names.as_slice()).expect("the same path, one directory at a time");
    assert_eq!(from_the_chain, identity_text(&direct), "the chain answers the directory the names lead to");
    assert_ne!(kept, from_the_chain, "the path past the bound ends in a directory of its own");
    remove(root.as_path());
}

/// The chain keeps every directory of a path exactly as deep as its bound,
/// and keeps nothing of a path one directory deeper.
#[test]
fn a_path_at_the_bound_is_kept_and_one_past_it_is_not() {
    let root: PathBuf = fixture("chain-bound");
    let at_bound: Vec<OsString> = deeply_nested(root.as_path(), MOST_DIRECTORIES_HELD);
    let mut one_past: Vec<OsString> = at_bound.clone();
    // The one component past the bound, made inside the deepest directory the fixture has.
    one_past.push(OsString::from("d"));
    let mut deeper: PathBuf = root.clone();
    for name in &one_past {
        deeper.push(name.as_os_str());
    }
    std::fs::create_dir(deeper.as_path()).expect("one directory past the bound");
    let held: File = open_directory(root.as_path()).expect("the root of the fixture");
    let mut chain: DirectoryChain<'_> = DirectoryChain::new(&held).expect("a chain");
    let kept: String = chain.directory(at_bound.as_slice()).map(identity_text).expect("a path at the bound is answered");
    assert_eq!(chain.held_names().len(), MOST_DIRECTORIES_HELD, "the chain keeps every directory of a path at its bound");
    let walked: String = chain.directory(one_past.as_slice()).map(identity_text).expect("a path past the bound is answered");
    assert_eq!(chain.held_names().len(), 0, "the chain keeps nothing of a path past its bound");
    assert_ne!(kept, walked, "the two paths end in different directories");
    // The answer a path past the bound left is not the one a shallower path gets afterwards: the chain opens that path
    //  again and keeps it, so nothing of the deep walk leaks into it.
    let shallow: Vec<OsString> = at_bound.iter().take(2).cloned().collect();
    let resumed: String = chain.directory(shallow.as_slice()).map(identity_text).expect("a shallow path after a deep one is answered");
    assert_eq!(chain.held_names().len(), 2, "the chain keeps the two directories of the shallow path");
    let direct_shallow: File = open_directory_beneath(&held, shallow.as_slice()).expect("the shallow path, directly");
    assert_eq!(resumed, identity_text(&direct_shallow), "the shallow path answers the directory its names lead to");
    remove(root.as_path());
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
