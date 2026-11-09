//! Controls for the index rewrite on disk:
//! which linked worktrees count,
//! every outcome line of one index file (missing,
//!  declined,
//! locked,
//! unreadable,
//! rewritten),
//! and the walk over the checkout's indexes.

use super::*;
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use std::ffi::OsStr;
use crate::subvolume_error::Interrupt;
use crate::subvolume_sync::{SyncOptions, sync_tree};
use crate::subvolume_test_support::{PlainPlatform, make_checkout};
use crate::test_support::{fixture, git, git_text, remove};

/// A checkout with two linked worktrees inside it and one outside,
/// and the three administrative directories.
fn checkout_with_worktrees(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "inside-b"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "inside-a"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "../outside"]);
    return (parent, root);
}

/// Copy a working tree and its Git directory with a pass,
/// so every file has a new inode and change time.
fn copy_of(source: &Path, destination: &Path) {
    std::fs::create_dir(destination).expect("destination");
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None };
    sync_tree(&PlainPlatform::new(), source, destination, options, &Interrupt::never()).expect("copy");
}

/// Only worktrees inside the checkout are found,
/// sorted by path;
/// the checkout itself,
/// one outside it,
/// one whose
///  directory is gone,
/// and an administrative directory with no `gitdir` file are not.
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

/// A file with no index is reported as such;
/// a file that is not an index,
/// or one Git cannot have written,
/// is declined
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

/// A copied index is rewritten through its lock:
/// the report counts the entries,
/// every stat field now matches the file,
///  Git finds the tree clean,
/// and no lock stays behind.
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

/// The lock is taken before the index is read:
/// with another command's lock in place the index is not even opened,
/// so a
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

