//! Controls that run the migration on the real platform:
//! the btrfs ioctls,
//! `FICLONE` and the kernel's exchange on a
//!  real btrfs filesystem.
//! Each needs a btrfs fixture root;
//! on the container gate (an overlay) they print one line each
//!  and the run states that this is known (`fs_btrfs_test_support`),
//! and the host task `native:test:host:btrfs` runs them.

use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::fs_kind::BTRFS_SUBVOLUME_ROOT_INODE;
use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::{LinuxPlatform, Platform};
use crate::subvolume_preflight::{Layout, layout_of};
use crate::subvolume_test_support::{checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::{git_text, remove};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The exit status of a finished run.
const DONE: i32 = 0;

/// The exit status of a failure after the journal began.
const FAILED: i32 = 2;

/// A checkout with a linked worktree inside a fresh btrfs fixture,
/// or `None` after the loud report.
fn btrfs_checkout(name: &str) -> Option<(PathBuf, PathBuf)> {
    let parent: PathBuf = btrfs_fixture(name)?;
    remove(parent.as_path());
    return Some(checkout_with_worktree(name));
}

/// The inode of a path,
/// which is 256 for the root of every btrfs subvolume.
fn inode(path: &Path) -> u64 {
    return std::fs::metadata(path).expect("metadata").ino();
}

/// The clone count printed by a pass line such as `copy: 12 entries, 6 files copied (6 cloned), ...`.
fn cloned_in(lines: &[String], label: &str) -> u64 {
    let line: &String = lines.iter().find(|candidate: &&String| { return candidate.starts_with(label); }).expect("the pass line");
    let before: &str = line.split(" cloned)").next().expect("a clone count");
    return before.rsplit('(').next().expect("a number").parse::<u64>().expect("a clone count");
}

/// The migration on the real platform:
/// the root and the nested worktree are subvolume roots (inode 256),
/// file data was
///  cloned,
/// Git is clean and the old tree is kept.
#[test]
fn a_real_migration_makes_subvolumes_and_clones_file_data() {
    let Some((parent, root)) = btrfs_checkout("btrfs-migrate") else { return };
    let (code, lines) = run_command(&LinuxPlatform::default(), &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    assert_eq!(inode(root.as_path()), BTRFS_SUBVOLUME_ROOT_INODE);
    assert_eq!(inode(root.join("wt").as_path()), BTRFS_SUBVOLUME_ROOT_INODE, "the linked worktree is a nested subvolume");
    assert_ne!(inode(layout.old_path.as_path()), BTRFS_SUBVOLUME_ROOT_INODE, "the old tree is an ordinary directory");
    assert!(cloned_in(lines.as_slice(), "copy:") > 0, "{lines:#?}");
    assert_eq!(git_text(root.as_path(), &["status", "--porcelain"]), "");
    assert_eq!(git_text(root.join("wt").as_path(), &["status", "--porcelain"]), "");
    assert!(layout.old_path.join("README.md").exists() && !layout.new_path.exists());
    remove(parent.as_path());
}

/// What:
///  A migration killed at `point` and the migration that follows it,
///  in a checkout of this control's own.
/// Why:
///   The kill is taken on the real platform,
/// so the point is two real migrations,
/// each copying the tree into
///  subvolumes and rewriting the index:
/// measured on 2026-11-09 on this host,
/// one point took 96.7 s and another 175.0 s while the
///  disk was busy,
/// and the four points in one control took 271.8 s before its third point was reached,
/// which is the runner's
///  per-test limit (`slow-timeout` 60 s times `terminate-after` 5 in `.config/nextest.toml`).
///  A control per point keeps each one's cost to its own two migrations,
/// and no limit is raised for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [code] = migrate(root, { interruptAt: point }); expect(code).toBe(FAILED);
/// const [code2] = migrate(root, {}); expect(code2).toBe(DONE);
/// ```
fn a_kill_at(point: &str) {
    let Some((parent, root)) = btrfs_checkout(format!("btrfs-resume-{point}").as_str()) else { return };
    let (code, lines) = run_command(&LinuxPlatform::default(), &migrate_arguments(), root.as_path(), &Interrupt::at(point, 0));
    assert_eq!(code, FAILED, "{point}: {lines:#?}");
    let (code2, lines2) = run_command(&LinuxPlatform::default(), &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{point}: {lines2:#?}");
    assert_eq!(inode(root.as_path()), BTRFS_SUBVOLUME_ROOT_INODE, "{point}");
    assert_eq!(git_text(root.as_path(), &["status", "--porcelain"]), "", "{point}");
    assert!(!root.join(".git/index.lock").exists(), "{point}");
    remove(parent.as_path());
}

/// A kill after the copy of the tree is survived on the real platform.
#[test]
fn a_real_migration_resumes_after_a_kill_after_the_copy() {
    a_kill_at("after-copy");
}

/// A kill after the index lock is taken is survived on the real platform.
#[test]
fn a_real_migration_resumes_after_a_kill_while_the_lock_is_held() {
    a_kill_at("after-lock");
}

/// A kill after the exchange of the trees is survived on the real platform.
#[test]
fn a_real_migration_resumes_after_a_kill_after_the_exchange() {
    a_kill_at("after-exchange");
}

/// A kill after the catch-up between the two trees is survived on the real platform.
#[test]
fn a_real_migration_resumes_after_a_kill_after_the_catchup() {
    a_kill_at("after-catchup");
}

/// The rollback on the real platform restores an ordinary directory and removes both subvolumes,
/// by the destroy call where
///  the mount allows it and by emptying and `rmdir` where it does not;
/// the test checks whichever applies on this mount.
#[test]
fn a_real_rollback_removes_the_subvolumes_whatever_the_mount_option_is() {
    let Some((parent, root)) = btrfs_checkout("btrfs-rollback") else { return };
    let (code, lines) = run_command(&LinuxPlatform::default(), &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let allowed: bool = removal_allowed(root.as_path());
    let arguments: MigrateArguments = MigrateArguments { mode: MigrateMode::Rollback, ..migrate_arguments() };
    let (code2, lines2) = run_command(&LinuxPlatform::default(), &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "removal allowed: {allowed}: {lines2:#?}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    assert_ne!(inode(root.as_path()), BTRFS_SUBVOLUME_ROOT_INODE);
    assert!(!layout.new_path.exists() && !layout.old_path.exists() && !layout.journal_path.exists());
    assert_eq!(git_text(root.as_path(), &["status", "--porcelain"]), "");
    assert_eq!(LinuxPlatform::default().is_subvolume_root(root.join("wt").as_path()), Ok(false));
    remove(parent.as_path());
}

/// The deletion of the old tree on the real platform leaves the subvolume checkout and its nested worktree alone.
#[test]
fn a_real_deletion_of_the_old_tree_keeps_the_subvolumes() {
    let Some((parent, root)) = btrfs_checkout("btrfs-remove-old") else { return };
    let (code, lines) = run_command(&LinuxPlatform::default(), &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let arguments: MigrateArguments = MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() };
    let (code2, lines2) = run_command(&LinuxPlatform::default(), &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code2, DONE, "{lines2:#?}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    assert!(!layout.old_path.exists() && !layout.journal_path.exists());
    assert_eq!((inode(root.as_path()), inode(root.join("wt").as_path())), (BTRFS_SUBVOLUME_ROOT_INODE, BTRFS_SUBVOLUME_ROOT_INODE));
    assert_eq!(git_text(root.as_path(), &["status", "--porcelain"]), "");
    remove(parent.as_path());
}

/// Two names of one file,
/// one inside a linked worktree and one outside it,
/// are valid before the migration and cannot be
///  linked afterwards (the worktree is a nested subvolume,
/// and a hard link cannot cross a subvolume):
/// the migration
///  copies the second name instead of failing,
/// and both names keep the content.
#[test]
fn a_hard_link_across_a_future_subvolume_boundary_becomes_a_copy_on_a_real_migration() {
    let Some((parent, root)) = btrfs_checkout("btrfs-hard-link-boundary") else { return };
    std::fs::hard_link(root.join("README.md"), root.join("wt/second-name-of-readme")).expect("a link into the linked worktree");
    let (code, lines) = run_command(&LinuxPlatform::default(), &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_eq!(inode(root.join("wt").as_path()), BTRFS_SUBVOLUME_ROOT_INODE);
    for name in ["README.md", "wt/second-name-of-readme"] {
        assert_eq!(std::fs::read(root.join(name)).expect("content"), b"readme\n", "{name}");
    }
    let ino = |name: &str| -> u64 { return std::fs::metadata(root.join(name)).expect("metadata").ino(); };
    assert_ne!(ino("README.md"), ino("wt/second-name-of-readme"), "the two names are separate files in two subvolumes");
    remove(parent.as_path());
}

/// The preview on the real platform reports the state of the mount option and changes nothing.
#[test]
fn a_real_preview_reports_the_removal_option_and_changes_nothing() {
    let Some((parent, root)) = btrfs_checkout("btrfs-preview") else { return };
    let arguments: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let (code, lines) = run_command(&LinuxPlatform::default(), &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let text: String = lines.join("\n");
    let expected: &str = if removal_allowed(root.as_path()) { "user_subvol_rm_allowed: active" } else { "user_subvol_rm_allowed: absent" };
    assert!(text.contains(expected) && text.contains("dry run complete"), "{text}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    assert!(!layout.new_path.exists() && !layout.journal_path.exists());
    assert_ne!(inode(root.as_path()), BTRFS_SUBVOLUME_ROOT_INODE);
    remove(parent.as_path());
}
