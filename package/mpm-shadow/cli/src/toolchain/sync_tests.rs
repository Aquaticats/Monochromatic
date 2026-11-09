//! Unit tests of `install`,
//! the exposure directory and the postinstall steps,
//! against the fake runner and a disposable
//! scope:
//! the tests assert the exact program lines started and that unchanged tools are left alone.

use std::fs;

use super::cargo_installs::{current_link, version_directory};
use super::context::Host;
use super::error::ErrorKind;
use super::exposure::{rebuild_exposure, run_postinstall_commands, run_postinstall_links, unresolvable_bins};
use super::fakes::{FakeHttp, FakeRunner, Rig, write_executable};
use super::fakes_cargo::finish_install;
use super::fixtures::{SAMPLE_PLAN, sample_lock, sample_scope_on_disk};
use super::launcher::{parse_record, record_path};
use super::model::{Postinstall, PostinstallRun};
use super::sync::{replacement_warning, run_sync, sync_order, sync_tools};
use super::sync_report_text::format_sync_report;

/// Keeps only the named tools and makes `cargo-binstall` a cargo crate,
/// so the scenario needs no `bin` CLI.
fn keep_only(rig: &mut Rig, ids: &[&str]) {
    rig.lock.tools.retain(|tool| return ids.contains(&tool.declaration.id.as_str()));
    for tool in &mut rig.lock.tools {
        if tool.declaration.id == "cargo-binstall" {
            tool.declaration.route = super::model::Route::CargoBinstall;
        }
    }
}

/// The `bin` directory the repository-published sample tool is searched in (below its `current` link).
fn shadow_bin_dir(rig: &Rig) -> std::path::PathBuf {
    return current_link(&rig.scope.toolchain_root, "monochromatic-mpm-shadow").join("bin");
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
    assert!(error.message.contains("mpm-shadow install --lockfile-only --offline"), "{error}");
    assert!(runner.calls().is_empty());
    assert!(http.requests().is_empty());
    assert!(!scope.toolchain_root.exists(), "nothing was installed");
}

#[test]
fn sync_installs_what_is_missing_in_order_with_the_locked_versions_and_skips_what_matches() {
    let mut rig = Rig::new();
    // Keep the scenario to tools whose installers need nothing else: cargo crates and the Rust toolchain.
    keep_only(&mut rig, &["rust", "cargo-binstall", "cargo:fd-find"]);
    finish_install(&version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3"), "fd-find", "1.2.3", &["fd"]);
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.unchanged, vec!["cargo:fd-find".to_string()]);
    assert_eq!(report.installed, vec!["rust".to_string(), "cargo-binstall".to_string()]);
    let lines = rig.runner.lines();
    assert!(lines.contains(&"rustup toolchain install --no-self-update nightly-2026-10-30".to_string()), "{lines:?}");
    let binstall_root = version_directory(&rig.scope.toolchain_root, "cargo-binstall", "1.2.3");
    assert!(
        lines.contains(&format!("cargo binstall --no-confirm --root {} --version 1.2.3 cargo-binstall", binstall_root.display())),
        "{lines:?}"
    );
    assert!(!lines.iter().any(|line| return line.contains("fd-find") && line.contains("--no-confirm")), "the matching crate is not reinstalled");
    let rust_position = lines.iter().position(|line| return line.starts_with("rustup toolchain install")).expect("rustup ran");
    let binstall_position = lines.iter().position(|line| return line.contains("--no-confirm")).expect("binstall ran");
    assert!(rust_position < binstall_position, "the Rust toolchain installs before cargo-binstall: {lines:?}");
}

/// The path `rustup which` prints for the locked channel when another step already installed it.
const LOCKED_TOOLCHAIN_PATH: &str = "/home/someone/.rustup/toolchains/nightly-2026-10-30-x86_64-unknown-linux-gnu/bin/rustc\n";

#[test]
fn a_current_toolchain_gets_the_declared_components_and_targets_another_step_left_out() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["rust"]);
    // The sample lock declares the components `clippy` and `rust-src`, and the target `aarch64-linux-android`.
    rig.runner.answer("rustup which --toolchain nightly-2026-10-30 rustc", LOCKED_TOOLCHAIN_PATH, 0);
    rig.runner.answer(
        "rustup component list --installed --toolchain nightly-2026-10-30",
        "cargo-x86_64-unknown-linux-gnu\nrust-src\n",
        0,
    );
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "x86_64-unknown-linux-gnu\n", 0);
    let report = sync_tools(&rig.context(), None);
    let lines = rig.runner.lines();
    assert!(
        !lines.iter().any(|line| return line.starts_with("rustup toolchain install")),
        "the locked toolchain is installed, so the sync must not install it again: {lines:?}"
    );
    assert!(lines.contains(&"rustup component add clippy --toolchain nightly-2026-10-30".to_string()), "{lines:?}");
    assert!(lines.contains(&"rustup target add aarch64-linux-android --toolchain nightly-2026-10-30".to_string()), "{lines:?}");
    assert!(!lines.contains(&"rustup component add rust-src --toolchain nightly-2026-10-30".to_string()), "the installed component is not added again: {lines:?}");
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    assert!(!report.unchanged.contains(&"rust".to_string()), "a tool whose declared target was missing was not left alone: {report:?}");
    assert_eq!(report.installed, vec!["rust".to_string()]);
    let text = format_sync_report(&report);
    assert!(text.contains("added the declared component clippy"), "{text}");
    assert!(text.contains("added the declared target aarch64-linux-android"), "{text}");
    assert!(!text.contains("added the declared component rust-src"), "{text}");
}

