//! Unit tests of the resolvers and of `update`,
//! against a table-driven fake network and a recording fake runner.
//! No
//! test reaches a registry or starts a package manager.

use std::fs;
use std::path::PathBuf;

use super::error::ErrorKind;
use super::fakes::{FakeHttp, FakeRunner, write_executable};
use super::fixtures::{sample_lock, sample_scope_on_disk};
use super::hash::sha256_hex;
use super::http::{HttpError, HttpGet, HttpResponse};
use super::layout::tool_install_dir;
use super::lock::{Lock, parse_lock, render_lock};
use super::lock_cmd::{LockMode, run_lock};
use super::model::{Artifact, Request};
use super::scope::{EnvMap, Scope};
use super::update::{OutcomeKind, UpdateReport, format_update_report, run_update};

pub(super) const NPM_URL: &str = "https://registry.npmjs.org/wrangler";
pub(super) const PYPI_URL: &str = "https://pypi.org/pypi/slopo/json";
pub(super) const CRATES_URL: &str = "https://crates.io/api/v1/crates/fd-find";
const RUST_URL: &str = "https://static.rust-lang.org/dist/channel-rust-nightly.toml";

/// A plan of four tools resolved by registries and the Rust manifest,
/// with `{version}` placeholders for the toolchain.
pub(super) const REGISTRY_PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    "cargo:fd-find": { "request": "cargo:fd-find@latest", "route": "cargo-binstall", "manager": "cargo", "packageId": "fd-find", "bins": ["fd"] },
    "npm:wrangler": { "request": "npm:wrangler@latest", "route": "mpm-pnpm", "manager": "pnpm", "packageId": "wrangler", "bins": ["wrangler"] },
    "pipx:slopo": { "request": "pipx:slopo@latest", "route": "mpm-uvx", "manager": "uvx", "packageId": "slopo", "bins": ["slopo"] },
    "rust": {
      "request": "rust@nightly", "route": "rustup-toolchain", "manager": "rustup", "packageId": "{version}",
      "bins": ["rustc"], "env": { "RUSTUP_TOOLCHAIN": "{version}" }
    },
  },
}
"#;

pub(super) fn serve_registries(http: &FakeHttp, crate_version: &str, npm_version: &str, pypi_version: &str, nightly_date: &str) {
    http.serve(CRATES_URL, format!("{{\"crate\": {{\"max_stable_version\": \"{crate_version}\"}}}}").as_bytes());
    http.serve(NPM_URL, format!("{{\"dist-tags\": {{\"latest\": \"{npm_version}\", \"next\": \"9.0.0-beta\"}}}}").as_bytes());
    http.serve(PYPI_URL, format!("{{\"info\": {{\"version\": \"{pypi_version}\"}}}}").as_bytes());
    http.serve(RUST_URL, format!("date = \"{nightly_date}\"\n[pkg.rust]\nversion = \"1.103.0-nightly (abc 2026-10-29)\"\n").as_bytes());
}

pub(super) fn registry_scope() -> (tempfile::TempDir, Scope, EnvMap) {
    let (directory, scope, env) = sample_scope_on_disk();
    fs::write(&scope.plan_file, REGISTRY_PLAN).expect("write the plan");
    fs::remove_file(&scope.lock_file).expect("start without a lock");
    return (directory, scope, env);
}

pub(super) fn linux() -> super::context::Host {
    return super::context::Host { platform: "linux-x64".to_string(), family: Some(super::model::OsFamily::Linux) };
}

pub(super) fn outcome_of<'a>(report: &'a UpdateReport, id: &str) -> &'a OutcomeKind {
    return &report.outcomes.iter().find(|outcome| return outcome.id == id).expect("an outcome for the tool").kind;
}

pub(super) fn read_lock(scope: &Scope) -> Lock {
    return parse_lock(&fs::read_to_string(&scope.lock_file).expect("read the lock"), "lock").expect("parses").lock;
}


//region update

#[test]
fn the_first_update_resolves_every_tool_writes_the_lock_and_passes_the_gate() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let runner = FakeRunner::new();
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("updated");
    assert!(report.wrote);
    assert!(report.outcomes.iter().all(|outcome| return matches!(outcome.kind, OutcomeKind::Added { .. })), "{:?}", report.outcomes);
    assert!(runner.calls().is_empty(), "nothing was installed");

    let lock = read_lock(&scope);
    assert_eq!(lock.tool("cargo:fd-find").expect("fd").version, "10.5.0");
    assert_eq!(lock.tool("npm:wrangler").expect("wrangler").version, "4.156.0");
    let rust = lock.tool("rust").expect("rust");
    assert_eq!(rust.version, "nightly-2026-10-30");
    assert_eq!(rust.declaration.package_id.as_deref(), Some("nightly-2026-10-30"), "the placeholder is expanded in the lock");
    assert_eq!(rust.declaration.env, vec![("RUSTUP_TOOLCHAIN".to_string(), "nightly-2026-10-30".to_string())]);
    run_lock(&scope, LockMode::Check).expect("the written lock passes the gate");
}

