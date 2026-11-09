//! Controls for the rollback and for the deletion of the old tree:
//! the rollback from every state the migration can be
//!  stopped in,
//! the deepest-first removal of nested subvolumes with and without the kernel's destroy call,
//! the refusals,
//!  and the deletion of the old tree with and without unreadable files.
//! They run on the stand-in platform.

use super::*;
use crate::fs_kind::FilesystemKind;
use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight::layout_of;
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, index_inode, migrate_arguments, run_command};
use std::os::unix::fs::MetadataExt;
use crate::test_support::{git_text, remove};
use std::cell::RefCell;
use std::fs::File;

/// The exit status of a refusal.
const REFUSED: i32 = 1;

/// The exit status of a finished run.
const DONE: i32 = 0;

/// The exit status of a failure after the journal began.
const FAILED: i32 = 2;

/// A platform that records which subvolumes the destroy call was asked to remove,
/// and can refuse the call.
struct Recording {
    inner: PlainPlatform,
    destroyed: RefCell<Vec<PathBuf>>,
    destroy_fails: bool,
}

/// Everything is delegated except the destroy call and the removal answer.
impl Platform for Recording {
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        return self.inner.filesystem_facts(path);
    }
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return self.inner.is_subvolume_root(path);
    }
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.create_subvolume(path);
    }
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        self.destroyed.borrow_mut().push(path.to_path_buf());
        if self.destroy_fails {
            return Err(PrimitiveError { operation: "BTRFS_IOC_SNAP_DESTROY", path: path.to_path_buf(), errno: libc::EPERM });
        }
        return self.inner.destroy_subvolume(path);
    }
    fn removal_allowed(&self, path: &Path) -> bool {
        return self.inner.removal_allowed(path);
    }
    fn is_mount_point(&self, path: &Path) -> bool {
        return self.inner.is_mount_point(path);
    }
    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return self.inner.exchange(first, second);
    }
    fn flushes_to_disk(&self) -> bool {
        return self.inner.flushes_to_disk();
    }
}

/// A platform whose chosen exchange takes effect and then fails,
/// as a kill right after the kernel call would leave it.
struct KillAfterExchange {
    inner: PlainPlatform,
    /// Which exchange call (counting from 1) fails after it took effect.
    kill_on: u32,
    calls: std::cell::Cell<u32>,
}

/// Everything is delegated;
/// the chosen exchange reports a failure after it ran.
impl Platform for KillAfterExchange {
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        return self.inner.filesystem_facts(path);
    }
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return self.inner.is_subvolume_root(path);
    }
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.create_subvolume(path);
    }
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.destroy_subvolume(path);
    }
    fn removal_allowed(&self, path: &Path) -> bool {
        return self.inner.removal_allowed(path);
    }
    fn is_mount_point(&self, path: &Path) -> bool {
        return self.inner.is_mount_point(path);
    }
    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        self.calls.set(self.calls.get() + 1);
        self.inner.exchange(first, second)?;
        if self.calls.get() == self.kill_on {
            return Err(PrimitiveError { operation: "renameat2(RENAME_EXCHANGE)", path: first.to_path_buf(), errno: libc::EINTR });
        }
        return Ok(());
    }
    fn flushes_to_disk(&self) -> bool {
        return self.inner.flushes_to_disk();
    }
}

/// A recording platform that allows removal.
fn recording(destroy_fails: bool) -> Recording {
    return Recording { inner: PlainPlatform::new(), destroyed: RefCell::new(Vec::new()), destroy_fails };
}

/// The arguments of a rollback.
fn rollback_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::Rollback, ..migrate_arguments() };
}

/// The arguments of the deletion of the old tree.
fn remove_old_arguments(discard_unreadable: bool) -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::RemoveOld, discard_unreadable, ..migrate_arguments() };
}

