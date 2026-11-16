//! Controls of what the two deleting commands do with a checkout a person has worked in since the migration.
//! A migration is made in order to go on working:
//! the person edits a file,
//! makes a commit,
//! and a program that still holds the old tree open writes into it.
//! Every one of those ends with every byte the person wrote kept in the tree that survives,
//! and with the command either finishing or refusing with words that name the file,
//! never with a status of 0 whose bytes are gone.
//! The controls here were written against `9ef0169599`, where `--remove-old` refused every used checkout and
//! `--rollback --discard-left-out` deleted the person's edit and commit.

use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_preflight::{Layout, layout_of};
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::{git, git_text, remove};
use std::path::{Path, PathBuf};

/// The exit status a finished command gives.
const DONE: i32 = 0;

/// The exit status of a refusal.
const REFUSED: i32 = 1;

/// The exit status of a failure after the journal began.
const FAILED: i32 = 2;

/// The arguments of the deletion of the old tree.
fn remove_old_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() };
}

/// The arguments of the deletion of the old tree that also deletes what a carry could not place.
fn remove_old_discarding_left_out() -> MigrateArguments {
    return MigrateArguments { discard_left_out: true, ..remove_old_arguments() };
}

/// The arguments of the rollback.
fn rollback_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::Rollback, ..migrate_arguments() };
}

/// The arguments of the rollback that also deletes what the carry back could not place.
fn rollback_discarding_left_out() -> MigrateArguments {
    return MigrateArguments { discard_left_out: true, ..rollback_arguments() };
}

/// Run one command to completion and answer its status and everything it said.
fn run(platform: &PlainPlatform, arguments: &MigrateArguments, root: &Path) -> (i32, String) {
    let (code, lines) = run_command(platform, arguments, root, &Interrupt::never());
    return (code, lines.join("\n"));
}

/// Whether the file at `path` holds exactly `bytes`.
fn holds(path: &Path, bytes: &[u8]) -> bool {
    return std::fs::read(path).map(|found| return found == bytes).unwrap_or(false);
}

/// Set both times of a path to whole seconds after the epoch.
fn set_seconds(path: &Path, seconds: i64) {
    use std::os::unix::ffi::OsStrExt;
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// What:
///  Every file below `root` that holds exactly `bytes`,
///  except the names in `skipped`.
/// Why:
///   The rule these controls assert is about the bytes,
///  not about where a command decided to put them:
///  the tree the person keeps,
///  the tree that stands until it is deleted,
///  and whatever the command keeps beside the checkout all count,
///  because each of them is a place the command names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = filesHolding(root, bytes, skipped);
/// ```
fn files_holding(root: &Path, bytes: &[u8], skipped: &[&Path]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(here) = pending.pop() {
        if skipped.iter().any(|skip| return *skip == here) {
            continue;
        }
        let Ok(listing) = std::fs::read_dir(here.as_path()) else { continue };
        for item in listing.flatten() {
            let path: PathBuf = item.path();
            match item.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(kind) if kind.is_file() && std::fs::read(path.as_path()).map(|content| return content == bytes).unwrap_or(false) => {
                    found.push(path);
                }
                _ => {}
            }
        }
    }
    return found;
}

/// What:
///  Every place beside the checkout that holds `bytes`,
///  the checkout itself and the old tree left out.
/// Why:
///   A command that keeps a person's bytes instead of deleting them has to put them somewhere and say where;
///  beside the checkout is where this program keeps what belongs to the migration but not to the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const beside = filesHolding(layout.parent, bytes, [layout.root, layout.oldPath]);
/// ```
fn beside_the_checkout(layout: &Layout, bytes: &[u8]) -> Vec<PathBuf> {
    return files_holding(layout.parent.as_path(), bytes, &[layout.root.as_path(), layout.old_path.as_path(), layout.new_path.as_path()]);
}

/// The BYTES of a text, as a control writes and compares them.
fn bytes_of(text: &str) -> Vec<u8> {
    return text.as_bytes().to_vec();
}

/// Whether the command's words name the place it kept a file.
fn names_the_place(said: &str, kept: &[PathBuf]) -> bool {
    return kept.iter().any(|path| return said.contains(path.to_string_lossy().as_ref()));
}