#[test]
fn a_second_update_with_nothing_new_writes_nothing() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let report = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("second");
    assert!(!report.wrote);
    assert!(report.outcomes.iter().all(|outcome| return outcome.kind == OutcomeKind::Unchanged), "{:?}", report.outcomes);
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
    assert!(format_update_report(&report, false).contains("is already current"));
}

#[test]
fn a_moved_registry_updates_the_versions_and_the_placeholder_follows_the_toolchain() {
    let (_directory, scope, env) = registry_scope();
    let first = FakeHttp::new();
    serve_registries(&first, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &first, &linux(), false).expect("first");
    let second = FakeHttp::new();
    serve_registries(&second, "10.6.0", "4.156.0", "0.9.0", "2026-11-03");
    let report = run_update(&scope, &env, &FakeRunner::new(), &second, &linux(), false).expect("second");
    assert!(report.wrote);
    assert_eq!(
        outcome_of(&report, "cargo:fd-find"),
        &OutcomeKind::Updated { old_version: "10.5.0".to_string(), new_version: "10.6.0".to_string() }
    );
    assert_eq!(outcome_of(&report, "npm:wrangler"), &OutcomeKind::Unchanged);
    let rust = read_lock(&scope).tool("rust").expect("rust").clone();
    assert_eq!(rust.version, "nightly-2026-11-03");
    assert_eq!(rust.declaration.package_id.as_deref(), Some("nightly-2026-11-03"));
    assert_eq!(rust.declaration.env[0].1, "nightly-2026-11-03");
    run_lock(&scope, LockMode::Check).expect("plan and lock still agree after the toolchain moved");
    let text = format_update_report(&report, false);
    assert!(text.contains("updated   cargo:fd-find  10.5.0 -> 10.6.0"), "{text}");
    assert!(text.contains("wrote toolchain.lock.json (2 tools moved"), "{text}");
}

#[test]
fn check_reports_movement_without_writing_or_starting_anything() {
    let (_directory, scope, env) = registry_scope();
    let first = FakeHttp::new();
    serve_registries(&first, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &first, &linux(), false).expect("first");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let second = FakeHttp::new();
    serve_registries(&second, "10.6.0", "4.156.0", "0.9.0", "2026-10-30");
    let runner = FakeRunner::new();
    let report = run_update(&scope, &env, &runner, &second, &linux(), true).expect("checked");
    assert!(report.has_movement());
    assert!(!report.wrote);
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
    assert!(runner.calls().is_empty());
    assert!(format_update_report(&report, true).contains("nothing was written or installed"));
}

#[test]
fn a_failed_lookup_writes_nothing_and_names_the_url() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    http.fail(PYPI_URL, "timed out");
    let report = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("a report");
    assert!(report.has_failures());
    assert!(!report.wrote);
    assert!(!scope.lock_file.exists(), "no lock is written when any lookup failed");
    let text = format_update_report(&report, false);
    assert!(text.contains(&format!("failed    pipx:slopo  {PYPI_URL}: timed out")), "{text}");
    assert!(text.contains("toolchain.lock.json was not written"), "{text}");
}

#[test]
fn a_changed_pin_resolves_to_the_pinned_version_without_a_lookup() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    fs::write(&scope.plan_file, REGISTRY_PLAN.replace(
        "\"npm:wrangler\": { \"request\": \"npm:wrangler@latest\"",
        "\"npm:wrangler\": { \"request\": \"npm:wrangler@3.0.0\"",
    ))
    .expect("pin wrangler");
    let quiet = FakeHttp::new();
    serve_registries(&quiet, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let report = run_update(&scope, &env, &FakeRunner::new(), &quiet, &linux(), false).expect("updated");
    assert_eq!(
        outcome_of(&report, "npm:wrangler"),
        &OutcomeKind::Updated { old_version: "4.156.0".to_string(), new_version: "3.0.0".to_string() }
    );
    assert!(!quiet.requests().contains(&NPM_URL.to_string()), "a pin needs no registry lookup");
    run_update(&scope, &env, &FakeRunner::new(), &quiet, &linux(), false).expect("again");
    assert_eq!(*outcome_of(&run_update(&scope, &env, &FakeRunner::new(), &quiet, &linux(), true).expect("checked"), "npm:wrangler"), OutcomeKind::Pinned);
}

