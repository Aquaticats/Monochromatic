//! Unit tests of what an `install` argument names and of adding it to a plan:
//! the entries that land,
//! the requests that are refused,
//! and the files a stopped run leaves behind.
//! Every test works in a scope of its own (a plan,
//! a lock,
//! a toolchain root and a `HOME` under a disposable directory),
//! with the network and the programs stood in for,
//! so nothing here reads the real plan,
//! writes the real toolchain or reaches the real registries.

use std::fs;

use super::context::{Context, Host};
use super::error::{ErrorKind, ToolchainError};
use super::fakes::{FakeHttp, FakeRunner};
use super::install_argument::{AddRefusal, Named, add_named_tools, named, nothing_stops_the_run};
use super::lock::{Lock, render_lock};
use super::model::{LockedTool, OsFamily};
use super::plan::{Plan, parse_plan};
use super::publish::revision_of;
use super::scope::{EnvMap, Scope, global_scope, repo_scope};
use super::sync::{FrozenReason, LockPolicy, run_sync_with, run_sync_with_named};

/// The plan of a fixture scope:
/// a comment above the one entry it holds,
/// a trailing comma,
/// and an entry order of its own.
const PLAN: &str = r#"// The plan of this fixture: people write these comments.
{
  "schemaVersion": 1,
  "tools": {
    // tokei counts lines; the numbers are compared while it is pinned.
    "tokei": {"request": "cargo:tokei@latest", "route": "cargo-binstall", "bins": ["tokei"]},
    // Nothing follows this note; it is here to be kept.
  },
}
"#;

/// The version the fixture lock holds every tool at.
const VERSION: &str = "1.0.0";

/// The crates.io document for `fastmod`,
/// as the resolver and the binaries lookup read it.
const FASTMOD_CRATE: &str = r#"{
  "crate": { "id": "fastmod", "max_stable_version": "0.4.4" },
  "versions": [ { "num": "0.4.4", "bin_names": ["fastmod"] } ]
}"#;

/// The crates.io document for `tokei`.
const TOKEI_CRATE: &str = r#"{
  "crate": { "id": "tokei", "max_stable_version": "15.0.0" },
  "versions": [ { "num": "15.0.0", "bin_names": ["tokei"] } ]
}"#;

/// A lock for `plan`: every tool at `VERSION`, as a hand edit and `update` would leave it.
fn lock_for(plan: &Plan) -> Lock {
    let mut tools: Vec<LockedTool> = Vec::new();
    for declaration in &plan.tools {
        tools.push(LockedTool { declaration: declaration.clone(), version: VERSION.to_string(), artifacts: Vec::new() });
    }
    return Lock { tools };
}

/// What a test needs: a scope on disk, its environment, the fakes and the host.
struct Fixture {
    /// Keeps the disposable directory alive until the fixture drops.
    directory: tempfile::TempDir,
    /// The scope the commands work in.
    scope: Scope,
    /// The environment with `HOME` inside the directory.
    env: EnvMap,
    /// The recording stand-in for every program.
    runner: FakeRunner,
    /// The table-driven stand-in for the network.
    http: FakeHttp,
    /// A Linux x64 machine.
    host: Host,
}

/// `PLAN` with the two installers the npm and the Go routes run,
/// as this repository's plan and the machine's plan both hold them.
fn plan_with_installers() -> String {
    return PLAN.replace(
        "    // Nothing follows",
        "    \"go\": {\"request\": \"go@1.22.0\", \"route\": \"archive\", \"bins\": [\"go\", \"gofmt\"]},\n    \"pnpm\": {\"request\": \"github:pnpm/pnpm@10.0.0\", \"route\": \"bin-sourced\", \"bins\": [\"pnpm\"]},\n    // Nothing follows",
    );
}

impl Fixture {
    /// A repository scope with `PLAN` and the lock it implies.
    fn repo() -> Fixture {
        return Fixture::repo_with(PLAN);
    }

    /// A repository scope whose plan also holds the npm and Go installers.
    fn repo_with_installers() -> Fixture {
        return Fixture::repo_with(&plan_with_installers());
    }

