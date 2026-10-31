//! Unit tests of the per-route observers and the status report, against a recording fake runner and a disposable
//! scope. Every test also asserts what was started, which proves nothing reached the real host.

use std::fs;

use super::context::Host;
use super::fakes::{Rig, write_executable};
use super::layout::tool_install_dir;
use super::model::OsFamily;
use super::observe::{format_status, observe_all, observe_tool};


#[test]
fn a_cargo_binstall_tool_is_read_from_the_binstall_list() {
    let rig = Rig::new();
    rig.runner.answer("cargo binstall --list", "Package  Cargo  Binstall  Source\nfd-find  v1.2.3  v1.2.3  prebuilt\n", 0);
    let tool = rig.lock.tool("cargo:fd-find").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
    assert_eq!(rig.runner.lines(), vec!["cargo binstall --list".to_string()]);
}

#[test]
fn a_cargo_binstall_tool_missing_from_the_list_is_absent() {
    let rig = Rig::new();
    rig.runner.answer("cargo binstall --list", "Package  Cargo  Binstall  Source\n", 0);
    let tool = rig.lock.tool("cargo:fd-find").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
}

#[test]
fn the_toolchain_environment_leads_path_with_the_exposure_directory_and_sets_home() {
    let rig = Rig::new();
    let tool = rig.lock.tool("cargo:fd-find").expect("tool").clone();
    observe_tool(&rig.context(), &tool);
    let call = &rig.runner.calls()[0];
    let env = call.env.as_ref().expect("an explicit environment");
    assert_eq!(env.get("HOME").map(String::as_str), Some(rig.scope.home.to_str().expect("utf-8")));
    let path = env.get("PATH").expect("PATH");
    assert!(path.starts_with(rig.scope.exposure_dir.to_str().expect("utf-8")), "{path}");
}

#[test]
fn an_npm_tool_is_absent_until_pnpm_is_installed_and_then_read_from_its_json_listing() {
    let rig = Rig::new();
    let tool = rig.lock.tool("npm:wrangler").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
    assert!(rig.runner.calls().is_empty(), "no pnpm is installed, so nothing is started");

    let pnpm_home = rig.scope.toolchain_bin_dir.join("pnpm");
    write_executable(&pnpm_home);
    rig.runner.answer("pnpm list --global", "[{\"dependencies\": {\"wrangler\": {\"version\": \"4.1.0\"}}}]", 0);
    assert_eq!(observe_tool(&rig.context(), &tool), Some("4.1.0".to_string()));
}

#[test]
fn an_npm_listing_that_is_not_json_reads_as_absent() {
    let rig = Rig::new();
    write_executable(&rig.scope.toolchain_bin_dir.join("pnpm"));
    rig.runner.answer("pnpm list", "ERR_PNPM_SOMETHING", 1);
    let tool = rig.lock.tool("npm:wrangler").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
}

#[test]
fn a_python_tool_is_read_from_the_uv_tool_list_by_its_underscored_name() {
    let rig = Rig::new();
    rig.install_binary("uv", "uv");
    rig.runner.answer("uv tool list", "slopo v0.9.0\n- slopo\n", 0);
    let tool = rig.lock.tool("pipx:slopo").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), Some("0.9.0".to_string()));
}

#[test]
fn a_python_tool_is_absent_when_uv_itself_is_not_installed() {
    let rig = Rig::new();
    let tool = rig.lock.tool("pipx:slopo").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
}

#[test]
fn a_rust_toolchain_is_the_listed_name_that_starts_with_the_locked_channel() {
    let rig = Rig::new();
    rig.runner.answer(
        "rustup toolchain list",
        "stable-x86_64-unknown-linux-gnu\nnightly-2026-10-30-x86_64-unknown-linux-gnu (active, default)\n",
        0,
    );
    let tool = rig.lock.tool("rust").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), Some("nightly-2026-10-30-x86_64-unknown-linux-gnu".to_string()));
    rig.runner.answer("rustup toolchain list", "stable-x86_64-unknown-linux-gnu\n", 0);
}

#[test]
fn a_bin_sourced_tool_is_read_from_bin_list_with_the_scope_config() {
    let rig = Rig::new();
    rig.install_binary("bin", "bin");
    rig.runner.answer("bin list", "/x/pnpm  v1.2.3  https://example.test  ok\n", 0);
    let tool = rig.lock.tool("pnpm").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
    let call = &rig.runner.calls()[0];
    let env = call.env.as_ref().expect("an explicit environment");
    assert_eq!(
        env.get("BIN_CONFIG").map(String::as_str),
        Some(rig.scope.bin_config_file.to_str().expect("utf-8"))
    );
}

#[test]
fn a_bin_sourced_tool_is_absent_without_the_bin_cli() {
    let rig = Rig::new();
    let tool = rig.lock.tool("pnpm").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
    assert!(rig.runner.calls().is_empty());
}

#[test]
fn an_archive_tool_is_its_version_directory() {
    let rig = Rig::new();
    let tool = rig.lock.tool("node").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
    fs::create_dir_all(tool_install_dir(&rig.scope.toolchain_root, "node", "0.9.0")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), Some("0.9.0".to_string()), "another version is reported as the observed one");
    fs::create_dir_all(tool_install_dir(&rig.scope.toolchain_root, "node", "1.2.3")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
}

#[test]
fn the_android_sdk_is_present_when_its_command_line_tools_directory_exists() {
    let rig = Rig::new();
    let tool = rig.lock.tool("android-sdk").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
    fs::create_dir_all(tool_install_dir(&rig.scope.toolchain_root, "android-sdk", "23.0").join("cmdline-tools").join("23.0")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), Some("23.0".to_string()));
}

#[test]
fn the_status_report_marks_matches_and_drift_and_skips_tools_for_other_operating_systems() {
    let mut rig = Rig::new();
    rig.runner.answer("cargo binstall --list", "Package  Cargo  Binstall  Source\nfd-find  v1.2.3  v1.2.3  prebuilt\nmonochromatic-mpm-shadow  v0.0.1  v0.0.1  prebuilt\n", 0);
    rig.host = Host { platform: "macos-arm64".to_string(), family: Some(OsFamily::Macos) };
    let observations = observe_all(&rig.context(), &rig.lock.tools);
    assert!(observations.iter().all(|observation| return observation.id != "cargo:monochromatic-mpm-shadow"), "linux-only tool is skipped on macOS");
    let fd = observations.iter().find(|observation| return observation.id == "cargo:fd-find").expect("fd");
    assert!(fd.in_sync);
    let node = observations.iter().find(|observation| return observation.id == "node").expect("node");
    assert!(!node.in_sync);
    assert_eq!(node.observed, None);

    let text = format_status(&observations);
    assert!(text.contains("ok      cargo:fd-find  1.2.3\n"), "{text}");
    assert!(text.contains("drift   node  locked=1.2.3  observed=(absent)\n"), "{text}");
    assert!(text.contains("tools drifted from toolchain.lock.json"), "{text}");
}

#[test]
fn a_fully_matching_report_says_so() {
    let observations = vec![super::observe::Observation {
        id: "a".to_string(),
        locked: "1".to_string(),
        observed: Some("1".to_string()),
        in_sync: true,
    }];
    assert!(format_status(&observations).ends_with("toolchain status: 1 tools in sync with toolchain.lock.json\n"));
}
