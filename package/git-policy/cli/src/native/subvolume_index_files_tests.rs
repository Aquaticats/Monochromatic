//! Controls for the index rewrite on disk: which linked worktrees count, every outcome line of one index file (missing,
//!  declined, locked, unreadable, rewritten), and the walk over the checkout's indexes.

use super::*;
use std::ffi::OsStr;
use crate::subvolume_error::Interrupt;
use crate::subvolume_sync::{SyncOptions, sync_tree};
use crate::subvolume_test_support::{PlainPlatform, make_checkout};
use crate::test_support::{fixture, git, git_text, remove};

/// A checkout with two linked worktrees inside it and one outside, and the three administrative directories.
fn checkout_with_worktrees(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "inside-b"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "inside-a"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "../outside"]);
    return (parent, root);
}

/// Copy a working tree and its Git directory with a pass, so every file has a new inode and change time.
fn copy_of(source: &Path, destination: &Path) {
    std::fs::create_dir(destination).expect("destination");
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None };
    sync_tree(&PlainPlatform::new(), source, destination, options, &Interrupt::never()).expect("copy");
}

/// Only worktrees inside the checkout are found, sorted by path; the checkout itself, one outside it, one whose
///  directory is gone, and an administrative directory with no `gitdir` file are not.
#[test]
fn only_existing_linked_worktrees_inside_the_checkout_are_found() {
    let (parent, root) = checkout_with_worktrees("index-files-worktrees");
    std::fs::remove_dir_all(parent.join("outside")).expect("outside is irrelevant either way");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "gone"]);
    std::fs::remove_dir_all(root.join("gone")).expect("the directory of a worktree Git would prune");
    std::fs::create_dir(root.join(".git/worktrees/orphan")).expect("an administrative directory with no gitdir file");
    let found: Vec<LinkedWorktree> = linked_worktrees_inside(root.as_path());
    let worktrees: Vec<PathBuf> = found.iter().map(|linked: &LinkedWorktree| { return linked.worktree.clone(); }).collect();
    assert_eq!(worktrees, vec![root.join("inside-a"), root.join("inside-b")]);
    assert!(found.iter().all(|linked: &LinkedWorktree| { return linked.admin_directory.starts_with(root.join(".git/worktrees")); }));
    assert_eq!(linked_worktrees_inside(parent.join("absent").as_path()), Vec::<LinkedWorktree>::new());
    remove(parent.as_path());
}

/// A file with no index is reported as such; a file that is not an index, or one Git cannot have written, is declined
///  with its reason and left as it was.
#[test]
fn a_missing_and_a_declined_index_are_reported_and_left_alone() {
    let parent: PathBuf = fixture("index-files-declined");
    let index: PathBuf = parent.join("index");
    assert_eq!(refresh_index_file(index.as_path(), parent.as_path()), "no index file");
    std::fs::write(index.as_path(), b"not an index at all").expect("file");
    let line: String = refresh_index_file(index.as_path(), parent.as_path());
    assert!(line.starts_with("skipped (") && line.contains("no DIRC signature"), "{line}");
    assert_eq!(std::fs::read(index.as_path()).expect("unchanged"), b"not an index at all");
    assert!(!parent.join("index.lock").exists());
    remove(parent.as_path());
}

/// An index that cannot be read says why.
#[test]
fn an_unreadable_index_is_reported() {
    let parent: PathBuf = fixture("index-files-unreadable");
    std::fs::create_dir(parent.join("index")).expect("a directory where the index would be");
    let line: String = refresh_index_file(parent.join("index").as_path(), parent.as_path());
    assert!(line.starts_with("skipped (cannot read it:"), "{line}");
    remove(parent.as_path());
}

/// A copied index is rewritten through its lock: the report counts the entries, every stat field now matches the file,
///  Git finds the tree clean, and no lock stays behind.
#[test]
fn a_copied_index_is_rewritten_through_its_lock() {
    let parent: PathBuf = fixture("index-files-rewrite");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let before: String = git_text(copy.as_path(), &["ls-files", "--debug", "--", "README.md"]);
    let line: String = refresh_index_file(copy.join(".git/index").as_path(), copy.as_path());
    assert!(line.ends_with("entries refreshed") && !line.starts_with("0 of"), "{line}");
    let after: String = git_text(copy.as_path(), &["ls-files", "--debug", "--", "README.md"]);
    assert_ne!(before, after, "the recorded inode changed");
    assert_eq!(git_text(copy.as_path(), &["status", "--porcelain"]), "");
    assert!(!copy.join(".git/index.lock").exists());
    remove(parent.as_path());
}

/// A lock held by a running Git command leaves the index as copied and says the first status will be slow.
#[test]
fn a_held_lock_leaves_the_index_alone_and_says_so() {
    let parent: PathBuf = fixture("index-files-locked");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let before: Vec<u8> = std::fs::read(copy.join(".git/index")).expect("index");
    std::fs::write(copy.join(".git/index.lock"), b"").expect("lock");
    let line: String = refresh_index_file(copy.join(".git/index").as_path(), copy.as_path());
    assert!(line.contains("index.lock exists") && line.contains("slow"), "{line}");
    assert_eq!(std::fs::read(copy.join(".git/index")).expect("index"), before);
    remove(parent.as_path());
}

