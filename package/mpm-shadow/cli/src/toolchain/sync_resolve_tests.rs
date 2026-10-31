//! Unit tests of the CI detection and of `sync` under a lock policy:
//! a local sync resolves a plan entry the lock lacks (or holds for an older request) and only that entry,
//! a frozen sync (the flag,
//! or CI) refuses and names `mpm-shadow update <ids>`,
//! and what `lock` fixes offline,
//! or a hand-edited lock,
//! is refused as before.

use std::fs;

use clap::Command;

use super::ci_detect::is_ci;
use super::commands::subcommands;
use super::error::ErrorKind;
use super::fakes::{FakeHttp, FakeRunner};
use super::lock_cmd::{LockMode, run_lock};
use super::scope::EnvMap;
use super::sync::{FrozenReason, LockPolicy, format_sync_report, run_sync, run_sync_with};
use super::update_named_tests::{VITE_LINE, VITE_URL, WRANGLER_LINE, add_vite, locked_scope, moved_network};
use super::update_tests::{CRATES_URL, NPM_URL, linux, read_lock, serve_registries};

//region CI detection

fn ci(pairs: &[(&str, &str)]) -> bool {
    return is_ci(&EnvMap::from_pairs(pairs));
}

#[test]
fn the_variables_ci_info_treats_as_ci_make_a_ci_environment() {
    let allowed: [&[(&str, &str)]; 12] = [
        &[("CI", "true")],
        &[("CI", "1")],
        &[("GITHUB_ACTIONS", "true")],
        &[("GITLAB_CI", "true")],
        &[("BUILD_NUMBER", "17")],
        &[("RUN_ID", "x")],
        &[("CONTINUOUS_INTEGRATION", "true")],
        &[("CI_NAME", "codeship")],
        &[("CI", "woodpecker")],
        &[("NOW_BUILDER", "1")],
        &[("VERCEL", "1")],
        &[("NODE", "/app/.heroku/node/bin/node")],
    ];
    for pairs in allowed {
        assert!(ci(pairs), "{pairs:?} should be CI");
    }
}

#[test]
fn an_ordinary_shell_and_an_explicit_ci_false_are_not_ci() {
    let rejected: [&[(&str, &str)]; 6] = [
        &[],
        &[("HOME", "/h"), ("PATH", "/bin")],
        &[("JENKINS_URL", "http://ci")],
        &[("TASK_ID", "t")],
        &[("NODE", "/usr/bin/node")],
        &[("CI", "")],
    ];
    for pairs in rejected {
        assert!(!ci(pairs), "{pairs:?} should not be CI");
    }
    assert!(!ci(&[("CI", "false"), ("GITHUB_ACTIONS", "true"), ("BUILD_NUMBER", "9")]), "CI=false turns detection off");
}

//endregion

//region sync under a policy

fn names_in(error: &str) -> bool {
    return error.contains("mpm-shadow update npm:vite");
}

#[test]
fn a_local_sync_resolves_a_new_entry_alone_and_keeps_every_other_row_identical() {
    let (_directory, scope, env) = locked_scope();
    let before = read_lock(&scope);
    add_vite(&scope);
    let http = moved_network();
    let runner = FakeRunner::new();
    let report = run_sync_with(&scope, &env, &runner, &http, &linux(), LockPolicy::Resolve).expect("synced");
    assert_eq!(report.resolved, vec![("npm:vite".to_string(), "7.1.0".to_string())]);
    let requests = http.requests();
    assert!(requests.contains(&VITE_URL.to_string()), "{requests:?}");
    assert!(!requests.contains(&NPM_URL.to_string()) && !requests.contains(&CRATES_URL.to_string()), "no other tool is looked up: {requests:?}");
    let after = read_lock(&scope);
    assert_eq!(after.tool("npm:vite").expect("vite").version, "7.1.0");
    for id in ["cargo:fd-find", "npm:wrangler", "pipx:slopo", "rust"] {
        assert_eq!(after.tool(id), before.tool(id), "{id} did not move although the registries did");
    }
    run_lock(&scope, LockMode::Check).expect("the written lock passes the gate");
    assert!(format_sync_report(&report).contains("toolchain sync: locked npm:vite 7.1.0"), "{}", format_sync_report(&report));
    assert!(!runner.calls().is_empty(), "the install step ran after the lock was written");
}

#[test]
fn a_frozen_sync_refuses_a_new_entry_names_update_with_its_id_and_changes_nothing() {
    for reason in [FrozenReason::Flag, FrozenReason::Ci] {
        let (_directory, scope, env) = locked_scope();
        add_vite(&scope);
        let lock_before = fs::read_to_string(&scope.lock_file).expect("read");
        let http = moved_network();
        let runner = FakeRunner::new();
        let error = run_sync_with(&scope, &env, &runner, &http, &linux(), LockPolicy::Frozen(reason)).expect_err("refused");
        assert_eq!(error.kind, ErrorKind::Drift);
        assert!(names_in(&error.message), "{reason:?}: {error}");
        match reason {
            FrozenReason::Flag => assert!(error.message.contains("`--frozen-lock` was given"), "{error}"),
            FrozenReason::Ci => assert!(error.message.contains("a CI environment was detected"), "{error}"),
        }
        assert!(http.requests().is_empty() && runner.calls().is_empty(), "nothing is looked up or installed");
        assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), lock_before);
    }
}