    /// A repository scope with `plan` and the lock it implies.
    fn repo_with(plan: &str) -> Fixture {
        let directory = crate::test_scratch::scratch();
        let home = directory.path().join("home");
        let work = directory.path().join("work");
        fs::create_dir_all(&home).expect("make the home");
        fs::create_dir_all(&work).expect("make the worktree");
        let env = EnvMap::from_pairs(&[("HOME", home.to_str().expect("utf-8"))]);
        let scope = repo_scope(&work, &env).expect("a scope");
        return Fixture::on(directory, scope, env, plan);
    }

    /// The machine-global scope with `PLAN` and the lock it implies.
    fn global() -> Fixture {
        let directory = crate::test_scratch::scratch();
        let home = directory.path().join("home");
        fs::create_dir_all(&home).expect("make the home");
        let env = EnvMap::from_pairs(&[("HOME", home.to_str().expect("utf-8"))]);
        let scope = global_scope(&env).expect("a scope");
        fs::create_dir_all(&scope.root).expect("make the config directory");
        return Fixture::on(directory, scope, env, PLAN);
    }

    /// Writes `plan` and its lock into `scope` and builds the fakes.
    fn on(directory: tempfile::TempDir, scope: Scope, env: EnvMap, plan_text: &str) -> Fixture {
        let plan = parse_plan(plan_text, "the fixture plan").expect("the fixture plan parses");
        fs::write(&scope.plan_file, plan_text).expect("write the plan");
        fs::write(&scope.lock_file, render_lock(&lock_for(&plan))).expect("write the lock");
        return Fixture {
            directory,
            scope,
            env,
            runner: FakeRunner::new(),
            http: FakeHttp::new(),
            host: Host { platform: "linux-x64".to_string(), family: Some(OsFamily::Linux) },
        };
    }

    /// A context borrowing this fixture,
    /// with no locked tool (the add step resolves none).
    fn context(&self) -> Context<'_> {
        return Context { scope: &self.scope, env: &self.env, runner: &self.runner, http: &self.http, host: &self.host, tools: &[] };
    }

    /// The plan's text as the file holds it.
    fn plan_text(&self) -> String {
        return fs::read_to_string(&self.scope.plan_file).expect("read the plan");
    }

    /// The lock's text as the file holds it.
    fn lock_text(&self) -> String {
        return fs::read_to_string(&self.scope.lock_file).expect("read the lock");
    }

    /// Serves the crates.io documents a `cargo:` request needs.
    fn serve_crates_io(&self) {
        self.http.serve("https://crates.io/api/v1/crates/fastmod", FASTMOD_CRATE.as_bytes());
        self.http.serve("https://crates.io/api/v1/crates/tokei", TOKEI_CRATE.as_bytes());
    }

    /// Makes every `cargo binstall` leave the executables and the record a real one leaves.
    fn fake_binstall(&self, bins: &'static [&'static str]) {
        self.runner.on("cargo binstall", super::fakes_cargo::fake_binstall(bins));
    }
}

/// The arguments `install` was given.
fn arguments(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| return (*item).to_string()).collect();
}

#[test]
fn a_request_the_plan_lacks_is_named_as_a_new_entry_with_its_route_and_its_binaries() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    let plan = parse_plan(PLAN, "the fixture plan").expect("parses");
    let named_tool = named(&plan, &fixture.context(), "cargo:fastmod", "plan").expect("a new entry");
    let Named::New(declaration) = named_tool else {
        panic!("a request the plan lacks is a new entry: {named_tool:?}");
    };
    assert_eq!(declaration.id, "cargo:fastmod");
    assert_eq!(declaration.request.text(), "cargo:fastmod@latest", "the newest is what an argument without a version asks for");
    assert_eq!(declaration.route, super::model::Route::CargoBinstall);
    assert_eq!(declaration.bins, vec!["fastmod".to_string()], "the binary name comes from crates.io's own bin_names");
    assert_eq!(fixture.plan_text(), PLAN, "a declaration is built in memory; nothing was written");
}

