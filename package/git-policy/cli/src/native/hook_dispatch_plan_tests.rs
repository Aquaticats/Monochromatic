//! Controls for the dispatch plan: quoting, config combination, the JSON spelling and real-Git reads.

use super::*;
use crate::config_snapshot::read_config_snapshot;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// One environment pair.
fn pair(name: &str, value: &str) -> (OsString, OsString) {
    return (OsString::from(name), OsString::from(value));
}

/// A plan with fixed values, for spelling tests.
fn sample_plan(config_parameters: ConfigParameters, skip_hook_lock: bool) -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from("/usr/bin/git"),
        disabled_events: vec![String::from("commit-msg")],
        config_parameters,
        worktree_root: String::from("/repo"),
        lease: String::from("{\"directory\":\"/d\",\"token\":\"t\"}"),
        hook_lock_path: String::from("/repo/.git/cli-git/hook.lock"),
        skip_hook_lock,
        named_hooks_possible: true,
    };
}

/// Single quotes close around each `'` and `!`, as Git's `sq_quote_buf` does.
#[test]
fn quoting_closes_the_quote_around_quotes_and_bangs() {
    assert_eq!(quote_config_parameter("user.name=O'Brien"), "'user.name=O'\\''Brien'");
    assert_eq!(quote_config_parameter("a.b=x!y"), "'a.b=x'\\!'y'");
    assert_eq!(quote_config_parameter("a.b=1"), "'a.b=1'");
}

/// Every `-c` value is collected, including a `-c` that is itself a value.
#[test]
fn global_overrides_pair_each_dash_c_with_its_next_token() {
    let arguments: Vec<OsString> = vec![
        OsString::from("-c"),
        OsString::from("a.b=1"),
        OsString::from("--no-pager"),
    ];
    assert_eq!(global_config_overrides(arguments.as_slice()), Ok(vec![String::from("a.b=1")]));
    let repeated: Vec<OsString> = vec![
        OsString::from("-c"),
        OsString::from("-c"),
        OsString::from("x"),
    ];
    assert_eq!(
        global_config_overrides(repeated.as_slice()),
        Ok(vec![String::from("-c"), String::from("x")])
    );
    let dangling: Vec<OsString> = vec![OsString::from("-c")];
    assert_eq!(global_config_overrides(dangling.as_slice()), Ok(Vec::new()));
}

/// A non-UTF-8 override cannot be written into JSON, so it is refused.
#[cfg(unix)]
#[test]
fn global_overrides_refuse_non_utf8_values() {
    use std::os::unix::ffi::OsStringExt;
    let arguments: Vec<OsString> = vec![
        OsString::from("-c"),
        OsString::from_vec(vec![0xff, 0xfe]),
    ];
    assert_eq!(
        global_config_overrides(arguments.as_slice()),
        Err(HookPlanError::NotUtf8("global -c option"))
    );
}

/// Inherited text comes first, an empty inherited value counts as none, and no entry means absence.
#[test]
fn combined_parameters_keep_git_order_and_absence() {
    assert_eq!(
        combine_config_parameters("", &[]),
        Ok(ConfigParameters::Absent)
    );
    assert_eq!(
        combine_config_parameters("", &[String::from("a.b=1")]),
        Ok(ConfigParameters::Present(String::from("'a.b=1'")))
    );
    assert_eq!(
        combine_config_parameters("'x=1'", &[String::from("a.b=1")]),
        Ok(ConfigParameters::Present(String::from("'x=1' 'a.b=1'")))
    );
    assert_eq!(
        combine_config_parameters("'x=1'", &[]),
        Ok(ConfigParameters::Present(String::from("'x=1'")))
    );
}

/// The plan's environment value is read from the injected environment, last entry winning.
#[test]
fn inherited_parameters_come_from_the_injected_environment() {
    let environment: Vec<(OsString, OsString)> = vec![
        pair("GIT_CONFIG_PARAMETERS", "'old=1'"),
        pair("GIT_CONFIG_PARAMETERS", "'x=1'"),
    ];
    assert_eq!(
        inherited_config_parameters(environment.as_slice()),
        Ok(String::from("'x=1'"))
    );
    assert_eq!(inherited_config_parameters(&[]), Ok(String::new()));
}