#[test]
fn a_current_toolchain_that_already_has_its_declared_parts_is_left_alone() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["rust"]);
    rig.runner.answer("rustup which --toolchain nightly-2026-10-30 rustc", LOCKED_TOOLCHAIN_PATH, 0);
    rig.runner.answer(
        "rustup component list --installed --toolchain nightly-2026-10-30",
        "clippy-x86_64-unknown-linux-gnu\nrust-src\n",
        0,
    );
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "x86_64-unknown-linux-gnu\naarch64-linux-android\n", 0);
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.unchanged, vec!["rust".to_string()]);
    assert!(report.installed.is_empty(), "{report:?}");
    assert!(report.reconciled.is_empty(), "nothing was added, so nothing is reported as added: {report:?}");
    let lines = rig.runner.lines();
    assert!(
        !lines.iter().any(|line| return line.contains(" add ")),
        "every declared part is installed, so the sync adds nothing: {lines:?}"
    );
    assert!(!format_sync_report(&report).contains("added the declared"), "{}", format_sync_report(&report));
}

/// What a lock row whose `packageId` names a channel, and whose version records the release that channel resolved to,
/// must get from a sync: the toolchain is recognised and its parts are listed under the channel,
/// so nothing is installed again.
#[test]
fn a_toolchain_row_naming_a_channel_is_recognised_and_reconciled_under_that_channel() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["rust"]);
    for tool in &mut rig.lock.tools {
        tool.declaration.package_id = Some("stable".to_string());
        tool.version = "1.90.0".to_string();
    }
    rig.runner.answer("rustup which --toolchain stable rustc", "/home/someone/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc\n", 0);
    // The channel still holds the release the lock records, so the row is current.
    rig.runner.answer("rustc --version", "rustc 1.90.0 (abc 2026-09-01)\n", 0);
    rig.runner.answer(
        "rustup component list --installed --toolchain stable",
        "clippy-x86_64-unknown-linux-gnu\nrust-src\n",
        0,
    );
    rig.runner.answer("rustup target list --installed --toolchain stable", "x86_64-unknown-linux-gnu\naarch64-linux-android\n", 0);
    let report = sync_tools(&rig.context(), None);
    let lines = rig.runner.lines();
    assert!(
        !lines.iter().any(|line| return line.starts_with("rustup toolchain install")),
        "the channel the lock names is installed, so it is not installed again: {lines:?}"
    );
    assert!(
        lines.iter().any(|line| return line.starts_with("rustup component list --installed --toolchain stable")),
        "the parts are listed for the channel, not for the release: {lines:?}"
    );
    assert!(lines.iter().all(|line| return !line.contains(" add ")), "every declared part is installed: {lines:?}");
    assert_eq!(report.unchanged, vec!["rust".to_string()], "{report:?}");
}

#[test]
fn a_failed_target_add_fails_the_tool_and_keeps_the_component_it_added_to_the_report() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["rust"]);
    rig.runner.answer("rustup which --toolchain nightly-2026-10-30 rustc", LOCKED_TOOLCHAIN_PATH, 0);
    rig.runner.answer("rustup component list --installed --toolchain nightly-2026-10-30", "rust-src\n", 0);
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "x86_64-unknown-linux-gnu\n", 0);
    rig.runner.fail("rustup target add aarch64-linux-android", "error: component 'rust-std' for target 'aarch64-linux-android' is unavailable");
    let report = sync_tools(&rig.context(), None);
    assert!(!report.unchanged.contains(&"rust".to_string()), "a failed add was reported as unchanged: {report:?}");
    assert!(report.installed.is_empty(), "a tool with a failed add is not installed: {report:?}");
    assert_eq!(report.failed.len(), 1, "{report:?}");
    assert_eq!(report.failed[0].0, "rust");
    assert!(report.failed[0].1.contains("is unavailable"), "{:?}", report.failed);
    assert_eq!(report.reconciled.len(), 1, "the component added before the failure stays reported: {report:?}");
    assert!(report.reconciled[0].contains("added the declared component clippy"), "{:?}", report.reconciled);
    let text = format_sync_report(&report);
    assert!(text.contains("added the declared component clippy"), "{text}");
    assert!(text.contains("rust failed: rustup target add (aarch64-linux-android) failed for rust"), "{text}");
}

