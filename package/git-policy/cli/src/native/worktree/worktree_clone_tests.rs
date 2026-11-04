//! Controls for the clone rung: the plain-path refusal, the clone of files, links, modes and times with the source's index
//!  installed, a source that changed after the check, and the clearing of a partial destination.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{git, git_context, git_text, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use crate::worktree_snapshot_source::common_source;
use crate::worktree_clean::stored_stats;
use crate::subvolume_index::StatFields;

/// An ordinary-directory repository with an executable file, a link, a nested file, aged, and a `--no-checkout` worktree.
fn clone_fixture(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), ".gitignore", b"ignored/\n");
    tracked(repo.as_path(), "plain.txt", b"plain\n");
    tracked(repo.as_path(), "bin/tool.sh", b"#!/bin/sh\n");
    std::fs::set_permissions(repo.join("bin/tool.sh"), std::fs::Permissions::from_mode(0o755)).expect("executable");
    tracked(repo.as_path(), "empty.txt", b"");
    std::os::unix::fs::symlink("plain.txt", repo.join("link")).expect("link");
    crate::test_support::age(repo.join("link").as_path());
    commit_all(repo.as_path(), "files");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    return (repo, destination, admin);
}

/// A path that climbs out, is absolute, is empty or enters `.git` is refused; a plain one gives its names.
#[test]
fn only_plain_paths_become_entries() {
    assert_eq!(entry_names(b"a/b.txt"), Ok(vec![OsString::from("a"), OsString::from("b.txt")]));
    for path in [b"../x".as_slice(), b"/etc/passwd", b"", b".git/config", b"a/../../b", b"a/.git/x"] {
        assert!(entry_names(path).is_err(), "{}", String::from_utf8_lossy(path));
    }
}

/// Files are cloned with the source's bytes, mode and modification time, the link has the same target, and the installed
///  index makes the new worktree clean at once.
#[test]
fn btrfs_files_links_modes_and_times_are_cloned_and_the_index_is_installed() {
    let Some(root) = btrfs_fixture("clone-files") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    clone_into(&source, destination.as_path(), admin.as_path()).expect("clones");
    assert_eq!(std::fs::read(destination.join("plain.txt")).expect("plain"), b"plain\n");
    assert_eq!(std::fs::read_link(destination.join("link")).expect("link"), PathBuf::from("plain.txt"));
    for name in ["plain.txt", "bin/tool.sh", "empty.txt"] {
        let (from, to) = (std::fs::metadata(repo.join(name)).expect("source"), std::fs::metadata(destination.join(name)).expect("clone"));
        assert_eq!((to.mode() & 0o7777, to.mtime(), to.mtime_nsec(), to.len()), (from.mode() & 0o7777, from.mtime(), from.mtime_nsec(), from.len()), "{name}");
    }
    let status: String = git_text(destination.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(status, "", "the first status of the clone is clean");
    remove(root.as_path());
}

/// A source that changed after the check is refused by the comparison with its own index; the entries the run created stay
///  (nothing is removed by a name), and the reason counts them.
#[test]
fn btrfs_a_changed_source_is_refused_and_the_partial_destination_is_left_for_the_checkout() {
    let Some(root) = btrfs_fixture("clone-changed") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    std::fs::write(repo.join("plain.txt"), b"PLAIN\n").expect("edit after the check");
    crate::test_support::age(repo.join("plain.txt").as_path());
    let reason: String = clone_into(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("the source changed while it was cloned"), "{reason}");
    assert!(!admin.join("index").exists(), "no index for a refused clone");
    assert!(reason.contains("entries it created stay"), "{reason}");
    assert!(destination.join(".gitignore").exists() || destination.join("bin").exists(), "what was cloned stays");
    remove(root.as_path());
}

/// A tracked file the source lost ends the rung with the file's name.
#[test]
fn btrfs_a_missing_source_file_ends_the_rung() {
    let Some(root) = btrfs_fixture("clone-missing") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    std::fs::remove_file(repo.join("plain.txt")).expect("remove");
    let reason: String = clone_into(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("plain.txt"), "{reason}");
    remove(root.as_path());
}

/// Clone with the destination and the administrative directory opened as the ladder holds them.
fn clone_into(source: &SnapshotSource, destination: &Path, admin: &Path) -> Result<Cloned, String> {
    let held: File = open_root(destination).expect("destination");
    let administrative: File = open_root(admin).expect("administrative directory");
    return clone_tracked_files(source, &held, &administrative);
}

/// Clone one entry through the given handles, with the stat fields of the objects made discarded.
fn entry_through(handles: (&File, &File), components: &[&str], entry: &StatEntry, source: &SnapshotSource, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<(), String> {
    let names: Vec<OsString> = components.iter().map(OsString::from).collect();
    let work: Work<'_> = Work { entry, names: names.as_slice(), index_seconds: source.clean.index_seconds };
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    return clone_entry(handles.0, handles.1, &work, (created, &mut stats), seams);
}

/// The frozen entries of the source's index and handles on the source root and the destination, as `clone_all` holds them.
fn handles(source: &SnapshotSource, repo: &Path, destination: &Path) -> (Vec<StatEntry>, File, File) {
    let entries: Vec<StatEntry> = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length).expect("entries");
    return (entries, open_root(repo).expect("source root"), open_root(destination).expect("destination"));
}

/// The entry of one tracked path.
fn entry_of<'a>(entries: &'a [StatEntry], path: &str) -> &'a StatEntry {
    return entries.iter().find(|entry: &&StatEntry| { return entry.path == path.as_bytes(); }).expect("a tracked path");
}

