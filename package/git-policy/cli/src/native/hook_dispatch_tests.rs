//! Controls for the dispatcher: start-name recognition, the dispatched environment, disabled events,
//! status propagation and the hook lock's release.

use super::*;
use crate::test_support::{fixture, remove};
use crate::hook_dispatch_entries::write_hook_dispatch;
use crate::hook_dispatch_plan::{ConfigParameters, HookDispatchPlan, encode_hook_dispatch_plan};
use crate::transaction_registry::TRANSACTION_ROOT_NAME;

/// A transaction identifier in the registry's spelling.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// One environment pair.
fn pair(name: &str, value: &str) -> (OsString, OsString) {
    return (OsString::from(name), OsString::from(value));
}

/// A transaction hooks directory `<root>/<TRANSACTION>/hooks` holding the given plan.
fn hooks_with_plan(root: &Path, plan: &HookDispatchPlan) -> PathBuf {
    let hooks: PathBuf = root
        .join(TRANSACTION_ROOT_NAME)
        .join(TRANSACTION)
        .join("hooks");
    std::fs::create_dir_all(&hooks).expect("hooks directory");
    std::fs::write(
        hooks.join(PLAN_FILENAME),
        format!("{}\n", encode_hook_dispatch_plan(plan)),
    )
    .expect("plan");
    return hooks;
}

/// A plan whose real Git is the given executable and whose lock is at the given directory.
fn plan_for(git_path: &Path, hooks_path: &Path, lock: &Path, skip_hook_lock: bool) -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from(git_path.to_str().expect("UTF-8")),
        hooks_path: String::from(hooks_path.to_str().expect("UTF-8")),
        disabled_events: Vec::new(),
        config_parameters: ConfigParameters::Absent,
        worktree_root: String::from("/repo"),
        lease: String::from("lease"),
        hook_lock_path: String::from(lock.to_str().expect("UTF-8")),
        skip_hook_lock,
        named_hooks_possible: true,
    };
}

/// Events in a real transaction hooks directory are recognized with their event and directory.
#[cfg(unix)]
#[test]
fn preparation_entries_in_a_transaction_are_recognized() {
    let root: PathBuf = fixture("dispatch-recognized");
    let plan: HookDispatchPlan = plan_for(Path::new("/usr/bin/git"), Path::new("/h"), Path::new("/l"), true);
    let hooks: PathBuf = hooks_with_plan(root.as_path(), &plan);
    for event in PREPARATION_HOOK_EVENTS {
        let target: Option<DispatchTarget> = dispatch_target_for_start_name(hooks.join(event).as_path());
        assert_eq!(
            target,
            Some(DispatchTarget {
                event,
                hooks_directory: hooks.clone(),
            }),
            "{event}"
        );
    }
    remove(root.as_path());
}

/// Every other start name behaves as `git`: post-commit, unknown names, other directories,
/// other transaction shapes and a plan that is missing or not a file.
#[cfg(unix)]
#[test]
fn every_other_start_name_is_not_a_dispatch() {
    let root: PathBuf = fixture("dispatch-refused");
    let plan: HookDispatchPlan = plan_for(Path::new("/usr/bin/git"), Path::new("/h"), Path::new("/l"), true);
    let hooks: PathBuf = hooks_with_plan(root.as_path(), &plan);
    assert_eq!(dispatch_target_for_start_name(hooks.join("post-commit").as_path()), None);
    assert_eq!(dispatch_target_for_start_name(hooks.join("pre-push").as_path()), None);
    assert_eq!(dispatch_target_for_start_name(hooks.join("pre-commit.exe").as_path()), None);
    assert_eq!(dispatch_target_for_start_name(hooks.join("pre-commit").parent().expect("parent").join("pre-commit").as_path()), Some(DispatchTarget {
        event: "pre-commit",
        hooks_directory: hooks.clone(),
    }));
    let outside: PathBuf = root.join("elsewhere").join("pre-commit");
    assert_eq!(dispatch_target_for_start_name(outside.as_path()), None);
    let not_hooks: PathBuf = root
        .join(TRANSACTION_ROOT_NAME)
        .join(TRANSACTION)
        .join("pre-commit");
    assert_eq!(dispatch_target_for_start_name(not_hooks.as_path()), None);
    let not_transaction: PathBuf = root
        .join(TRANSACTION_ROOT_NAME)
        .join("not-a-transaction")
        .join("hooks")
        .join("pre-commit");
    assert_eq!(dispatch_target_for_start_name(not_transaction.as_path()), None);
    let wrong_root: PathBuf = root
        .join("other-root")
        .join(TRANSACTION)
        .join("hooks")
        .join("pre-commit");
    assert_eq!(dispatch_target_for_start_name(wrong_root.as_path()), None);
    // A plan beside the entry leaves only the registry root name to refuse the start name.
    let wrong_root_hooks: PathBuf = wrong_root.parent().expect("hooks").to_path_buf();
    std::fs::create_dir_all(&wrong_root_hooks).expect("wrong root hooks");
    std::fs::write(wrong_root_hooks.join(PLAN_FILENAME), format!("{}\n", encode_hook_dispatch_plan(&plan)))
        .expect("wrong root plan");
    assert_eq!(dispatch_target_for_start_name(wrong_root.as_path()), None);
    std::fs::remove_file(hooks.join(PLAN_FILENAME)).expect("remove plan");
    assert_eq!(dispatch_target_for_start_name(hooks.join("pre-commit").as_path()), None);
    std::fs::create_dir(hooks.join(PLAN_FILENAME)).expect("plan as a directory");
    assert_eq!(dispatch_target_for_start_name(hooks.join("pre-commit").as_path()), None);
    remove(root.as_path());
}