#[test]
fn sync_keeps_going_after_a_failure_and_reports_every_failure() {
    let mut rig = Rig::new();
    keep_only(&mut rig, &["cargo-binstall", "cargo:fd-find"]);
    let first_root = version_directory(&rig.scope.toolchain_root, "cargo-binstall", "1.2.3");
    let second_root = version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3");
    rig.runner.fail(&format!("cargo binstall --no-confirm --root {}", first_root.display()), "boom one");
    rig.runner.fail(&format!("cargo binstall --no-confirm --root {}", second_root.display()), "boom two");
    let report = sync_tools(&rig.context(), None);
    assert!(report.installed.is_empty());
    assert_eq!(report.failed.len(), 2);
    assert!(report.failed[0].1.contains("boom one"));
    assert!(report.failed[1].1.contains("boom two"));
    let text = format_sync_report(&report);
    assert!(text.contains("0 installed, 0 unchanged, 2 failed"), "{text}");
    assert!(text.contains("toolchain install: cargo:fd-find failed: cargo binstall failed for cargo:fd-find: boom two"), "{text}");
}

#[test]
fn a_second_sync_of_a_current_machine_installs_nothing() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id == "cargo:fd-find");
    finish_install(&version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3"), "fd-find", "1.2.3", &["fd"]);
    let report = sync_tools(&rig.context(), None);
    assert!(report.installed.is_empty());
    assert_eq!(report.unchanged, vec!["cargo:fd-find".to_string()]);
    assert!(rig.runner.calls().is_empty(), "the installed version is read from disk");
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
    rig.lock.tools.retain(|tool| return tool.declaration.id == "pipx:slopo" || tool.declaration.id == "uv");
    rig.install_binary("uv", "uv");
    // The tool is listed at another version before the install, and at the locked one after it: the route reads it back.
    rig.runner.answer("uv tool list", "slopo v0.9.0\n- slopo\n", 0);
    rig.runner.answer_after("uv tool install", "uv tool list", "slopo v1.2.3\n- slopo\n", 0);
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.installed, vec!["pipx:slopo".to_string()]);
    assert_eq!(report.warnings.len(), 1);
    assert!(format_sync_report(&report).starts_with("toolchain install: warning: pipx:slopo is installed at 0.9.0"));
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
    let bin = shadow_bin_dir(&rig);
    write_executable(&bin.join("mpm-shadow"));
    let tools = vec![rig.lock.tool("cargo:monochromatic-mpm-shadow").expect("shadow")];
    let created = run_postinstall_links(&rig.context(), &tools).expect("linked");
    assert_eq!(created, vec!["mpm".to_string()]);
    let link = rig.scope.root.join(".cache").join("mpm-shadow-bin").join("mpm");
    assert_eq!(
        fs::read_link(&link).expect("a symlink"),
        rig.scope.launcher_dir.join("mpm-shadow"),
        "a repository-published tool is reached through this tree's launcher, which names the program in its record"
    );
    let record = parse_record(&fs::read_to_string(record_path(&rig.scope.launcher_dir.join("mpm-shadow"))).expect("a record")).expect("parses");
    assert_eq!(record.real, bin.join("mpm-shadow"));
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
    write_executable(&shadow_bin_dir(&rig).join("mpm-shadow"));
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
    write_executable(&shadow_bin_dir(&rig).join("mpm-shadow"));
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
    // The archive is unpacked into the staging directory `--directory` names.
    rig.runner.on(
        "tar",
        Box::new(|request| {
            let position = request.args.iter().position(|argument| return argument == "--directory").expect("a directory");
            write_executable(&std::path::PathBuf::from(&request.args[position + 1]).join("bin").join("node"));
        }),
    );
    let report = sync_tools(&rig.context(), None);
    assert_eq!(report.installed, vec!["node".to_string()]);
    assert_eq!(report.missing_bins, vec!["npm".to_string(), "npx".to_string()]);
    assert!(format_sync_report(&report).contains("unresolved bins: npm, npx"));
    assert!(rig.scope.exposure_dir.join("node").exists());
}

/// A `PATH` that cannot hold the exposure directory refuses the postinstall command:
/// running it on the inherited `PATH` alone would miss the tools that directory links,
/// and the caller learns why instead of the run looking like a success.
#[test]
fn a_path_that_cannot_hold_the_exposure_directory_refuses_the_postinstall_command() {
    let mut rig = Rig::new();
    write_executable(&shadow_bin_dir(&rig).join("mpm-shadow"));
    rig.lock.tools[7].declaration.options.postinstall =
        Some(Postinstall::Run(PostinstallRun { source_bin: "mpm-shadow".to_string(), args: vec!["link".to_string()] }));
    rig.scope.exposure_dir = rig.scope.root.join("a:b");
    let tools = vec![rig.lock.tool("cargo:monochromatic-mpm-shadow").expect("shadow")];
    let outcome = run_postinstall_commands(&rig.context(), &tools);
    assert!(outcome.succeeded.is_empty(), "{outcome:?}");
    assert!(outcome.failed[0].1.contains("cannot be put on PATH"), "{outcome:?}");
    assert!(rig.runner.calls().is_empty(), "no command ran on the inherited PATH alone");
}