/// Under a lock the migration holds,
/// the index is rewritten and the lock stays for its owner to release;
/// a lock that
///  holds other text is another command's,
/// and the index is left as copied.
#[test]
fn a_rewrite_under_the_held_lock_keeps_the_lock_and_a_foreign_lock_blocks_it() {
    let parent: PathBuf = fixture("index-files-held");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let index: PathBuf = copy.join(".git/index");
    let before: Vec<u8> = std::fs::read(index.as_path()).expect("index");
    std::fs::write(copy.join(".git/index.lock"), b"git add is writing here\n").expect("foreign lock");
    let blocked: String = refresh_index_file_locked(index.as_path(), copy.as_path(), IndexLock::Held("our token\n".into()));
    assert!(blocked.contains("held by another Git command"), "{blocked}");
    assert_eq!(std::fs::read(index.as_path()).expect("index"), before);
    std::fs::write(copy.join(".git/index.lock"), b"our token\n").expect("our lock");
    let line: String = refresh_index_file_locked(index.as_path(), copy.as_path(), IndexLock::Held("our token\n".into()));
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

/// Whether the running user is root,
/// which ignores directory permissions.
fn unsafe_effective_user_is_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// The walk reports the checkout's index first,
/// then each linked worktree's,
/// counts them,
/// and refreshes each one.
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
    let count: usize = refresh_all_indexes(copy.as_path(), IndexLock::Acquire, &[], false, &mut |text: &str| { lines.push(text.to_string()); })
        .expect("the indexes were refreshed");
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
///  modification time,
/// as a tool that preserves times does:
/// `racy.txt` has the time of the index file itself (Git cannot
///  trust such an entry),
/// `quiet.txt` has an older time (only its change time shows the edit).
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

/// An entry modified in the same second as the index file is racy:
/// the rewrite leaves its stat fields alone,
/// so Git
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
///  when the tree it was copied from is given as the reference,
/// so Git hashes it and sees the edit.
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

/// A linked worktree whose directory name is not UTF-8 is found,
/// because the recorded path is read as bytes.
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

/// The hash length comes from `extensions.objectformat`:
/// 32 for sha256,
/// 20 when the setting is absent or another,
/// nothing
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

/// The stat fields of a file as `lstat` reports them now,
/// and the fields an index entry recorded for the same file.
fn recorded_and_present() -> (StatFields, StoredStat) {
    let now: StatFields = StatFields { ctime_seconds: 10, ctime_nanos: 11, mtime_seconds: 20, mtime_nanos: 21, dev: 5, ino: 30, uid: 40, gid: 50, size: 60 };
    let stored: StoredStat = StoredStat { ctime_seconds: 10, ctime_nanos: 11, mtime_seconds: 20, mtime_nanos: 21, ino: 30, uid: 40, gid: 50, size: 60 };
    return (now, stored);
}

/// An entry matches its file when every field it recorded agrees,
/// whatever the device (a subvolume has its own),
/// and each
///  field that differs alone makes it stale;
/// this is the positive control of the nanosecond rule that follows.
#[test]
fn an_entry_matches_a_file_when_every_recorded_field_agrees_and_not_when_one_differs() {
    let (now, stored) = recorded_and_present();
    assert!(matches_stored(&now, &stored));
    assert!(matches_stored(&StatFields { dev: now.dev + 1, ..now }, &stored), "the device is not compared");
    let changed: [StatFields; 8] = [
        StatFields { size: now.size + 1, ..now },
        StatFields { ino: now.ino + 1, ..now },
        StatFields { uid: now.uid + 1, ..now },
        StatFields { gid: now.gid + 1, ..now },
        StatFields { ctime_seconds: now.ctime_seconds + 1, ..now },
        StatFields { ctime_nanos: now.ctime_nanos + 1, ..now },
        StatFields { mtime_seconds: now.mtime_seconds + 1, ..now },
        StatFields { mtime_nanos: now.mtime_nanos + 1, ..now },
    ];
    for (index, other) in changed.iter().enumerate() {
        assert!(!matches_stored(other, &stored), "field {index} differs");
    }
}

/// A recorded nanosecond count of zero (an index written by a Git built without nanosecond support) matches any present
///  count;
/// a recorded non-zero count must equal the present one.
/// Both rules apply to the change time and the modification
///  time on their own.
#[test]
fn a_recorded_zero_nanosecond_count_matches_any_and_a_non_zero_one_must_agree() {
    let (now, stored) = recorded_and_present();
    let other_nanos: u32 = now.mtime_nanos + 1;
    let without_mtime_nanos: StoredStat = StoredStat { mtime_nanos: 0, ..stored };
    assert!(matches_stored(&StatFields { mtime_nanos: other_nanos, ..now }, &without_mtime_nanos), "mtime recorded as zero matches any");
    assert!(matches_stored(&now, &without_mtime_nanos));
    assert!(!matches_stored(&StatFields { mtime_nanos: other_nanos, ..now }, &stored), "mtime recorded non-zero must agree");
    let without_ctime_nanos: StoredStat = StoredStat { ctime_nanos: 0, ..stored };
    assert!(matches_stored(&StatFields { ctime_nanos: now.ctime_nanos + 1, ..now }, &without_ctime_nanos), "ctime recorded as zero matches any");
    assert!(!matches_stored(&StatFields { ctime_nanos: now.ctime_nanos + 1, ..now }, &stored), "ctime recorded non-zero must agree");
    assert!(matches_stored(&now, &stored), "equal non-zero counts agree");
}

/// A seed index of a one-file repository,
/// and an administrative directory with no index.
fn seed_and_admin(name: &str) -> (PathBuf, Vec<u8>, PathBuf, std::fs::File, StatFields) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = std::fs::canonicalize(crate::test_support::repository(root.as_path(), "repo")).expect("canonical");
    std::fs::write(repo.join("a.txt"), b"alpha\n").expect("file");
    // Older than the index, as the entries of a proven-clean source are; a newer entry is racy and is never refreshed.
    crate::test_support::age(repo.join("a.txt").as_path());
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let seed: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    let admin: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("admin");
    std::fs::create_dir(admin.as_path()).expect("admin");
    let held: std::fs::File = crate::fs_beneath::open_root(admin.as_path()).expect("handle");
    let real: StatFields = fields_of(&std::fs::symlink_metadata(repo.join("a.txt")).expect("metadata"));
    return (root, seed, admin, held, StatFields { ino: 6, ..real });
}

