//! Unit tests of `sync`, the exposure directory and the postinstall steps, against the fake runner and a disposable
//! scope: the tests assert the exact program lines started and that unchanged tools are left alone.

use std::fs;

use super::context::Host;
use super::error::ErrorKind;
use super::exposure::{rebuild_exposure, run_postinstall_commands, run_postinstall_links, unresolvable_bins};
use super::fakes::{FakeHttp, FakeRunner, Rig, write_executable};
use super::fixtures::{SAMPLE_PLAN, sample_lock, sample_scope_on_disk};
use super::layout::tool_install_dir;
use super::model::{Postinstall, PostinstallRun};
use super::sync::{format_sync_report, replacement_warning, run_sync, sync_order, sync_tools};

/// Keeps only the named tools and makes `cargo-binstall` a cargo crate, so the scenario needs no `bin` CLI.
fn keep_only(rig: &mut Rig, ids: &[&str]) {
    rig.lock.tools.retain(|tool| return ids.contains(&tool.declaration.id.as_str()));
    for tool in &mut rig.lock.tools {
        if tool.declaration.id == "cargo-binstall" {
            tool.declaration.route = super::model::Route::CargoBinstall;
        }
    }
}

#[test]
fn staging_tools_come_first_in_staging_order_then_the_rest_in_lock_order() {
    let lock = sample_lock();
    let ids: Vec<&str> = sync_order(&lock.tools).iter().map(|tool| return tool.declaration.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "bin",
            "uv",
            "rust",
            "cargo-binstall",
            "node",
            "pnpm",
            "cargo:fd-find",
            "cargo:monochromatic-mpm-shadow",
            "npm:wrangler",
            "pipx:slopo",
            "android-sdk"
        ]
    );
}

#[test]
fn a_replacement_warning_names_both_versions_and_only_appears_for_a_different_installed_version() {
    let warning = replacement_warning("node", &Some("25.0.0".to_string()), "26.13.0").expect("a different version warns");
    assert!(warning.contains("node is installed at 25.0.0 but this tree's lock pins 26.13.0"), "{warning}");
    assert_eq!(replacement_warning("node", &Some("v26.13.0".to_string()), "26.13.0"), None);
    assert_eq!(replacement_warning("node", &None, "26.13.0"), None);
}

#[test]
fn sync_refuses_when_the_plan_and_lock_disagree_before_starting_anything() {
    let (_directory, scope, env) = sample_scope_on_disk();
    fs::write(&scope.plan_file, SAMPLE_PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]")).expect("edit the plan");
    let runner = FakeRunner::new();
    let http = FakeHttp::new();
    let host = Host { platform: "linux-x64".to_string(), family: Some(super::model::OsFamily::Linux) };
    let error = run_sync(&scope, &env, &runner, &http, &host).expect_err("the lock is stale");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("mpm-shadow lock"), "{error}");
    assert!(runner.calls().is_empty());
    assert!(http.requests().is_empty());
    assert!(!scope.toolchain_root.exists(), "nothing was installed");
}

#[test]
fn sync_installs_what_is_missing_in_order_with_the_locked_versions_and_skips_what_matches() {
    let mut rig = Rig::new();
    // Keep the scenario to tools whose installers need nothing else: cargo crates and the Rust toolchain.
    keep_only(&mut rig, &["rust", "cargo-binstall", "cargo:fd-find"]);
    rig.runner.answer("cargo binstall --list", "Package  Cargo  Binstall  Source\nfd-find  v1.2.3  v1.2.3  prebuilt\n", 0);
    rig.runner.answer("rustup toolchain list", "stable-x86_64-unknown-linux-gnu\n", 0);
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.unchanged, vec!["cargo:fd-find".to_string()]);
    assert_eq!(report.installed, vec!["rust".to_string(), "cargo-binstall".to_string()]);
    let lines = rig.runner.lines();
    assert!(lines.contains(&"rustup toolchain install --no-self-update nightly-2026-10-30".to_string()), "{lines:?}");
    assert!(lines.contains(&"cargo binstall --no-confirm --version 1.2.3 cargo-binstall".to_string()), "{lines:?}");
    assert!(!lines.iter().any(|line| return line.contains("fd-find") && line.contains("--no-confirm")), "the matching crate is not reinstalled");
    let rust_position = lines.iter().position(|line| return line.starts_with("rustup toolchain install")).expect("rustup ran");
    let binstall_position = lines.iter().position(|line| return line.contains("--no-confirm")).expect("binstall ran");
    assert!(rust_position < binstall_position, "the Rust toolchain installs before cargo-binstall: {lines:?}");
}