#[test]
fn an_id_the_plan_holds_is_installed_as_it_is_and_the_plan_is_byte_for_byte_unchanged() {
    let fixture = Fixture::repo();
    fixture.fake_binstall(&["tokei"]);
    let before = fixture.plan_text();
    let report = add_named_tools(&fixture.context(), &arguments(&["tokei"]), None, nothing_stops_the_run).expect("named");
    assert_eq!(report.existing, vec!["tokei".to_string()]);
    assert!(report.added.is_empty());
    assert!(fixture.http.requests().is_empty(), "an id the plan holds needs no lookup");
    assert_eq!(fixture.plan_text(), before, "the plan is byte for byte as it was");
    let synced = run_sync_with(&fixture.scope, &fixture.env, &fixture.runner, &fixture.http, &fixture.host, LockPolicy::Resolve)
        .expect("the install runs");
    assert_eq!(synced.installed, vec!["tokei".to_string()]);
    assert!(fixture.runner.lines().iter().any(|line| return line.starts_with("cargo binstall --no-confirm")), "{:?}", fixture.runner.lines());
    assert_eq!(fixture.plan_text(), before, "installing it wrote nothing into the plan");
}

#[test]
fn a_new_request_is_added_to_the_plan_and_installed() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    fixture.fake_binstall(&["fastmod"]);
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
    assert_eq!(report.added.len(), 1);
    assert_eq!(report.added_ids(), vec!["cargo:fastmod".to_string()]);
    let plan_text = fixture.plan_text();
    let expected = r#"// The plan of this fixture: people write these comments.
{
  "schemaVersion": 1,
  "tools": {
    // tokei counts lines; the numbers are compared while it is pinned.
    "tokei": {"request": "cargo:tokei@latest", "route": "cargo-binstall", "bins": ["tokei"]},
    "cargo:fastmod": {
      "request": "cargo:fastmod@latest",
      "route": "cargo-binstall",
      "bins": [
        "fastmod"
      ]
    },
    // Nothing follows this note; it is here to be kept.
  },
}
"#;
    assert_eq!(plan_text, expected, "the entry lands last and every other byte is the one the person wrote");
    let synced = run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Resolve,
        &report.added_ids(),
    )
    .expect("the install runs");
    assert!(synced.resolved.iter().any(|(id, version)| return id == "cargo:fastmod" && version == "0.4.4"), "{synced:?}");
    assert!(synced.installed.contains(&"cargo:fastmod".to_string()), "{synced:?}");
    assert!(
        fixture.runner.lines().iter().any(|line| return line.contains("--version 0.4.4 fastmod")),
        "{:?}",
        fixture.runner.lines()
    );
    let lock: serde_json::Value = serde_json::from_str(&fixture.lock_text()).expect("the lock is JSON");
    assert_eq!(lock["tools"]["cargo:fastmod"]["version"], "0.4.4", "{lock}");
    assert_eq!(lock["tools"]["cargo:fastmod"]["bins"][0], "fastmod");
    assert_eq!(lock["tools"]["tokei"]["version"], VERSION, "no other row moved");
}

#[test]
fn a_bare_name_that_is_neither_an_id_nor_a_request_is_refused_and_nothing_is_written() {
    let fixture = Fixture::repo();
    let before = fixture.plan_text();
    let error = add_named_tools(&fixture.context(), &arguments(&["fastmod"]), None, nothing_stops_the_run).expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("\"fastmod\""), "{error}");
    assert!(error.message.contains("cargo:"), "the prefixes it could take are named: {error}");
    assert!(error.message.contains("README.md"), "the entry shapes are named: {error}");
    assert_eq!(fixture.plan_text(), before);
    assert!(fixture.http.requests().is_empty());
    assert!(fixture.runner.calls().is_empty());
}

#[test]
fn two_tools_in_one_command_are_both_added_in_the_order_given() {
    let fixture = Fixture::repo_with_installers();
    fixture.serve_crates_io();
    fixture.http.serve(
        "https://registry.npmjs.org/is-odd/latest",
        br#"{"name": "is-odd", "version": "3.0.1", "bin": {"is-odd": "index.js"}}"#,
    );
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod", "npm:is-odd"]), None, nothing_stops_the_run)
        .expect("both are added");
    assert_eq!(report.added_ids(), vec!["cargo:fastmod".to_string(), "npm:is-odd".to_string()]);
    let plan = parse_plan(&fixture.plan_text(), "the plan").expect("the new text parses");
    let ids: Vec<&str> = plan.tools.iter().map(|tool| return tool.id.as_str()).collect();
    assert_eq!(ids, vec!["tokei", "go", "pnpm", "cargo:fastmod", "npm:is-odd"]);
    let npm = plan.tools.iter().find(|tool| return tool.id == "npm:is-odd").expect("the npm entry");
    assert_eq!(npm.route, super::model::Route::MpmPnpm);
    assert_eq!(npm.bins, vec!["is-odd".to_string()], "npm's own bin field names the command");
}

