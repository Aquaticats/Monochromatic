//! Unit tests of `update <tool>...`:
//! the pattern matcher and the selection with allowed and rejected names,
//! and the command against a fake network:
//! only the named rows move,
//! a new plan entry is resolved alone by naming it,
//! and anything the lock cannot hold that the names left out is refused before a lookup.

use std::fs;

use clap::Command;

use super::agree::compare;
use super::commands::subcommands;
use super::error::ErrorKind;
use super::fakes::{FakeHttp, FakeRunner};
use super::lock::parse_lock;
use super::lock_cmd::{LockMode, run_lock};
use super::plan::{Plan, parse_plan};
use super::scope::{EnvMap, Scope};
use super::update::{OutcomeKind, format_update_report, run_update, run_update_for};
use super::update_select::{matches_pattern, select_tools};
use super::update_tests::{CRATES_URL, NPM_URL, REGISTRY_PLAN, linux, outcome_of, read_lock, registry_scope, serve_registries};

pub(super) const VITE_URL: &str = "https://registry.npmjs.org/vite";

pub(super) const WRANGLER_LINE: &str =
    "    \"npm:wrangler\": { \"request\": \"npm:wrangler@latest\", \"route\": \"mpm-pnpm\", \"manager\": \"pnpm\", \"packageId\": \"wrangler\", \"bins\": [\"wrangler\"] },\n";

pub(super) const VITE_LINE: &str =
    "    \"npm:vite\": { \"request\": \"npm:vite@latest\", \"route\": \"mpm-pnpm\", \"manager\": \"pnpm\", \"packageId\": \"vite\", \"bins\": [\"vite\"] },\n";

pub(super) fn names(given: &[&str]) -> Vec<String> {
    return given.iter().map(|name| return (*name).to_string()).collect();
}

fn registry_plan() -> Plan {
    return parse_plan(REGISTRY_PLAN, "registry plan").expect("the plan parses");
}

/// The state every named-update test starts from:
/// one full update against the first registry answers.
pub(super) fn locked_scope() -> (tempfile::TempDir, Scope, EnvMap) {
    let (directory, scope, env) = registry_scope();
    let first = FakeHttp::new();
    serve_registries(&first, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &first, &linux(), false).expect("the first update");
    return (directory, scope, env);
}

/// Every registry has moved on.
pub(super) fn moved_network() -> FakeHttp {
    let http = FakeHttp::new();
    serve_registries(&http, "10.6.0", "4.157.0", "0.9.1", "2026-11-03");
    http.serve(VITE_URL, b"{\"dist-tags\": {\"latest\": \"7.1.0\"}}");
    return http;
}

pub(super) fn add_vite(scope: &Scope) {
    let plan = fs::read_to_string(&scope.plan_file).expect("read the plan");
    assert!(plan.contains(WRANGLER_LINE), "the fixture plan changed shape");
    fs::write(&scope.plan_file, plan.replace(WRANGLER_LINE, &format!("{WRANGLER_LINE}{VITE_LINE}"))).expect("add vite");
}

//region the pattern matcher

#[test]
fn a_pattern_matches_the_ids_it_describes() {
    let allowed: [(&str, &str); 8] = [
        ("npm:wrangler", "npm:wrangler"),
        ("npm:*", "npm:wrangler"),
        ("*", "go:github.com/charmbracelet/crush"),
        ("*:wrangler", "npm:wrangler"),
        ("go:*/crush", "go:github.com/charmbracelet/crush"),
        ("a*b*c", "aXXbYYc"),
        ("a*", "a"),
        ("*a*", "banana"),
    ];
    for (pattern, text) in allowed {
        assert!(matches_pattern(pattern, text), "{pattern} should match {text}");
    }
}

#[test]
fn a_pattern_does_not_match_what_it_does_not_describe() {
    let rejected: [(&str, &str); 7] = [
        ("npm:wrangler", "npm:wrangler2"),
        ("npm:wrang", "npm:wrangler"),
        ("npm:*", "pipx:slopo"),
        ("a*b", "ac"),
        ("*c", "cab"),
        ("", "a"),
        ("a", ""),
    ];
    for (pattern, text) in rejected {
        assert!(!matches_pattern(pattern, text), "{pattern} should not match {text}");
    }
    assert!(matches_pattern("", ""), "the empty pattern matches the empty text only");
}

#[test]
fn a_pathological_pattern_finishes_on_a_long_text() {
    let text: String = "a".repeat(20_000);
    let pattern: String = format!("{}b", "a*".repeat(200));
    assert!(!matches_pattern(&pattern, &text));
}

