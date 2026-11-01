//! Controls for the event switches: Git's spellings, the last value, the snapshot's answers and its declines.

use super::*;
use crate::hook_dispatch_plan::{HookDispatchPlan, HookPlanInputs, compute_hook_dispatch_plan};
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::path::PathBuf;

/// The inputs of a plan that only the configuration decides.
fn plan_inputs<'a>(context: &'a GitContext, repository_root: &'a Path) -> HookPlanInputs<'a> {
    return HookPlanInputs {
        context,
        cwd: repository_root,
        common_directory: repository_root,
        lease: "lease",
        concurrent_commits: false,
        inherited_lease_valid: false,
        snapshot: None,
    };
}

/// A snapshot parsed from `(scope, record)` pairs in the `git config --list -z --show-scope` shape.
fn listing_snapshot(records: &[(&str, &str)]) -> ConfigSnapshot {
    let mut bytes: Vec<u8> = Vec::new();
    for (scope, record) in records {
        bytes.extend_from_slice(scope.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(record.as_bytes());
        bytes.push(0);
    }
    return ConfigSnapshot::parse(bytes.as_slice(), true).expect("a well-formed listing");
}


/// A stand-in Git that logs each start and fails, so a plan that starts Git shows in the log and in its result.
fn failing_stand_in(root: &Path) -> (GitContext, PathBuf) {
    let log: PathBuf = root.join("starts.log");
    let path: PathBuf = root.join("git");
    let content: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexit 99\n", log.display());
    crate::test_support::executable(path.as_path(), content.as_bytes());
    let context: GitContext = GitContext {
        real_git: path,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    return (context, log);
}


/// A snapshot that models the switches answers the plan without starting Git, whatever `core.hooksPath` holds.
#[test]
fn a_snapshot_answers_the_plan_without_starting_git() {
    let root: PathBuf = fixture("hook-plan-snapshot-answers");
    let (context, log) = failing_stand_in(root.as_path());
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("global", "core.hookspath\nglobal-hooks"),
        ("local", "core.hookspath\n~/hooks"),
        ("local", "hook.commit-msg.enabled\nfalse"),
        ("command", "hook.pre-commit.enabled\nno"),
        ("local", "hook.post-commit.enabled\nyes"),
    ]);
    let inputs: HookPlanInputs<'_> = HookPlanInputs {
        snapshot: Some(&snapshot),
        ..plan_inputs(&context, root.as_path())
    };
    let plan: HookDispatchPlan = compute_hook_dispatch_plan(&inputs).expect("plan");
    assert_eq!(
        plan.disabled_events,
        vec![String::from("pre-commit"), String::from("commit-msg")]
    );
    assert!(!log.exists(), "the plan started Git: {:?}", std::fs::read_to_string(log.as_path()));
    remove(root.as_path());
}


/// A value the snapshot does not model makes the plan start Git for that key, as it did before.
#[test]
fn a_value_the_snapshot_does_not_model_starts_git() {
    let root: PathBuf = fixture("hook-plan-snapshot-declines");
    let (context, log) = failing_stand_in(root.as_path());
    let bad_switch: ConfigSnapshot = listing_snapshot(&[("local", "hook.pre-commit.enabled\nmaybe")]);
    let switch_inputs: HookPlanInputs<'_> = HookPlanInputs {
        snapshot: Some(&bad_switch),
        ..plan_inputs(&context, root.as_path())
    };
    assert!(matches!(compute_hook_dispatch_plan(&switch_inputs), Err(HookPlanError::ConfigRead(_))));
    let started: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert!(started.contains("config --type=bool --get-regexp"), "{started}");
    remove(root.as_path());
}


/// The one batched read agrees with Git's own `--get` for every spelling of a switch:
///  `False`, `no` and `0` disable, a valueless key and `true` enable,
///  and the last of repeated values decides.
#[test]
fn event_switches_follow_git_spellings_and_the_last_value() {
    let root: PathBuf = fixture("hook-plan-spellings");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(
        repository_root.join(".git").join("config"),
        [
            "[hook \"pre-commit\"]\n\tenabled = False\n",
            "[hook \"commit-msg\"]\n\tenabled = no\n\tenabled = true\n",
            "[hook \"post-commit\"]\n\tenabled = 0\n",
            "[hook \"prepare-commit-msg\"]\n\tenabled\n",
        ]
        .concat(),
    )
    .expect("config");
    let context: GitContext = git_context();
    let plan: HookDispatchPlan =
        compute_hook_dispatch_plan(&plan_inputs(&context, repository_root.as_path())).expect("plan");
    assert_eq!(
        plan.disabled_events,
        vec![String::from("pre-commit"), String::from("post-commit")]
    );
    remove(root.as_path());
}


/// A repository with no switch disables nothing, and an unreadable switch stops planning with Git's message.
#[test]
fn missing_switches_disable_nothing_and_a_bad_switch_stops_planning() {
    let root: PathBuf = fixture("hook-plan-switch-errors");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let plan: HookDispatchPlan =
        compute_hook_dispatch_plan(&plan_inputs(&context, repository_root.as_path())).expect("plan");
    assert_eq!(plan.disabled_events, Vec::<String>::new());
    git(repository_root.as_path(), &["config", "hook.post-commit.enabled", "false"]);
    git(repository_root.as_path(), &["config", "hook.pre-commit.enabled", "maybe"]);
    let Err(HookPlanError::ConfigRead(message)) =
        compute_hook_dispatch_plan(&plan_inputs(&context, repository_root.as_path()))
    else {
        panic!("a bad switch must stop planning");
    };
    assert!(message.contains("bad boolean config value 'maybe'"), "{message}");
    remove(root.as_path());
}