/// The index is installed under the lock with the stat fields of the provider,
/// and no lock is left.
#[test]
fn an_index_is_installed_under_the_lock_with_the_stat_fields_it_was_given() {
    let (root, seed, admin, held, fields) = seed_and_admin("install-index-ok");
    let report: String = install_index_in(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20).expect("installed");
    assert_eq!(report, "1 of 1 entries refreshed");
    assert!(admin.join("index").is_file() && !admin.join("index.lock").exists());
    let entries = crate::worktree_clean::stored_stats(std::fs::read(admin.join("index")).expect("index").as_slice(), 20).expect("entries");
    assert_eq!(entries[0].stored.ino, 6, "the fields of the provider");
    remove(root.as_path());
}

/// An entry the provider does not know is left as the seed has it.
#[test]
fn an_entry_the_provider_does_not_know_keeps_its_stored_fields() {
    let (root, seed, admin, held, _fields) = seed_and_admin("install-index-unknown");
    let report: String = install_index_in(&held, seed.as_slice(), &mut |_path: &[u8]| { return None; }, 20).expect("installed");
    assert_eq!(report, "0 of 1 entries refreshed");
    assert_eq!(std::fs::read(admin.join("index")).expect("index").len(), seed.len());
    remove(root.as_path());
}

/// An index that stands,
/// or a lock another process holds,
/// ends the install with both left alone.
#[test]
fn an_existing_index_or_a_held_lock_ends_the_install_untouched() {
    let (root, seed, admin, held, fields) = seed_and_admin("install-index-refused");
    std::fs::write(admin.join("index.lock"), b"theirs").expect("their lock");
    assert!(install_index_in(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20).expect_err("locked").contains("index.lock exists"));
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("lock"), b"theirs");
    assert!(!admin.join("index").exists());
    std::fs::remove_file(admin.join("index.lock")).expect("their lock goes");
    std::fs::write(admin.join("index"), b"theirs").expect("their index");
    assert!(install_index_in(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20).expect_err("an index stands").contains("appeared"));
    assert_eq!(std::fs::read(admin.join("index")).expect("index"), b"theirs");
    assert!(!admin.join("index.lock").exists(), "our lock went");
    remove(root.as_path());
}

/// An index that another command wrote between the check and the publication is not replaced.
#[test]
fn an_index_written_before_the_publication_is_not_replaced() {
    let (root, seed, admin, held, fields) = seed_and_admin("install-index-raced");
    let mut write_theirs = || { std::fs::write(admin.join("index"), b"theirs").expect("their index"); };
    let error: String = install_index_in_with(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20, &mut InstallSeams { before_publish: &mut write_theirs, after_publish: &mut || {} }).expect_err("refused");
    assert!(error.contains("cannot be put in place"), "{error}");
    assert_eq!(std::fs::read(admin.join("index")).expect("index"), b"theirs");
    assert!(!admin.join("index.lock").exists(), "our lock went");
    remove(root.as_path());
}

/// A lock name that another process took while the index was written does not change what is published:
/// the file that was
///  written is the one that gets the name of the index,
/// and the other process's lock stays.
#[test]
fn a_lock_replaced_before_the_publication_does_not_change_the_file_published() {
    let (root, seed, admin, held, fields) = seed_and_admin("install-index-lock-replaced");
    let mut replace = || {
        std::fs::rename(admin.join("index.lock"), admin.join("index.lock.moved")).expect("our lock moved away");
        std::fs::write(admin.join("index.lock"), b"theirs").expect("their lock");
    };
    install_index_in_with(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20, &mut InstallSeams { before_publish: &mut replace, after_publish: &mut || {} }).expect("installed");
    let entries = crate::worktree_clean::stored_stats(std::fs::read(admin.join("index")).expect("index").as_slice(), 20).expect("what was written is a valid index");
    assert_eq!(entries[0].stored.ino, 6, "the index is the file written here");
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("lock"), b"theirs", "their lock stays");
    remove(root.as_path());
}

/// The lock name the next Git process takes between the publication and the release is not removed.
#[test]
fn a_lock_taken_right_after_the_publication_stays() {
    let (root, seed, admin, held, fields) = seed_and_admin("install-index-after");
    // The lock name is taken over by the next writer between the publication and the release.
    let mut take_lock = || {
        std::fs::rename(admin.join("index.lock"), admin.join("index.lock.moved")).expect("our lock moves");
        std::fs::write(admin.join("index.lock"), b"the next writer").expect("their lock");
    };
    install_index_in_with(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20, &mut InstallSeams { before_publish: &mut || {}, after_publish: &mut take_lock }).expect("installed");
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("lock"), b"the next writer", "released only while the name still holds our file");
    assert!(admin.join("index").is_file());
    remove(root.as_path());
}