//endregion

//region the selection

fn selected(patterns: &[&str]) -> Vec<String> {
    return select_tools(&registry_plan(), &names(patterns)).expect("a selection").into_iter().collect();
}

#[test]
fn no_pattern_selects_every_tool_and_names_select_only_those() {
    assert_eq!(selected(&[]).len(), 4);
    assert_eq!(selected(&["npm:wrangler"]), vec!["npm:wrangler".to_string()]);
    assert_eq!(selected(&["npm:wrangler", "npm:wrangler", "rust"]), vec!["npm:wrangler".to_string(), "rust".to_string()]);
}

#[test]
fn a_glob_selects_by_pattern_and_a_bang_excludes() {
    assert_eq!(selected(&["*:*"]), vec!["cargo:fd-find".to_string(), "npm:wrangler".to_string(), "pipx:slopo".to_string()]);
    assert_eq!(selected(&["*:*", "!npm:wrangler"]), vec!["cargo:fd-find".to_string(), "pipx:slopo".to_string()]);
    assert_eq!(selected(&["!rust"]).len(), 3, "only exclusions select everything else");
    assert!(selected(&["rust", "!rust"]).is_empty(), "an exclusion can leave nothing selected");
}

#[test]
fn an_unknown_name_is_refused_with_every_valid_id_listed() {
    for patterns in [vec!["npm:wranglr"], vec!["npm:wrangler", "nope"], vec!["!nope"], vec!["nope:*"], vec![""]] {
        let error = select_tools(&registry_plan(), &names(&patterns)).expect_err("refused");
        assert_eq!(error.kind, ErrorKind::Config, "{patterns:?}");
        assert_eq!(error.exit_code(), 2);
        assert!(error.message.contains("update names no tool of the plan"), "{error}");
        for id in ["cargo:fd-find", "npm:wrangler", "pipx:slopo", "rust"] {
            assert!(error.message.contains(id), "{id} is a valid id and must be listed: {error}");
        }
    }
}

//endregion

//region the command

#[test]
fn naming_a_tool_moves_only_that_row_and_looks_up_only_that_tool() {
    let (_directory, scope, env) = locked_scope();
    let before = read_lock(&scope);
    let http = moved_network();
    let report = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["npm:wrangler"])).expect("updated");
    assert!(report.wrote);
    assert_eq!(
        outcome_of(&report, "npm:wrangler"),
        &OutcomeKind::Updated { old_version: "4.156.0".to_string(), new_version: "4.157.0".to_string() }
    );
    for id in ["cargo:fd-find", "pipx:slopo", "rust"] {
        assert_eq!(outcome_of(&report, id), &OutcomeKind::Kept, "{id}");
    }
    assert_eq!(http.requests(), vec![NPM_URL.to_string()], "no other tool is looked up");

    let after = read_lock(&scope);
    assert_eq!(after.tool("npm:wrangler").expect("wrangler").version, "4.157.0");
    for id in ["cargo:fd-find", "pipx:slopo", "rust"] {
        assert_eq!(after.tool(id), before.tool(id), "{id} is byte-identical in meaning: version, artifacts and declaration");
    }
    run_lock(&scope, LockMode::Check).expect("the written lock passes the gate");
    let text = format_update_report(&report, false);
    assert!(text.contains("kept      cargo:fd-find"), "{text}");
    assert!(text.contains("updated   npm:wrangler  4.156.0 -> 4.157.0"), "{text}");
    assert!(text.contains("wrote toolchain.lock.json (1 tools moved"), "{text}");
}

#[test]
fn a_glob_updates_every_matching_row_and_an_exclusion_leaves_one_out() {
    let (_directory, scope, env) = locked_scope();
    let http = moved_network();
    let report = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["*:*", "!pipx:slopo"])).expect("updated");
    assert!(matches!(outcome_of(&report, "cargo:fd-find"), OutcomeKind::Updated { .. }));
    assert!(matches!(outcome_of(&report, "npm:wrangler"), OutcomeKind::Updated { .. }));
    assert_eq!(outcome_of(&report, "pipx:slopo"), &OutcomeKind::Kept);
    assert_eq!(outcome_of(&report, "rust"), &OutcomeKind::Kept);
    assert_eq!(read_lock(&scope).tool("pipx:slopo").expect("slopo").version, "0.9.0");
}

