//! Controls for the clone conversion fence beside the clone:
//! the seed the destination's index is installed from,
//! and the fence the other thread answers.
//! Why:
//!   The ordering is the option:
//! the index must hold no stat data for a path Git still has to write,
//!  and the fence must answer the same on its own thread as it does on the caller's.
//! A Git run is needed and no reflink is,
//! so these controls need a repository on any filesystem.

use super::*;
use crate::subvolume_index_files::fields_of;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::worktree_clean::stored_stats;
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use crate::worktree_snapshot_source::common_source;
use crate::fs_beneath::open_root;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  A repository whose `*.txt` paths Git's checkout writes with carriage returns,
/// and beside them a path no rule matches:
/// the first is a path the fence leaves to Git,
/// the second one it may carry.
/// Why:
///   The two answers are what the seed must tell apart,
/// so one fixture holds one of each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repositoryOf({ '.gitattributes': '*.txt text eol=crlf\n', 'notes.txt': 'line one\nline two\n', 'plain.dat': 'line one\nline two\n' });
/// ```
fn converting_fixture(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "notes.txt", b"line one\nline two\n");
    tracked(repo.as_path(), "plain.dat", b"line one\nline two\n");
    commit_all(repo.as_path(), "fixture");
    return (root, repo);
}

/// What:
///  The fence of a converting fixture,
/// and the source's index as the seed that holds the written path's stat data zeroed.
/// Why:
///   The seed is what makes the install honest,
/// so the control reads the two entries the fence told apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(sizeOf(seed, 'notes.txt')).toBe(0);
/// ```
#[test]
fn the_seed_zeroes_the_stat_data_of_the_path_the_fence_leaves_to_git() {
    let (root, repo) = converting_fixture("fence-seed-zeroed");
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let fence: Fence = fence_the_clone(&source, &git_context()).expect("a fence");
    assert_eq!(fence.git_writes, vec![b"notes.txt".to_vec()], "the carriage-return rule leaves one path to Git");
    let size_of = |seed: &[u8], path: &str| -> u32 {
        let entries = stored_stats(seed, source.clean.hash_length).expect("entries");
        return entries.iter().find(|entry| return entry.path == path.as_bytes()).expect("a tracked path").stored.size;
    };
    let plain: u32 = size_of(source.clean.index_bytes.as_slice(), "plain.dat");
    assert_ne!(size_of(source.clean.index_bytes.as_slice(), "notes.txt"), 0, "the source's index records the file's size");
    let seed: Vec<u8> = seed_without_stat(&source, &fence).expect("a seed");
    assert_eq!(size_of(seed.as_slice(), "notes.txt"), 0, "the written path's stat data is gone, so nothing vouches for its bytes until Git writes them");
    assert_eq!(size_of(seed.as_slice(), "plain.dat"), plain, "the carried path keeps the stat data the clone's file matches");
    remove(root.as_path());
}

/// The fence answers the same on the thread it runs beside the clone as it does on the caller's.
#[test]
fn the_fence_beside_the_clone_answers_what_the_fence_before_it_answers() {
    let (root, repo) = converting_fixture("fence-beside-answers");
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let before: Fence = fence_the_clone(&source, &git_context()).expect("a fence");
    let (state, work): AJob<(SnapshotSource, GitContext), Result<Fence, String>> = fence_beside_the_clone(&source, &git_context());
    let beside: Fence = work(state).expect("a fence beside the clone");
    assert_eq!(beside.git_writes, before.git_writes);
    assert_eq!(beside.carried, before.carried);
    // The seed is a function of the answer, so the same answer gives the same bytes.
    assert_eq!(seed_without_stat(&source, &beside).expect("a seed"), seed_without_stat(&source, &before).expect("a seed"));
    remove(root.as_path());
}

/// A source index the reader cannot walk is the rung's reason,
/// never a seed that vouches for a path.
#[test]
fn a_seed_declines_an_index_it_cannot_walk() {
    let (root, repo) = converting_fixture("fence-seed-unwalkable");
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let fence: Fence = run_the_fence((source.clone(), git_context())).expect("a fence");
    let mut broken = source.clone();
    broken.clean.index_bytes = b"DIRC".to_vec();
    let reason: String = seed_without_stat(&broken, &fence).expect_err("refused");
    assert!(reason.contains("a layout the reader does not know"), "{reason}");
    remove(root.as_path());
}

