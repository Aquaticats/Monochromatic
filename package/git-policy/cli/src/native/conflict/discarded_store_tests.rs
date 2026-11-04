//! What:
//!  Controls of the folder of saved copies opened through descriptors: which links it refuses, what a walk counts and removes,
//!  how deep it goes, and that walks which meet the same directory share its files without a lock.
//! Why:
//!  The removal is only as safe as the steps under it, so each step is checked on a real directory in a fixture: a link is
//!  never entered or followed, a name that is another directory now is noticed, and a tree that is too deep is kept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = await Store.open(common, 'cli-git/discarded'); const counted = await Promise.all([walk(store), walk(store)]);
//! ```

use super::{Identity, MAX_DEPTH, OpenFailure, Store, Tally};
use crate::command_test_support::{fixture, remove};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The folder of a fixture repository, made, and the common directory above it.
fn folder_in(root: &Path) -> (PathBuf, PathBuf) {
    let common: PathBuf = root.join("repo.git");
    let folder: PathBuf = common.join("cli-git/discarded");
    std::fs::create_dir_all(&folder).expect("the folder");
    return (common, folder);
}

/// A folder is opened when it exists, is nothing when it is absent, and is refused when a link stands in it.
#[test]
fn a_link_at_either_component_is_refused_and_an_absent_folder_is_nothing() {
    let root: PathBuf = fixture("store-open");
    let (common, folder) = folder_in(root.as_path());
    assert!(Store::open(common.as_path(), "cli-git/discarded").expect("opens").is_some());
    assert!(Store::open(root.as_path(), "cli-git/discarded").expect("absent is not a failure").is_none());
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    std::fs::remove_dir(&folder).expect("remove the folder");
    std::os::unix::fs::symlink(&elsewhere, &folder).expect("link");
    assert!(matches!(Store::open(common.as_path(), "cli-git/discarded"), Err(OpenFailure::Link)));
    std::fs::remove_file(&folder).expect("remove the link");
    std::fs::write(&folder, "a file").expect("a file where the folder belongs");
    assert!(matches!(Store::open(common.as_path(), "cli-git/discarded"), Err(OpenFailure::Link)), "a file is not a folder");
    remove(root.as_path());
}

/// The common directory may be reached through a link, because Git names it; only what is below it is refused.
#[test]
fn a_link_at_the_common_directory_itself_is_followed() {
    let root: PathBuf = fixture("store-common-link");
    let (common, _) = folder_in(root.as_path());
    let through: PathBuf = root.join("through");
    std::os::unix::fs::symlink(&common, &through).expect("link");
    assert!(Store::open(through.as_path(), "cli-git/discarded").expect("opens").is_some());
    remove(root.as_path());
}

/// A survey counts files by their sizes with relative names and never enters a link; a removal takes the same files and the
/// subdirectories and leaves the directory itself.
#[test]
fn a_survey_counts_and_a_removal_takes_the_same_files_and_never_enters_a_link() {
    let root: PathBuf = fixture("store-walk");
    let (common, folder) = folder_in(root.as_path());
    let entry: PathBuf = folder.join("2026-11-02T14-01-07");
    std::fs::create_dir_all(entry.join("a/b")).expect("tree");
    std::fs::write(entry.join("top.txt"), "12345").expect("file");
    std::fs::write(entry.join("a/mid.txt"), "123").expect("file");
    std::fs::write(entry.join("a/b/deep.txt"), "1").expect("file");
    let outside: PathBuf = root.join("elsewhere");
    std::fs::create_dir(&outside).expect("elsewhere");
    std::fs::write(outside.join("precious.txt"), "not ours").expect("file");
    std::os::unix::fs::symlink(&outside, entry.join("link")).expect("link");
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    let directory = store.open_directory(OsStr::new("2026-11-02T14-01-07")).expect("opens").expect("exists");
    let surveyed = directory.survey().expect("surveyed");
    let mut names: Vec<String> = surveyed.names.clone();
    names.sort();
    assert_eq!(names, vec!["a/b/deep.txt", "a/mid.txt", "link", "top.txt"]);
    assert_eq!(surveyed.bytes, 5 + 3 + 1 + u128::try_from(outside.as_os_str().len()).expect("small"), "a link counts by its own size");
    let emptied = directory.empty().expect("emptied");
    assert_eq!(emptied.bytes, surveyed.bytes);
    assert_eq!(std::fs::read_dir(&entry).expect("the directory stays").count(), 0);
    assert_eq!(std::fs::read_to_string(outside.join("precious.txt")).expect("untouched"), "not ours");
    remove(root.as_path());
}

/// A file of a directory is removed before any of its subdirectories, so a failure below leaves the same remainder each run.
#[test]
fn files_go_before_subdirectories() {
    let root: PathBuf = fixture("store-order");
    let (common, folder) = folder_in(root.as_path());
    let entry: PathBuf = folder.join("2026-11-02T14-01-07");
    std::fs::create_dir_all(entry.join("sub")).expect("tree");
    std::fs::write(entry.join("sub/inner.txt"), "x").expect("file");
    std::fs::write(entry.join("z-last-by-name.txt"), "y").expect("file");
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    let directory = store.open_directory(OsStr::new("2026-11-02T14-01-07")).expect("opens").expect("exists");
    let emptied = directory.empty().expect("emptied");
    assert_eq!(emptied.names, vec!["z-last-by-name.txt", "sub/inner.txt"]);
    remove(root.as_path());
}

