//! Unit tests of `list`:
//! it refuses when the plan and lock disagree (starting nothing),
//! and otherwise reports what a
//! fake machine holds.

use std::fs;

use super::cargo_installs::version_directory;
use super::context::Host;
use super::error::ErrorKind;
use super::fakes::FakeRunner;
use super::fakes_cargo::finish_install;
use super::fixtures::{SAMPLE_PLAN, sample_lock, sample_scope_on_disk};
use super::lock::render_lock;
use super::model::OsFamily;
use super::status_cmd::run_status;

fn linux() -> Host {
    return Host { platform: "linux-x64".to_string(), family: Some(OsFamily::Linux) };
}

#[test]
fn status_refuses_when_the_plan_and_lock_disagree_and_starts_nothing() {
    let (_directory, scope, env) = sample_scope_on_disk();
    fs::write(&scope.plan_file, SAMPLE_PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]")).expect("edit the plan");
    let runner = FakeRunner::new();
    let error = run_status(&scope, &env, &runner, &linux()).expect_err("the lock is stale");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("mpm-shadow install --lockfile-only --offline"), "{error}");
    assert!(error.message.contains("cargo:fd-find differs between the plan and the lock in: bins"), "{error}");
    assert!(runner.calls().is_empty(), "nothing is started when the files disagree");
}

#[test]
fn status_refuses_a_hand_edited_lock_by_its_digest() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let text = fs::read_to_string(&scope.lock_file).expect("read");
    fs::write(&scope.lock_file, text.replace("\"version\": \"1.2.3\"", "\"version\": \"7.7.7\"")).expect("edit");
    let runner = FakeRunner::new();
    let error = run_status(&scope, &env, &runner, &linux()).expect_err("the digest fails");
    assert!(error.message.contains("edited by hand"), "{error}");
    assert!(runner.calls().is_empty());
}

#[test]
fn status_without_a_lock_names_update() {
    let (_directory, scope, env) = sample_scope_on_disk();
    fs::remove_file(&scope.lock_file).expect("remove");
    let error = run_status(&scope, &env, &FakeRunner::new(), &linux()).expect_err("no lock");
    assert!(error.message.contains("mpm-shadow update"), "{error}");
}

#[test]
fn status_reports_drift_for_tools_a_fake_machine_lacks() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let runner = FakeRunner::new();
    finish_install(&version_directory(&scope.toolchain_root, "fd-find", "1.2.3"), "fd-find", "1.2.3", &["fd"]);
    let report = run_status(&scope, &env, &runner, &linux()).expect("a report");
    assert!(report.drifted);
    assert!(report.text.contains("ok      cargo:fd-find  1.2.3\n"), "{}", report.text);
    assert!(report.text.contains("drift   node  locked=1.2.3  observed=(absent)\n"), "{}", report.text);
    assert!(
        runner.lines().iter().all(|line| return line.starts_with("rustup toolchain list")),
        "only read-only queries run: {:?}",
        runner.lines()
    );
}

#[test]
fn status_reports_in_sync_when_everything_matches() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let mut lock = sample_lock();
    lock.tools.retain(|tool| return tool.declaration.id == "cargo:fd-find");
    let plan_text = "{ \"schemaVersion\": 1, \"tools\": { \"cargo:fd-find\": { \"request\": \"cargo:fd-find@latest\", \"route\": \"cargo-binstall\", \"manager\": \"cargo\", \"packageId\": \"fd-find\", \"bins\": [\"fd\"] } } }";
    fs::write(&scope.plan_file, plan_text).expect("write the plan");
    fs::write(&scope.lock_file, render_lock(&lock)).expect("write the lock");
    let runner = FakeRunner::new();
    finish_install(&version_directory(&scope.toolchain_root, "fd-find", "1.2.3"), "fd-find", "1.2.3", &["fd"]);
    let report = run_status(&scope, &env, &runner, &linux()).expect("a report");
    assert!(!report.drifted, "{}", report.text);
    assert!(report.text.ends_with("toolchain list: 1 tools in sync with toolchain.lock.json\n"), "{}", report.text);
}