#[test]
fn the_plain_run_sync_is_frozen() {
    let (_directory, scope, env) = locked_scope();
    add_vite(&scope);
    let http = moved_network();
    let error = run_sync(&scope, &env, &FakeRunner::new(), &http, &linux()).expect_err("refused");
    assert!(names_in(&error.message), "{error}");
    assert!(http.requests().is_empty());
}

#[test]
fn a_changed_asset_option_re_resolves_only_that_tool() {
    let (_directory, scope, env) = locked_scope();
    let before = read_lock(&scope);
    let plan = fs::read_to_string(&scope.plan_file).expect("read the plan");
    assert!(plan.contains(WRANGLER_LINE.trim_end_matches('\n')), "the fixture plan changed shape");
    fs::write(
        &scope.plan_file,
        plan.replace("\"bins\": [\"wrangler\"] }", "\"bins\": [\"wrangler\"], \"options\": { \"asset\": \"wrangler-*\" } }"),
    )
    .expect("change the asset option");
    let http = moved_network();
    let report = run_sync_with(&scope, &env, &FakeRunner::new(), &http, &linux(), LockPolicy::Resolve).expect("synced");
    assert_eq!(report.resolved, vec![("npm:wrangler".to_string(), "4.157.0".to_string())]);
    let after = read_lock(&scope);
    assert_eq!(after.tool("npm:wrangler").expect("wrangler").declaration.options.asset.as_deref(), Some("wrangler-*"));
    for id in ["cargo:fd-find", "pipx:slopo", "rust"] {
        assert_eq!(after.tool(id), before.tool(id), "{id} is not the changed tool and must not move");
    }
    assert!(!http.requests().contains(&CRATES_URL.to_string()));
    run_lock(&scope, LockMode::Check).expect("plan and lock agree");

    // The same edit under a frozen policy is refused, naming the tool.
    let (_other, frozen_scope, frozen_env) = locked_scope();
    let frozen_plan = fs::read_to_string(&frozen_scope.plan_file).expect("read the plan");
    fs::write(
        &frozen_scope.plan_file,
        frozen_plan.replace("\"bins\": [\"wrangler\"] }", "\"bins\": [\"wrangler\"], \"options\": { \"asset\": \"wrangler-*\" } }"),
    )
    .expect("change the asset option");
    let error = run_sync_with(&frozen_scope, &frozen_env, &FakeRunner::new(), &FakeHttp::new(), &linux(), LockPolicy::Frozen(FrozenReason::Ci))
        .expect_err("refused");
    assert!(error.message.contains("mpm-shadow update npm:wrangler") && error.message.contains("options.asset"), "{error}");
}

#[test]
fn a_re_edited_entry_that_resolves_to_the_same_version_is_still_locked_and_reported() {
    let (_directory, scope, env) = locked_scope();
    let before = read_lock(&scope);
    let plan = fs::read_to_string(&scope.plan_file).expect("read the plan");
    fs::write(
        &scope.plan_file,
        plan.replace("\"bins\": [\"fd\"] }", "\"bins\": [\"fd\"], \"options\": { \"asset\": \"fd-*\" } }"),
    )
    .expect("change the asset option");
    let quiet = FakeHttp::new();
    serve_registries(&quiet, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let report = run_sync_with(&scope, &env, &FakeRunner::new(), &quiet, &linux(), LockPolicy::Resolve).expect("synced");
    assert_eq!(report.resolved, vec![("cargo:fd-find".to_string(), "10.5.0".to_string())]);
    assert_eq!(quiet.requests(), vec![CRATES_URL.to_string()], "only the re-edited tool is looked up");
    let after = read_lock(&scope);
    assert_eq!(after.tool("cargo:fd-find").expect("fd").declaration.options.asset.as_deref(), Some("fd-*"));
    assert_eq!(after.tool("cargo:fd-find").expect("fd").version, "10.5.0");
    assert_eq!(after.tool("npm:wrangler"), before.tool("npm:wrangler"));
    run_lock(&scope, LockMode::Check).expect("plan and lock agree");
}

#[test]
fn a_failed_lookup_of_the_new_entry_stops_before_anything_is_written_or_installed() {
    let (_directory, scope, env) = locked_scope();
    add_vite(&scope);
    let lock_before = fs::read_to_string(&scope.lock_file).expect("read");
    let http = moved_network();
    http.fail(VITE_URL, "timed out");
    let runner = FakeRunner::new();
    let error = run_sync_with(&scope, &env, &runner, &http, &linux(), LockPolicy::Resolve).expect_err("stopped");
    assert_eq!(error.kind, ErrorKind::Failure);
    assert!(error.message.contains("npm:vite") && error.message.contains("timed out"), "{error}");
    assert!(error.message.contains("wrote nothing and installed nothing"), "{error}");
    assert!(runner.calls().is_empty());
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), lock_before);
}