#[test]
fn the_same_argument_twice_adds_one_entry() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod", "cargo:fastmod"]), None, nothing_stops_the_run)
        .expect("added once");
    assert_eq!(report.added.len(), 1);
    assert_eq!(parse_plan(&fixture.plan_text(), "the plan").expect("parses").tools.len(), 2);
}

#[test]
fn a_crate_whose_binaries_would_take_a_name_the_plan_uses_is_refused() {
    let fixture = Fixture::repo();
    // `cargo:tokei` is not the id the plan holds, but its binary name is the one `tokei` declares.
    fixture.http.serve("https://crates.io/api/v1/crates/tokei", TOKEI_CRATE.as_bytes());
    let before = fixture.plan_text();
    let error = add_named_tools(&fixture.context(), &arguments(&["cargo:tokei"]), None, nothing_stops_the_run).expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("tokei"), "{error}");
    assert!(error.message.contains("is already there"), "the refusal names the collision rather than the JSON parser: {error}");
    assert_eq!(fixture.plan_text(), before, "nothing was written");
}

#[test]
fn a_pinned_wrong_range_is_refused_when_crates_io_has_no_such_version() {
    let fixture = Fixture::repo();
    let error = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod@9.9.9"]), None, nothing_stops_the_run).expect_err("no such version");
    assert!(error.message.contains("crates.io/api/v1/crates/fastmod/9.9.9"), "{error}");
    assert!(error.message.contains("404"), "{error}");
}

#[test]
fn a_pinned_version_reads_the_binaries_of_that_version() {
    let fixture = Fixture::repo();
    fixture.http.serve(
        "https://crates.io/api/v1/crates/fastmod/0.4.4",
        br#"{"version": {"num": "0.4.4", "bin_names": ["fastmod", "fastmod-helper"]}}"#,
    );
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod@0.4.4"]), None, nothing_stops_the_run).expect("added");
    assert_eq!(report.added[0].request.text(), "cargo:fastmod@0.4.4");
    assert_eq!(report.added[0].bins, vec!["fastmod".to_string(), "fastmod-helper".to_string()], "every name is listed once, in order");
}

#[test]
fn an_npm_package_whose_bin_is_one_text_gets_the_packages_own_name() {
    let fixture = Fixture::repo_with_installers();
    fixture.http.serve("https://registry.npmjs.org/tic80/latest", br#"{"name": "tic80", "version": "1.0.0", "bin": "bin/tic80.js"}"#);
    let report = add_named_tools(&fixture.context(), &arguments(&["npm:tic80"]), None, nothing_stops_the_run).expect("added");
    assert_eq!(report.added[0].bins, vec!["tic80".to_string()]);
}

#[test]
fn a_scoped_npm_package_is_named_and_looked_up_whole() {
    let fixture = Fixture::repo_with_installers();
    fixture.http.serve(
        "https://registry.npmjs.org/@actions%2Flanguageserver/latest",
        br#"{"name": "@actions/languageserver", "version": "0.3.61", "bin": {"actions-languageserver": "index.js"}}"#,
    );
    let report = add_named_tools(&fixture.context(), &arguments(&["npm:@actions/languageserver"]), None, nothing_stops_the_run).expect("added");
    assert_eq!(report.added[0].id, "npm:@actions/languageserver");
    assert_eq!(report.added[0].request.text(), "npm:@actions/languageserver@latest");
    assert_eq!(report.added[0].bins, vec!["actions-languageserver".to_string()]);
}

#[test]
fn a_go_module_gets_the_last_segment_of_its_path_as_the_program() {
    let fixture = Fixture::repo_with_installers();
    let report = add_named_tools(
        &fixture.context(),
        &arguments(&["go:github.com/charmbracelet/crush"]),
        None,
        nothing_stops_the_run,
    )
    .expect("added");
    assert_eq!(report.added[0].route, super::model::Route::GoInstall);
    assert_eq!(report.added[0].bins, vec!["crush".to_string()]);
    assert!(fixture.http.requests().is_empty(), "a Go module's program name needs no lookup");
}

#[test]
fn a_go_module_that_ends_in_a_major_version_suffix_is_refused() {
    let fixture = Fixture::repo_with_installers();
    let error = add_named_tools(&fixture.context(), &arguments(&["go:example.com/x/y/v2"]), None, nothing_stops_the_run).expect_err("refused");
    assert!(error.message.contains("v2"), "{error}");
    assert!(error.message.contains("go:example.com/x/y/v2/cmd/"), "the addable form is named: {error}");
}

#[test]
fn an_npm_request_is_refused_before_any_lookup_when_the_plan_holds_no_pnpm() {
    let fixture = Fixture::repo();
    let before = fixture.plan_text();
    let error = add_named_tools(&fixture.context(), &arguments(&["npm:is-odd"]), None, nothing_stops_the_run).expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("`pnpm`"), "{error}");
    assert!(error.message.contains(&fixture.scope.plan_file.display().to_string()), "{error}");
    assert_eq!(fixture.plan_text(), before, "nothing was written");
    assert!(fixture.http.requests().is_empty(), "the registry is not asked for a tool that could never install");
}

