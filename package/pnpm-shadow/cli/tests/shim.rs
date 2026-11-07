//! Integration tests:
//! the built binary started as `pnpm` and as `pnpm-shadow`,
//! against a fake pnpm on PATH.

/// Imports the link helper and the executable-mode helper.
use std::os::unix::fs::{PermissionsExt, symlink};
/// Imports `.signal()` on an exit status.
use std::os::unix::process::ExitStatusExt;
/// Imports the path types.
use std::path::{Path, PathBuf};
/// Imports the child-process builder and its captured output.
use std::process::{Command, Output};

/// The built `pnpm-shadow` binary,
/// which Cargo builds before running these tests.
const SHADOW_BINARY: &str = env!("CARGO_BIN_EXE_pnpm-shadow");

/// Where integration-test scratch goes:
/// the package's build directory,
/// never the system temporary directory.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// What the fake pnpm prints first,
/// so the tests can tell its output from the shadow's.
const FAKE_MARKER: &str = "fake-pnpm-ran";

/// The fake pnpm's exit code,
/// taken from the environment variable of this name.
const EXIT_VARIABLE: &str = "FAKE_PNPM_EXIT";

/// When this variable is `1` the fake pnpm kills itself with SIGTERM after printing.
const KILL_VARIABLE: &str = "FAKE_PNPM_KILL";

/// The number of the termination signal on Linux and macOS,
/// written once and named.
const SIGTERM_NUMBER: i32 = 15;

/// An exit code no default maps to,
/// so a pass-through is visible.
const DISTINCT_EXIT_CODE: i32 = 9;

/// One test world:
/// a directory holding the shadow linked as `pnpm`,
/// a directory holding the fake real pnpm,
/// and a
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
    let script: String = format!(
        "#!/bin/sh\necho \"{FAKE_MARKER} $@\"\nif [ \"${{{KILL_VARIABLE}:-}}\" = 1 ]; then kill -TERM $$; fi\nif [ \"$1\" = nested ]; then pnpm inner; fi\nexit \"${{{EXIT_VARIABLE}:-0}}\"\n"
    );
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
    assert!(stderr.contains("warning:"));
    // The wrapper names itself only after `--version` and `--help`, never in a warning.
    assert!(!stderr.contains("pnpm-shadow"));
    assert!(!stderr.to_lowercase().contains("shadow"));
    assert!(stderr.contains("left-pad"));
    assert_eq!(output.status.code(), Some(DISTINCT_EXIT_CODE));
}

#[test]
fn a_clean_workspace_is_silent_and_pnpm_runs() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["list"], 0);

    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), format!("{FAKE_MARKER} list"));
    assert_eq!(output.status.code(), Some(0));
}

/// The text the notice prints for `--version`,
/// from the fake's own path.
fn expected_version_line(world: &World) -> String {
    return format!(
        "pnpm-shadow {}, shadowing pnpm at {}\n",
        env!("CARGO_PKG_VERSION"),
        world.fake_directory.join("pnpm").display()
    );
}

#[test]
fn version_prints_pnpms_text_then_one_line_naming_the_wrapper_and_the_real_pnpm() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let long = run_shadow(&world, &path, &["--version"], 0);
    let short = run_shadow(&world, &path, &["-v"], 0);

    let expected_long = format!("{FAKE_MARKER} --version\n{}", expected_version_line(&world));
    let expected_short = format!("{FAKE_MARKER} -v\n{}", expected_version_line(&world));
    assert_eq!(String::from_utf8_lossy(&long.stdout), expected_long);
    assert_eq!(String::from_utf8_lossy(&short.stdout), expected_short);
    assert!(long.stderr.is_empty());
}

#[test]
fn help_prints_pnpms_text_then_a_section_naming_the_wrapper() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    for word in ["--help", "-h", "help"] {
        let output = run_shadow(&world, &path, &[word], 0);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.starts_with(&format!("{FAKE_MARKER} {word}\n\n")), "word {word}");
        assert!(stdout.contains(&expected_version_line(&world)), "word {word}");
        assert!(stdout.contains("never blocks pnpm"), "word {word}");
    }
}