/// A file replaced between the look and the open by another file of the same size and time is refused by the descriptor's
///  own metadata, and nothing of it reaches the destination.
#[test]
fn btrfs_a_file_swapped_between_the_look_and_the_open_is_not_cloned() {
    let Some(root) = btrfs_fixture("clone-swap-file") else { return };
    let (repo, destination, _admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let (entries, source_root, destination_root) = handles(&source, repo.as_path(), destination.as_path());
    let modified = std::fs::metadata(repo.join("plain.txt")).expect("metadata").modified().expect("time");
    let mut swap = || {
        std::fs::write(repo.join("swap.tmp"), b"PLAIN\n").expect("the other file, one byte of difference per letter, same size");
        std::fs::File::options().write(true).open(repo.join("swap.tmp")).expect("open").set_modified(modified).expect("the same time");
        std::fs::rename(repo.join("swap.tmp"), repo.join("plain.txt")).expect("swap");
    };
    let mut created: Vec<Created> = Vec::new();
    let reason: String = entry_through((&source_root, &destination_root), &["plain.txt"], entry_of(entries.as_slice(), "plain.txt"), &source, &mut created, &mut Seams { after_lookup: &mut swap, after_clone: &mut || {}, before_index: &mut || {} })
        .expect_err("refused");
    assert!(reason.contains("the source changed while it was cloned") && reason.contains("differs"), "{reason}");
    assert!(!destination.join("plain.txt").exists() && created.is_empty(), "the other file's bytes were not cloned");
    remove(root.as_path());
}

/// A file written after its data was cloned is refused, and the entry it made is on the list of what stays.
#[test]
fn btrfs_a_file_written_during_the_clone_is_refused_and_listed() {
    let Some(root) = btrfs_fixture("clone-write-during") else { return };
    let (repo, destination, _admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let (entries, source_root, destination_root) = handles(&source, repo.as_path(), destination.as_path());
    let mut append = || {
        let mut file = std::fs::OpenOptions::new().append(true).open(repo.join("plain.txt")).expect("open");
        std::io::Write::write_all(&mut file, b"more\n").expect("append");
    };
    let mut created: Vec<Created> = Vec::new();
    let reason: String = entry_through((&source_root, &destination_root), &["plain.txt"], entry_of(entries.as_slice(), "plain.txt"), &source, &mut created, &mut Seams { after_lookup: &mut || {}, after_clone: &mut append, before_index: &mut || {} })
        .expect_err("refused");
    assert!(reason.contains("written during the clone"), "{reason}");
    assert_eq!(created.iter().map(|entry: &Created| { return entry.names.clone(); }).collect::<Vec<_>>(), vec![vec![OsString::from("plain.txt")]]);
    remove(root.as_path());
}

/// A parent directory renamed away and replaced after the handle was taken is read through the handle: the original's
///  bytes are cloned, never the stand-in's.
#[test]
fn btrfs_a_parent_directory_swapped_after_it_was_opened_is_read_through_its_handle() {
    let Some(root) = btrfs_fixture("clone-swap-parent") else { return };
    let (repo, destination, _admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let (entries, _source_root, destination_root) = handles(&source, repo.as_path(), destination.as_path());
    let held_parent: File = open_root(repo.join("bin").as_path()).expect("the parent, opened first");
    std::fs::rename(repo.join("bin"), repo.join("bin.orig")).expect("renamed away");
    std::fs::create_dir(repo.join("bin")).expect("stand-in");
    std::fs::write(repo.join("bin/tool.sh"), b"#!/bin/sh\n").expect("a stand-in of the same size");
    let mut created: Vec<Created> = Vec::new();
    let mut seams: Seams<'_> = Seams { after_lookup: &mut || {}, after_clone: &mut || {}, before_index: &mut || {} };
    let mut tool: StatEntry = entry_of(entries.as_slice(), "bin/tool.sh").clone();
    tool.path = b"tool.sh".to_vec();
    let outcome = entry_through((&held_parent, &destination_root), &["tool.sh"], &tool, &source, &mut created, &mut seams);
    assert_eq!(outcome, Ok(()), "the original inode matches its entry although the path now names another");
    assert_ne!(std::fs::metadata(destination.join("tool.sh")).expect("clone").ino(), std::fs::metadata(repo.join("bin/tool.sh")).expect("stand-in").ino());
    assert_eq!(std::fs::read(destination.join("tool.sh")).expect("clone"), std::fs::read(repo.join("bin.orig/tool.sh")).expect("original"));
    remove(root.as_path());
}

/// A failed run removes nothing: a file and a directory that were in the destination before stay, and so do the clones that
///  were made, which the reason counts.
#[test]
fn btrfs_a_failed_run_removes_nothing_and_counts_what_stays() {
    let Some(root) = btrfs_fixture("clone-created-only") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    std::fs::write(destination.join("keep.txt"), b"mine").expect("a file that was there");
    std::fs::create_dir(destination.join("bin")).expect("a directory that was there");
    std::fs::write(destination.join("bin/mine"), b"mine").expect("its content");
    std::fs::write(repo.join("plain.txt"), b"PLAIN\n").expect("edit after the check");
    crate::test_support::age(repo.join("plain.txt").as_path());
    let reason: String = clone_into(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("the source changed while it was cloned") && reason.contains("entries it created stay"), "{reason}");
    let mut left: Vec<String> = walk(destination.as_path());
    left.sort();
    for kept in [".git", "bin", "bin/mine", "keep.txt"] {
        assert!(left.iter().any(|name: &String| { return name == kept; }), "{kept} stays: {left:?}");
    }
    assert!(left.iter().any(|name: &String| { return name == "bin/tool.sh" || name == ".gitignore"; }), "the clones that were made stay: {left:?}");
    remove(root.as_path());
}

/// Every path below a directory, relative, directories included.
fn walk(top: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![top.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory.as_path()).expect("listing").flatten() {
            found.push(entry.path().strip_prefix(top).expect("below").to_string_lossy().into_owned());
            if entry.file_type().is_ok_and(|kind: std::fs::FileType| { return kind.is_dir(); }) {
                pending.push(entry.path());
            }
        }
    }
    return found;
}

/// The index that was proven clean is the one cloned from: a source that moved to another commit after the check, with its
///  own consistent index, is refused, never cloned as the other commit.
#[test]
fn btrfs_the_frozen_index_is_used_not_a_second_read_of_the_file() {
    let Some(root) = btrfs_fixture("clone-frozen") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    tracked(repo.as_path(), "later.txt", b"later\n");
    commit_all(repo.as_path(), "later");
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD~1"]);
    let reason: String = clone_into(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("later.txt"), "{reason}");
    assert!(!admin.join("index").exists());
    remove(root.as_path());
}

/// The report of what a failed run left names how many entries stay, a few of their names, and the command that lists the
///  untracked ones.
#[test]
fn the_report_of_what_stays_names_the_count_the_examples_and_the_command() {
    let created: Vec<Created> = vec![
        Created { names: vec![OsString::from("dir")], directory: true },
        Created { names: vec![OsString::from("dir"), OsString::from("a.txt")], directory: false },
        Created { names: vec![OsString::from("b.txt")], directory: false },
    ];
    let report: String = leftover_report(created.as_slice(), Path::new("/base/topic"));
    assert!(report.starts_with("3 entries") && report.contains("dir/a.txt, b.txt") && report.contains("git -C /base/topic status"), "{report}");
}

/// A file replaced after it was cloned, with the same size and time, is not certified: the index records the inode that was
///  cloned, so Git sees the replacement as changed.
#[test]
fn btrfs_a_destination_file_replaced_before_the_index_is_not_certified() {
    let Some(root) = btrfs_fixture("clone-replaced-before-index") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let modified = std::fs::metadata(repo.join("plain.txt")).expect("metadata").modified().expect("time");
    let mut replace = || {
        std::fs::rename(destination.join("plain.txt"), destination.join("plain.orig")).expect("moved");
        std::fs::write(destination.join("plain.txt"), b"PLAIN\n").expect("a replacement of the same size");
        std::fs::File::options().write(true).open(destination.join("plain.txt")).expect("open").set_modified(modified).expect("the same time");
    };
    let mut directories: Directories = HashMap::new();
    let held: File = open_root(destination.as_path()).expect("destination");
    directories.insert(Vec::new(), held.try_clone().expect("clone handle"));
    let administrative: File = open_root(admin.as_path()).expect("admin");
    let mut created: Vec<Created> = Vec::new();
    clone_with(&source, &mut directories, &administrative, &mut created, &mut Seams { after_lookup: &mut || {}, after_clone: &mut || {}, before_index: &mut replace }).expect("clones");
    let entries = stored_stats(std::fs::read(admin.join("index")).expect("index").as_slice(), source.clean.hash_length).expect("entries");
    let recorded = entry_of(entries.as_slice(), "plain.txt").stored.ino;
    assert_eq!(recorded, std::fs::metadata(destination.join("plain.orig")).expect("the clone").ino() as u32, "the inode that was cloned");
    assert_ne!(recorded, std::fs::metadata(destination.join("plain.txt")).expect("the replacement").ino() as u32);
    remove(root.as_path());
}

/// A subdirectory the run walks that is another device is refused, so nothing is created across a subvolume.
#[test]
fn btrfs_a_destination_directory_on_another_device_is_refused() {
    let Some(root) = btrfs_fixture("clone-device") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    crate::fs_btrfs_ioctl::subvolume_create_at(destination.join("bin").as_path()).expect("a subvolume where bin goes");
    let reason: String = clone_into(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("another device"), "{reason}");
    assert!(!destination.join("bin/tool.sh").exists(), "nothing was created in it");
    remove(root.as_path());
}

/// A link replaced after it was made, by another of the same length, is not certified: the index keeps the stat fields the
///  source's entry had, which match neither link, instead of recording those of whatever the name holds by then.
#[test]
fn btrfs_a_destination_link_replaced_before_the_index_is_not_certified() {
    let Some(root) = btrfs_fixture("clone-link-replaced-before-index") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let mut replace = || {
        std::fs::remove_file(destination.join("link")).expect("the clone's link goes");
        std::os::unix::fs::symlink("plainXtxt", destination.join("link")).expect("a link of the same length");
    };
    let mut directories: Directories = HashMap::new();
    let held: File = open_root(destination.as_path()).expect("destination");
    directories.insert(Vec::new(), held.try_clone().expect("clone handle"));
    let administrative: File = open_root(admin.as_path()).expect("admin");
    let mut created: Vec<Created> = Vec::new();
    clone_with(&source, &mut directories, &administrative, &mut created, &mut Seams { after_lookup: &mut || {}, after_clone: &mut || {}, before_index: &mut replace }).expect("clones");
    let entries = stored_stats(std::fs::read(admin.join("index")).expect("index").as_slice(), source.clean.hash_length).expect("entries");
    let seed = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length).expect("seed");
    let recorded = entry_of(entries.as_slice(), "link").stored;
    assert_eq!(recorded.ino, entry_of(seed.as_slice(), "link").stored.ino, "the stored fields are the source's, not the live name's");
    assert_ne!(recorded.ino, std::fs::symlink_metadata(destination.join("link")).expect("replacement").ino() as u32);
    remove(root.as_path());
}