/// A refresh under its own lock never touches the lock name after the new index was renamed over the index:
/// a Git command that
///  takes the lock right after keeps it.
#[test]
fn a_refresh_leaves_the_lock_of_the_next_command_alone() {
    let parent: PathBuf = fixture("index-files-next-lock");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let index: PathBuf = copy.join(".git/index");
    let lock: PathBuf = copy.join(".git/index.lock");
    let mut next_command = || { std::fs::write(lock.as_path(), b"the next command's lock").expect("their lock"); };
    let line: String = refresh_index_file_seamed(index.as_path(), copy.as_path(), &RefreshOptions { lock: IndexLock::Acquire, references: &[], hash_length: None }, &mut next_command);
    assert!(line.ends_with("entries refreshed"), "{line}");
    assert_eq!(std::fs::read(lock.as_path()).expect("lock"), b"the next command's lock");
    remove(parent.as_path());
}

/// A refresh that did not publish removes its own lock,
/// and only if the name still holds it.
#[test]
fn a_refresh_that_published_nothing_removes_its_own_lock_only() {
    let parent: PathBuf = fixture("index-files-own-lock");
    std::fs::create_dir(parent.join("tree")).expect("tree");
    let missing: PathBuf = parent.join("tree/index");
    let line: String = refresh_index_file(missing.as_path(), parent.join("tree").as_path());
    assert_eq!(line, "no index file");
    assert!(!parent.join("tree/index.lock").exists(), "its own lock is removed");
    let lock: PathBuf = parent.join("tree/index.lock");
    let mut taken_over = || {
        std::fs::remove_file(lock.as_path()).expect("our lock goes");
        std::fs::write(lock.as_path(), b"theirs").expect("their lock");
    };
    let again: String = refresh_index_file_seamed(missing.as_path(), parent.join("tree").as_path(), &RefreshOptions { lock: IndexLock::Acquire, references: &[], hash_length: None }, &mut taken_over);
    assert_eq!(again, "no index file");
    assert_eq!(std::fs::read(lock.as_path()).expect("lock"), b"theirs", "a lock that is not ours stays");
    remove(parent.as_path());
}

/// A staging name that already exists,
/// as a link to a user's file or as a file,
/// is neither followed nor overwritten:
/// the
///  refresh under a held lock is refused and the file the link names keeps its content.
#[test]
fn an_existing_staging_name_is_never_followed_or_overwritten() {
    let parent: PathBuf = fixture("index-files-staging");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let copy: PathBuf = parent.join("copy");
    copy_of(root.as_path(), copy.as_path());
    let index: PathBuf = copy.join(".git/index");
    let before: Vec<u8> = std::fs::read(index.as_path()).expect("index");
    std::fs::write(copy.join(".git/index.lock"), b"our token\n").expect("our lock");
    let victim: PathBuf = parent.join("victim.txt");
    std::fs::write(victim.as_path(), b"a user's file").expect("victim");
    std::os::unix::fs::symlink(victim.as_path(), copy.join(".git/index.cli-git-new")).expect("a link at the staging name");
    let line: String = refresh_index_file_locked(index.as_path(), copy.as_path(), IndexLock::Held("our token\n".into()));
    assert!(line.starts_with("skipped"), "{line}");
    assert_eq!(std::fs::read(victim.as_path()).expect("victim"), b"a user's file", "the link was not followed");
    assert_eq!(std::fs::read(index.as_path()).expect("index"), before);
    assert!(std::fs::symlink_metadata(copy.join(".git/index.cli-git-new")).expect("link").file_type().is_symlink(), "the staging name was left alone");
    std::fs::remove_file(copy.join(".git/index.cli-git-new")).expect("link removed");
    std::fs::write(copy.join(".git/index.cli-git-new"), b"somebody's file").expect("a file at the staging name");
    let again: String = refresh_index_file_locked(index.as_path(), copy.as_path(), IndexLock::Held("our token\n".into()));
    assert!(again.starts_with("skipped"), "{again}");
    assert_eq!(std::fs::read(copy.join(".git/index.cli-git-new")).expect("kept"), b"somebody's file");
    remove(parent.as_path());
}

