//! Controls for the migration:
//! the happy path with a linked worktree,
//! the preview,
//! every refusal that reaches the
//!  command,
//! the resume after a simulated kill at every journal step,
//! the tolerance of unreadable files and the
//!  refusal of unreadable directories,
//! the catch-up,
//! and the index stat rewrite.
//! They run on the stand-in platform,
//! so
//!  they run on any filesystem,
//! including the container gate's overlay.

use super::*;
use crate::fs_kind::FilesystemKind;
use crate::subvolume_arguments::{MigrateArguments, MigrateMode, MIGRATE_RISK_NOTICE};
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight::layout_of;
use crate::subvolume_run::Run;
use crate::subvolume_sync_attrs::make_fifo;
use crate::subvolume_test_support::{Hooked, PlainPlatform, checkout_with_worktree, index_inode, make_checkout, migrate_arguments, run_command};
use crate::test_support::{fixture, git_text, remove};
use std::os::unix::fs::MetadataExt;

/// The exit status the command gives a finished,
/// previewed or already finished migration.
const DONE: i32 = 0;

/// The exit status of a refusal.
const REFUSED: i32 = 1;

/// The exit status of a failure after the journal began.
const FAILED: i32 = 2;

/// Whether the tests run as root,
/// which can read every file and so cannot make an unreadable one.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// The files whose bytes,
/// modes and links the migrated tree must carry,
/// compared with the old tree.
fn assert_same_tree(original: &Path, copy: &Path) {
    for path in ["README.md", "src/main.rs", "src/deep/er/leaf.txt", "empty", "script.sh", ".gitignore", "ignored/node_modules/pkg/index.js"] {
        assert_eq!(std::fs::read(original.join(path)).expect("original"), std::fs::read(copy.join(path)).expect("copy"), "{path}");
        let (left, right) = (std::fs::metadata(original.join(path)).expect("original"), std::fs::metadata(copy.join(path)).expect("copy"));
        assert_eq!(left.mode(), right.mode(), "{path} mode");
        assert_eq!((left.mtime(), left.mtime_nsec()), (right.mtime(), right.mtime_nsec()), "{path} modification time");
    }
    assert_eq!(std::fs::read_link(copy.join("link")).expect("link"), PathBuf::from("src/main.rs"));
    assert_eq!(std::fs::metadata(copy.join("src/main.rs")).expect("main").ino(), std::fs::metadata(copy.join("ignored/main-link.rs")).expect("link").ino(), "hard links stay one file");
}

/// A migrated tree:
/// a subvolume,
/// Git-clean,
/// with its index stat fields refreshed,
/// and no lock left.
fn assert_migrated(platform: &PlainPlatform, root: &Path, old: &Path) {
    assert_eq!(platform.is_subvolume_root(root), Ok(true));
    assert_same_tree(old, root);
    // Before the first status, which refreshes the index file itself and would hide a missing rewrite.
    for path in ["README.md", "src/main.rs", "script.sh"] {
        assert_eq!(index_inode(root, path), std::fs::metadata(root.join(path)).expect("file").ino(), "{path} index inode");
    }
    assert_eq!(git_text(root, &["status", "--porcelain"]), "", "the first status finds nothing dirty");
    assert!(!root.join(".git/index.lock").exists());
    assert!(!old.join(".git/index.lock").exists());
}

/// The happy path:
/// the checkout becomes a subvolume with its worktree nested,
/// the old tree is kept,
/// Git is clean.
#[test]
fn a_checkout_with_a_linked_worktree_is_migrated_and_stays_clean() {
    let (parent, root) = checkout_with_worktree("migrate-happy");
    let platform: PlainPlatform = PlainPlatform::new();
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert_migrated(&platform, root.as_path(), layout.old_path.as_path());
    assert_eq!(platform.is_subvolume_root(root.join("wt").as_path()), Ok(true), "the linked worktree is a nested subvolume");
    assert_eq!(git_text(root.join("wt").as_path(), &["status", "--porcelain"]), "");
    assert!(git_text(root.as_path(), &["worktree", "list"]).contains(root.join("wt").to_str().expect("path")));
    assert!(layout.old_path.join("README.md").exists(), "the old tree is kept");
    assert!(layout.journal_path.exists() && !layout.new_path.exists());
    let text: String = lines.join("\n");
    for expected in ["first delta", "last delta (under the lock)", "catch-up", "index stat rewrite", "is now a btrfs subvolume", "--remove-old", "--rollback"] {
        assert!(text.contains(expected), "{expected} missing from {text}");
    }
    remove(parent.as_path());
}