/// Whether the running user is root,
/// which cannot make an unreadable file.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// A checkout migrated to the end.
fn migrated(name: &str, platform: &dyn Platform) -> (PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let (code, lines) = run_command(platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    return (parent, root);
}

/// After a rollback the original directory is back as an ordinary directory,
/// clean,
/// with nothing left beside it.
fn assert_original_again(platform: &dyn Platform, root: &Path) {
    let layout = layout_of(root).expect("layout");
    assert_eq!(platform.is_subvolume_root(root), Ok(false));
    assert!(root.join("README.md").exists() && root.join("wt/README.md").exists());
    // Before the first status, which refreshes the index file itself and would hide a missing rewrite.
    for path in ["README.md", "src/main.rs"] {
        assert_eq!(index_inode(root, path), std::fs::metadata(root.join(path)).expect("file").ino(), "{path}: the restored index describes the restored files");
    }
    assert_eq!(git_text(root, &["status", "--porcelain"]), "");
    assert!(!layout.new_path.exists() && !layout.old_path.exists() && !layout.journal_path.exists() && !layout.unreadable_path.exists());
    assert!(!root.join(".git/index.lock").exists());
}

/// A finished migration is undone:
/// a file written to the new root since is carried back,
/// the nested subvolumes are
///  removed deepest first through the destroy call,
/// and the journal is gone.
#[test]
fn a_finished_migration_is_rolled_back_with_newer_files_and_nested_subvolumes_deepest_first() {
    let platform: Recording = recording(false);
    let (parent, root) = migrated("rollback-finished", &platform);
    std::fs::write(root.join("ignored/written-after.txt"), b"after the migration\n").expect("new file");
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_original_again(&platform, root.as_path());
    assert_eq!(std::fs::read(root.join("ignored/written-after.txt")).expect("carried back"), b"after the migration\n");
    let layout = layout_of(root.as_path()).expect("layout");
    assert_eq!(*platform.destroyed.borrow(), vec![layout.new_path.join("wt"), layout.new_path.clone()]);
    let text: String = lines.join("\n");
    assert!(text.contains("carry back") && text.contains("rolled back") && text.contains("nested subvolumes first"), "{text}");
    remove(parent.as_path());
}

/// Where the destroy call is not allowed or fails,
/// the subvolumes are emptied and removed instead,
/// with the same end.
#[test]
fn the_removal_falls_back_to_emptying_when_the_destroy_call_is_not_available() {
    let mut refused: PlainPlatform = PlainPlatform::new();
    refused.removal = false;
    let (parent, root) = migrated("rollback-no-option", &refused);
    let (code, lines) = run_command(&refused, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_original_again(&refused, root.as_path());
    remove(parent.as_path());
    let failing: Recording = recording(true);
    let (parent2, root2) = migrated("rollback-destroy-fails", &failing);
    let (code2, lines2) = run_command(&failing, &rollback_arguments(), root2.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    assert_original_again(&failing, root2.as_path());
    assert_eq!(failing.destroyed.borrow().len(), 2, "the call was tried for both subvolumes before the fallback");
    remove(parent2.as_path());
}

/// A rollback stopped at either of its own steps is finished by running it again.
#[test]
fn a_rollback_stopped_at_each_step_is_finished_by_the_next_run() {
    for point in ["rollback-before-exchange", "rollback-after-exchange"] {
        let platform: PlainPlatform = PlainPlatform::new();
        let (parent, root) = migrated(format!("rollback-resume-{point}").as_str(), &platform);
        let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::at(point, 0));
        assert_eq!(code, FAILED, "{point}: {lines:#?}");
        let (code2, lines2) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(code2, DONE, "{point}: {lines2:#?}");
        assert_original_again(&platform, root.as_path());
        remove(parent.as_path());
    }
}

/// A rollback killed right after its own exchange,
/// before the rename or the journal record,
/// must not swap the trees
///  a second time:
/// that would put the new subvolume back in the checkout's place and delete the original directory.
#[test]
fn a_rollback_killed_right_after_its_exchange_does_not_exchange_again() {
    let platform: KillAfterExchange = KillAfterExchange { inner: PlainPlatform::new(), kill_on: 2, calls: std::cell::Cell::new(0) };
    let (parent, root) = migrated("rollback-kill-after-swap", &platform);
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, FAILED, "{lines:#?}");
    let (code2, lines2) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    assert_eq!(platform.calls.get(), 2, "the second run did not exchange again");
    assert_original_again(&platform, root.as_path());
    remove(parent.as_path());
}

/// A rollback killed after the rename of the new subvolume but before the journal record is finished by the next run,
///  which must not refuse because the old tree's name is gone.
#[test]
fn a_rollback_killed_after_the_rename_before_the_record_is_finished() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("rollback-kill-after-rename", &platform);
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::at("rollback-after-rename", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let (code2, lines2) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    assert_original_again(&platform, root.as_path());
    remove(parent.as_path());
}

/// A migration run again while a rollback has swapped the trees is refused and changes nothing:
/// only the rollback can
///  finish the job.
#[test]
fn a_migration_is_refused_while_a_rollback_is_half_done() {
    let platform: KillAfterExchange = KillAfterExchange { inner: PlainPlatform::new(), kill_on: 2, calls: std::cell::Cell::new(0) };
    let (parent, root) = migrated("rollback-half-then-migrate", &platform);
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, FAILED, "{lines:#?}");
    let (code2, lines2) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code2, REFUSED, "{lines2:#?}");
    assert!(lines2.join("\n").contains("--rollback"), "{lines2:#?}");
    let (code3, lines3) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code3, DONE, "{lines3:#?}");
    assert_original_again(&platform, root.as_path());
    remove(parent.as_path());
}