/// A name that is a link or a file is not opened as a directory, and a name that is gone is nothing.
#[test]
fn a_link_a_file_and_a_gone_name_are_not_directories() {
    let root: PathBuf = fixture("store-kinds");
    let (common, folder) = folder_in(root.as_path());
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, folder.join("a-link")).expect("link");
    std::fs::write(folder.join("a-file"), "x").expect("file");
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    for name in ["a-link", "a-file"] {
        let error = store.open_directory(OsStr::new(name)).expect_err(name);
        assert_eq!(error.kind(), std::io::ErrorKind::NotADirectory, "{name}");
        assert_eq!(store.directory_identity(OsStr::new(name)).expect("inspected"), None, "{name}");
    }
    assert!(store.open_directory(OsStr::new("gone")).expect("not an error").is_none());
    assert_eq!(store.directory_identity(OsStr::new("gone")).expect("inspected"), None);
    remove(root.as_path());
}

/// A directory replaced under its name is another directory: the identity differs, and the removal of the name refuses.
#[test]
fn a_replaced_directory_has_another_identity_and_its_name_is_not_removed() {
    let root: PathBuf = fixture("store-identity");
    let (common, folder) = folder_in(root.as_path());
    let name: &OsStr = OsStr::new("2026-11-02T14-01-07");
    std::fs::create_dir(folder.join(name)).expect("directory");
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    let judged: Identity = store.directory_identity(name).expect("inspected").expect("a directory");
    std::fs::rename(folder.join(name), folder.join("aside")).expect("aside");
    std::fs::create_dir(folder.join(name)).expect("a replacement");
    assert!(!store.remove_empty_directory(name, judged).expect("refused without an error"), "the replacement was removed");
    assert!(folder.join(name).is_dir());
    remove(root.as_path());
}

/// A directory that holds something is never removed by the call that removes the empty directory.
#[test]
fn a_directory_with_content_is_not_removed_as_empty() {
    let root: PathBuf = fixture("store-not-empty");
    let (common, folder) = folder_in(root.as_path());
    let name: &OsStr = OsStr::new("2026-11-02T14-01-07");
    std::fs::create_dir(folder.join(name)).expect("directory");
    std::fs::write(folder.join(name).join("kept.txt"), "x").expect("file");
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    let judged: Identity = store.directory_identity(name).expect("inspected").expect("a directory");
    assert!(store.remove_empty_directory(name, judged).is_err());
    assert!(folder.join(name).join("kept.txt").exists());
    remove(root.as_path());
}

/// A tree deeper than the bound is stopped at, with what was met before it, and nothing below the bound is entered.
#[test]
fn a_tree_deeper_than_the_bound_is_stopped_at() {
    let root: PathBuf = fixture("store-deep");
    let (common, folder) = folder_in(root.as_path());
    let mut deep: PathBuf = folder.join("2026-11-02T14-01-07");
    for _level in 0..MAX_DEPTH + 2 {
        deep.push("d");
    }
    std::fs::create_dir_all(&deep).expect("a deep tree");
    std::fs::write(deep.join("bottom.txt"), "x").expect("file");
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    let directory = store.open_directory(OsStr::new("2026-11-02T14-01-07")).expect("opens").expect("exists");
    let interrupted = directory.survey().expect_err("too deep");
    assert!(interrupted.tally.names.is_empty());
    assert!(interrupted.error.to_string().contains("deeper"), "{}", interrupted.error);
    assert!(deep.join("bottom.txt").exists());
    remove(root.as_path());
}

/// Two walks of one directory share its files without a lock: each file is counted by the walk whose removal of it succeeded,
/// so across the walks every file is counted once and neither fails on a file or a directory the other removed.
#[test]
fn walks_emptying_one_directory_together_count_each_file_once() {
    let root: PathBuf = fixture("store-shared-empty");
    let (common, folder) = folder_in(root.as_path());
    let directory: PathBuf = folder.join("2026-11-02T14-01-07");
    std::fs::create_dir_all(directory.join("inner")).expect("a directory");
    for number in 0..SHARED_FILES {
        std::fs::write(directory.join(format!("file-{number}")), "x").expect("a file");
        std::fs::write(directory.join("inner").join(format!("deep-{number}")), "y").expect("a deeper file");
    }
    let store: Store = Store::open(common.as_path(), "cli-git/discarded").expect("opens").expect("exists");
    let name = std::ffi::OsString::from("2026-11-02T14-01-07");
    let starting: std::sync::Barrier = std::sync::Barrier::new(SHARED_WALKS);
    let tallies: Vec<Tally> = std::thread::scope(|scope| {
        let walks: Vec<_> = (0..SHARED_WALKS)
            .map(|_| {
                return scope.spawn(|| {
                    let handle = store.open_directory(name.as_os_str()).expect("opens").expect("exists");
                    starting.wait();
                    return handle.empty().expect("a walk fails on nothing another walk removed");
                });
            })
            .collect();
        return walks.into_iter().map(|walk| { return walk.join().expect("a walk finished"); }).collect();
    });
    let mut counted: Vec<String> = tallies.into_iter().flat_map(|tally| { return tally.names; }).collect();
    counted.sort();
    let before: usize = counted.len();
    counted.dedup();
    assert_eq!(counted.len(), before, "a file was counted by more than one walk");
    assert_eq!(counted.len(), SHARED_FILES * 2, "every file was counted by some walk");
    remove(root.as_path());
}

/// How many walks empty one directory together, and how many files each of its two levels holds.
const SHARED_WALKS: usize = 4;
const SHARED_FILES: usize = 400;