#[test]
fn check_with_a_name_reports_only_that_tool_and_writes_nothing() {
    let (_directory, scope, env) = locked_scope();
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let http = moved_network();
    let runner = FakeRunner::new();
    let report = run_update_for(&scope, &env, &runner, &http, &linux(), true, &names(&["pipx:slopo"])).expect("checked");
    assert!(report.has_movement(), "slopo would move");
    assert!(!report.wrote);
    assert_eq!(outcome_of(&report, "cargo:fd-find"), &OutcomeKind::Kept, "fd-find moved on the network but was not named");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
    assert!(runner.calls().is_empty());
    assert!(format_update_report(&report, true).contains("1 tools would move"));

    let quiet = FakeHttp::new();
    serve_registries(&quiet, "10.6.0", "4.156.0", "0.9.0", "2026-10-30");
    let unmoved = run_update_for(&scope, &env, &FakeRunner::new(), &quiet, &linux(), true, &names(&["npm:wrangler"])).expect("checked");
    assert!(!unmoved.has_movement(), "wrangler did not move, so the check exits clean although fd-find did: {:?}", unmoved.outcomes);
}

#[test]
fn an_unknown_name_is_refused_before_any_lookup_and_changes_nothing() {
    let (_directory, scope, env) = locked_scope();
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let http = moved_network();
    let error = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["npm:wrangler", "npm:wranglr"]))
        .expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("npm:wranglr"), "{error}");
    assert!(error.message.contains("cargo:fd-find") && error.message.contains("pipx:slopo"), "{error}");
    assert!(http.requests().is_empty(), "nothing is looked up when a name is wrong, not even the valid one");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
}

#[test]
fn naming_a_new_plan_entry_resolves_it_alone_and_moves_no_other_row() {
    let (_directory, scope, env) = locked_scope();
    let before = read_lock(&scope);
    add_vite(&scope);
    assert!(run_lock(&scope, LockMode::Check).is_err(), "the lock does not hold vite yet");
    let http = moved_network();
    let report = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["npm:vite"])).expect("updated");
    assert_eq!(outcome_of(&report, "npm:vite"), &OutcomeKind::Added { version: "7.1.0".to_string() });
    assert_eq!(http.requests(), vec![VITE_URL.to_string()], "only the new entry is looked up");
    let after = read_lock(&scope);
    assert_eq!(after.tool("npm:vite").expect("vite").version, "7.1.0");
    for id in ["cargo:fd-find", "npm:wrangler", "pipx:slopo", "rust"] {
        assert_eq!(after.tool(id), before.tool(id), "{id} did not move although the registries did");
    }
    run_lock(&scope, LockMode::Check).expect("plan and lock agree after resolving the new entry alone");
}

#[test]
fn a_new_entry_the_names_left_out_is_refused_naming_it_and_looks_nothing_up() {
    let (_directory, scope, env) = locked_scope();
    add_vite(&scope);
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let http = moved_network();
    let error = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["npm:wrangler"])).expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("npm:vite"), "{error}");
    assert!(error.message.contains("mpm-shadow update ... npm:vite"), "{error}");
    assert!(http.requests().is_empty());
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
    let checked = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), true, &names(&["npm:wrangler"])).expect_err("check refuses too");
    assert!(checked.message.contains("npm:vite"), "{checked}");
}

#[test]
fn an_unnamed_tool_with_a_changed_pin_is_refused_but_an_unnamed_tool_with_a_changed_bin_list_is_carried() {
    let (_directory, scope, env) = locked_scope();
    let plan = fs::read_to_string(&scope.plan_file).expect("read the plan");
    fs::write(&scope.plan_file, plan.replace("\"npm:wrangler@latest\"", "\"npm:wrangler@3.0.0\"")).expect("pin wrangler");
    let http = moved_network();
    let error = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["pipx:slopo"])).expect_err("refused");
    assert!(error.message.contains("npm:wrangler"), "{error}");
    assert!(http.requests().is_empty());

    // Restore the pin and change only what installs and links: a row the lock can hold as it is.
    fs::write(&scope.plan_file, plan.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]")).expect("edit the bins");
    let report = run_update_for(&scope, &env, &FakeRunner::new(), &moved_network(), &linux(), false, &names(&["pipx:slopo"])).expect("updated");
    assert_eq!(outcome_of(&report, "cargo:fd-find"), &OutcomeKind::Kept);
    let lock = read_lock(&scope);
    assert_eq!(lock.tool("cargo:fd-find").expect("fd").version, "10.5.0", "the version stays");
    assert_eq!(lock.tool("cargo:fd-find").expect("fd").declaration.bins, vec!["fd".to_string(), "fdfind".to_string()], "the bins follow the plan");
    run_lock(&scope, LockMode::Check).expect("plan and lock agree");
}