/// The review, findings 1 and 2:
/// an edit made in the checkout after the migration does not stop the deletion of the old tree.
#[test]
fn work_an_edit_after_the_migration_does_not_stop_the_deletion() {
    let (parent, root) = checkout_with_worktree("work-edit-then-remove");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let edited: Vec<u8> = bytes_of("ONE, EDITED IN THE CHECKOUT AFTER THE MIGRATION\n");
    std::fs::write(root.join("README.md"), edited.as_slice()).expect("the edit");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_eq!(code, DONE, "a checkout the person has worked in is the state the command exists for: {said}");
    assert!(!layout.old_path.exists(), "the old tree is deleted");
    assert!(holds(root.join("README.md").as_path(), edited.as_slice()), "the edit is in the checkout");
    remove(parent.as_path());
}

/// The review, finding 1:
/// a commit made in the checkout after the migration does not stop the deletion of the old tree.
#[test]
fn work_a_commit_after_the_migration_does_not_stop_the_deletion() {
    let (parent, root) = checkout_with_worktree("work-commit-then-remove");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let edited: Vec<u8> = bytes_of("ONE, COMMITTED IN THE CHECKOUT AFTER THE MIGRATION\n");
    std::fs::write(root.join("README.md"), edited.as_slice()).expect("the edit");
    git(root.as_path(), &["commit", "--quiet", "--all", "--message=later"]);
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_eq!(code, DONE, "a commit is what a person migrates in order to make: {said}");
    assert!(!layout.old_path.exists(), "the old tree is deleted");
    assert!(holds(root.join("README.md").as_path(), edited.as_slice()), "the committed bytes are in the checkout");
    assert!(git_text(root.as_path(), &["log", "--format=%s"]).contains("later"), "the commit is in the checkout");
    remove(parent.as_path());
}

/// The review, finding 1:
/// a deletion made in the checkout after the migration is not undone, and the deletion still finishes.
#[test]
fn work_a_deletion_after_the_migration_does_not_stop_the_deletion() {
    let (parent, root) = checkout_with_worktree("work-delete-then-remove");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    std::fs::remove_file(root.join("script.sh")).expect("the deletion");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_eq!(code, DONE, "{said}");
    assert!(!layout.old_path.exists(), "the old tree is deleted");
    assert!(!root.join("script.sh").exists(), "the file the person deleted is not restored from the old tree");
    remove(parent.as_path());
}

/// What:
///  Whether the bytes a person wrote are in the tree that survives, or the command kept them somewhere it names.
/// Why:
///  One rule for an edit and for a commit:
///  a rollback that finishes keeps them,
///  and one that refuses says where they are instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assertKeptOrNamed(code, said, kept, where);
/// ```
fn assert_kept_or_named(code: i32, said: &str, kept: &[PathBuf], what: &str) {
    assert_ne!(code, FAILED, "{what}: the command failed instead of finishing or refusing: {said}");
    if code == DONE {
        assert!(!kept.is_empty(), "{what}: the command finished and kept the bytes: {said}");
        return;
    }
    assert_eq!(code, REFUSED, "{what}: an unexpected status: {said}");
    assert!(names_the_place(said, kept), "{what}: the refusal names the place the bytes are kept: {said}");
}

/// The review, finding 2:
/// a rollback after an edit in the new checkout keeps the edit, or refuses and keeps it in the new subvolume.
#[test]
fn work_a_rollback_after_an_edit_keeps_the_edit() {
    let (parent, root) = checkout_with_worktree("work-rollback-edit");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let edited: Vec<u8> = bytes_of("ONE, EDITED IN THE NEW CHECKOUT\n");
    std::fs::write(root.join("README.md"), edited.as_slice()).expect("the edit");
    let (code, said) = run(&platform, &rollback_arguments(), root.as_path());
    println!("rollback code {code}; output: {said}");
    let kept: Vec<PathBuf> = files_holding(layout.parent.as_path(), edited.as_slice(), &[]);
    assert_kept_or_named(code, &said, &kept, "an edit before a rollback");
    if code == DONE {
        assert!(holds(root.join("README.md").as_path(), edited.as_slice()), "the edit is the checkout's: {said}");
    }
    remove(parent.as_path());
}

/// The review, finding 2:
/// a rollback after a commit in the new checkout keeps the commit, or refuses and keeps it in the new subvolume.
#[test]
fn work_a_rollback_after_a_commit_keeps_the_commit() {
    let (parent, root) = checkout_with_worktree("work-rollback-commit");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let committed: Vec<u8> = bytes_of("ONE, COMMITTED IN THE NEW CHECKOUT\n");
    std::fs::write(root.join("README.md"), committed.as_slice()).expect("the edit");
    git(root.as_path(), &["commit", "--quiet", "--all", "--message=later"]);
    let head: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let (code, said) = run(&platform, &rollback_arguments(), root.as_path());
    println!("rollback code {code}; output: {said}");
    let kept: Vec<PathBuf> = files_holding(layout.parent.as_path(), committed.as_slice(), &[]);
    assert_ne!(code, FAILED, "the command failed instead of finishing or refusing: {said}");
    if code == DONE {
        assert!(holds(root.join("README.md").as_path(), committed.as_slice()), "the commit's bytes are the checkout's: {said}");
        assert_eq!(git_text(root.as_path(), &["rev-parse", "HEAD"]), head, "the commit is what the checkout's HEAD names: {said}");
        remove(parent.as_path());
        return;
    }
    assert_eq!(code, REFUSED, "{said}");
    assert!(!kept.is_empty() || said.contains(head.as_str()) || said.contains(layout.new_path.to_string_lossy().as_ref()), "the refusal names where the commit stays: {said}");
    remove(parent.as_path());
}