#[test]
fn what_lock_fixes_offline_is_still_refused_naming_lock_under_either_policy() {
    for policy in [LockPolicy::Resolve, LockPolicy::Frozen(FrozenReason::Ci)] {
        let (_directory, scope, env) = locked_scope();
        let plan = fs::read_to_string(&scope.plan_file).expect("read the plan");
        fs::write(&scope.plan_file, plan.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]")).expect("edit the bins");
        let http = moved_network();
        let error = run_sync_with(&scope, &env, &FakeRunner::new(), &http, &linux(), policy).expect_err("refused");
        assert!(error.message.contains("`mpm-shadow lock`"), "{policy:?}: {error}");
        assert!(http.requests().is_empty(), "{policy:?}: nothing is looked up for an offline fix");
    }
}

#[test]
fn a_hand_edited_lock_is_refused_under_either_policy_before_any_lookup() {
    for policy in [LockPolicy::Resolve, LockPolicy::Frozen(FrozenReason::Flag)] {
        let (_directory, scope, env) = locked_scope();
        add_vite(&scope);
        let text = fs::read_to_string(&scope.lock_file).expect("read");
        fs::write(&scope.lock_file, text.replace("\"version\": \"4.156.0\"", "\"version\": \"0.0.1\"")).expect("hand edit");
        let http = moved_network();
        let error = run_sync_with(&scope, &env, &FakeRunner::new(), &http, &linux(), policy).expect_err("refused");
        assert!(error.message.contains("edited by hand"), "{policy:?}: {error}");
        assert!(http.requests().is_empty());
    }
}

/// pnpm:
/// the CI default of `--frozen-lockfile` is "true,
/// if a lockfile is present",
/// so with no lockfile `pnpm install` resolves and writes one,
/// in CI too.
#[test]
fn a_missing_lock_is_resolved_and_written_in_full_when_not_explicitly_frozen() {
    for policy in [LockPolicy::Resolve, LockPolicy::Frozen(FrozenReason::Ci)] {
        let (_directory, scope, env) = locked_scope();
        fs::remove_file(&scope.lock_file).expect("delete the lock");
        let http = moved_network();
        let runner = FakeRunner::new();
        let report = run_sync_with(&scope, &env, &runner, &http, &linux(), policy).expect("synced");
        let mut ids: Vec<String> = report.resolved.iter().map(|(id, _)| return id.clone()).collect();
        ids.sort();
        assert_eq!(ids, vec!["cargo:fd-find", "npm:wrangler", "pipx:slopo", "rust"], "{policy:?}: every entry is resolved");
        let lock = read_lock(&scope);
        assert_eq!(lock.tool("npm:wrangler").expect("wrangler").version, "4.157.0", "{policy:?}: the newest versions are locked");
        run_lock(&scope, LockMode::Check).expect("the written lock passes the gate");
        assert!(!runner.calls().is_empty(), "{policy:?}: the install step ran after the lock was written");
    }
}

/// pnpm:
/// `pnpm install --frozen-lockfile` with no lockfile fails with ERR_PNPM_NO_LOCKFILE.
#[test]
fn a_missing_lock_is_refused_when_frozen_by_the_flag() {
    let (_directory, scope, env) = locked_scope();
    fs::remove_file(&scope.lock_file).expect("delete the lock");
    let http = moved_network();
    let runner = FakeRunner::new();
    for result in [
        run_sync_with(&scope, &env, &runner, &http, &linux(), LockPolicy::Frozen(FrozenReason::Flag)),
        run_sync(&scope, &env, &runner, &http, &linux()),
    ] {
        let error = result.expect_err("refused");
        assert_eq!(error.kind, ErrorKind::Drift);
        assert!(error.message.contains("`--frozen-lock` forbids sync from creating it"), "{error}");
        assert!(error.message.contains("run `mpm-shadow update`"), "{error}");
    }
    assert!(!scope.lock_file.exists(), "nothing was written");
    assert!(http.requests().is_empty() && runner.calls().is_empty());
}

//endregion

//region the grammar

fn sync_command() -> Command {
    return subcommands().into_iter().find(|command| return command.get_name() == "sync").expect("a sync verb");
}

#[test]
fn the_sync_grammar_takes_either_frozen_switch_but_not_both() {
    let plain = sync_command().try_get_matches_from(["sync"]).expect("plain");
    assert!(!plain.get_flag("frozen-lock") && !plain.get_flag("no-frozen-lock"));
    assert!(sync_command().try_get_matches_from(["sync", "--frozen-lock"]).expect("frozen").get_flag("frozen-lock"));
    assert!(sync_command().try_get_matches_from(["sync", "--no-frozen-lock", "--global"]).expect("unfrozen").get_flag("no-frozen-lock"));
    assert!(sync_command().try_get_matches_from(["sync", "--frozen-lock", "--no-frozen-lock"]).is_err());
}

//endregion

#[test]
fn the_fixture_lines_are_the_ones_the_plan_edits_replace() {
    assert!(WRANGLER_LINE.contains("npm:wrangler") && VITE_LINE.contains("npm:vite"));
}