#[test]
fn sync_keeps_going_after_a_failure_and_reports_every_failure() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["cargo-binstall", "cargo:fd-find"]);
    rig.runner.fail("cargo binstall --no-confirm --version 1.2.3 cargo-binstall", "boom one");
    rig.runner.fail("cargo binstall --no-confirm --version 1.2.3 fd-find", "boom two");
    let report = sync_tools(&rig.context(), None);
    assert!(report.installed.is_empty());
    assert_eq!(report.failed.len(), 2);
    assert!(report.failed[0].1.contains("boom one"));
    assert!(report.failed[1].1.contains("boom two"));
    let text = format_sync_report(&report);
    assert!(text.contains("0 installed, 0 unchanged, 2 failed"), "{text}");
    assert!(text.contains("toolchain sync: cargo:fd-find failed: cargo binstall failed for cargo:fd-find: boom two"), "{text}");
}

#[test]
fn a_second_sync_of_a_current_machine_installs_nothing() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id == "cargo:fd-find");
    rig.runner.answer("cargo binstall --list", "fd-find  v1.2.3  v1.2.3  prebuilt\n", 0);
    let report = sync_tools(&rig.context(), None);
    assert!(report.installed.is_empty());
    assert_eq!(report.unchanged, vec!["cargo:fd-find".to_string()]);
    assert_eq!(rig.runner.lines(), vec!["cargo binstall --list".to_string()]);
}

#[test]
fn sync_skips_tools_limited_to_other_operating_systems() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id == "cargo:monochromatic-mpm-shadow");
    rig.host = Host { platform: "macos-arm64".to_string(), family: Some(super::model::OsFamily::Macos) };
    let report = sync_tools(&rig.context(), None);
    assert!(report.installed.is_empty() && report.unchanged.is_empty() && report.failed.is_empty());
    assert!(rig.runner.calls().is_empty());
}

#[test]
fn sync_can_be_limited_to_named_tools() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["cargo-binstall", "cargo:fd-find"]);
    let only = vec!["cargo:fd-find".to_string()];
    let report = sync_tools(&rig.context(), Some(&only));
    assert_eq!(report.installed, vec!["cargo:fd-find".to_string()]);
    assert!(rig.runner.lines().iter().all(|line| return !line.ends_with(" cargo-binstall")));
}

#[test]
fn sync_warns_before_replacing_a_different_installed_version() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id == "cargo:fd-find");
    rig.runner.answer("cargo binstall --list", "fd-find  v0.9.0  v0.9.0  prebuilt\n", 0);
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.installed, vec!["cargo:fd-find".to_string()]);
    assert_eq!(report.warnings.len(), 1);
    assert!(format_sync_report(&report).starts_with("toolchain sync: warning: cargo:fd-find is installed at 0.9.0"));
}

#[test]
fn the_exposure_directory_links_every_declared_bin_and_aliases_link_their_primary() {
    let rig = Rig::new();
    let node = rig.install_binary("node", "node");
    rig.install_binary("node", "npm");
    write_executable(&rig.scope.toolchain_bin_dir.join("pnpm"));
    let tools = vec![rig.lock.tool("node").expect("node"), rig.lock.tool("pnpm").expect("pnpm")];
    let report = rebuild_exposure(&rig.context(), &tools).expect("rebuilt");
    assert_eq!(report.linked, vec!["node".to_string(), "npm".to_string(), "pnpm".to_string(), "pn".to_string()]);
    assert_eq!(report.missing, vec!["npx".to_string()]);
    assert_eq!(fs::read_link(rig.scope.exposure_dir.join("node")).expect("a symlink"), node);
    assert_eq!(
        fs::read_link(rig.scope.exposure_dir.join("pn")).expect("the alias is a link to the primary's program"),
        rig.scope.toolchain_bin_dir.join("pnpm")
    );
    assert_eq!(unresolvable_bins(&rig.scope, &tools), vec!["npx".to_string()]);
}

#[test]
fn rebuilding_the_exposure_directory_twice_is_stable() {
    let rig = Rig::new();
    rig.install_binary("node", "node");
    let tools = vec![rig.lock.tool("node").expect("node")];
    rebuild_exposure(&rig.context(), &tools).expect("first");
    rebuild_exposure(&rig.context(), &tools).expect("second");
    let names: Vec<String> = fs::read_dir(&rig.scope.exposure_dir)
        .expect("list")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, vec!["node".to_string()]);
}