/// The plan spells fields in the incumbent's order, with `JSON.stringify` escapes and a parameter kind.
#[test]
fn plans_spell_the_incumbent_fields_in_order() {
    let absent: String = encode_hook_dispatch_plan(&sample_plan(ConfigParameters::Absent, false));
    assert_eq!(
        absent,
        "{\"schemaVersion\":1,\"gitPath\":\"/usr/bin/git\",\
         \"disabledEvents\":[\"commit-msg\"],\"configParameters\":{\"kind\":\"absent\"},\
         \"worktreeRoot\":\"/repo\",\"lease\":\"{\\\"directory\\\":\\\"/d\\\",\\\"token\\\":\\\"t\\\"}\",\
         \"hookLockPath\":\"/repo/.git/cli-git/hook.lock\",\"skipHookLock\":false}"
    );
    let present: String = encode_hook_dispatch_plan(&sample_plan(
        ConfigParameters::Present(String::from("'a.b=\"1\"'")),
        true,
    ));
    assert!(present.contains(
        "\"configParameters\":{\"kind\":\"present\",\"value\":\"'a.b=\\\"1\\\"'\"}"
    ));
    assert!(present.ends_with("\"skipHookLock\":true}"));
}

/// Parsing reads back what encoding wrote and refuses plans of another shape.
#[test]
fn plans_round_trip_and_refuse_other_shapes() {
    let plan: HookDispatchPlan = sample_plan(ConfigParameters::Present(String::from("'x'")), false);
    assert_eq!(parse_hook_dispatch_plan(encode_hook_dispatch_plan(&plan).as_str()), Some(plan));
    let other_version: String = encode_hook_dispatch_plan(&sample_plan(ConfigParameters::Absent, false))
        .replace("\"schemaVersion\":1", "\"schemaVersion\":2");
    assert_eq!(parse_hook_dispatch_plan(other_version.as_str()), None);
    let unknown_event: String = encode_hook_dispatch_plan(&sample_plan(ConfigParameters::Absent, false))
        .replace("\"commit-msg\"", "\"pre-push\"");
    assert_eq!(parse_hook_dispatch_plan(unknown_event.as_str()), None);
    assert_eq!(parse_hook_dispatch_plan("{}"), None);
    assert_eq!(parse_hook_dispatch_plan("not json"), None);
}

/// A plan written before the hooks path was dropped still parses, and a new plan records no hooks path.
#[test]
fn plans_with_a_legacy_hooks_path_field_still_parse() {
    let current: String = encode_hook_dispatch_plan(&sample_plan(ConfigParameters::Absent, false));
    assert!(!current.contains("hooksPath"), "{current}");
    let legacy: String = current.replace("\"gitPath\":\"/usr/bin/git\",", "\"gitPath\":\"/usr/bin/git\",\"hooksPath\":\"/repo/.git/hooks\",");
    assert!(legacy.contains("hooksPath"));
    assert_eq!(
        parse_hook_dispatch_plan(legacy.as_str()),
        Some(sample_plan(ConfigParameters::Absent, false))
    );
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

/// Real Git supplies the switches and the caller's `-c` values.
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
        global_arguments: &[OsString::from("-c"), OsString::from("x.y=2")],
        common_directory: common.as_path(),
        worktree_root: repository_root.as_path(),
        lease: "{\"directory\":\"/d\",\"token\":\"t\"}",
        concurrent_commits: true,
        inherited_lease_valid: false,
        environment: &[],
        snapshot: None,
    })
    .expect("plan");
    assert_eq!(plan.disabled_events, vec![String::from("pre-commit")]);
    assert_eq!(plan.config_parameters, ConfigParameters::Present(String::from("'x.y=2'")));
    assert!(plan.skip_hook_lock);
    assert_eq!(plan.hook_lock_path, common.join("cli-git").join("hook.lock").to_str().expect("UTF-8"));
    remove(root.as_path());
}

/// The inputs of a plan that only the configuration decides.
fn plan_inputs<'a>(context: &'a GitContext, repository_root: &'a Path) -> HookPlanInputs<'a> {
    return HookPlanInputs {
        context,
        cwd: repository_root,
        global_arguments: &[],
        common_directory: repository_root,
        worktree_root: repository_root,
        lease: "lease",
        concurrent_commits: false,
        inherited_lease_valid: false,
        environment: &[],
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
        global_arguments: &[],
        common_directory: repository_root.join(".git").as_path(),
        worktree_root: repository_root.as_path(),
        lease: "lease",
        concurrent_commits: false,
        inherited_lease_valid: true,
        environment: &[],
        snapshot: None,
    })
    .expect("plan");
    assert!(plan.skip_hook_lock);
    assert_eq!(plan.config_parameters, ConfigParameters::Absent);
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
        global_arguments: &[],
        common_directory: repository_root.join(".git").as_path(),
        worktree_root: repository_root.as_path(),
        lease: "lease",
        concurrent_commits: false,
        inherited_lease_valid: false,
        environment: &[],
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
