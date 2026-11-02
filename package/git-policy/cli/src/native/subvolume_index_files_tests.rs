//! Controls for the index rewrite on disk: which linked worktrees count, every outcome line of one index file (missing,
//!  declined, locked, unreadable, rewritten), and the walk over the checkout's indexes.

use super::*;
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
    let count: usize = refresh_all_indexes(copy.as_path(), IndexLock::Acquire, &mut |text: &str| { lines.push(text.to_string()); });
    assert_eq!(count, 2);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(format!("index {}:", copy.join(".git/index").display()).as_str()), "{lines:?}");
    assert!(lines[1].contains(".git/worktrees/wt/index") && lines[1].ends_with("entries refreshed"), "{lines:?}");
    assert_eq!(git_text(copy.join("wt").as_path(), &["status", "--porcelain"]), "");
    remove(parent.as_path());
}
