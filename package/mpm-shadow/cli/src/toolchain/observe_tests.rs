//! Unit tests of the per-route observers and the status report,
//! against a recording fake runner and a disposable
//! scope.
//! Every test also asserts what was started,
//! which proves nothing reached the real host.

use std::fs;

use super::context::Host;
use super::cargo_installs::version_directory;
use super::fakes::{Rig, write_executable};
use super::fakes_cargo::finish_install;
use super::layout::tool_install_dir;
use super::model::OsFamily;
use super::observe::{format_status, observe_all, observe_tool, satisfies_lock};


#[test]
fn a_cargo_binstall_tool_is_read_from_its_version_directory_without_starting_a_program() {
    let rig = Rig::new();
    let tool = rig.lock.tool("cargo:fd-find").expect("tool").clone();
    finish_install(&version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3"), "fd-find", "1.2.3", &["fd"]);
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
    assert!(rig.runner.calls().is_empty(), "neither `cargo binstall --list` nor the shared cargo home is read");
}

#[test]
fn a_cargo_binstall_tool_without_a_finished_install_in_its_directory_is_absent() {
    let rig = Rig::new();
    let tool = rig.lock.tool("cargo:fd-find").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
    fs::create_dir_all(version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3").join("bin")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), None, "no cargo record, so the install did not finish");
}

#[test]
fn the_shared_cargo_home_record_is_not_read_for_the_binstall_route() {
    let rig = Rig::new();
    let tool = rig.lock.tool("cargo:fd-find").expect("tool").clone();
    fs::create_dir_all(rig.scope.home.join(".cargo")).expect("make");
    fs::write(rig.scope.home.join(".cargo").join(".crates.toml"), "[v1]\n\"fd-find 1.2.3 (registry+x)\" = [\"fd\"]\n").expect("write");
    assert_eq!(observe_tool(&rig.context(), &tool), None, "the legacy install belongs to the trees still on the TypeScript toolchain");
}

#[test]
fn the_toolchain_environment_leads_path_with_the_exposure_directory_and_sets_home() {
    let rig = Rig::new();
    let tool = rig.lock.tool("rust").expect("tool").clone();
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

/// pnpm writes the links of a global package relative to the spelling of its home,
/// and the store's spelling is the physical one (mise canonicalizes it):
/// a home reached through a symbolic link makes `pnpm add --global` write dangling links,
/// which `pnpm list --global` then cannot read.
/// So the listing runs with `PNPM_HOME` in its physical spelling.
#[cfg(unix)]
#[test]
fn the_npm_listing_runs_with_the_physical_spelling_of_the_pnpm_home() {
    let (rig, canonical_home) = Rig::through_symlinked_home();
    write_executable(&rig.scope.toolchain_bin_dir.join("pnpm"));
    rig.runner.answer("pnpm list --global", "[{\"dependencies\": {}}]", 0);
    let tool = rig.lock.tool("npm:wrangler").expect("tool").clone();
    observe_tool(&rig.context(), &tool);
    let call = &rig.runner.calls()[0];
    let env = call.env.as_ref().expect("an explicit environment");
    let physical_pnpm_home = canonical_home.join(".local").join("share").join("pnpm");
    assert_eq!(env.get("PNPM_HOME").map(String::as_str), Some(physical_pnpm_home.to_str().expect("utf-8")));
    let path = env.get("PATH").expect("PATH");
    let physical_bin = physical_pnpm_home.join("bin");
    assert!(
        std::env::split_paths(path).any(|directory| return directory == physical_bin),
        "pnpm refuses a global bin directory that PATH does not hold in the spelling of PNPM_HOME: {path}"
    );
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

/// A toolchain row names a channel, and `rustup toolchain list` prints that channel with the host triple:
/// the listing is read for the channel, never for the release the row records,
/// or another installed toolchain counts as this one.
#[test]
fn a_rust_toolchain_is_the_listed_name_of_the_channel_the_row_names() {
    let rig = Rig::new();
    // The row names the channel `stable`, and `1.90.0` is the release that channel resolved to.
    let mut tool = rig.lock.tool("rust").expect("tool").clone();
    tool.declaration.package_id = Some("stable".to_string());
    tool.version = "1.90.0".to_string();
    rig.runner.answer(
        "rustup toolchain list",
        "stable-x86_64-unknown-linux-gnu (active, default)\n1.90.0-x86_64-unknown-linux-gnu\n",
        0,
    );
    let observed = observe_tool(&rig.context(), &tool);
    assert_eq!(observed, Some("stable-x86_64-unknown-linux-gnu".to_string()), "the channel, not the release, is the toolchain");
    assert!(
        satisfies_lock(&tool, observed.as_deref().expect("observed")),
        "the toolchain the lock names satisfies it"
    );
}

/// The channel the row names is absent though a toolchain of the recorded release is installed:
/// that toolchain is a different one, so the row is not installed.
#[test]
fn a_toolchain_of_the_recorded_release_is_not_the_channel_the_row_names() {
    let rig = Rig::new();
    let mut tool = rig.lock.tool("rust").expect("tool").clone();
    tool.declaration.package_id = Some("stable".to_string());
    tool.version = "1.90.0".to_string();
    rig.runner.answer("rustup toolchain list", "1.90.0-x86_64-unknown-linux-gnu\n", 0);
    assert_eq!(observe_tool(&rig.context(), &tool), None, "`stable` is not installed, and 1.90.0 is another toolchain");
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
    fs::create_dir_all(tool_install_dir(&rig.scope.toolchain_root, "node", "0.9.0").expect("a plain name")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), Some("0.9.0".to_string()), "another version is reported as the observed one");
    fs::create_dir_all(tool_install_dir(&rig.scope.toolchain_root, "node", "1.2.3").expect("a plain name")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
}

/// What:
/// A Go-installed tool is read from the executables `go install` writes into its `bin` directory (the `GOBIN` the route
/// sets),
/// not from the version directory.
/// Why:
/// The route creates that directory before `go` runs,
/// so the directory alone answers "installed" for a tool whose binaries are missing,
/// and every later `install` would skip it.
#[test]
fn a_go_tool_is_absent_until_its_gobin_holds_the_executable_go_writes() {
    let rig = Rig::new();
    let mut tool = rig.tool("cargo:fd-find");
    tool.declaration.route = super::model::Route::GoInstall;
    let install = tool_install_dir(&rig.scope.toolchain_root, "cargo:fd-find", "1.2.3").expect("a plain name");
    fs::create_dir_all(install.join("bin")).expect("make the gobin the route makes before go runs");
    assert_eq!(observe_tool(&rig.context(), &tool), None, "an empty gobin is not an installed tool");
    write_executable(&install.join("bin").join("fd"));
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
    assert!(rig.runner.calls().is_empty(), "the observation starts nothing");
}

/// The other half of the same rule:
/// a declaration that names no binary is read from any executable in its `bin` directory,
/// so a `go install` that wrote nothing is still absent.
#[test]
fn a_go_tool_with_no_declared_bin_is_read_from_any_executable_in_its_gobin() {
    let rig = Rig::new();
    let mut tool = rig.tool("cargo:fd-find");
    tool.declaration.route = super::model::Route::GoInstall;
    tool.declaration.bins = Vec::new();
    let install = tool_install_dir(&rig.scope.toolchain_root, "cargo:fd-find", "1.2.3").expect("a plain name");
    fs::create_dir_all(install.join("bin")).expect("make");
    fs::write(install.join("bin").join("notes"), "not a program").expect("write a file no one can run");
    assert_eq!(observe_tool(&rig.context(), &tool), None, "a file no one can run is not an installed program");
    write_executable(&install.join("bin").join("anything"));
    assert_eq!(observe_tool(&rig.context(), &tool), Some("1.2.3".to_string()));
}

/// A Go tool left at an older version is reported by that version,
/// the way every other directory route reports one,
/// but only when that version's `bin` directory holds its executable.
#[test]
fn a_go_tool_is_read_from_an_older_version_that_still_holds_its_executable() {
    let rig = Rig::new();
    let mut tool = rig.tool("cargo:fd-find");
    tool.declaration.route = super::model::Route::GoInstall;
    let older = tool_install_dir(&rig.scope.toolchain_root, "cargo:fd-find", "0.9.0").expect("a plain name");
    fs::create_dir_all(older.join("bin")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), None, "a version directory holding no executable is not an install");
    write_executable(&older.join("bin").join("fd"));
    assert_eq!(observe_tool(&rig.context(), &tool), Some("0.9.0".to_string()), "the older finished version is what there is");
}

#[test]
fn the_android_sdk_is_present_when_its_command_line_tools_directory_exists() {
    let rig = Rig::new();
    let tool = rig.lock.tool("android-sdk").expect("tool").clone();
    assert_eq!(observe_tool(&rig.context(), &tool), None);
    fs::create_dir_all(tool_install_dir(&rig.scope.toolchain_root, "android-sdk", "23.0").expect("a plain name").join("cmdline-tools").join("23.0")).expect("make");
    assert_eq!(observe_tool(&rig.context(), &tool), Some("23.0".to_string()));
}

#[test]
fn the_status_report_marks_matches_and_drift_and_skips_tools_for_other_operating_systems() {
    let mut rig = Rig::new();
    finish_install(&version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3"), "fd-find", "1.2.3", &["fd"]);
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
    assert!(format_status(&observations).ends_with("toolchain list: 1 tools in sync with toolchain.lock.json\n"));
}

#[test]
fn an_observed_version_satisfies_a_repository_published_lock_by_the_alpha_number_read_as_a_number() {
    let rig = Rig::new();
    let tool = rig.tool("cargo:monochromatic-mpm-shadow");
    let mut locked = tool.clone();
    locked.version = "0.4.5-alpha.10".to_string();
    assert!(!super::observe::satisfies_lock(&locked, "0.4.5-alpha.9"), "alpha.9 is older than the locked alpha.10");
    locked.version = "0.4.5-alpha.9".to_string();
    assert!(super::observe::satisfies_lock(&locked, "0.4.5-alpha.10"), "alpha.10 is newer than the locked alpha.9");
}

/// The `bin` downloader of the bin-sourced route, as the fake toolchain would have installed it.
fn install_bin_downloader(rig: &Rig) -> std::path::PathBuf {
    let tool = rig.lock.tool("bin").expect("the downloader tool");
    let directory: std::path::PathBuf = tool_install_dir(&rig.scope.toolchain_root, "bin", &tool.version).expect("an install directory");
    let path: std::path::PathBuf = directory.join("bin");
    write_executable(&path);
    return path;
}

/// The positive control:
/// a row on standard output of a successful listing is read as the installed version.
/// Why:
/// A null result from the two tests below counts only when this one moves.
#[test]
fn a_bin_listing_row_on_stdout_is_read_as_installed() {
    let rig = Rig::new();
    let tool = rig.lock.tool("pnpm").expect("tool").clone();
    install_bin_downloader(&rig);
    rig.runner.answer("bin list", "pnpm  10.1.0  https://registry.example/pnpm", 0);
    assert_eq!(observe_tool(&rig.context(), &tool), Some("10.1.0".to_string()));
}

/// A row printed on stderr is not a listing:
/// every other observer reads stdout only, and a diagnostic that happens to look like a row must not
/// be recorded as an installed version.
#[test]
fn a_bin_listing_row_on_stderr_is_not_read_as_installed() {
    let rig = Rig::new();
    let tool = rig.lock.tool("pnpm").expect("tool").clone();
    install_bin_downloader(&rig);
    rig.runner.answer_streams("bin list", "", "pnpm  10.1.0  https://registry.example/pnpm", 0);
    assert_eq!(observe_tool(&rig.context(), &tool), None);
}

/// A failed listing is read as absent, not as installed:
/// a run that exited non-zero proves nothing about what is installed.
#[test]
fn a_failed_bin_listing_is_read_as_absent() {
    let rig = Rig::new();
    let tool = rig.lock.tool("pnpm").expect("tool").clone();
    install_bin_downloader(&rig);
    rig.runner.answer_streams("bin list", "pnpm  10.1.0  https://registry.example/pnpm", "", 1);
    assert_eq!(observe_tool(&rig.context(), &tool), None);
}