#[test]
fn a_commands_own_help_and_every_ordinary_run_print_exactly_pnpms_text() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let command_lines: [&[&str]; 3] = [&["install", "--help"], &["help", "install"], &["install"]];
    for command_line in command_lines {
        let output = run_shadow(&world, &path, command_line, 0);

        let expected = format!("{FAKE_MARKER} {}\n", command_line.join(" "));
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "{command_line:?}");
    }
}

#[test]
fn a_lone_version_or_help_word_does_no_scan_even_in_a_workspace_with_a_violation() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    for word in ["--version", "-v", "--help", "-h", "help"] {
        let output = run_shadow(&world, &path, &[word], 0);

        assert!(output.stderr.is_empty(), "word {word} must not scan");
        assert!(String::from_utf8_lossy(&output.stdout).contains("shadowing pnpm at"), "word {word}");
    }
}

#[test]
fn a_commands_own_help_does_no_scan_because_it_installs_nothing() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    for command_line in [["install", "--help"], ["add", "-h"]] {
        let output = run_shadow(&world, &path, &command_line, 0);

        assert!(output.stderr.is_empty(), "{command_line:?} must not scan");
    }
}

#[test]
fn only_commands_that_resolve_or_change_dependencies_warn() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let warns: [&[&str]; 7] = [
        &["add", "x"],
        &["i"],
        &["update"],
        &["remove", "x"],
        &["--filter", "app", "add", "y"],
        &["-r", "install"],
        &["recursive", "install"],
    ];
    for command_line in warns {
        let output = run_shadow(&world, &path, command_line, 0);

        assert!(String::from_utf8_lossy(&output.stderr).contains("left-pad"), "{command_line:?} must warn");
    }

    let silent: [&[&str]; 6] = [&["run", "build"], &["list"], &["root"], &["exec", "ls"], &["test"], &["--filter", "install", "list"]];
    for command_line in silent {
        let output = run_shadow(&world, &path, command_line, 0);

        assert!(output.stderr.is_empty(), "{command_line:?} must not scan");
    }
}

#[test]
fn a_signal_that_kills_pnpm_reaches_the_caller_as_that_signal() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = Command::new(world.shadow_directory.join("pnpm"))
        .arg("install")
        .current_dir(&world.workspace)
        .env("PATH", &path)
        .env(KILL_VARIABLE, "1")
        .output()
        .expect("start the shadow");

    // The shadow replaced itself with pnpm, so the caller sees pnpm's own death by signal and no exit code.
    assert_eq!(output.status.signal(), Some(SIGTERM_NUMBER));
    assert_eq!(output.status.code(), None);
}

/// How long a case may run before `timeout` kills it:
/// a loop is a failure,
/// not a hang.
const LIMIT_SECONDS: &str = "30";

/// `timeout` by absolute path:
/// the child's `PATH` holds only the shadow and the fake pnpm,
/// and Rust resolves a bare program
/// name against the `PATH` it is given.
const TIMEOUT_PROGRAM: &str = "/usr/bin/timeout";

/// The exit code `timeout` gives a command it had to kill.
const TIMED_OUT_EXIT_CODE: i32 = 124;

