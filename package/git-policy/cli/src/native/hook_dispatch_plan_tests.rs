//! Controls for the hook plan: the JSON spelling, the lock path and the configuration reads.

use super::*;
use crate::config_snapshot::read_config_snapshot;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// A plan with fixed values, for spelling tests.
fn sample_plan(skip_hook_lock: bool) -> HookDispatchPlan {
    return HookDispatchPlan {
        disabled_events: vec![String::from("commit-msg")],
        lease: String::from("{\"directory\":\"/d\",\"token\":\"t\"}"),
        hook_lock_path: String::from("/repo/.git/cli-git/hook.lock"),
        skip_hook_lock,
        named_hooks_possible: true,
    };
}

/// The plan spells its fields in order with `JSON.stringify` escapes, and records no executable, no hooks path and
///  no config parameters: Git never starts the wrapper for a hook, so nothing needs them.
#[test]
fn plans_spell_the_lease_record_in_order() {
    let spelled: String = encode_hook_dispatch_plan(&sample_plan(false));
    assert_eq!(
        spelled,
        "{\"schemaVersion\":1,\
         \"disabledEvents\":[\"commit-msg\"],\
         \"lease\":\"{\\\"directory\\\":\\\"/d\\\",\\\"token\\\":\\\"t\\\"}\",\
         \"hookLockPath\":\"/repo/.git/cli-git/hook.lock\",\"skipHookLock\":false}"
    );
    for forbidden in ["gitPath", "hooksPath", "configParameters", "worktreeRoot"] {
        assert!(!spelled.contains(forbidden), "{forbidden} in {spelled}");
    }
    assert!(encode_hook_dispatch_plan(&sample_plan(true)).ends_with("\"skipHookLock\":true}"));
}

/// The hook lock sits under the common directory, and paths normalize like Node's `path`.
#[test]
fn lock_and_paths_normalize_lexically() {
    assert_eq!(
        hook_lock_directory(Path::new("/repo/.git")),
        PathBuf::from("/repo/.git/cli-git/hook.lock")
    );
    assert_eq!(
        hook_lock_directory(Path::new("/repo/./wt//.git/")),
        PathBuf::from("/repo/wt/.git/cli-git/hook.lock")
    );
    assert_eq!(normalized_path(Path::new("/repo/../../x/")), PathBuf::from("/x"));
}

/// Real Git supplies the switches.
#[test]
fn plans_read_real_git_configuration() {
    let root: PathBuf = fixture("hook-plan-reads");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    git(
        repository_root.as_path(),
        &["config", "hook.pre-commit.enabled", "false"],
    );
    git(
        repository_root.as_path(),
        &["config", "hooks.concurrentCommits", "true"],
    );
    let common: PathBuf = repository_root.join(".git");
    let plan: HookDispatchPlan = compute_hook_dispatch_plan(&HookPlanInputs {
        context: &git_context(),
        cwd: repository_root.as_path(),
        common_directory: common.as_path(),
        lease: "{\"directory\":\"/d\",\"token\":\"t\"}",
        concurrent_commits: true,
        inherited_lease_valid: false,
        snapshot: None,
    })
    .expect("plan");
    assert_eq!(plan.disabled_events, vec![String::from("pre-commit")]);
    assert!(plan.skip_hook_lock);
    assert_eq!(plan.hook_lock_path, common.join("cli-git").join("hook.lock").to_str().expect("UTF-8"));
    remove(root.as_path());
}

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

/// An inherited valid lease skips the lock.
#[test]
fn plans_skip_the_lock_for_inherited_leases() {
    let root: PathBuf = fixture("hook-plan-inherited");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    let plan: HookDispatchPlan = compute_hook_dispatch_plan(&HookPlanInputs {
        context: &git_context(),
        cwd: repository_root.as_path(),
        common_directory: repository_root.join(".git").as_path(),
        lease: "lease",
        concurrent_commits: false,
        inherited_lease_valid: true,
        snapshot: None,
    })
    .expect("plan");
    assert!(plan.skip_hook_lock);
    remove(root.as_path());
}

