//! Controls for the hook lock and lease decisions: which events can run a hook, when the lock is held, and what the
//!  lease variable carries.

use super::*;
use crate::test_support::{fixture, remove};

/// A plan with a lock at `lock`, the given switches and `named` hooks possible.
fn plan(lock: &Path, named: bool, skip: bool, disabled: &[&str]) -> HookDispatchPlan {
    return HookDispatchPlan {
        disabled_events: disabled.iter().map(|name| { return String::from(*name); }).collect(),
        lease: String::from("the-lease"),
        hook_lock_path: lock.display().to_string(),
        skip_hook_lock: skip,
        named_hooks_possible: named,
    };
}

/// An event runs a hook only when the configuration can define one and the user did not disable the event.
#[test]
fn an_event_may_run_only_with_named_hooks_and_no_switch_off() {
    let lock: PathBuf = PathBuf::from("/unused/hook.lock");
    assert!(event_may_run(&plan(lock.as_path(), true, false, &[]), "pre-commit"));
    assert!(!event_may_run(&plan(lock.as_path(), false, false, &[]), "pre-commit"));
    assert!(!event_may_run(&plan(lock.as_path(), true, false, &["pre-commit"]), "pre-commit"));
    assert!(event_may_run(&plan(lock.as_path(), true, false, &["pre-commit"]), "commit-msg"));
}

/// The preparation events decide together: one enabled event is enough, and all three disabled is none.
#[test]
fn preparation_hooks_may_run_while_one_event_is_enabled() {
    let lock: PathBuf = PathBuf::from("/unused/hook.lock");
    let two_off: HookDispatchPlan = plan(lock.as_path(), true, false, &["pre-commit", "commit-msg", "post-commit"]);
    assert!(preparation_hooks_may_run(&two_off));
    let all_off: HookDispatchPlan =
        plan(lock.as_path(), true, false, &["pre-commit", "prepare-commit-msg", "commit-msg"]);
    assert!(!preparation_hooks_may_run(&all_off));
    assert!(!preparation_hooks_may_run(&plan(lock.as_path(), false, false, &[])));
}

/// The lock is taken only when a run can find a hook and the plan does not skip it, and it is released on drop.
#[test]
fn the_lock_is_held_only_when_a_run_can_find_a_hook() {
    let root: PathBuf = fixture("hook-run-lock");
    let lock: PathBuf = root.join("cli-git").join("hook.lock");
    let events: [&str; 3] = PREPARATION_HOOK_EVENTS;
    for (named, skip, disabled, expect_lock) in [
        (false, false, vec![], false),
        (true, true, vec![], false),
        (true, false, vec!["pre-commit", "prepare-commit-msg", "commit-msg"], false),
        (true, false, vec![], true),
    ] {
        let held = hold_hook_lock(&plan(lock.as_path(), named, skip, &disabled), &events).expect("decision");
        assert_eq!(held.is_some(), expect_lock, "named {named}, skip {skip}, disabled {disabled:?}");
        assert_eq!(lock.exists(), expect_lock, "named {named}, skip {skip}, disabled {disabled:?}");
        drop(held);
        assert!(!lock.exists(), "dropping the guard releases the lock");
    }
    remove(root.as_path());
}

/// The lease variable carries the plan's lease under the name nested wrappers read.
#[test]
fn the_lease_variable_carries_the_plans_lease() {
    let (name, value): (OsString, OsString) = lease_variable(&plan(Path::new("/l"), true, false, &[]));
    assert_eq!(name, OsString::from("CLI_GIT_PREPARATION_LEASE"));
    assert_eq!(value, OsString::from("the-lease"));
}

/// The absent hooks directory sits beside the hooks directory in the transaction and is never created.
#[test]
fn the_absent_hooks_directory_is_named_but_never_created() {
    let root: PathBuf = fixture("hook-run-no-hooks");
    let absent: PathBuf = transaction_no_hooks_directory(root.as_path());
    assert_eq!(absent, root.join("no-hooks"));
    assert!(!absent.exists());
    remove(root.as_path());
}
