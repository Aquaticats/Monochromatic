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