/// The preview creates nothing,
/// not even a journal,
/// and says how to run it for real.
#[test]
fn the_dry_run_changes_nothing() {
    let (parent, root) = checkout_with_worktree("migrate-dry");
    let platform: PlainPlatform = PlainPlatform::new();
    let arguments: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(!layout.new_path.exists() && !layout.old_path.exists() && !layout.journal_path.exists());
    assert_eq!(platform.is_subvolume_root(root.as_path()), Ok(false));
    let text: String = lines.join("\n");
    assert!(text.contains("dry run complete") && text.contains("linked worktrees inside the checkout that become nested subvolumes: 1"), "{text}");
    remove(parent.as_path());
}

/// A filesystem that is not btrfs is refused with the cause and the remedies,
/// before anything is created.
#[test]
fn a_filesystem_other_than_btrfs_is_refused_with_its_name() {
    let (parent, root) = checkout_with_worktree("migrate-not-btrfs");
    for (kind, word) in [(FilesystemKind::Overlayfs, "overlayfs"), (FilesystemKind::Other, "a filesystem other than btrfs")] {
        let mut platform: PlainPlatform = PlainPlatform::new();
        platform.kind = kind;
        let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(code, REFUSED);
        let text: String = lines.join("\n");
        assert!(text.contains(word) && text.contains("What to do:") && text.contains("Nothing was changed."), "{text}");
    }
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(!layout.new_path.exists() && !layout.journal_path.exists());
    remove(parent.as_path());
}

/// A rerun after a finished migration,
/// and a plain subvolume with no journal,
/// both report "nothing to do" with status 0.
#[test]
fn an_already_migrated_checkout_is_reported_and_left_alone() {
    let (parent, root) = checkout_with_worktree("migrate-again");
    let platform: PlainPlatform = PlainPlatform::new();
    assert_eq!(run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never()).0, DONE);
    let (again_code, again_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(again_code, DONE);
    let text: String = again_lines.join("\n");
    assert!(text.contains("already a btrfs subvolume") && text.contains("--remove-old"), "{text}");
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::remove_file(layout.journal_path.as_path()).expect("forget the journal");
    let (forgotten_code, forgotten_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(forgotten_code, DONE, "{forgotten_lines:#?}");
    assert!(forgotten_lines.join("\n").contains("already a btrfs subvolume"));
    remove(parent.as_path());
}

/// A kill at any step is survived:
/// the rerun resumes and ends in the same migrated state.
/// Each point is a place a
///  real kill can land;
/// the hook stops the run exactly there,
/// and nothing but the disk and the journal survives.
#[test]
fn a_rerun_after_a_simulated_kill_at_every_step_completes() {
    let points: [(&str, u64); 12] = [
        ("after-create", 0),
        ("after-nested", 0),
        ("copy-entry", 0),
        ("copy-entry", 5),
        ("after-copy", 0),
        ("after-delta1", 0),
        ("after-lock", 0),
        ("before-exchange", 0),
        ("after-exchange", 0),
        ("after-rename", 0),
        ("after-catchup", 0),
        ("after-indexes", 0),
    ];
    // A kill point shares nothing with another: each has its own fixture, its own platform and its own journal, so the
    //  points walk side by side and one point's waits for the disk are another point's chance to work.
    // What decides this control's own time is the tree pass of each point, which is filesystem latency rather than work
    //  (`AGENTS.md` rule `SUF`):
    // measured on 2026-11-09: the twelve points one after another took 384 s with the host's load average at 119 and
    //  110 s once the durability switch of the stand-in reached the claim and the index rewrites, and the same points
    //  side by side took 25.2 s at a load average of 115.
    let mut failed: Vec<&str> = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<(&str, std::thread::ScopedJoinHandle<'_, ()>)> = points
            .iter()
            .enumerate()
            .map(|(index, killed_at)| {
                let (stop_at, after): (&str, u64) = *killed_at;
                let name: String = format!("migrate-kill-{index}");
                return (stop_at, scope.spawn(move || { return kill_point_at(name.as_str(), stop_at, after); }));
            })
            .collect();
        for (point, handle) in handles {
            if handle.join().is_err() {
                failed.push(point);
            }
        }
    });
    assert!(failed.is_empty(), "the kill points that failed: {failed:?}");
}