/// A config read that Git fails with a code other than 1 stops planning with Git's message.
#[test]
fn a_failing_config_read_stops_planning() {
    let root: PathBuf = fixture("hook-plan-failure");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repository_root.join(".git").join("config"), b"[core\n").expect("broken config");
    let result = compute_hook_dispatch_plan(&HookPlanInputs {
        context: &git_context(),
        cwd: repository_root.as_path(),
        common_directory: repository_root.join(".git").as_path(),
        lease: "lease",
        concurrent_commits: false,
        inherited_lease_valid: false,
        snapshot: None,
    });
    assert!(matches!(result, Err(HookPlanError::ConfigRead(_))));
    remove(root.as_path());
}

/// A new preparation lease names the directory and a fresh token in the incumbent's spelling.
#[test]
fn new_leases_name_the_directory_with_fresh_tokens() {
    let directory: PathBuf = PathBuf::from("/repo/.git/cli-git-transactions/x");
    let first: String = new_preparation_lease(directory.as_path()).expect("lease");
    let second: String = new_preparation_lease(directory.as_path()).expect("lease");
    assert!(first.starts_with("{\"directory\":\"/repo/.git/cli-git-transactions/x\",\"token\":\""));
    assert_ne!(first, second);
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

/// A hook defined by name, by its event or by its command, makes the plan keep every event; other hook keys do not.
#[test]
fn named_hook_keys_decide_whether_the_plan_keeps_every_event() {
    let root: PathBuf = fixture("hook-plan-named-hooks");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    for (text, expected) in [
        ("", false),
        ("[hook \"pre-commit\"]\n\tenabled = false\n[hook]\n\tjobs = 2\n", false),
        ("[hook \"mine\"]\n\tevent = pre-commit\n", true),
        ("[hook \"mine\"]\n\tcommand = true\n", true),
        ("[hook \"a.b\"]\n\tevent = commit-msg\n\tcommand = true\n\tenabled = false\n", true),
    ] {
        std::fs::write(
            repository_root.join(".git").join("config"),
            format!("[core]\n\trepositoryformatversion = 0\n{text}"),
        )
        .expect("config");
        let snapshot: ConfigSnapshot = read_config_snapshot(&context, repository_root.as_path()).expect("snapshot");
        let plan: HookDispatchPlan = compute_hook_dispatch_plan(&HookPlanInputs {
            snapshot: Some(&snapshot),
            ..plan_inputs(&context, repository_root.as_path())
        })
        .expect("plan");
        assert_eq!(plan.named_hooks_possible, expected, "{text}");
    }
    remove(root.as_path());
}

/// For each configuration, the plan from the snapshot equals the plan from Git's own reads, errors included.
#[test]
fn the_snapshot_plan_equals_the_plan_from_git_reads() {
    let root: PathBuf = fixture("hook-plan-snapshot-differential");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    for text in [
        "",
        "[hook \"pre-commit\"]\n\tenabled = false\n[hook \"post-commit\"]\n\tenabled = off\n",
        "[hook \"commit-msg\"]\n\tenabled = no\n\tenabled = true\n",
        "[hook \"prepare-commit-msg\"]\n\tenabled\n",
        "[hook \"pre-commit\"]\n\tenabled = maybe\n",
        "[hook \"pre-commit\"]\n\tenabled = 2\n[hook \"commit-msg\"]\n\tenabled = 0\n",
        "[hook \"Pre-Commit\"]\n\tenabled = false\n",
        "[hook \"pre-commit\"]\n\tenabled = 1k\n",
    ] {
        std::fs::write(
            repository_root.join(".git").join("config"),
            format!("[core]\n\trepositoryformatversion = 0\n{text}"),
        )
        .expect("config");
        let snapshot: Option<ConfigSnapshot> = read_config_snapshot(&context, repository_root.as_path());
        let from_snapshot = compute_hook_dispatch_plan(&HookPlanInputs {
            snapshot: snapshot.as_ref(),
            ..plan_inputs(&context, repository_root.as_path())
        });
        let from_git = compute_hook_dispatch_plan(&plan_inputs(&context, repository_root.as_path()));
        // Only the snapshot can say that the configuration names no hooks; without it the plan assumes it may.
        if let (Ok(counted), Ok(spawned)) = (&from_snapshot, &from_git) {
            assert!(!counted.named_hooks_possible, "{text}");
            assert!(spawned.named_hooks_possible, "{text}");
        }
        let comparable = from_git.map(|plan| {
            return HookDispatchPlan { named_hooks_possible: false, ..plan };
        });
        assert_eq!(from_snapshot, comparable, "{text}");
    }
    remove(root.as_path());
}