#[test]
fn a_go_request_is_refused_when_the_plan_holds_no_go() {
    let fixture = Fixture::repo();
    let before = fixture.plan_text();
    let error = add_named_tools(
        &fixture.context(),
        &arguments(&["go:github.com/charmbracelet/crush@1.2.3"]),
        None,
        nothing_stops_the_run,
    )
    .expect_err("refused");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("`go`"), "{error}");
    assert_eq!(fixture.plan_text(), before);
}

#[test]
fn a_route_a_request_cannot_describe_is_refused_with_the_plan_and_the_readme_named() {
    let fixture = Fixture::repo();
    let before = fixture.plan_text();
    for argument in ["github:FiloSottile/age", "pipx:isort", "git:https://example.invalid/repo@ref:abc", "node"] {
        let error = add_named_tools(&fixture.context(), &arguments(&[argument]), None, nothing_stops_the_run).expect_err("refused");
        assert_eq!(error.kind, ErrorKind::Config, "{argument}");
        assert!(error.message.contains("package/mpm-shadow/cli/README.md"), "{argument}: {error}");
        assert!(error.message.contains(&fixture.scope.plan_file.display().to_string()), "{argument}: {error}");
    }
    assert_eq!(fixture.plan_text(), before);
    assert!(fixture.http.requests().is_empty());
}

#[test]
fn a_request_carrying_a_quote_a_space_a_newline_a_dot_dot_a_leading_dash_or_a_semicolon_is_refused_and_nothing_is_written() {
    let fixture = Fixture::repo();
    let before = fixture.plan_text();
    let hostile = [
        "cargo:fastmod\"@latest",
        "cargo:fast\r\nmod",
        "cargo:../fastmod",
        "-cargo:fastmod",
        "cargo:fastmod;rm -rf /",
        "cargo:fast mod",
        "cargo:fastmod\nnpm:is-odd",
    ];
    for argument in hostile {
        let error = add_named_tools(&fixture.context(), &arguments(&[argument]), None, nothing_stops_the_run).expect_err("refused");
        assert_eq!(error.kind, ErrorKind::Config, "{argument:?}");
        assert!(error.message.contains("\\n") || !argument.contains('\n'), "a newline is escaped in the message, never printed: {error}");
        assert!(!error.message.contains('\n'), "{argument:?}: {error}");
    }
    assert_eq!(fixture.plan_text(), before);
    assert!(fixture.http.requests().is_empty());
    assert!(fixture.runner.calls().is_empty());
}