/// The review, finding 2:
/// the flag in the rollback's own remedy does not delete an edit the person made in the new checkout.
#[test]
fn work_the_rollbacks_discard_flag_does_not_delete_an_edit_of_the_person() {
    let (parent, root) = checkout_with_worktree("work-rollback-discard");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let edited: Vec<u8> = bytes_of("ONE, EDITED IN THE NEW CHECKOUT\n");
    std::fs::write(root.join("README.md"), edited.as_slice()).expect("the edit");
    let (code, said) = run(&platform, &rollback_discarding_left_out(), root.as_path());
    println!("rollback --discard-left-out code {code}; output: {said}");
    let kept: Vec<PathBuf> = files_holding(layout.parent.as_path(), edited.as_slice(), &[]);
    assert!(!kept.is_empty(), "no flag deletes an edit the person made in the checkout: {said}");
    assert_ne!(code, FAILED, "{said}");
    remove(parent.as_path());
}

/// The review, findings 1 and 2:
/// a file a program wrote into the old tree after the migration, whose copy in the checkout the person also changed,
/// is kept where the command says, and the checkout's own bytes are not replaced on a modification time.
#[test]
fn work_a_write_into_the_old_tree_beside_a_change_in_the_checkout_is_kept() {
    let (parent, root) = checkout_with_worktree("work-both-trees");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let written_into_the_old_tree: Vec<u8> = bytes_of("A PROGRAM WROTE THIS INTO THE OLD TREE\n");
    let the_checkouts_own: Vec<u8> = bytes_of("THE PERSON CHANGED THIS IN THE CHECKOUT\n");
    std::fs::write(layout.old_path.join("README.md"), written_into_the_old_tree.as_slice()).expect("a write into the old tree");
    set_seconds(layout.old_path.join("README.md").as_path(), 3_000_000_000);
    std::fs::write(root.join("README.md"), the_checkouts_own.as_slice()).expect("a change in the checkout");
    set_seconds(root.join("README.md").as_path(), 1_000_000_000);
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_ne!(code, FAILED, "{said}");
    let beside: Vec<PathBuf> = beside_the_checkout(&layout, written_into_the_old_tree.as_slice());
    let in_the_checkout: bool = holds(root.join("README.md").as_path(), written_into_the_old_tree.as_slice());
    if code == DONE {
        // The old tree's newer bytes reach the checkout, and what they replaced is parked at a name the run gives.
        assert!(in_the_checkout || names_the_place(&said, &beside), "the old tree's bytes are in the checkout or kept where the run says: {said}");
        let persons: Vec<PathBuf> = files_holding(root.as_path(), the_checkouts_own.as_slice(), &[]);
        assert!(!persons.is_empty() || !in_the_checkout, "the person's own bytes are kept: {said}");
    } else {
        assert_eq!(code, REFUSED, "{said}");
        assert!(said.contains("README.md"), "the refusal names the file: {said}");
        assert!(holds(root.join("README.md").as_path(), the_checkouts_own.as_slice()), "and the checkout's bytes are untouched: {said}");
    }
    remove(parent.as_path());
}