#[test]
fn the_postinstall_link_makes_mpm_point_at_mpm_shadow_in_the_declared_directory() {
    let rig = Rig::new();
    let cargo_bin = rig.scope.home.join(".cargo").join("bin");
    write_executable(&cargo_bin.join("mpm-shadow"));
    let tools = vec![rig.lock.tool("cargo:monochromatic-mpm-shadow").expect("shadow")];
    let created = run_postinstall_links(&rig.context(), &tools).expect("linked");
    assert_eq!(created, vec!["mpm".to_string()]);
    let link = rig.scope.root.join(".cache").join("mpm-shadow-bin").join("mpm");
    assert_eq!(fs::read_link(&link).expect("a symlink"), cargo_bin.join("mpm-shadow"));
}

#[test]
fn a_postinstall_link_is_skipped_when_the_program_is_not_installed() {
    let rig = Rig::new();
    let tools = vec![rig.lock.tool("cargo:monochromatic-mpm-shadow").expect("shadow")];
    assert!(run_postinstall_links(&rig.context(), &tools).expect("nothing to do").is_empty());
}

#[test]
fn a_postinstall_command_runs_from_the_scope_root_with_the_exposure_directory_first_on_path() {
    let mut rig = Rig::new();
    let cargo_bin = rig.scope.home.join(".cargo").join("bin");
    write_executable(&cargo_bin.join("mpm-shadow"));
    rig.lock.tools[7].declaration.options.postinstall =
        Some(Postinstall::Run(PostinstallRun { source_bin: "mpm-shadow".to_string(), args: vec!["link".to_string(), ".cache/x".to_string()] }));
    let tools = vec![rig.lock.tool("cargo:monochromatic-mpm-shadow").expect("shadow")];
    let outcome = run_postinstall_commands(&rig.context(), &tools);
    assert_eq!(outcome.succeeded, vec!["cargo:monochromatic-mpm-shadow".to_string()]);
    let call = &rig.runner.calls()[0];
    assert_eq!(call.cwd.as_ref(), Some(&rig.scope.root));
    assert_eq!(call.args, vec!["link".to_string(), ".cache/x".to_string()]);
    let path = call.env.as_ref().expect("env").get("PATH").expect("PATH").clone();
    assert!(path.starts_with(rig.scope.exposure_dir.to_str().expect("utf-8")), "{path}");
}

#[test]
fn a_failing_postinstall_command_is_reported_with_its_output() {
    let mut rig = Rig::new();
    write_executable(&rig.scope.home.join(".cargo").join("bin").join("mpm-shadow"));
    rig.lock.tools[7].declaration.options.postinstall =
        Some(Postinstall::Run(PostinstallRun { source_bin: "mpm-shadow".to_string(), args: vec!["link".to_string()] }));
    rig.runner.fail("mpm-shadow link", "cannot link");
    let tools = vec![rig.lock.tool("cargo:monochromatic-mpm-shadow").expect("shadow")];
    let outcome = run_postinstall_commands(&rig.context(), &tools);
    assert!(outcome.succeeded.is_empty());
    assert!(outcome.failed[0].1.contains("exit code 1"), "{:?}", outcome.failed);
    assert!(outcome.failed[0].1.contains("cannot link"), "{:?}", outcome.failed);
}

#[test]
fn a_full_sync_report_lists_unresolved_bins() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id == "node");
    rig.runner.answer("tar", "", 0);
    let node_url = "https://nodejs.example/node.tar.gz";
    rig.http.serve(node_url, b"x");
    rig.lock.tools[0].artifacts =
        vec![("linux-x64".to_string(), super::model::Artifact { url: node_url.to_string(), sha256: super::hash::sha256_hex(b"x") })];
    let dir = tool_install_dir(&rig.scope.toolchain_root, "node", "1.2.3");
    rig.runner.on("tar", Box::new(move |_request| write_executable(&dir.join("bin").join("node"))));
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.installed, vec!["node".to_string()]);
    assert_eq!(report.missing_bins, vec!["npm".to_string(), "npx".to_string()]);
    assert!(format_sync_report(&report).contains("unresolved bins: npm, npx"));
    assert!(rig.scope.exposure_dir.join("node").exists());
}