#[test]
fn a_frozen_run_refuses_a_plan_whose_other_entries_differ_too() {
    // A resolution edit on a tool the command line did not name, made after the lock was written.
    let fixture = Fixture::repo();
    let edited = PLAN.replace("cargo:tokei@latest", "cargo:tokei@1.2.3");
    fs::write(&fixture.scope.plan_file, &edited).expect("edit the plan");
    fixture.serve_crates_io();
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
    let lock_before = fixture.lock_text();
    let error = run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Frozen(FrozenReason::Flag),
        &report.added_ids(),
    )
    .expect_err("tokei needs a lookup the command line did not ask for");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert_eq!(fixture.lock_text(), lock_before, "a frozen run writes no lock here");
}

#[test]
fn a_frozen_run_refuses_a_plan_whose_bins_edit_it_could_have_written_without_a_lookup() {
    let fixture = Fixture::repo();
    let edited = PLAN.replace("\"bins\": [\"tokei\"]", "\"bins\": [\"tokei\", \"tokei2\"]");
    fs::write(&fixture.scope.plan_file, &edited).expect("edit the plan");
    fixture.serve_crates_io();
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
    let lock_before = fixture.lock_text();
    let error = run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Frozen(FrozenReason::Flag),
        &report.added_ids(),
    )
    .expect_err("a frozen run writes nothing but the entries it was told to add");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert_eq!(fixture.lock_text(), lock_before);
}

#[test]
fn a_frozen_check_and_an_offline_run_refuse_a_new_entry_before_the_plan_is_touched() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    let before = fixture.plan_text();
    for refusal in [AddRefusal::FrozenCheck, AddRefusal::Offline] {
        let error = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), Some(refusal), nothing_stops_the_run).expect_err("refused");
        assert_eq!(error.kind, ErrorKind::Config);
        assert!(error.message.contains("was not added"), "{error}");
    }
    assert_eq!(fixture.plan_text(), before);
    assert!(fixture.http.requests().is_empty(), "a refusal needs no lookup");
}

#[test]
fn a_frozen_run_resolves_the_entry_its_own_command_line_added() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    fixture.fake_binstall(&["fastmod"]);
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
    let synced = run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Frozen(FrozenReason::Flag),
        &report.added_ids(),
    )
    .expect("a frozen run locks what its command line added, as `pnpm add --frozen-lockfile` does");
    assert!(synced.resolved.iter().any(|(id, _)| return id == "cargo:fastmod"), "{synced:?}");
    let lock: serde_json::Value = serde_json::from_str(&fixture.lock_text()).expect("the lock is JSON");
    assert_eq!(lock["tools"]["cargo:fastmod"]["version"], "0.4.4");
}

#[test]
fn a_frozen_run_still_refuses_a_plan_entry_its_command_line_did_not_name() {
    let fixture = Fixture::repo();
    let edited = PLAN.replace("cargo:tokei@latest", "cargo:tokei@1.2.3");
    fs::write(&fixture.scope.plan_file, edited).expect("edit the plan");
    let error = run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Frozen(FrozenReason::Flag),
        &["cargo:fastmod".to_string()],
    )
    .expect_err("nothing but the named entry is resolved when frozen");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("mpm-shadow update"), "{error}");
}

#[test]
fn a_run_stopped_between_the_plans_write_and_the_locks_leaves_files_a_later_run_finishes() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    fixture.fake_binstall(&["fastmod"]);
    let lock_before = fixture.lock_text();
    // The fault point stands where a kill would land: the plan is published, the lock is not touched.
    let stopped = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, || {
        return Err(ToolchainError::failure("killed between the plan's write and the lock's"));
    })
    .expect_err("stopped");
    assert!(stopped.message.contains("killed"), "{stopped}");
    assert!(fixture.plan_text().contains("\"cargo:fastmod\""), "the plan holds the new entry");
    assert_eq!(fixture.lock_text(), lock_before, "the lock was not written");
    // A later run of the same command finishes it: the plan is ahead of the lock, so the entry is resolved and installed.
    let synced = run_sync_with(&fixture.scope, &fixture.env, &fixture.runner, &fixture.http, &fixture.host, LockPolicy::Resolve)
        .expect("a plain install finishes it");
    assert!(synced.resolved.iter().any(|(id, _)| return id == "cargo:fastmod"), "{synced:?}");
    let plan = parse_plan(&fixture.plan_text(), "the plan").expect("the plan parses");
    let lock: serde_json::Value = serde_json::from_str(&fixture.lock_text()).expect("the lock is JSON");
    for declaration in &plan.tools {
        if declaration.id == "cargo:fastmod" {
            continue;
        }
        assert_eq!(lock["tools"][&declaration.id]["version"], VERSION, "{} keeps the row it had", declaration.id);
    }
    assert_eq!(lock["tools"]["cargo:fastmod"]["version"], "0.4.4");
}