/// A migration stopped before the exchange is undone from each earlier stop;
/// the migration's lock is removed.
#[test]
fn a_migration_stopped_before_the_exchange_is_rolled_back_from_every_earlier_stop() {
    for point in ["after-create", "after-nested", "copy-entry", "after-copy", "after-delta1", "after-lock", "before-exchange"] {
        let platform: PlainPlatform = PlainPlatform::new();
        let (parent, root) = checkout_with_worktree(format!("rollback-early-{point}").as_str());
        let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at(point, 0));
        assert_eq!(code, FAILED, "{point}: {lines:#?}");
        let (code2, lines2) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(code2, DONE, "{point}: {lines2:#?}");
        assert_original_again(&platform, root.as_path());
        remove(parent.as_path());
    }
}

/// A migration stopped between the exchange and the rename of the old tree is rolled back too.
#[test]
fn a_migration_stopped_between_the_exchange_and_the_rename_is_rolled_back() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("rollback-mid-exchange");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("after-exchange", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let (code2, lines2) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    assert_original_again(&platform, root.as_path());
    remove(parent.as_path());
}

/// Without a journal there is nothing recorded to undo:
/// status 1,
/// nothing changed.
#[test]
fn a_rollback_without_a_journal_is_refused() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("rollback-no-journal");
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, REFUSED, "{lines:#?}");
    let text: String = lines.join("\n");
    assert!(text.contains("no migration journal") && text.contains("Nothing was changed."), "{text}");
    assert!(root.join("README.md").exists());
    remove(parent.as_path());
}

/// A recorded exchange whose old tree has gone is refused with the two ways forward,
/// and nothing is deleted.
#[test]
fn a_rollback_whose_old_tree_is_missing_is_refused() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("rollback-old-missing", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::remove_dir_all(layout.old_path.as_path()).expect("delete the old tree by hand");
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, REFUSED, "{lines:#?}");
    let text: String = lines.join("\n");
    assert!(text.contains("old tree") && text.contains("is missing") && text.contains("move it back"), "{text}");
    assert_eq!(platform.is_subvolume_root(root.as_path()), Ok(true));
    assert!(layout.journal_path.exists());
    remove(parent.as_path());
}

/// The preview of the two other modes changes nothing and says what would happen.
#[test]
fn the_dry_run_of_the_other_modes_changes_nothing() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("rollback-dry", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    for (arguments, words) in [
        (MigrateArguments { dry_run: true, ..rollback_arguments() }, "restore the original directory"),
        (MigrateArguments { dry_run: true, ..remove_old_arguments(false) }, "delete the original tree"),
    ] {
        let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
        assert_eq!(code, DONE, "{lines:#?}");
        assert!(lines.join("\n").contains(words), "{lines:#?}");
        assert!(layout.old_path.exists() && layout.journal_path.exists());
    }
    remove(parent.as_path());
}

