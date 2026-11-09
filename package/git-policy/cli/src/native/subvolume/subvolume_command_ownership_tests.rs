//! Controls for who owns a migration:
//! one live command per checkout,
//! whatever its mode,
//! named in the refusal of the second,
//!  and nothing left in the checkout's parent by a preview.

use super::*;
use crate::subvolume_arguments::MigrateMode;
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::remove;

/// The exit status of a finished migration.
const DONE: i32 = 0;

/// The names in a directory,
/// sorted.
fn names(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(directory).expect("listing").flatten().map(|item| { return item.file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    return found;
}

/// A command of any mode that starts while a migration of the same checkout runs is refused,
/// names the live owner by its
///  process id,
/// and changes nothing;
/// the first command still finishes.
#[test]
fn a_second_command_on_one_checkout_is_refused_whatever_its_mode() {
    let (parent, root) = checkout_with_worktree("ownership-second-command");
    let platform: PlainPlatform = PlainPlatform::new();
    let mut inner: Vec<(MigrateMode, i32, Vec<String>)> = Vec::new();
    let mut outer_lines: Vec<String> = Vec::new();
    let outer_code: i32 = execute(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never(), &mut |text: &str| {
        outer_lines.push(text.to_string());
        if inner.is_empty() && text.starts_with("checkout ") {
            for mode in [MigrateMode::Migrate, MigrateMode::Rollback, MigrateMode::RemoveOld] {
                let arguments: MigrateArguments = MigrateArguments { mode, ..migrate_arguments() };
                let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
                inner.push((mode, code, lines));
            }
        }
    });
    assert_eq!(inner.len(), 3, "the second commands ran while the first announced");
    for (mode, code, lines) in &inner {
        let text: String = lines.join("\n");
        assert_eq!(*code, REFUSED_EXIT_CODE, "{mode:?}: {text}");
        assert!(text.contains(format!("process {}", std::process::id()).as_str()) && text.contains("Nothing was changed."), "{mode:?}: {text}");
    }
    assert_eq!(outer_code, DONE, "{outer_lines:#?}");
    remove(parent.as_path());
}

/// A preview and a refused command leave no claim file in the checkout's parent directory.
#[test]
fn a_preview_leaves_no_claim_behind() {
    let (parent, root) = checkout_with_worktree("ownership-preview");
    let before: Vec<String> = names(parent.as_path());
    let arguments: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let (code, lines) = run_command(&PlainPlatform::new(), &arguments, root.as_path(), &Interrupt::never());
    assert_eq!(code, DONE, "{lines:#?}");
    assert_eq!(names(parent.as_path()), before);
    remove(parent.as_path());
}

/// A conversion of a linked worktree that is running owns it:
/// a migration of the checkout is refused before it starts,
/// names
///  the conversion's process,
/// and creates nothing,
/// even while the conversion holds no `index.lock` (during its first copy).
#[test]
fn a_migration_is_refused_while_a_worktree_is_being_converted() {
    use crate::process_identity::process_birth_identity;
    use crate::single_owner_claim::plant_claim_for_tests;
    use crate::worktree_convert_journal::CLAIM_PREFIX;
    let (parent, root) = checkout_with_worktree("ownership-conversion-running");
    // The stand-in must outlive this test however slow the host is: the migration reads the claim's owner when its
    // preflight reaches it, and a stand-in that has ended by then makes the claim stale, so the migration does not refuse.
    // Five minutes is six times the longest run seen on a loaded host (53.7 s on 2026-11-09), and the process group is ended
    // with the test thread anyway.
    let mut conversion: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("300")).expect("sleep");
    let pid: i64 = i64::from(conversion.id());
    let birth: String = process_birth_identity(pid).expect("identity").expect("alive");
    plant_claim_for_tests(root.join(".git/worktrees/wt").as_path(), std::ffi::OsStr::new(CLAIM_PREFIX), 1, pid, birth.as_str());
    let before: Vec<String> = names(parent.as_path());
    let (code, lines) = run_command(&PlainPlatform::new(), &migrate_arguments(), root.as_path(), &Interrupt::never());
    let text: String = lines.join("\n");
    assert_eq!(code, REFUSED_EXIT_CODE, "{text}");
    assert!(text.contains(format!("process {pid}").as_str()) && text.contains("Nothing was changed."), "{text}");
    // The refused command's own claim file, released again, is the only thing it leaves in the parent directory.
    let created: Vec<String> = names(parent.as_path()).into_iter().filter(|name: &String| { return !before.contains(name) && !name.starts_with(".cli-git-migrate-checkout.owner."); }).collect();
    assert_eq!(created, Vec::<String>::new(), "nothing was created beside the checkout");
    conversion.kill().expect("kill");
    conversion.wait().expect("wait");
    remove(parent.as_path());
}
