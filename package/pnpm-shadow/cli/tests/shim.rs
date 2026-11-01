//! Integration tests: the built binary started as `pnpm` and as `pnpm-shadow`, against a fake pnpm on PATH.

/// Imports the link helper and the executable-mode helper.
use std::os::unix::fs::{PermissionsExt, symlink};
/// Imports the path types.
use std::path::{Path, PathBuf};
/// Imports the child-process builder and its captured output.
use std::process::{Command, Output};

/// The built `pnpm-shadow` binary, which Cargo builds before running these tests.
const SHADOW_BINARY: &str = env!("CARGO_BIN_EXE_pnpm-shadow");

/// Where integration-test scratch goes: the package's build directory, never the system temporary directory.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// What the fake pnpm prints first, so the tests can tell its output from the shadow's.
const FAKE_MARKER: &str = "fake-pnpm-ran";

/// The fake pnpm's exit code, taken from the environment variable of this name.
const EXIT_VARIABLE: &str = "FAKE_PNPM_EXIT";

/// An exit code no default maps to, so a pass-through is visible.
const DISTINCT_EXIT_CODE: i32 = 9;

/// One test world: a directory holding the shadow linked as `pnpm`, a directory holding the fake real pnpm, and a
/// workspace directory to run in.
struct World {
    _guard: tempfile::TempDir,
    shadow_directory: PathBuf,
    fake_directory: PathBuf,
    workspace: PathBuf,
}

fn make_world() -> World {
    std::fs::create_dir_all(SCRATCH_ROOT).expect("make the scratch root");
    let guard: tempfile::TempDir = tempfile::tempdir_in(SCRATCH_ROOT).expect("make a scratch directory");
    let shadow_directory: PathBuf = guard.path().join("shadow");
    let fake_directory: PathBuf = guard.path().join("fake");
    let workspace: PathBuf = guard.path().join("workspace");
    for directory in [&shadow_directory, &fake_directory, &workspace] {
        std::fs::create_dir_all(directory).expect("make a directory");
    }
    symlink(SHADOW_BINARY, shadow_directory.join("pnpm")).expect("link the shadow as pnpm");

    let fake: PathBuf = fake_directory.join("pnpm");
    let script: String = format!("#!/bin/sh\necho \"{FAKE_MARKER} $@\"\nexit \"${{{EXIT_VARIABLE}:-0}}\"\n");
    std::fs::write(&fake, script).expect("write the fake pnpm");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).expect("make it executable");
    return World { _guard: guard, shadow_directory, fake_directory, workspace };
}

fn path_of(directories: &[&Path]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for directory in directories {
        parts.push(directory.to_string_lossy().to_string());
    }
    return parts.join(":");
}

fn run_shadow(world: &World, path: &str, arguments: &[&str], fake_exit: i32) -> Output {
    return Command::new(world.shadow_directory.join("pnpm"))
        .args(arguments)
        .current_dir(&world.workspace)
        .env("PATH", path)
        .env(EXIT_VARIABLE, fake_exit.to_string())
        .output()
        .expect("start the shadow");
}

fn write_workspace(world: &World, manifest: &str) {
    std::fs::write(world.workspace.join("pnpm-workspace.yaml"), "packages: []\n").expect("write the marker");
    std::fs::write(world.workspace.join("package.json"), manifest).expect("write the manifest");
}

const LITERAL: &str = r#"{ "dependencies": { "left-pad": "1.3.0" } }"#;
const CLEAN: &str = r#"{ "dependencies": { "x": "catalog:" } }"#;

#[test]
fn a_violation_warns_on_stderr_and_pnpm_runs_with_its_own_output_and_status() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["install", "--frozen-lockfile"], DISTINCT_EXIT_CODE);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stdout.trim_end(), format!("{FAKE_MARKER} install --frozen-lockfile"));
    assert!(stderr.contains("pnpm-shadow: warning:"));
    assert!(stderr.contains("left-pad"));
    assert_eq!(output.status.code(), Some(DISTINCT_EXIT_CODE));
}

#[test]
fn a_clean_workspace_is_silent_and_pnpm_runs() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["--version"], 0);

    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), format!("{FAKE_MARKER} --version"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_directory_outside_any_workspace_is_silent_and_pnpm_runs() {
    let world = make_world();
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["help"], 0);

    assert!(output.stderr.is_empty());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn the_warning_repeats_on_every_run() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let first = run_shadow(&world, &path, &["list"], 0);
    let second = run_shadow(&world, &path, &["list"], 0);

    assert!(String::from_utf8_lossy(&first.stderr).contains("pnpm-shadow: warning:"));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn with_no_real_pnpm_on_path_it_exits_127_after_the_warning() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory]);

    let output = run_shadow(&world, &path, &["install"], 0);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("left-pad"));
    assert!(stderr.contains("no real pnpm found on PATH"));
    assert_eq!(output.status.code(), Some(127));
}

#[test]
fn started_under_its_own_name_it_answers_version_and_help_and_runs_no_pnpm() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.fake_directory]);

    let version = Command::new(SHADOW_BINARY)
        .arg("--version")
        .current_dir(&world.workspace)
        .env("PATH", &path)
        .output()
        .expect("start the shadow");
    let help = Command::new(SHADOW_BINARY)
        .arg("--help")
        .current_dir(&world.workspace)
        .env("PATH", &path)
        .output()
        .expect("start the shadow");

    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert!(String::from_utf8_lossy(&help.stdout).contains("Shadows pnpm"));
    assert!(version.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&help.stdout).contains(FAKE_MARKER));
}