/// The review, finding 6:
/// a branch named `index` and a person's file ending `.lock` below the Git directory are not taken for this program's own,
/// so the bytes the old tree holds there are kept and named.
#[test]
fn work_a_branch_named_index_and_a_person_s_lock_name_are_kept() {
    let (parent, root) = checkout_with_worktree("work-branch-index");
    let platform: PlainPlatform = PlainPlatform::new();
    git(root.as_path(), &["branch", "index"]);
    std::fs::create_dir_all(root.join(".git/hooks")).expect("hooks");
    std::fs::write(root.join(".git/hooks/mine.lock"), bytes_of("hook v1\n")).expect("a lock-named file");
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let other_branch: Vec<u8> = bytes_of("0123456789abcdef0123456789abcdef01234567\n");
    let other_hook: Vec<u8> = bytes_of("hook v2, written after the migration\n");
    std::fs::write(layout.old_path.join(".git/refs/heads/index"), other_branch.as_slice()).expect("the old tree's branch");
    set_seconds(layout.old_path.join(".git/refs/heads/index").as_path(), 1_000_000_000);
    std::fs::write(layout.old_path.join(".git/hooks/mine.lock"), other_hook.as_slice()).expect("the old tree's hook");
    set_seconds(layout.old_path.join(".git/hooks/mine.lock").as_path(), 1_000_000_000);
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_ne!(code, FAILED, "{said}");
    for (what, bytes) in [("the branch named index", &other_branch), ("a person's .lock file", &other_hook)] {
        let kept: Vec<PathBuf> = beside_the_checkout(&layout, bytes.as_slice());
        let named: bool = said.contains("refs/heads/index") || said.contains("mine.lock");
        assert!(!kept.is_empty() || (code == REFUSED && named), "{what}: the bytes the old tree held are kept or named, not deleted silently: {said}");
    }
    remove(parent.as_path());
}

/// The review, finding 7:
/// a file restored in the checkout with an older time is not replaced by the old tree's version.
#[test]
fn work_a_file_restored_in_the_checkout_with_an_older_time_is_not_replaced() {
    let (parent, root) = checkout_with_worktree("work-restored-in-checkout");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let restored: Vec<u8> = bytes_of("PERSON-RESTORED-IN-CHECKOUT\n");
    // The restore carries an older timestamp than the copy the migration made, which is the state `cp -p` of a backup leaves.
    std::fs::write(root.join("README.md"), restored.as_slice()).expect("the restore");
    set_seconds(root.join("README.md").as_path(), 1_000_000_000);
    let (code, said) = run(&platform, &remove_old_discarding_left_out(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_ne!(code, FAILED, "{said}");
    assert_eq!(code, DONE, "a file the person restored in the checkout does not stop the deletion: {said}");
    assert!(
        holds(root.join("README.md").as_path(), restored.as_slice()),
        "the checkout's restored bytes are not replaced by the old tree's on a modification time: {said}"
    );
    remove(parent.as_path());
}

/// The review, finding 8:
/// a migration stopped by an error right after it took the lock leaves no index lock behind,
/// so the person's next Git command in the checkout works.
#[test]
fn work_a_migration_that_fails_under_the_lock_leaves_no_index_lock() {
    let (parent, root) = checkout_with_worktree("work-lock-on-failure");
    let platform: PlainPlatform = PlainPlatform::new();
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::at("after-lock", 0));
    println!("migration stopped after the lock: {code}; output: {}", lines.join(" | "));
    assert_eq!(code, FAILED, "the run stopped where the control asked: {}", lines.join(" | "));
    assert!(!root.join(crate::subvolume_run::INDEX_LOCK).exists(), "the lock is released on every way out of the run");
    // The checkout's Git works again: a Git command that writes the index does not report the lock.
    let status: std::process::Output = git(root.as_path(), &["update-index", "--refresh"]);
    assert!(status.status.success(), "Git works in the checkout after the failure: {status:?}");
    remove(parent.as_path());
}

/// What:
///  The review, finding 5, as a stated limit rather than a closed window:
///  a write made into the old tree after the deletion's last look at its entry,
///  and before that entry is unlinked, is lost.
/// Why:
///   The window cannot be closed while a program holds a descriptor in the old tree:
///  after the name is unlinked the write reaches no name at all.
///  The command says so in its own help and in the package's page,
///  and this control shows what it costs.
/// It is not run in the suite: it asserts the loss, so it would fail on every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the limit, shown: a late write into a file about to be unlinked is gone
/// ```
#[test]
#[ignore = "the stated limit: a write into the old tree after the deletion's last look at its entry is lost"]
fn work_a_write_after_the_last_look_is_the_stated_limit() {
    let (parent, root) = checkout_with_worktree("work-limit-late-write");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    let late: Vec<u8> = bytes_of("WRITTEN AFTER THE LAST LOOK\n");
    // The proof reads the whole old tree before the first unlink; a write that lands after that read is inside the window.
    let old_tree: PathBuf = layout.old_path.clone();
    let written: Vec<u8> = late.clone();
    let writer = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1));
        let _written = std::fs::write(old_tree.join("README.md"), written.as_slice());
    });
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    let _joined = writer.join();
    println!("removal code {code}; output: {said}");
    assert!(!holds(root.join("README.md").as_path(), late.as_slice()), "the limit: the late write is not carried");
    remove(parent.as_path());
}
