//! Controls for who owns a migration: one live command per checkout, whatever its mode, named in the refusal of the second,
//!  and nothing left in the checkout's parent by a preview.

use super::*;
use crate::subvolume_arguments::MigrateMode;
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::remove;

/// The exit status of a finished migration.
const DONE: i32 = 0;

/// The names in a directory, sorted.
fn names(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(directory).expect("listing").flatten().map(|item| { return item.file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    return found;
}

/// A command of any mode that starts while a migration of the same checkout runs is refused, names the live owner by its
///  process id, and changes nothing; the first command still finishes.
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