#[test]
fn without_a_lock_a_named_update_must_name_every_tool_it_cannot_keep() {
    let (_directory, scope, env) = registry_scope();
    let http = moved_network();
    let error = run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &names(&["npm:wrangler"])).expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Drift);
    for id in ["cargo:fd-find", "pipx:slopo", "rust"] {
        assert!(error.message.contains(id), "{id}: {error}");
    }
    assert!(http.requests().is_empty());
    assert!(!scope.lock_file.exists());
    let everything = names(&["cargo:fd-find", "npm:wrangler", "pipx:slopo", "rust"]);
    run_update_for(&scope, &env, &FakeRunner::new(), &http, &linux(), false, &everything).expect("naming every tool writes the lock");
    assert!(scope.lock_file.exists());
}

#[test]
fn naming_every_tool_equals_a_full_update() {
    let (_directory, scope, env) = locked_scope();
    let full = FakeHttp::new();
    serve_registries(&full, "10.6.0", "4.157.0", "0.9.1", "2026-11-03");
    let first_lock = fs::read_to_string(&scope.lock_file).expect("read");
    run_update(&scope, &env, &FakeRunner::new(), &full, &linux(), false).expect("full update");
    let after_full = fs::read_to_string(&scope.lock_file).expect("read");
    fs::write(&scope.lock_file, &first_lock).expect("restore the first lock");
    let again = FakeHttp::new();
    serve_registries(&again, "10.6.0", "4.157.0", "0.9.1", "2026-11-03");
    run_update_for(&scope, &env, &FakeRunner::new(), &again, &linux(), false, &names(&["*"])).expect("by pattern");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), after_full);
}

//endregion

//region the refusals that now name update <ids>

#[test]
fn offline_lock_and_the_gate_name_update_with_the_new_tool_id() {
    let (_directory, scope, _env) = locked_scope();
    add_vite(&scope);
    for mode in [LockMode::Write, LockMode::Check] {
        let error = run_lock(&scope, mode).expect_err("refused");
        assert!(error.message.contains("`mpm-shadow update npm:vite`"), "{error}");
    }
    let loaded = parse_lock(&fs::read_to_string(&scope.lock_file).expect("read"), "lock").expect("parses");
    let verdict = compare(&registry_plan_with_vite(), &loaded);
    assert_eq!(verdict.ids_needing_update(), vec!["npm:vite".to_string()]);
}

fn registry_plan_with_vite() -> Plan {
    let text = REGISTRY_PLAN.replace(WRANGLER_LINE, &format!("{WRANGLER_LINE}{VITE_LINE}"));
    return parse_plan(&text, "plan with vite").expect("parses");
}

//endregion

//region the grammar

fn update_command() -> Command {
    return subcommands().into_iter().find(|command| return command.get_name() == "update").expect("an update verb");
}

#[test]
fn the_grammar_takes_any_number_of_tool_ids_with_global() {
    let none = update_command().try_get_matches_from(["update"]).expect("no names");
    assert!(none.get_many::<String>("tool").is_none());
    let several = update_command()
        .try_get_matches_from(["update", "npm:vite", "-g", "pipx:slopo"])
        .expect("names around the flags");
    let given: Vec<&String> = several.get_many::<String>("tool").expect("names").collect();
    assert_eq!(given, vec!["npm:vite", "pipx:slopo"]);
    assert!(several.get_flag("global"), "-g is pnpm's short for --global");
}

#[test]
fn the_grammar_rejects_an_unknown_flag_and_the_old_check_switch() {
    assert!(update_command().try_get_matches_from(["update", "--latest"]).is_err());
    assert!(update_command().try_get_matches_from(["update", "--check"]).is_err(), "outdated is the report");
}

#[test]
fn outdated_takes_the_same_names_as_update() {
    let outdated = subcommands().into_iter().find(|command| return command.get_name() == "outdated").expect("an outdated verb");
    let matches = outdated.try_get_matches_from(["outdated", "npm:*", "!npm:vite", "--global"]).expect("names and a flag");
    let given: Vec<&String> = matches.get_many::<String>("tool").expect("names").collect();
    assert_eq!(given, vec!["npm:*", "!npm:vite"]);
}

//endregion

#[test]
fn the_fixture_registry_urls_are_the_ones_the_resolvers_ask_for() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("updated");
    let requests = http.requests();
    assert!(requests.contains(&NPM_URL.to_string()) && requests.contains(&CRATES_URL.to_string()), "{requests:?}");
}