/// Replaces the shadow's link with the real toolchain's arrangement:
/// `pnpm` links to a launcher,
/// a separate executable
/// that starts the shadow under the name `pnpm`.
/// Its resolved path is therefore not the shadow's own executable path.
fn put_a_launcher_in_front(world: &World) {
    let launcher_directory: PathBuf = world.shadow_directory.join("..").join("launcher");
    std::fs::create_dir_all(&launcher_directory).expect("make the launcher directory");
    let launcher: PathBuf = launcher_directory.join("pnpm-shadow-launcher");
    std::fs::write(&launcher, format!("#!/bin/bash\nexec -a pnpm \"{SHADOW_BINARY}\" \"$@\"\n")).expect("write the launcher");
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).expect("make it executable");
    // The toolchain writes a record beside every launcher it makes; the shadow reads it to see that the launcher leads back to a shadow and
    // skips it without starting it. A launcher with no record is covered by `never_start_another_shadow.rs`.
    std::fs::write(
        launcher_directory.join("pnpm-shadow-launcher.launch"),
        format!("{{\"toolId\":\"cargo:monochromatic-pnpm-shadow\",\"lockFile\":\"/dev/null\",\"real\":\"{SHADOW_BINARY}\",\"scope\":\"repo\"}}"),
    )
    .expect("write the launcher record");
    let link: PathBuf = world.shadow_directory.join("pnpm");
    std::fs::remove_file(&link).expect("remove the direct link");
    symlink(&launcher, &link).expect("link pnpm to the launcher");
}

/// Runs the link under `timeout`,
/// so a loop fails the test instead of hanging it.
fn run_limited(world: &World, path: &str, arguments: &[&str]) -> Output {
    // The notice handshake keeps its marker files in the per-user runtime directory; this world gets one of its own.
    let runtime: PathBuf = world.shadow_directory.join("..").join("runtime");
    std::fs::create_dir_all(&runtime).expect("make the runtime directory");
    return Command::new(TIMEOUT_PROGRAM)
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg(LIMIT_SECONDS)
        .arg(world.shadow_directory.join("pnpm"))
        .args(arguments)
        .current_dir(&world.workspace)
        .env("PATH", path)
        .env(EXIT_VARIABLE, "0")
        .output()
        .expect("start timeout");
}

#[test]
fn through_a_launcher_both_start_paths_reach_the_real_pnpm_without_looping() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let exec_path = run_limited(&world, &path, &["install"]);
    let spawn_path = run_limited(&world, &path, &["--version"]);

    assert_ne!(exec_path.status.code(), Some(TIMED_OUT_EXIT_CODE), "the exec path looped");
    assert_ne!(spawn_path.status.code(), Some(TIMED_OUT_EXIT_CODE), "the spawn path looped");
    assert_eq!(String::from_utf8_lossy(&exec_path.stdout), format!("{FAKE_MARKER} install\n"));
    assert_eq!(exec_path.status.code(), Some(0));
    // The notice appears once, after pnpm's own line, whatever the chain between the link and pnpm.
    let version_stdout = String::from_utf8_lossy(&spawn_path.stdout);
    assert_eq!(version_stdout, format!("{FAKE_MARKER} --version\n{}", expected_version_line(&world)));
    assert_eq!(spawn_path.status.code(), Some(0));
}

#[test]
fn through_a_launcher_a_violation_warns_exactly_once() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_limited(&world, &path, &["install"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.matches("problem in the package manifests").count(), 1, "stderr: {stderr}");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_pnpm_that_pnpm_itself_runs_still_finds_the_real_pnpm_so_the_guard_does_not_leak() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    // The fake pnpm runs `pnpm inner` through PATH, which is the launcher chain again, from inside the real pnpm.
    let output = run_limited(&world, &path, &["nested"]);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{FAKE_MARKER} nested\n{FAKE_MARKER} inner\n"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_launcher_with_no_real_pnpm_behind_it_ends_with_127_and_does_not_loop() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory]);

    for arguments in [["install"], ["--version"]] {
        let output = run_limited(&world, &path, &arguments);

        assert_eq!(output.status.code(), Some(127), "{arguments:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("no real pnpm found on PATH"), "{arguments:?}");
    }
}

#[test]
fn a_failing_version_run_gets_no_notice() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["--version"], DISTINCT_EXIT_CODE);

    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("{FAKE_MARKER} --version\n"));
    assert_eq!(output.status.code(), Some(DISTINCT_EXIT_CODE));
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

    let first = run_shadow(&world, &path, &["install"], 0);
    let second = run_shadow(&world, &path, &["install"], 0);

    assert!(String::from_utf8_lossy(&first.stderr).contains("warning:"));
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