/// What:
///  What the clones leave in a destination:
/// every tracked file's bytes,
/// with the source's modification time,
/// so an entry's own fields describe the copied file.
/// Why:
///   The control is about the stat data the install records,
/// so the copy must be one such data would vouch for if the install did not refuse to record it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of entries) { copyFileSync(source, destination); utimesSync(destination, mtime, mtime); }
/// ```
fn copy_the_source_files(source: &SnapshotSource, destination: &Path) -> HashMap<Vec<u8>, StatFields> {
    let entries = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length).expect("entries");
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    for entry in entries.iter() {
        let from: PathBuf = source.root.join(OsStr::from_bytes(entry.path.as_slice()));
        let to: PathBuf = destination.join(OsStr::from_bytes(entry.path.as_slice()));
        std::fs::create_dir_all(to.parent().expect("parent")).expect("directories");
        std::fs::copy(from.as_path(), to.as_path()).expect("the copy");
        let modified = std::fs::metadata(from.as_path()).expect("the source's fields").modified().expect("a modification time");
        std::fs::File::options().write(true).open(to.as_path()).expect("the copy").set_modified(modified).expect("the source's time");
        stats.insert(entry.path.clone(), fields_of(&std::fs::metadata(to.as_path()).expect("the copy's fields")));
    }
    return stats;
}

/// What:
///  The index the clone rung installs after the fence answered,
/// read back entry by entry,
///  beside the bytes Git's own checkout writes for the same commit.
/// Why:
///   The order is only honest if no path Git still has to write is recorded clean,
/// so the control asserts the two things the install decides:
///  an entry the fence left to Git holds no stat data at all,
///  and the path's bytes at that moment are not the ones Git's checkout makes,
/// which is why a stat that vouched for them would have been a lie.
/// A plant that installs the source's index unchanged
/// (`seed_without_stat` returning it as it is,
/// the fence-before-the-clone order)
/// makes the first assertion fail and names the path,
///  the size the entry records and the size Git writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installBesideTheClone(admin, source, fence, stats);
/// expect(stored(index, 'notes.txt').size).toBe(0);
/// expect(sizeOnDisk('notes.txt')).not.toBe(referenceSize);
/// checkoutIndexThroughGit(admin, destination, context, fence.gitWrites, hashLength);
/// expect(trackedContent(destination)).toEqual(trackedContent(reference));
/// ```
#[test]
fn the_installed_index_vouches_for_no_path_git_still_has_to_write() {
    let (root, repo) = converting_fixture("fence-install-order");
    let destination: PathBuf = repo.parent().expect("the fixture root").join("topic");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "reference", "../reference"]);
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let fence: Fence = run_the_fence((source.clone(), git_context())).expect("a fence");
    assert_eq!(fence.git_writes, vec![b"notes.txt".to_vec()], "the rule leaves one path to Git");
    let stats: HashMap<Vec<u8>, StatFields> = copy_the_source_files(&source, destination.as_path());
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let held: std::fs::File = open_root(admin.as_path()).expect("the administrative directory");
    install_beside_the_clone(&held, &source, &fence, &stats).expect("the index");
    let entries = stored_stats(std::fs::read(admin.join("index")).expect("the installed index").as_slice(), source.clean.hash_length).expect("entries");
    let stored_of = |path: &str| -> u32 { return entries.iter().find(|entry| return entry.path == path.as_bytes()).expect("a tracked path").stored.size; };
    let gits_size: u64 = std::fs::metadata(repo.parent().expect("the fixture root").join("reference/notes.txt")).expect("Git's checkout").len();
    assert_eq!(stored_of("notes.txt"), 0, "the entry the fence left to Git records no stat data, so no status can skip the file: it records {} and Git writes {gits_size} bytes", stored_of("notes.txt"));
    assert_ne!(stored_of("plain.dat"), 0, "the carried path keeps the stat data of the file the clone made");
    assert_ne!(std::fs::metadata(destination.join("notes.txt")).expect("the copy").len(), gits_size, "the bytes on disk at this moment are not the ones Git's checkout writes, so a stat that vouched for them would have been a lie");
    crate::worktree_clone_conversion_fence::checkout_index_through_git(admin.as_path(), destination.as_path(), &git_context(), fence.git_writes.as_slice(), source.clean.hash_length).expect("the written path");
    assert_eq!(std::fs::metadata(destination.join("notes.txt")).expect("the written path").len(), gits_size, "Git wrote its own conversion");
    assert_eq!(git_text(destination.as_path(), &["status", "--porcelain=v1"]), "", "the worktree is clean once Git has written its paths");
    remove(root.as_path());
}