/// A lock name whose file was unlinked before the publication fails the install closed:
/// nothing is published and the other
///  process's lock stays.
#[test]
fn an_unlinked_lock_fails_the_install_closed() {
    let (root, seed, admin, held, fields) = seed_and_admin("install-index-lock-unlinked");
    let mut unlink = || {
        std::fs::remove_file(admin.join("index.lock")).expect("our lock is unlinked");
        std::fs::write(admin.join("index.lock"), b"theirs").expect("their lock");
    };
    let error: String = install_index_in_with(&held, seed.as_slice(), &mut |path: &[u8]| { return (path == b"a.txt").then_some(fields); }, 20, &mut InstallSeams { before_publish: &mut unlink, after_publish: &mut || {} }).expect_err("refused");
    assert!(error.contains("cannot be put in place"), "{error}");
    assert!(!admin.join("index").exists());
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("lock"), b"theirs");
    remove(root.as_path());
}

/// What:
///  A named pipe where the index belongs leaves the index skipped at once,
///  with the reason,
/// instead of
///  waiting for a writer that never comes.
/// Why:
///   Seeing another command write the index is the one thing this read must never do by waiting.
#[test]
fn a_named_pipe_where_the_index_belongs_is_skipped_at_once() {
    let parent: PathBuf = fixture("index-files-fifo");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let index: PathBuf = root.join(".git/index");
    std::fs::remove_file(index.as_path()).expect("remove the index");
    crate::subvolume_sync_attrs::make_fifo(index.as_path(), 0o600).expect("named pipe");
    let tree: PathBuf = root.clone();
    let read = crate::test_support::finished_within(move || { return refresh_index_file(index.as_path(), tree.as_path()); });
    let Some(line) = read else { panic!("reading the index did not finish within five seconds") };
    assert!(line.starts_with("skipped (cannot read it:") && line.contains("not a plain file"), "{line}");
    remove(parent.as_path());
}

/// What:
///  A named pipe where the repository's configuration belongs reads as no configuration at once,
/// instead of
///  waiting for a writer that never comes.
/// Why:
///   The configuration names the object format,
/// and a read that waits for it holds the command that owns a checkout.
#[test]
fn a_named_pipe_where_the_configuration_belongs_reads_as_none_at_once() {
    let parent: PathBuf = fixture("index-files-config-fifo");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let config: PathBuf = root.join(".git/config");
    std::fs::remove_file(config.as_path()).expect("remove the configuration");
    crate::subvolume_sync_attrs::make_fifo(config.as_path(), 0o600).expect("named pipe");
    let git_directory: PathBuf = root.join(".git");
    let read = crate::test_support::finished_within(move || { return repository_hash_length(git_directory.as_path()); });
    let Some(length) = read else { panic!("reading the configuration did not finish within five seconds") };
    assert_eq!(length, None);
    remove(parent.as_path());
}

/// What:
///  A named pipe where a registration's `gitdir` belongs is left out at once,
///  with the reason,
/// instead of
///  waiting for a writer that never comes.
/// Why:
///   The registrations are read before anything is copied,
/// so a read that waits stops the command before its first step.
#[test]
fn a_named_pipe_where_a_registration_gitdir_belongs_is_left_out_at_once() {
    let parent: PathBuf = fixture("index-files-gitdir-fifo");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "wt"]);
    let recorded: PathBuf = root.join(".git/worktrees/wt/gitdir");
    std::fs::remove_file(recorded.as_path()).expect("remove the gitdir file");
    crate::subvolume_sync_attrs::make_fifo(recorded.as_path(), 0o600).expect("named pipe");
    let scanned: PathBuf = root.clone();
    let read = crate::test_support::finished_within(move || { return scan_linked_worktrees(scanned.as_path()); });
    let Some(found) = read else { panic!("reading the registration did not finish within five seconds") };
    let scan: LinkedWorktreeScan = found.expect("the scan finishes");
    assert_eq!(scan.inside.len(), 0, "no registration is a worktree: {scan:?}");
    assert_eq!(scan.left_out.len(), 1, "{scan:?}");
    assert!(scan.left_out[0].reason.contains("not a plain file"), "{:?}", scan.left_out[0].reason);
    remove(parent.as_path());
}