/// The lock is taken before the index is read: with another command's lock in place the index is not even opened, so a
///  newer index installed by that command can never be replaced by a rewrite of the older one the read returned.
#[test]
fn the_lock_is_taken_before_the_index_is_read() {
    let parent: PathBuf = fixture("index-files-lock-first");
    std::fs::create_dir(parent.join("index")).expect("a directory where the index would be: reading it fails");
    std::fs::write(parent.join("index.lock"), b"another command\n").expect("the other command's lock");
    let line: String = refresh_index_file(parent.join("index").as_path(), parent.as_path());
    assert!(line.contains("index.lock exists"), "the lock was checked first: {line}");
    assert_eq!(std::fs::read(parent.join("index.lock")).expect("lock kept"), b"another command\n");
    remove(parent.as_path());
}

/// Under a lock the migration holds, the index is rewritten and the lock stays for its owner to release; a lock that
///  holds other text is another command's, and the index is left as copied.
#[test]
fn a_rewrite_under_the_held_lock_keeps_the_lock_and_a_foreign_lock_blocks_it() {
    let parent: PathBuf = fixture("index-files-held");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let index: PathBuf = copy.join(".git/index");
    let before: Vec<u8> = std::fs::read(index.as_path()).expect("index");
    std::fs::write(copy.join(".git/index.lock"), b"git add is writing here\n").expect("foreign lock");
    let blocked: String = refresh_index_file_locked(index.as_path(), copy.as_path(), IndexLock::Held("our token\n"));
    assert!(blocked.contains("held by another Git command"), "{blocked}");
    assert_eq!(std::fs::read(index.as_path()).expect("index"), before);
    std::fs::write(copy.join(".git/index.lock"), b"our token\n").expect("our lock");
    let line: String = refresh_index_file_locked(index.as_path(), copy.as_path(), IndexLock::Held("our token\n"));
    assert!(line.ends_with("entries refreshed") && !line.starts_with("0 of"), "{line}");
    assert_ne!(std::fs::read(index.as_path()).expect("index"), before, "the index was rewritten");
    assert_eq!(std::fs::read(copy.join(".git/index.lock")).expect("lock kept"), b"our token\n");
    assert!(!copy.join(".git/index.cli-git-new").exists(), "no sibling file is left");
    remove(parent.as_path());
}

/// A directory that does not allow a new file leaves the index as copied and says why.
#[test]
fn a_directory_that_refuses_the_lock_is_reported() {
    if unsafe_effective_user_is_root() {
        return;
    }
    let parent: PathBuf = fixture("index-files-no-lock");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    std::fs::set_permissions(copy.join(".git"), std::os::unix::fs::PermissionsExt::from_mode(0o500)).expect("read-only");
    let line: String = refresh_index_file(copy.join(".git/index").as_path(), copy.as_path());
    std::fs::set_permissions(copy.join(".git"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("restore");
    assert!(line.starts_with("skipped (cannot create the lock:"), "{line}");
    remove(parent.as_path());
}

/// Whether the running user is root, which ignores directory permissions.
fn unsafe_effective_user_is_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// The walk reports the checkout's index first, then each linked worktree's, counts them, and refreshes each one.
#[test]
fn every_index_of_the_checkout_is_refreshed_and_reported() {
    let parent: PathBuf = fixture("index-files-all");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "wt"]);
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    // The copy is a whole checkout of its own: its worktree records now name the copy's paths.
    std::fs::write(copy.join(".git/worktrees/wt/gitdir"), format!("{}\n", copy.join("wt/.git").display())).expect("gitdir");
    std::fs::write(copy.join("wt/.git"), format!("gitdir: {}\n", copy.join(".git/worktrees/wt").display())).expect("gitfile");
    let mut lines: Vec<String> = Vec::new();
    let count: usize = refresh_all_indexes(copy.as_path(), IndexLock::Acquire, &[], &mut |text: &str| { lines.push(text.to_string()); });
    assert_eq!(count, 2);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(format!("index {}:", copy.join(".git/index").display()).as_str()), "{lines:?}");
    assert!(lines[1].contains(".git/worktrees/wt/index") && lines[1].ends_with("entries refreshed"), "{lines:?}");
    assert_eq!(git_text(copy.join("wt").as_path(), &["status", "--porcelain"]), "");
    remove(parent.as_path());
}