/// The old tree of a finished migration is deleted with the journal;
/// the checkout and its worktree stay intact.
#[test]
fn the_old_tree_is_deleted_after_a_finished_migration() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("remove-old-finished", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    let (code, lines) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert!(!layout.old_path.exists() && !layout.journal_path.exists());
    assert_eq!(platform.is_subvolume_root(root.as_path()), Ok(true));
    assert_eq!(git_text(root.as_path(), &["status", "--porcelain"]), "");
    assert!(lines.join("\n").contains("the old tree is deleted"));
    let (code2, lines2) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code2, REFUSED, "a second deletion has nothing to delete: {lines2:#?}");
    remove(parent.as_path());
}

/// The deletion is refused for a migration that is not finished,
/// and the old tree stays.
#[test]
fn the_old_tree_of_an_unfinished_migration_is_not_deleted() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("remove-old-unfinished");
    let (stopped, _stopped_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("after-indexes", 0));
    assert_eq!(stopped, FAILED);
    let layout = layout_of(root.as_path()).expect("layout");
    let (code, lines) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code, REFUSED, "{lines:#?}");
    assert!(lines.join("\n").contains("not recorded as finished") && layout.old_path.exists());
    remove(parent.as_path());
}

/// Files this user could not read exist only in the old tree:
/// the deletion is refused with both remedies,
/// and the
///  explicit discard deletes them with the tree.
#[test]
fn unreadable_files_hold_back_the_deletion_unless_discarded() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("remove-old-unreadable");
    std::fs::write(root.join("ignored/secret.log"), b"secret").expect("log");
    std::fs::set_permissions(root.join("ignored/secret.log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    let (code2, lines2) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code2, REFUSED, "{lines2:#?}");
    let text: String = lines2.join("\n");
    assert!(text.contains("1 file(s) that this user may not read") && text.contains("sudo cp --archive") && text.contains("--discard-unreadable"), "{text}");
    assert!(layout.old_path.join("ignored/secret.log").exists() && layout.unreadable_path.exists());
    let (code3, lines3) = run_command(&platform, &remove_old_arguments(true), root.as_path(), &Interrupt::never());
    assert_eq!(code3, DONE, "{lines3:#?}");
    assert!(!layout.old_path.exists() && !layout.unreadable_path.exists() && !layout.journal_path.exists());
    remove(parent.as_path());
}

/// What:
///  The unreadable-file list is replaced whole,
/// never rewritten in place.
/// Why:
///  The late-writes pass of `--remove-old` records a file it finds unreadable by rewriting the
/// list;
///  the old writer truncated the file and then wrote it,
/// so a kill between the two left an empty list,
///  which the next run read as "nothing was recorded" and deleted what it protected.
/// A replaced file
///  leaves the earlier record whole for a reader that opened it,
/// and a kill leaves either the old record or the
///  new one.
#[test]
fn rewriting_the_unreadable_list_replaces_it_and_never_truncates_it() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("remove-old-list-replaced");
    std::fs::write(root.join("ignored/secret-a.log"), b"secret a").expect("a");
    std::fs::set_permissions(root.join("ignored/secret-a.log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable a");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    let first: std::fs::Metadata = std::fs::metadata(layout.unreadable_path.as_path()).expect("the first list");
    assert!(std::fs::read_to_string(layout.unreadable_path.as_path()).expect("list").contains("secret-a.log"));
    // A reader that opened the record before the rewrite keeps the record it opened.
    let mut opened: File = File::open(layout.unreadable_path.as_path()).expect("open the list");
    // A second unreadable file, written to the old tree after the catch-up, is recorded by the late-writes pass.
    std::fs::write(layout.old_path.join("ignored/secret-b.log"), b"secret b").expect("b");
    std::fs::set_permissions(layout.old_path.join("ignored/secret-b.log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable b");
    let (code2, lines2) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code2, REFUSED, "{lines2:#?}");
    let second: std::fs::Metadata = std::fs::metadata(layout.unreadable_path.as_path()).expect("the rewritten list");
    assert_ne!(first.ino(), second.ino(), "the list is replaced, not rewritten in place");
    let mut kept: String = String::new();
    std::io::Read::read_to_string(&mut opened, &mut kept).expect("read the list that handle opened");
    assert!(kept.contains("secret-a.log"), "a reader that opened the earlier record still reads it whole: {kept:?}");
    let rewritten: String = std::fs::read_to_string(layout.unreadable_path.as_path()).expect("list");
    assert!(rewritten.contains("secret-a.log") && rewritten.contains("secret-b.log"), "{rewritten}");
    remove(parent.as_path());
}

/// A file that exists only in the new subvolume and that this user may not read cannot be carried back:
/// the rollback is
///  refused before the exchange,
/// nothing is deleted,
/// and the explicit discard deletes it with the subvolume.
#[test]
fn a_rollback_does_not_destroy_an_unreadable_file_that_only_the_new_subvolume_holds() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("rollback-unreadable-new", &platform);
    let secret: PathBuf = root.join("ignored/new-secret.log");
    std::fs::write(secret.as_path(), b"only here").expect("log");
    std::fs::set_permissions(secret.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable");
    let layout = layout_of(root.as_path()).expect("layout");
    let (code, lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, REFUSED, "{lines:#?}");
    let text: String = lines.join("\n");
    assert!(text.contains("exist only in the new subvolume") && text.contains("new-secret.log") && text.contains("--discard-unreadable"), "{text}");
    assert!(secret.exists() && platform.is_subvolume_root(root.as_path()) == Ok(true), "the checkout is still the new subvolume");
    assert!(!layout.unreadable_path.exists(), "the new root's unreadable files do not enter the old tree's list");
    let arguments: MigrateArguments = MigrateArguments { discard_unreadable: true, ..rollback_arguments() };
    let (code2, lines2) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    assert_original_again(&platform, root.as_path());
    remove(parent.as_path());
}

/// A manifest of unreadable files that cannot be read is not an empty manifest:
/// the deletion stops,
/// and the old tree
///  with the files only it holds stays.
#[test]
fn an_unreadable_manifest_stops_the_deletion_instead_of_counting_as_empty() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("remove-old-manifest-unreadable");
    std::fs::write(root.join("ignored/secret.log"), b"secret").expect("log");
    std::fs::set_permissions(root.join("ignored/secret.log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::set_permissions(layout.unreadable_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("hide the manifest");
    let (code2, lines2) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    std::fs::set_permissions(layout.unreadable_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("restore");
    assert_eq!(code2, FAILED, "{lines2:#?}");
    let text: String = lines2.join("\n");
    assert!(text.contains("unreadable.txt"), "the message names the manifest: {text}");
    assert!(layout.old_path.join("ignored/secret.log").exists() && layout.journal_path.exists());
    remove(parent.as_path());
}

/// Set the modification time of a path to whole seconds after the epoch.
fn set_seconds(path: &Path, seconds: i64) {
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// What a process that kept a descriptor in the old tree wrote after the catch-up is carried into the checkout before the
///  old tree is deleted:
/// a changed file and a new file;
/// a file that was deleted from the checkout on purpose stays
///  deleted.
#[test]
fn writes_to_the_old_tree_after_the_catch_up_are_carried_before_it_is_deleted() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("remove-old-late-writes", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::write(layout.old_path.join("README.md"), b"saved by a late writer\n").expect("late change");
    std::fs::write(layout.old_path.join("ignored/late.txt"), b"untracked work").expect("late file");
    // A file the person deleted from the new checkout, last modified long before the migration.
    std::fs::remove_file(root.join("src/main.rs")).expect("deleted on purpose");
    set_seconds(layout.old_path.join("src/main.rs").as_path(), 1_000_000_000);
    let (code, lines) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_eq!(std::fs::read(root.join("README.md")).expect("carried"), b"saved by a late writer\n");
    assert_eq!(std::fs::read(root.join("ignored/late.txt")).expect("carried"), b"untracked work");
    assert!(!root.join("src/main.rs").exists(), "a file deleted on purpose is not brought back");
    let text: String = lines.join("\n");
    assert!(text.contains("carried README.md") && text.contains("carried ignored/late.txt"), "{text}");
    assert!(!layout.old_path.exists());
    remove(parent.as_path());
}

/// A path at the old tree's place that is a symbolic link is never followed or deleted.
#[test]
fn an_old_tree_path_that_is_a_symbolic_link_is_refused() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("remove-old-symlink", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    let elsewhere: PathBuf = parent.join("elsewhere");
    std::fs::rename(layout.old_path.as_path(), elsewhere.as_path()).expect("move the old tree away");
    std::os::unix::fs::symlink(elsewhere.as_path(), layout.old_path.as_path()).expect("link in its place");
    std::fs::write(elsewhere.join("ignored/newer.txt"), b"fresh").expect("a file the carry would copy if it followed the link");
    let (code, lines) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code, REFUSED, "{lines:#?}");
    assert!(lines.join("\n").contains("is a symbolic link") && elsewhere.join("README.md").exists());
    assert!(!root.join("ignored/newer.txt").exists(), "nothing is carried from where the link points");
    remove(parent.as_path());
}

/// A failure while deleting is reported with the path and the instruction to run again;
/// the journal stays.
#[test]
fn a_deletion_that_fails_is_reported_and_keeps_the_journal() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("remove-old-fails", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    // The old tree cannot be unlinked from a parent directory this user cannot write. The command writes its claim there
    // first, so the directory becomes unwritable when the command announces the deletion.
    let mut lines: Vec<String> = Vec::new();
    let code: i32 = crate::subvolume_command::execute(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never(), &mut |text: &str| {
        lines.push(text.to_string());
        if text.starts_with("deleting ") {
            std::fs::set_permissions(parent.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o500)).expect("parent not writable");
        }
    });
    std::fs::set_permissions(parent.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("restore");
    assert_eq!(code, FAILED, "{lines:#?}");
    let text: String = lines.join("\n");
    assert!(text.contains("cannot delete") && text.contains("Run the command again"), "{text}");
    assert!(layout.journal_path.exists());
    remove(parent.as_path());
}

/// Directories of the old tree that the owner cannot write (mode 555 and 500) do not stop its deletion.
#[test]
fn the_old_tree_with_read_only_directories_is_deleted() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = migrated("remove-old-read-only", &platform);
    let layout = layout_of(root.as_path()).expect("layout");
    for path in ["ignored/closed", "ignored/closed/inner", "ignored/node_modules"] {
        let directory: PathBuf = layout.old_path.join(path);
        std::fs::create_dir_all(directory.as_path()).expect("directory");
        std::fs::write(directory.join("file"), b"x").ok();
    }
    for (path, mode) in [("ignored/closed/inner", 0o500), ("ignored/closed", 0o555), ("ignored/node_modules", 0o500)] {
        std::fs::set_permissions(layout.old_path.join(path), std::os::unix::fs::PermissionsExt::from_mode(mode)).expect("mode");
    }
    let (code, lines) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert!(!layout.old_path.exists() && !layout.journal_path.exists());
    // The late-write pass carried the read-only directories into the checkout, with their modes; the cleanup needs them writable.
    for path in ["ignored/closed/inner", "ignored/closed", "ignored/node_modules"] {
        std::fs::set_permissions(root.join(path), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("restore");
    }
    remove(parent.as_path());
}

/// A rollback on a mount without unprivileged subvolume removal empties the new subvolume even when it holds read-only
///  directories.
#[test]
fn a_rollback_empties_a_subvolume_that_holds_read_only_directories() {
    if running_as_root() {
        return;
    }
    let mut refused: PlainPlatform = PlainPlatform::new();
    refused.removal = false;
    let (parent, root) = migrated("rollback-read-only", &refused);
    std::fs::create_dir_all(root.join("ignored/closed/inner")).expect("directories");
    std::fs::write(root.join("ignored/closed/inner/file"), b"x").expect("file");
    for (path, mode) in [("ignored/closed/inner", 0o500), ("ignored/closed", 0o555)] {
        std::fs::set_permissions(root.join(path), std::os::unix::fs::PermissionsExt::from_mode(mode)).expect("mode");
    }
    let (code, lines) = run_command(&refused, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_eq!(refused.is_subvolume_root(root.as_path()), Ok(false));
    // The restored directory keeps the read-only modes it was given; the fixture cleanup needs them writable.
    for path in ["ignored/closed", "ignored/closed/inner"] {
        std::fs::set_permissions(root.join(path), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("restore");
    }
    remove(parent.as_path());
}

/// Nested subvolumes inside nested subvolumes are destroyed deepest first,
/// and the top one last,
/// whatever order the
///  directory listing returns them in.
#[test]
fn subvolumes_nested_several_levels_deep_are_destroyed_deepest_first() {
    let platform: Recording = recording(false);
    let parent: PathBuf = crate::test_support::fixture("rollback-deep-nesting");
    let top: PathBuf = parent.join("top");
    platform.create_subvolume(top.as_path()).expect("top");
    platform.create_subvolume(top.join("a").as_path()).expect("a");
    platform.create_subvolume(top.join("a/b").as_path()).expect("b");
    std::fs::create_dir_all(top.join("a/b/plain")).expect("an ordinary directory");
    platform.create_subvolume(top.join("a/b/plain/c").as_path()).expect("c");
    platform.create_subvolume(top.join("z").as_path()).expect("z");
    let layout = layout_of(parent.join("checkout").as_path()).expect("layout");
    let mut say = |_text: &str| {};
    let never: Interrupt = Interrupt::never();
    let mut run = crate::subvolume_run::Run::open(&platform, &layout, &never, &mut say).expect("run");
    remove_subvolume_tree(&mut run, top.as_path()).expect("removal");
    let destroyed: Vec<PathBuf> = platform.destroyed.borrow().clone();
    assert_eq!(destroyed.len(), 5, "{destroyed:?}");
    assert_eq!(destroyed.last(), Some(&top));
    let position = |path: PathBuf| -> usize { return destroyed.iter().position(|candidate: &PathBuf| { return *candidate == path; }).expect("destroyed"); };
    assert!(position(top.join("a/b/plain/c")) < position(top.join("a/b")), "{destroyed:?}");
    assert!(position(top.join("a/b")) < position(top.join("a")), "{destroyed:?}");
    assert!(!top.exists());
    remove(parent.as_path());
}

/// What:
///  A list larger than the plain-file reader's bound is still read,
///  so it guards the deletion instead of blocking it.
/// Why:
///   The writer had no bound and the reader refused a list over a mebibyte,
/// so the migration could write a safeguard it
///  then refused to reopen,
/// and both cleanup modes call `Run::open` before they do anything.
#[test]
fn a_list_over_the_plain_file_bound_still_guards_the_deletion() {
    if running_as_root() {
        return;
    }
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("remove-old-list-long");
    std::fs::write(root.join("ignored/secret.log"), b"secret").expect("log");
    std::fs::set_permissions(root.join("ignored/secret.log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    let mut text: String = std::fs::read_to_string(layout.unreadable_path.as_path()).expect("the list");
    text.push_str(format!("{}\n", "p".repeat(1 << 20)).as_str());
    std::fs::write(layout.unreadable_path.as_path(), text).expect("a longer list");
    let (code2, lines2) = run_command(&platform, &remove_old_arguments(false), root.as_path(), &Interrupt::never());
    assert_eq!(code2, REFUSED, "{lines2:#?}");
    let said: String = lines2.join("\n");
    assert!(said.contains("may not read") && !said.contains("larger than"), "{said}");
    remove(parent.as_path());
}

/// What:
///  A rollback refuses to delete a directory at the staged subvolume's name that is not a subvolume.
/// Why:
///   The removal fell back to deleting a plain directory when the name held no subvolume,
/// so a process that replaced
///  the staged directory with one holding user data had that data deleted with no identity check.
#[test]
fn a_rollback_does_not_delete_a_directory_that_replaced_the_staged_subvolume() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("rollback-replaced-stage");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("before-exchange", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::rename(layout.new_path.as_path(), parent.join("staged-aside")).expect("move the stage aside");
    std::fs::create_dir_all(layout.new_path.as_path()).expect("a replacement directory");
    std::fs::write(layout.new_path.join("theirs"), b"their data").expect("their file");
    let (code2, lines2) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code2, REFUSED, "{lines2:#?}");
    let said: String = lines2.join("\n");
    assert!(said.contains("not a subvolume") && said.contains("the subvolume the migration created"), "{said}");
    assert_eq!(std::fs::read(layout.new_path.join("theirs")).expect("their file"), b"their data", "their data survives");
    assert!(parent.join("staged-aside").exists(), "the staged subvolume is left where it was moved");
    remove(parent.as_path());
}

/// What:
///  A rollback preview refuses the stage that is not a subvolume,
/// in the words the real rollback uses.
/// Why:
///   The preview answered the refusals of the mode it previews,
/// but the stage-type guard lives in the rollback
///  itself,
/// so the preview promised a restore the real command refuses,
/// on state it could have read.
#[test]
fn a_rollback_preview_names_the_stage_that_is_not_a_subvolume() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("rollback-preview-stage-type");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("before-exchange", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::rename(layout.new_path.as_path(), parent.join("staged-aside")).expect("move the stage aside");
    std::fs::create_dir_all(layout.new_path.as_path()).expect("a replacement directory");
    let preview: MigrateArguments = MigrateArguments { dry_run: true, ..rollback_arguments() };
    let (preview_code, preview_lines) = run_command(&platform, &preview, root.as_path(), &Interrupt::never());
    assert_eq!(preview_code, REFUSED, "{preview_lines:#?}");
    let said: String = preview_lines.join("\n");
    assert!(said.contains("not a subvolume") && said.contains("the subvolume the migration created"), "{said}");
    assert!(!said.contains("would restore"), "a preview promises nothing the run refuses: {said}");
    let (real_code, real_lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(real_code, REFUSED, "{real_lines:#?}");
    // The real run also states the risk it would carry before it reaches the refusal; the refusal itself is the same.
    let refusals = |said_now: &[String]| -> Vec<String> {
        return said_now.iter().filter(|line: &&String| { return line.contains("refused:") || line.starts_with("  What to do:") || line.as_str() == "Nothing was changed."; }).cloned().collect();
    };
    assert_eq!(refusals(preview_lines.as_slice()), refusals(real_lines.as_slice()), "the preview says what the real run says");
    assert!(layout.new_path.exists(), "the replacement directory is untouched");
    remove(parent.as_path());
}

/// What:
///  A rollback does not delete a different subvolume placed at the stage's name.
/// Why:
///  The guard required only a subvolume root,
/// and the removal destroys whatever subvolume the name holds,
/// so a
///  subvolume another process made there,
/// holding bytes that exist nowhere else,
/// was destroyed.
/// The journal
///  records the identity of the stage it made,
/// which tells the two apart.
#[test]
fn a_rollback_does_not_delete_another_subvolume_placed_at_the_stage() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("rollback-foreign-stage");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("before-exchange", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::rename(layout.new_path.as_path(), parent.join("staged-aside")).expect("move the stage aside");
    platform.create_subvolume(layout.new_path.as_path()).expect("another subvolume at the stage's name");
    std::fs::write(layout.new_path.join("theirs"), b"their data").expect("their file");
    let (rollback_code, rollback_lines) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(rollback_code, REFUSED, "{rollback_lines:#?}");
    let said: String = rollback_lines.join("\n");
    assert!(said.contains("not the subvolume the migration created"), "{said}");
    assert_eq!(std::fs::read(layout.new_path.join("theirs")).expect("their file"), b"their data", "their data survives");
    remove(parent.as_path());
}

/// What:
///  A rollback preview of a migration stopped before the exchange is not refused,
///  because the real rollback can perform it.
/// Why:
///   The preview answered the rollback's refusals without first requiring that the migration exchanged
/// the trees,
///  so it reported the missing old tree of a stop that never had one and refused a rollback the real
///  mode finishes.
#[test]
fn a_rollback_preview_before_the_exchange_is_not_refused() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("rollback-preview-before-exchange");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("before-exchange", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(layout.new_path.exists() && !layout.old_path.exists(), "the stage exists and no old tree does");
    let preview: MigrateArguments = MigrateArguments { dry_run: true, ..rollback_arguments() };
    let (code2, lines2) = run_command(&platform, &preview, root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    let said: String = lines2.join("\n");
    assert!(said.contains("would restore") && !said.contains("refused"), "{said}");
    // The preview promised what the real mode does.
    let (code3, lines3) = run_command(&platform, &rollback_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code3, DONE, "{lines3:#?}");
    assert_original_again(&platform, root.as_path());
    remove(parent.as_path());
}
