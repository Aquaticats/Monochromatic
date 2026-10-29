//! Controls for the dispatch plan: quoting, config combination, the JSON spelling and real-Git reads.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// One environment pair.
fn pair(name: &str, value: &str) -> (OsString, OsString) {
    return (OsString::from(name), OsString::from(value));
}

/// A plan with fixed values, for spelling tests.
fn sample_plan(config_parameters: ConfigParameters, skip_hook_lock: bool) -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from("/usr/bin/git"),
        hooks_path: String::from("/repo/.git/hooks"),
        disabled_events: vec![String::from("commit-msg")],
        config_parameters,
        worktree_root: String::from("/repo"),
        lease: String::from("{\"directory\":\"/d\",\"token\":\"t\"}"),
        hook_lock_path: String::from("/repo/.git/cli-git/hook.lock"),
        skip_hook_lock,
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
        "{\"schemaVersion\":1,\"gitPath\":\"/usr/bin/git\",\"hooksPath\":\"/repo/.git/hooks\",\
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

/// The hook lock sits under the common directory, and the hooks path normalizes like Node's `path`.
#[test]
fn lock_and_hooks_paths_normalize_lexically() {
    assert_eq!(
        hook_lock_directory(Path::new("/repo/.git")),
        PathBuf::from("/repo/.git/cli-git/hook.lock")
    );
    assert_eq!(
        default_hooks_directory(Path::new("/repo/.git/")),
        PathBuf::from("/repo/.git/hooks")
    );
    assert_eq!(
        resolve_hooks_path(Path::new("/repo/./wt//"), Path::new("../shared/./hooks")),
        PathBuf::from("/repo/shared/hooks")
    );
    assert_eq!(
        resolve_hooks_path(Path::new("/repo"), Path::new("/abs/../x/")),
        PathBuf::from("/x")
    );
    assert_eq!(
        resolve_hooks_path(Path::new("/"), Path::new("../../up")),
        PathBuf::from("/up")
    );
}

/// Real Git supplies the hooks path, the switches and the caller's `-c` values.
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
    })
    .expect("plan");
    assert_eq!(plan.disabled_events, vec![String::from("pre-commit")]);
    assert_eq!(plan.hooks_path, common.join("hooks").to_str().expect("UTF-8"));
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

/// A relative `core.hooksPath` resolves under the worktree root, and an inherited valid lease skips the lock.
#[test]
fn plans_resolve_relative_hooks_paths_and_inherited_leases() {
    let root: PathBuf = fixture("hook-plan-relative");
    let repository_root: PathBuf = repository(root.as_path(), "repo");
    git(
        repository_root.as_path(),
        &["config", "core.hooksPath", "../shared-hooks"],
    );
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
    })
    .expect("plan");
    let expected: PathBuf = resolve_hooks_path(repository_root.as_path(), Path::new("../shared-hooks"));
    assert_eq!(plan.hooks_path, expected.to_str().expect("UTF-8"));
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