/// Relative start names are read as written, against the current directory.
#[test]
fn relative_start_names_are_not_recognized_without_their_directories() {
    assert_eq!(dispatch_target_for_start_name(Path::new("pre-commit")), None);
    assert_eq!(dispatch_target_for_start_name(Path::new("")), None);
}

/// The dispatched environment restores or removes the config parameters and sets the worktree and lease.
#[test]
fn the_dispatched_environment_carries_the_plan() {
    let mut plan: HookDispatchPlan = plan_for(Path::new("/g"), Path::new("/h"), Path::new("/l"), false);
    let inherited: Vec<(OsString, OsString)> = vec![
        pair("PATH", "/bin"),
        pair("GIT_CONFIG_PARAMETERS", "'stale=1'"),
        pair("GIT_WORK_TREE", "/elsewhere"),
    ];
    let absent: Vec<(OsString, OsString)> = dispatch_environment(inherited.as_slice(), &plan);
    assert!(absent.contains(&pair("PATH", "/bin")));
    assert!(!absent.iter().any(|(name, _)| { return name == "GIT_CONFIG_PARAMETERS"; }));
    assert!(absent.contains(&pair("GIT_WORK_TREE", "/repo")));
    assert!(absent.contains(&pair("CLI_GIT_PREPARATION_LEASE", "lease")));
    plan.config_parameters = ConfigParameters::Present(String::from("'a.b=1'"));
    let present: Vec<(OsString, OsString)> = dispatch_environment(inherited.as_slice(), &plan);
    assert!(present.contains(&pair("GIT_CONFIG_PARAMETERS", "'a.b=1'")));
    assert_eq!(present.iter().filter(|(name, _)| { return name == "GIT_CONFIG_PARAMETERS"; }).count(), 1);
}

/// A disabled event returns success without starting Git or taking the lock.
#[test]
fn a_disabled_event_succeeds_without_starting_git() {
    let root: PathBuf = fixture("dispatch-disabled");
    let mut plan: HookDispatchPlan = plan_for(
        root.join("no-such-git").as_path(),
        root.as_path(),
        root.join("no-such-dir").join("hook.lock").as_path(),
        false,
    );
    plan.disabled_events = vec![String::from("pre-commit")];
    let hooks: PathBuf = hooks_with_plan(root.as_path(), &plan);
    let target: DispatchTarget = DispatchTarget {
        event: "pre-commit",
        hooks_directory: hooks,
    };
    assert_eq!(run_hook_dispatch(&target, &[], &[]), 0);
    remove(root.as_path());
}

/// A failure to start Git is reported and exits 1, and the lock is still released.
#[test]
fn a_missing_git_exits_one_and_releases_the_lock() {
    let root: PathBuf = fixture("dispatch-missing-git");
    let lock: PathBuf = root.join("cli-git").join("hook.lock");
    std::fs::create_dir_all(root.join("cli-git")).expect("cli-git directory");
    let plan: HookDispatchPlan = plan_for(
        root.join("no-such-git").as_path(),
        root.as_path(),
        lock.as_path(),
        false,
    );
    let hooks: PathBuf = hooks_with_plan(root.as_path(), &plan);
    let target: DispatchTarget = DispatchTarget {
        event: "commit-msg",
        hooks_directory: hooks,
    };
    assert_eq!(run_hook_dispatch(&target, &[], &[]), 1);
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A malformed plan exits 1 without starting Git.
#[test]
fn a_malformed_plan_exits_one() {
    let root: PathBuf = fixture("dispatch-malformed");
    let hooks: PathBuf = root
        .join(TRANSACTION_ROOT_NAME)
        .join(TRANSACTION)
        .join("hooks");
    std::fs::create_dir_all(&hooks).expect("hooks directory");
    std::fs::write(hooks.join(PLAN_FILENAME), b"{}").expect("plan");
    let target: DispatchTarget = DispatchTarget {
        event: "pre-commit",
        hooks_directory: hooks,
    };
    assert_eq!(run_hook_dispatch(&target, &[], &[]), 1);
    remove(root.as_path());
}

/// The entries the writer makes are recognized by the dispatcher (a round trip through both modules).
#[cfg(unix)]
#[test]
fn written_entries_are_recognized_by_the_dispatcher() {
    let root: PathBuf = fixture("dispatch-round-trip");
    let plan: HookDispatchPlan = plan_for(Path::new("/usr/bin/git"), Path::new("/h"), Path::new("/l"), true);
    let hooks: PathBuf = root
        .join(TRANSACTION_ROOT_NAME)
        .join(TRANSACTION)
        .join("hooks");
    std::fs::create_dir_all(hooks.parent().expect("transaction directory")).expect("transaction directory");
    write_hook_dispatch(hooks.as_path(), &plan, Path::new("/opt/native/bin/git")).expect("write");
    let target: Option<DispatchTarget> =
        dispatch_target_for_start_name(hooks.join("commit-msg").as_path());
    assert_eq!(target.map(|found| { return found.event; }), Some("commit-msg"));
    remove(root.as_path());
}