#[test]
fn update_refuses_a_hand_edited_lock() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    let text = fs::read_to_string(&scope.lock_file).expect("read");
    fs::write(&scope.lock_file, text.replace("\"version\": \"4.156.0\"", "\"version\": \"0.0.1\"")).expect("hand edit");
    let error = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect_err("the digest fails");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("edited by hand"), "{error}");
    assert!(error.message.contains("delete it and run `mpm-shadow update`"), "{error}");
}

#[test]
fn deleting_the_lock_and_updating_rebuilds_it_from_the_plan_alone() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    let first = fs::read_to_string(&scope.lock_file).expect("read");
    fs::remove_file(&scope.lock_file).expect("delete the lock");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("rebuilt");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), first, "the rebuilt lock is byte for byte the first one");
}

#[test]
fn a_brand_new_downloaded_tool_is_just_a_new_plan_entry() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    let plan = fs::read_to_string(&scope.plan_file).expect("read");
    let edited = plan.replace(
        "    \"cargo:fd-find\"",
        "    \"age\": { \"request\": \"github:FiloSottile/age@latest\", \"route\": \"archive\", \"bins\": [\"age\"] },\n    \"cargo:fd-find\"",
    );
    fs::write(&scope.plan_file, edited).expect("add the entry");
    http.serve(
        "https://api.github.com/repos/FiloSottile/age/releases/latest",
        b"{\"tag_name\": \"v1.3.2\", \"assets\": [\
          {\"name\": \"age-v1.3.2-linux-amd64.tar.gz\", \"browser_download_url\": \"https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz\", \"digest\": \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}, \
          {\"name\": \"age-v1.3.2-linux-arm64.tar.gz\", \"browser_download_url\": \"https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-arm64.tar.gz\", \"digest\": \"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"}]}",
    );
    let report = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("updated");
    assert_eq!(outcome_of(&report, "age"), &OutcomeKind::Added { version: "1.3.2".to_string() });
    let lock = read_lock(&scope);
    let age = lock.tool("age").expect("age is locked");
    assert_eq!(age.artifacts.len(), 4);
    assert_eq!(age.artifacts[2].1.sha256, "a".repeat(64));
    run_lock(&scope, LockMode::Check).expect("the gate passes");
}

/// A scope whose lock holds node and pnpm (pinned archives that a fake `tar` unpacks) and an unlocked-looking npm tool
/// whose registry lookup the test controls.
fn auto_sync_rig() -> (tempfile::TempDir, Scope, EnvMap, FakeHttp, FakeRunner) {
    let (directory, scope, env) = sample_scope_on_disk();
    let plan = "{ \"schemaVersion\": 1, \"tools\": {
      \"node\": { \"request\": \"node@1.2.3\", \"route\": \"archive\", \"bins\": [\"node\", \"npm\", \"npx\"] },
      \"pnpm\": { \"request\": \"github:pnpm/pnpm@1.2.3\", \"route\": \"archive\", \"bins\": [\"pnpm\"] },
      \"npm:wrangler\": { \"request\": \"npm:wrangler@latest\", \"route\": \"mpm-pnpm\", \"manager\": \"pnpm\", \"packageId\": \"wrangler\", \"bins\": [\"wrangler\"] }
    } }";
    fs::write(&scope.plan_file, plan).expect("write the plan");
    let http = FakeHttp::new();
    let mut lock = sample_lock();
    lock.tools.retain(|tool| return ["node", "pnpm", "npm:wrangler"].contains(&tool.declaration.id.as_str()));
    for tool in &mut lock.tools {
        match tool.declaration.id.as_str() {
            "node" | "pnpm" => {
                tool.declaration.request = Request::parse(&format!("{}@1.2.3", if tool.declaration.id == "node" { "node" } else { "github:pnpm/pnpm" })).expect("parses");
                if tool.declaration.id == "pnpm" {
                    tool.declaration.route = super::model::Route::Archive;
                    tool.declaration.options.bin_aliases = None;
                    tool.declaration.bins = vec!["pnpm".to_string()];
                }
                let url = format!("https://example.invalid/{}.tar.gz", tool.declaration.id);
                http.serve(&url, tool.declaration.id.as_bytes());
                tool.artifacts = vec![("linux-x64".to_string(), Artifact { url, sha256: sha256_hex(tool.declaration.id.as_bytes()) })];
            }
            _ => {
                tool.version = "4.0.0".to_string();
                tool.declaration.options.allow_builds = None;
            }
        }
    }
    fs::write(&scope.lock_file, render_lock(&lock)).expect("write the lock");
    let runner = FakeRunner::new();
    let root = scope.toolchain_root.clone();
    runner.on(
        "tar",
        Box::new(move |request| {
            let into = PathBuf::from(request.args.last().expect("tar --directory <dir>"));
            assert!(into.starts_with(&root), "the unpack target is inside the disposable toolchain root");
            for name in ["node", "npm", "npx", "pnpm"] {
                write_executable(&into.join(name));
            }
        }),
    );
    return (directory, scope, env, http, runner);
}