/// Set the modification time of a file or of an index to whole seconds after the epoch.
fn set_seconds(path: &Path, seconds: i64) {
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// A repository with two tracked files whose content is then replaced by the same number of bytes and the same
///  modification time, as a tool that preserves times does: `racy.txt` has the time of the index file itself (Git cannot
///  trust such an entry), `quiet.txt` has an older time (only its change time shows the edit).
fn repository_with_silent_edits(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = crate::test_support::repository(parent.as_path(), "checkout");
    let (racy, quiet): (i64, i64) = (1_600_000_000, 1_599_000_000);
    std::fs::write(root.join("racy.txt"), b"aaaa").expect("racy");
    std::fs::write(root.join("quiet.txt"), b"cccc").expect("quiet");
    set_seconds(root.join("racy.txt").as_path(), racy);
    set_seconds(root.join("quiet.txt").as_path(), quiet);
    git(root.as_path(), &["add", "--all"]);
    git(root.as_path(), &["commit", "--quiet", "--message=files"]);
    std::fs::write(root.join("racy.txt"), b"bbbb").expect("racy edit");
    std::fs::write(root.join("quiet.txt"), b"dddd").expect("quiet edit");
    set_seconds(root.join("racy.txt").as_path(), racy);
    set_seconds(root.join("quiet.txt").as_path(), quiet);
    set_seconds(root.join(".git/index").as_path(), racy);
    return (parent, root);
}

/// An entry modified in the same second as the index file is racy: the rewrite leaves its stat fields alone, so Git
///  hashes the file and sees the edit instead of finding the copy clean.
#[test]
fn a_racy_entry_is_left_for_git_to_hash() {
    let (parent, root) = repository_with_silent_edits("index-files-racy");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let line: String = refresh_index_file(copy.join(".git/index").as_path(), copy.as_path());
    assert!(line.ends_with("entries refreshed"), "{line}");
    let status: String = git_text(copy.as_path(), &["status", "--porcelain"]);
    assert!(status.contains(" M racy.txt"), "the racy edit is seen: {status:?}");
    remove(parent.as_path());
}

/// A file that changed after Git recorded it (its change time moved while its size and time did not) is not refreshed
///  when the tree it was copied from is given as the reference, so Git hashes it and sees the edit.
#[test]
fn a_file_that_changed_after_git_looked_is_not_refreshed_when_the_original_is_the_reference() {
    let (parent, root) = repository_with_silent_edits("index-files-reference");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let references: [PathBuf; 1] = [root.clone()];
    let options: RefreshOptions<'_> = RefreshOptions { lock: IndexLock::Acquire, references: &references, hash_length: None };
    let line: String = refresh_index_file_with(copy.join(".git/index").as_path(), copy.as_path(), &options);
    assert!(line.ends_with("entries refreshed"), "{line}");
    let status: String = git_text(copy.as_path(), &["status", "--porcelain"]);
    assert!(status.contains(" M quiet.txt") && status.contains(" M racy.txt"), "both edits are seen: {status:?}");
    remove(parent.as_path());
}

/// A linked worktree whose directory name is not UTF-8 is found, because the recorded path is read as bytes.
#[test]
fn a_linked_worktree_with_a_non_utf8_name_is_found() {
    use std::os::unix::ffi::OsStrExt;
    let parent: PathBuf = fixture("index-files-non-utf8");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let worktree: PathBuf = root.join(OsStr::from_bytes(b"work-\xff"));
    std::fs::create_dir(worktree.as_path()).expect("a directory whose name is not UTF-8");
    let admin: PathBuf = root.join(".git/worktrees/odd");
    std::fs::create_dir_all(admin.as_path()).expect("admin directory");
    let mut recorded: Vec<u8> = worktree.join(".git").as_os_str().as_bytes().to_vec();
    recorded.push(b'\n');
    std::fs::write(admin.join("gitdir"), recorded).expect("gitdir");
    let found: Vec<LinkedWorktree> = linked_worktrees_inside(root.as_path());
    assert_eq!(found, vec![LinkedWorktree { admin_directory: admin, worktree }]);
    remove(parent.as_path());
}

/// The hash length comes from `extensions.objectformat`: 32 for sha256, 20 when the setting is absent or another, nothing
///  when there is no configuration file.
#[test]
fn the_hash_length_is_read_from_the_repository_configuration() {
    let parent: PathBuf = fixture("index-files-hash-length");
    assert_eq!(repository_hash_length(parent.as_path()), None);
    for (text, expected) in [
        ("[core]\n\trepositoryformatversion = 0\n", 20),
        ("[core]\n\trepositoryformatversion = 1\n[extensions]\n\tobjectFormat = sha256\n", 32),
        ("[Extensions]\nOBJECTFORMAT=SHA256\n", 32),
        ("[extensions]\n\tobjectformat = sha1\n", 20),
        ("[core]\n\tobjectformat = sha256\n", 20),
        ("[extensions]\n\tworktreeConfig = true\n[core]\n\tobjectformat = sha256\n", 20),
    ] {
        std::fs::write(parent.join("config"), text).expect("config");
        assert_eq!(repository_hash_length(parent.as_path()), Some(expected), "{text:?}");
    }
    remove(parent.as_path());
}