/// One kill point:
/// a fresh checkout,
/// the run stopped at `point`,
/// and the rerun that resumes it and ends migrated.
fn kill_point_at(name: &str, point: &str, after: u64) {
    let (parent, root) = checkout_with_worktree(name);
    let platform: PlainPlatform = PlainPlatform::new();
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at(point, after));
    assert_eq!(code, FAILED, "{point}: {lines:#?}");
    assert!(lines.join("\n").contains("Run the command again to resume"), "{point}");
    let (resumed_code, resumed_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(resumed_code, DONE, "{point}: {resumed_lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert_migrated(&platform, root.as_path(), layout.old_path.as_path());
    assert_eq!(platform.is_subvolume_root(root.join("wt").as_path()), Ok(true), "{point}: nested worktree");
    assert!(!layout.new_path.exists(), "{point}: the new path is renamed away");
    remove(parent.as_path());
}

/// Files this user may not read are left in the old tree and listed;
/// the run still succeeds.
#[test]
fn unreadable_files_stay_in_the_old_tree_and_are_listed() {
    if running_as_root() {
        return;
    }
    let (parent, root) = checkout_with_worktree("migrate-unreadable-file");
    std::fs::write(root.join("ignored/root-owned.log"), b"secret").expect("log");
    std::fs::set_permissions(root.join("ignored/root-owned.log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable");
    std::fs::write(root.join("wt/ignored-log"), b"secret").expect("worktree log");
    std::fs::set_permissions(root.join("wt/ignored-log"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unreadable");
    let platform: PlainPlatform = PlainPlatform::new();
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(!root.join("ignored/root-owned.log").exists() && layout.old_path.join("ignored/root-owned.log").exists());
    let list: String = std::fs::read_to_string(layout.unreadable_path.as_path()).expect("the list");
    assert!(list.lines().any(|line| { return line == "ignored/root-owned.log"; }) && list.lines().any(|line| { return line == "wt/ignored-log"; }), "{list}");
    let text: String = lines.join("\n");
    assert!(text.contains("may not read") && text.contains("exist only in the old tree"), "{text}");
    assert_eq!(git_text(root.as_path(), &["status", "--porcelain"]), "");
    remove(parent.as_path());
}

/// A directory this user cannot list stops the run before the exchange,
/// naming it,
/// and nothing is lost.
#[test]
fn an_unreadable_directory_stops_the_run_before_the_exchange() {
    if running_as_root() {
        return;
    }
    let (parent, root) = checkout_with_worktree("migrate-unreadable-dir");
    std::fs::create_dir(root.join("ignored/locked")).expect("directory");
    std::fs::write(root.join("ignored/locked/file"), b"x").expect("file");
    std::fs::set_permissions(root.join("ignored/locked"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("unlistable");
    let platform: PlainPlatform = PlainPlatform::new();
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    std::fs::set_permissions(root.join("ignored/locked"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("restore");
    assert_eq!(code, FAILED, "{lines:#?}");
    let text: String = lines.join("\n");
    assert!(text.contains("ignored/locked") && text.contains("cannot be listed or searched") && text.contains("--rollback"), "{text}");
    assert_eq!(platform.is_subvolume_root(root.as_path()), Ok(false), "the checkout was not exchanged");
    assert!(root.join("README.md").exists());
    remove(parent.as_path());
}

/// A file written to the old tree after the exchange is carried over by the catch-up on the rerun.
#[test]
fn a_write_to_the_old_tree_after_the_exchange_is_carried_over() {
    let (parent, root) = checkout_with_worktree("migrate-catchup");
    let platform: PlainPlatform = PlainPlatform::new();
    let (stopped_code, _stopped_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("after-rename", 0));
    assert_eq!(stopped_code, FAILED);
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::write(layout.old_path.join("late.txt"), b"written by a straggler\n").expect("late file");
    let (resumed_code, resumed_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(resumed_code, DONE, "{resumed_lines:#?}");
    assert_eq!(std::fs::read(root.join("late.txt")).expect("carried over"), b"written by a straggler\n");
    remove(parent.as_path());
}

/// Without the index rewrite the copied index keeps the old inodes;
/// with it they match.
/// The first proves that the
///  second control can fail.
#[test]
fn the_index_rewrite_can_be_switched_off_and_then_the_inodes_differ() {
    let (parent, root) = checkout_with_worktree("migrate-no-refresh");
    let platform: PlainPlatform = PlainPlatform::new();
    let arguments: MigrateArguments = MigrateArguments { refresh_indexes: false, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_ne!(index_inode(root.as_path(), "README.md"), std::fs::metadata(root.join("README.md")).expect("file").ino());
    remove(parent.as_path());
}

/// Without the nested-worktree switch the linked worktree stays an ordinary directory inside the new tree.
#[test]
fn the_nested_worktree_switch_keeps_worktrees_ordinary() {
    let (parent, root) = checkout_with_worktree("migrate-no-nested");
    let platform: PlainPlatform = PlainPlatform::new();
    let arguments: MigrateArguments = MigrateArguments { nested_worktrees: false, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_eq!(platform.is_subvolume_root(root.as_path()), Ok(true));
    assert_eq!(platform.is_subvolume_root(root.join("wt").as_path()), Ok(false));
    assert!(root.join("wt/README.md").exists());
    remove(parent.as_path());
}

/// A process whose working directory is inside the checkout is named in a warning.
#[test]
fn a_process_inside_the_checkout_is_named_in_the_warning() {
    let (parent, root) = checkout_with_worktree("migrate-process-warning");
    let mut child: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("60").current_dir(root.as_path())).expect("sleep");
    let platform: PlainPlatform = PlainPlatform::new();
    let arguments: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
    let pid: String = child.id().to_string();
    child.kill().expect("stop the sleeper");
    child.wait().expect("reap the sleeper");
    assert_eq!(code, DONE);
    let text: String = lines.join("\n");
    assert!(text.contains("WARNING") && text.contains(format!("pid {pid} (sleep)").as_str()), "{text}");
    remove(parent.as_path());
}

/// A platform that makes a foreign Git command's lock file appear at `lock` during the first copy,
/// after the preflight.
fn lock_taker(lock: PathBuf) -> Hooked {
    return Hooked::new(Box::new(move |call: u32| {
        if call == 0 {
            std::fs::write(lock.as_path(), b"git add is writing here\n").expect("the foreign lock");
        }
    }));
}

/// A lock taken by another Git command after the preflight is never taken for the migration's own:
/// the migration stops
///  before the exchange,
/// leaves that lock alone,
/// and finishes once the other command is done.
#[test]
fn an_index_lock_taken_by_another_git_command_after_the_preflight_stops_the_migration() {
    for relative in [".git/index.lock", ".git/worktrees/wt/index.lock"] {
        let (parent, root) = checkout_with_worktree(format!("migrate-foreign-lock-{}", relative.replace('/', "-")).as_str());
        let platform: Hooked = lock_taker(root.join(relative));
        let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(code, FAILED, "{relative}: {lines:#?}");
        let text: String = lines.join("\n");
        assert!(text.contains("is not this migration's lock") && text.contains("index.lock"), "{relative}: {text}");
        let layout = layout_of(root.as_path()).expect("layout");
        assert_eq!(platform.inner.is_subvolume_root(root.as_path()), Ok(false), "{relative}: the trees were not exchanged");
        assert!(!layout.old_path.exists(), "{relative}");
        assert_eq!(std::fs::read(root.join(relative)).expect("foreign lock kept"), b"git add is writing here\n", "{relative}");
        let (refused, _lines) = run_command(&platform.inner, &migrate_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(refused, REFUSED, "{relative}: the preflight refuses while the other command's lock stands");
        std::fs::remove_file(root.join(relative)).expect("the other command finishes");
        let (done, lines2) = run_command(&platform.inner, &migrate_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(done, DONE, "{relative}: {lines2:#?}");
        assert_migrated(&platform.inner, root.as_path(), layout.old_path.as_path());
        remove(parent.as_path());
    }
}

/// The migration's locks stand in the new checkout (the main index and the linked worktree's) from the exchange until
///  the indexes are rewritten,
/// so no Git command reads or writes a tree that is being completed;
/// none is left after.
#[test]
fn the_new_checkout_holds_the_index_locks_until_the_indexes_are_rewritten() {
    let main: &str = ".git/index.lock";
    let linked: &str = ".git/worktrees/wt/index.lock";
    for point in ["after-exchange", "after-catchup", "after-indexes"] {
        let platform: PlainPlatform = PlainPlatform::new();
        let (parent, root) = checkout_with_worktree(format!("migrate-held-{point}").as_str());
        let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at(point, 0));
        assert_eq!(code, FAILED, "{point}: {lines:#?}");
        let layout = layout_of(root.as_path()).expect("layout");
        for lock in [main, linked] {
            let content: Vec<u8> = std::fs::read(root.join(lock)).unwrap_or_default();
            assert!(content.starts_with(b"monochromatic-git-policy-cli migrate-to-subvolume"), "{point}: {lock} is held by the migration in the new checkout");
        }
        if point != "after-exchange" {
            assert!(!layout.old_path.join(main).exists() && !layout.old_path.join(linked).exists(), "{point}: the old tree's locks are dropped");
        }
        let (code2, lines2) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
        assert_eq!(code2, DONE, "{point}: {lines2:#?}");
        assert_migrated(&platform, root.as_path(), layout.old_path.as_path());
        assert!(!root.join(linked).exists() && !layout.old_path.join(linked).exists(), "{point}");
        remove(parent.as_path());
    }
}

/// The migration holds the linked worktrees' index locks too,
/// in the old tree before its last copy and in the new tree
///  before the exchange.
#[test]
fn the_linked_worktrees_index_locks_are_held_in_both_trees_before_the_exchange() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("migrate-worktree-lock");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("before-exchange", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    for tree in [root.as_path(), layout.new_path.as_path()] {
        for lock in [".git/index.lock", ".git/worktrees/wt/index.lock"] {
            assert!(tree.join(lock).exists(), "{} holds {lock}", tree.display());
        }
    }
    remove(parent.as_path());
}

/// How often a notice stands among the lines a run said.
fn said(lines: &[String], notice: &str) -> usize {
    return lines.iter().filter(|line: &&String| { return line.as_str() == notice; }).count();
}

/// A run that changes the checkout says the risk once,
/// before the first line about the checkout,
/// and never the conversion's own notice:
/// one command prints one notice.
#[test]
fn a_run_says_the_risk_once_before_it_changes_anything() {
    let (parent, root) = checkout_with_worktree("migrate-risk");
    let platform: PlainPlatform = PlainPlatform::new();
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_eq!(lines.first().map(String::as_str), Some(MIGRATE_RISK_NOTICE), "before anything is changed: {lines:#?}");
    assert_eq!(said(lines.as_slice(), MIGRATE_RISK_NOTICE), 1, "said once: {lines:#?}");
    assert_eq!(said(lines.as_slice(), crate::worktree_convert_arguments::RISK_NOTICE), 0, "the conversion's notice is not this command's: {lines:#?}");
    remove(parent.as_path());
}

/// The preview and a refusal before the first step say no risk,
/// because they change nothing.
#[test]
fn a_preview_and_a_refusal_say_no_risk() {
    let (parent, root) = checkout_with_worktree("migrate-risk-none");
    let platform: PlainPlatform = PlainPlatform::new();
    let preview: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let (preview_code, preview_lines) = run_command(&platform, &preview, root.as_path(), &Interrupt::never());
    assert_eq!(preview_code, DONE, "{preview_lines:#?}");
    assert_eq!(said(preview_lines.as_slice(), MIGRATE_RISK_NOTICE), 0, "a preview changes nothing: {preview_lines:#?}");
    let mut unsuitable: PlainPlatform = PlainPlatform::new();
    unsuitable.kind = FilesystemKind::Overlayfs;
    let (refused_code, refused_lines) = run_command(&unsuitable, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(refused_code, REFUSED, "{refused_lines:#?}");
    assert_eq!(said(refused_lines.as_slice(), MIGRATE_RISK_NOTICE), 0, "a refusal leaves the checkout alone: {refused_lines:#?}");
    remove(parent.as_path());
}

/// The two modes that delete a tree each say the risk once,
/// and neither repeats the conversion's notice.
#[test]
fn the_deleting_modes_say_the_risk_once() {
    let (parent, root) = checkout_with_worktree("migrate-risk-delete");
    let platform: PlainPlatform = PlainPlatform::new();
    assert_eq!(run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never()).0, DONE);
    let rollback: MigrateArguments = MigrateArguments { mode: MigrateMode::Rollback, ..migrate_arguments() };
    let (rollback_code, rollback_lines) = run_command(&platform, &rollback, root.as_path(), &Interrupt::never());
    assert_eq!(rollback_code, DONE, "{rollback_lines:#?}");
    assert_eq!(said(rollback_lines.as_slice(), MIGRATE_RISK_NOTICE), 1, "said once by a rollback: {rollback_lines:#?}");
    assert_eq!(said(rollback_lines.as_slice(), crate::worktree_convert_arguments::RISK_NOTICE), 0, "{rollback_lines:#?}");
    assert_eq!(run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never()).0, DONE);
    let remove_old: MigrateArguments = MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() };
    let (remove_code, remove_lines) = run_command(&platform, &remove_old, root.as_path(), &Interrupt::never());
    assert_eq!(remove_code, DONE, "{remove_lines:#?}");
    assert_eq!(said(remove_lines.as_slice(), MIGRATE_RISK_NOTICE), 1, "said once by a removal of the old tree: {remove_lines:#?}");
    assert_eq!(said(remove_lines.as_slice(), crate::worktree_convert_arguments::RISK_NOTICE), 0, "{remove_lines:#?}");
    remove(parent.as_path());
}

/// What:
///  A named pipe where the unreadable-file list belongs ends the run at once,
///  naming the file,
/// instead of
///  waiting for a writer that never comes.
/// Why:
///   The list is what keeps files this user may not read from being deleted with the old tree,
/// so it may
///  never be counted as empty by a read that waits;
/// and the command that waits owns the checkout.
#[test]
fn a_named_pipe_where_the_unreadable_list_belongs_ends_the_run_at_once() {
    let parent: PathBuf = fixture("migrate-unreadable-fifo");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    make_fifo(layout.unreadable_path.as_path(), 0o600).expect("named pipe");
    let read = crate::test_support::finished_within(move || {
        let inside: Layout = layout_of(root.as_path()).expect("layout");
        let platform: PlainPlatform = PlainPlatform::new();
        let mut say = |_text: &str| {};
        return match Run::open(&platform, &inside, &Interrupt::never(), &mut say) {
            Ok(_) => String::from("opened"),
            Err(error) => error.to_string(),
        };
    });
    let Some(message) = read else { panic!("reading the list did not finish within five seconds") };
    assert!(message.contains("not a plain file") && message.contains("unreadable"), "{message}");
    remove(parent.as_path());
}

/// What:
///  A resume refuses a subvolume at the stage's name that is not the one the journal recorded creating,
///  and copies nothing into it.
/// Why:
///   The creation step is skipped when the journal records it as done,
/// and nothing compared the subvolume at that
///  name with the identity the journal recorded;
/// the initial copy ran into the other subvolume and the deleting
///  delta then removed the files it held,
/// which exist nowhere else.
#[test]
fn a_resume_refuses_a_stage_that_is_not_the_subvolume_the_journal_recorded() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("migrate-foreign-stage");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("after-create", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    let aside: PathBuf = parent.join("staged-aside");
    std::fs::rename(layout.new_path.as_path(), aside.as_path()).expect("move the stage aside");
    platform.create_subvolume(layout.new_path.as_path()).expect("another subvolume at the stage's name");
    std::fs::write(layout.new_path.join("theirs"), b"their data").expect("their file");
    let (resumed_code, resumed_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(resumed_code, REFUSED, "{resumed_lines:#?}");
    let said: String = resumed_lines.join("\n");
    assert!(said.contains("not the subvolume this migration created"), "{said}");
    assert_eq!(std::fs::read(layout.new_path.join("theirs")).expect("their file"), b"their data", "their data survives");
    assert!(aside.exists(), "the stage this migration made is left where it was moved");
    remove(parent.as_path());
}

/// Record the creation step as begun,
/// and drop the identity of the stage,
/// as a journal a kill left before that write,
/// or one an earlier release wrote,
/// has it.
fn record_the_creation_without_the_stage(journal: &Path) {
    let text: String = std::fs::read_to_string(journal).expect("the journal");
    let mut value: serde_json::Value = serde_json::from_str(text.as_str()).expect("a journal");
    let document = value.as_object_mut().expect("a journal is an object");
    let steps = document.get_mut("steps").and_then(serde_json::Value::as_object_mut).expect("a journal records steps");
    let create = steps.get_mut("create").and_then(serde_json::Value::as_object_mut).expect("the journal records the creation");
    create.insert(String::from("state"), serde_json::Value::String(String::from("started")));
    let values = document.get_mut("values").and_then(serde_json::Value::as_object_mut).expect("the journal records values");
    let identity = values.remove(crate::subvolume_journal::STAGE_ROOT);
    assert!(identity.is_some(), "the control starts from a journal that records the stage");
    if values.is_empty() {
        document.remove("values");
    }
    std::fs::write(journal, serde_json::to_string_pretty(&value).expect("a journal")).expect("write the journal");
}

/// What:
///  A resume adopts an empty subvolume at the stage's name when the journal records no identity for it.
/// Why:
///   The identity of the stage is a write of its own before the step is marked done,
/// so a kill before it leaves a journal that records
///  the creation and no identity;
/// a copy into an empty subvolume loses nothing,
/// so that state is finished rather than refused,
/// and every
///  other state at that name is left alone.
#[test]
fn a_resume_adopts_an_empty_stage_the_journal_records_no_identity_for() {
    let platform: PlainPlatform = PlainPlatform::new();
    let (parent, root) = checkout_with_worktree("migrate-unrecorded-stage");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("after-create", 0));
    assert_eq!(code, FAILED, "{lines:#?}");
    let layout = layout_of(root.as_path()).expect("layout");
    record_the_creation_without_the_stage(layout.journal_path.as_path());
    let (resumed_code, resumed_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(resumed_code, DONE, "{resumed_lines:#?}");
    assert_migrated(&platform, root.as_path(), layout.old_path.as_path());
    remove(parent.as_path());
}