#[test]
fn a_plan_another_run_wrote_meanwhile_is_not_overwritten() {
    let fixture = Fixture::repo();
    let before = fs::read(&fixture.scope.plan_file).expect("read the plan");
    let expected = revision_of(&Some(before));
    // Another mpm-shadow adds its tool after this run read the file, before this run writes.
    let other = PLAN.replace("// Nothing follows", "// Another run's tool.\n    \"cargo:other\": {\n      \"request\": \"cargo:other@latest\",\n      \"route\": \"cargo-binstall\",\n      \"bins\": [\"other\"]\n    },\n    // Nothing follows");
    fs::write(&fixture.scope.plan_file, &other).expect("the other run's write");
    let error = super::publish::publish_plan(&fixture.scope, &expected, &other.replace("cargo:other", "cargo:ours"))
        .expect_err("the publish must notice the other write");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("changed while this command was running"), "{error}");
    assert_eq!(fixture.plan_text(), other, "the other run's entry is still there and nothing was written over it");
}

#[test]
fn the_plan_keeps_the_mode_it_had() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::repo();
        fixture.serve_crates_io();
        fs::set_permissions(&fixture.scope.plan_file, fs::Permissions::from_mode(0o600)).expect("chmod the plan");
        add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
        let mode = fs::metadata(&fixture.scope.plan_file).expect("the plan exists").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the global plan is one the owner alone can read, and it stays that way");
    }
}

#[test]
fn the_machine_global_scope_adds_and_installs_the_same_way() {
    let fixture = Fixture::global();
    fixture.serve_crates_io();
    fixture.fake_binstall(&["fastmod"]);
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
    assert_eq!(report.added_ids(), vec!["cargo:fastmod".to_string()]);
    assert!(fixture.scope.plan_file.starts_with(fixture.directory.path()), "the global plan is under the disposable home");
    let synced = run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Resolve,
        &report.added_ids(),
    )
    .expect("the install runs");
    assert!(synced.installed.contains(&"cargo:fastmod".to_string()), "{synced:?}");
    assert!(fixture.scope.toolchain_root.starts_with(fixture.directory.path()), "the toolchain root is the disposable one");
}

#[test]
fn an_existing_id_in_the_machine_global_scope_is_installed_and_its_plan_untouched() {
    let fixture = Fixture::global();
    fixture.fake_binstall(&["tokei"]);
    let before = fixture.plan_text();
    let report = add_named_tools(&fixture.context(), &arguments(&["tokei"]), None, nothing_stops_the_run).expect("named");
    assert_eq!(report.existing, vec!["tokei".to_string()]);
    assert_eq!(fixture.plan_text(), before);
    let synced = run_sync_with(&fixture.scope, &fixture.env, &fixture.runner, &fixture.http, &fixture.host, LockPolicy::Resolve)
        .expect("the install runs");
    assert_eq!(synced.installed, vec!["tokei".to_string()]);
}

#[test]
fn a_binary_the_registry_lists_is_the_one_the_plan_links() {
    let fixture = Fixture::repo();
    fixture.serve_crates_io();
    fixture.fake_binstall(&["fastmod"]);
    let report = add_named_tools(&fixture.context(), &arguments(&["cargo:fastmod"]), None, nothing_stops_the_run).expect("added");
    run_sync_with_named(
        &fixture.scope,
        &fixture.env,
        &fixture.runner,
        &fixture.http,
        &fixture.host,
        LockPolicy::Resolve,
        &report.added_ids(),
    )
    .expect("the install runs");
    let link = fixture.scope.exposure_dir.join("fastmod");
    assert!(link.symlink_metadata().is_ok(), "the declared binary is linked in the exposure directory: {}", link.display());
}