#[test]
fn a_failed_npm_lookup_of_a_missing_locked_tool_syncs_the_missing_tools_then_retries_once() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    http.fail_first(NPM_URL, "connection reset", 1);
    http.serve(NPM_URL, b"{\"dist-tags\": {\"latest\": \"4.156.0\"}}");
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("updated");
    assert_eq!(
        outcome_of(&report, "npm:wrangler"),
        &OutcomeKind::Updated { old_version: "4.0.0".to_string(), new_version: "4.156.0".to_string() }
    );
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 2, "looked up exactly twice");
    assert_eq!(report.synced, vec!["node".to_string(), "pnpm".to_string(), "npm:wrangler".to_string()]);
    let lines = runner.lines();
    assert!(lines.iter().any(|line| return line == "pnpm add --global wrangler@4.0.0"), "the missing tool installs at its locked version: {lines:?}");
    assert!(report.wrote);
    assert_eq!(read_lock(&scope).tool("npm:wrangler").expect("wrangler").version, "4.156.0");
}

#[test]
fn a_second_failure_is_final_and_says_the_lookup_was_made_twice() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    http.fail(NPM_URL, "connection reset");
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("a report");
    assert!(report.has_failures());
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 2);
    match outcome_of(&report, "npm:wrangler") {
        OutcomeKind::Failed { reason } => {
            assert!(reason.contains("looked up twice"), "{reason}");
            assert!(reason.contains(NPM_URL), "{reason}");
        }
        other => panic!("expected a failure, found {other:?}"),
    }
    assert!(!report.wrote);
}

#[test]
fn an_installed_tool_whose_lookup_fails_is_not_synced() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    for name in ["node", "npm", "npx"] {
        write_executable(&tool_install_dir(&scope.toolchain_root, "node", "1.2.3").join(name));
    }
    write_executable(&tool_install_dir(&scope.toolchain_root, "pnpm", "1.2.3").join("pnpm"));
    runner.answer("pnpm list", "[{\"dependencies\": {\"wrangler\": {\"version\": \"4.0.0\"}}}]", 0);
    http.fail(NPM_URL, "503");
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("a report");
    assert!(report.synced.is_empty());
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 1, "no retry for an installed tool");
    assert!(report.has_failures());
}

#[test]
fn check_never_installs_even_when_a_lookup_fails() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    http.fail(NPM_URL, "connection reset");
    let report = run_update(&scope, &env, &runner, &http, &linux(), true).expect("a report");
    assert!(report.has_failures());
    assert!(report.synced.is_empty());
    assert!(runner.calls().is_empty(), "update --check starts no program");
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 1);
}

/// An `HttpGet` that rewrites the plan file the first time it is asked for anything,
/// as a person editing the plan
/// while an update waits on the network would.
struct EditingHttp<'a> {
    inner: &'a FakeHttp,
    plan: PathBuf,
}

impl HttpGet for EditingHttp<'_> {
    fn get(&self, url: &str, headers: &[(String, String)], max_bytes: usize) -> Result<HttpResponse, HttpError> {
        let current = fs::read_to_string(&self.plan).expect("read the plan");
        if !current.contains("// edited during the update") {
            fs::write(&self.plan, format!("{current}\n// edited during the update\n")).expect("edit the plan");
        }
        return self.inner.get(url, headers, max_bytes);
    }

    fn download(&self, url: &str, target: &std::path::Path, max_bytes: u64) -> Result<String, HttpError> {
        return self.inner.download(url, target, max_bytes);
    }
}

#[test]
fn an_update_does_not_publish_over_a_plan_that_changed_while_it_ran() {
    let (_directory, scope, env) = registry_scope();
    let inner = FakeHttp::new();
    serve_registries(&inner, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let http = EditingHttp { inner: &inner, plan: scope.plan_file.clone() };
    let error = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect_err("the plan moved");
    assert!(error.message.contains("toolchain.plan.jsonc changed while this command was running"), "{error}");
    assert!(!scope.lock_file.exists());
}

//endregion update
