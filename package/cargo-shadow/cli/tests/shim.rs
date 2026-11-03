//! Integration tests: the built binary started as `cargo` and as `cargo-shadow`, against a fake cargo on PATH.

use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The built binary, which Cargo builds before running these tests.
const SHADOW_BINARY: &str = env!("CARGO_BIN_EXE_cargo-shadow");

/// Where integration-test scratch goes: the package's build directory, never the system temporary directory.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// What the fake cargo prints first, so the tests can tell its output from the shadow's.
const FAKE_MARKER: &str = "fake-cargo-ran";

/// An exit code no default maps to, so a pass-through is visible.
const DISTINCT_EXIT_CODE: i32 = 9;

/// The crate root inside the catalog repository.
const CRATE_DIR: &str = "package/x/y";

/// The fake cargo: `update --locked` passes on a lock whose first line says satisfied; a plain `update` fixes it;
/// `metadata` prints an empty package list; anything else logs its arguments and prints the marker. Every call is logged.
const FAKE_CARGO: &str = "#!/bin/sh\n\
echo \"$@\" >> \"$FAKE_CARGO_LOG\"\n\
if [ \"$1\" = update ]; then\n\
  case \" $* \" in\n\
    *\" --locked \"*) head -n 1 Cargo.lock | grep -q satisfied; exit $? ;;\n\
  esac\n\
  printf '# satisfied\\nversion = 4\\n' > Cargo.lock\n\
  exit 0\n\
fi\n\
if [ \"$1\" = metadata ]; then\n\
  printf '{\"workspace_root\":\"%s\",\"packages\":[]}' \"$PWD\"\n\
  exit 0\n\
fi\n\
echo \"fake-cargo-ran $@\"\n\
exit \"${FAKE_CARGO_EXIT:-0}\"\n";

/// One test world: the shadow linked as `cargo`, a fake real cargo, and a catalog repository to run in.
struct World {
    _guard: tempfile::TempDir,
    shadow_directory: PathBuf,
    fake_directory: PathBuf,
    repository: PathBuf,
    log: PathBuf,
}

fn make_world() -> World {
    std::fs::create_dir_all(SCRATCH_ROOT).expect("make the scratch root");
    let guard: tempfile::TempDir = tempfile::tempdir_in(SCRATCH_ROOT).expect("make a scratch directory");
    let shadow_directory: PathBuf = guard.path().join("shadow");
    let fake_directory: PathBuf = guard.path().join("fake");
    let repository: PathBuf = guard.path().join("repository");
    for directory in [&shadow_directory, &fake_directory, &repository.join(CRATE_DIR)] {
        std::fs::create_dir_all(directory).expect("make a directory");
    }
    symlink(SHADOW_BINARY, shadow_directory.join("cargo")).expect("link the shadow as cargo");
    let fake: PathBuf = fake_directory.join("cargo");
    std::fs::write(&fake, FAKE_CARGO).expect("write the fake cargo");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).expect("make it executable");
    std::fs::write(repository.join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n")
        .expect("catalog");
    std::fs::write(
        repository.join(CRATE_DIR).join("Cargo.toml"),
        "[package]\nname = \"y\"\nversion = \"0.1.0\"\n\n[workspace]\n",
    )
    .expect("manifest");
    let log: PathBuf = guard.path().join("fake-cargo.log");
    return World { _guard: guard, shadow_directory, fake_directory, repository, log };
}

fn write_lock(world: &World, text: &str) {
    std::fs::write(world.repository.join(CRATE_DIR).join("Cargo.lock"), text).expect("write the lock");
}

fn read_lock(world: &World) -> String {
    return std::fs::read_to_string(world.repository.join(CRATE_DIR).join("Cargo.lock")).expect("read the lock");
}

fn log_lines(world: &World) -> Vec<String> {
    return std::fs::read_to_string(&world.log).unwrap_or_default().lines().map(|line| { return line.to_string(); }).collect();
}

fn path_of(directories: &[&Path]) -> String {
    return directories.iter().map(|directory| { return directory.to_string_lossy().to_string(); }).collect::<Vec<String>>().join(":");
}

/// Runs the shadow as `cargo` in the crate root with the given CI value.
fn run_shadow(world: &World, arguments: &[&str], ci: &str, fake_exit: i32) -> Output {
    let path: String = path_of(&[&world.shadow_directory, &world.fake_directory]);
    return Command::new(world.shadow_directory.join("cargo"))
        .args(arguments)
        .current_dir(world.repository.join(CRATE_DIR))
        .env("PATH", path)
        .env("CI", ci)
        .env("FAKE_CARGO_LOG", &world.log)
        .env("FAKE_CARGO_EXIT", fake_exit.to_string())
        .output()
        .expect("start the shadow");
}

const SATISFIED: &str = "# satisfied\nversion = 4\n";
const STALE: &str = "# stale\nversion = 4\n";

#[test]
fn a_command_that_resolves_nothing_goes_straight_to_cargo_with_its_own_output_and_status() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["fmt", "--check"], "", DISTINCT_EXIT_CODE);

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), format!("{FAKE_MARKER} fmt --check"));
    assert_eq!(output.status.code(), Some(DISTINCT_EXIT_CODE));
    assert_eq!(read_lock(&world), STALE, "a pass-through must not touch the lock");
    assert_eq!(log_lines(&world), vec!["fmt --check".to_string()]);
}

#[test]
fn a_stale_lock_is_updated_before_a_resolving_command_which_then_gets_its_arguments_untouched() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["check", "--workspace", "-q"], "", 0);

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), format!("{FAKE_MARKER} check --workspace -q"));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(read_lock(&world), SATISFIED);
    let last: Option<String> = log_lines(&world).last().cloned();
    assert_eq!(last, Some("check --workspace -q".to_string()), "no option may be added to the user's command");
    assert!(String::from_utf8_lossy(&output.stderr).contains(CRATE_DIR));
}

#[test]
fn in_ci_a_stale_lock_fails_naming_it_and_cargo_never_runs() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["check"], "true", 0);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains(CRATE_DIR));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(FAKE_MARKER));
    assert_eq!(read_lock(&world), STALE);
}

#[test]
fn the_users_own_locked_or_frozen_option_asks_for_the_same_failure_outside_ci() {
    for option in ["--locked", "--frozen"] {
        let world = make_world();
        write_lock(&world, STALE);

        let output = run_shadow(&world, &["check", option], "", 0);

        assert_eq!(output.status.code(), Some(1), "{option}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(CRATE_DIR), "{option}");
        assert_eq!(read_lock(&world), STALE, "{option}");
    }
}

#[test]
fn an_up_to_date_workspace_is_checked_on_the_first_call_and_the_second_starts_no_check_at_all() {
    let world = make_world();
    write_lock(&world, SATISFIED);

    run_shadow(&world, &["check"], "", 0);
    let after_first: usize = log_lines(&world).len();
    let second = run_shadow(&world, &["check"], "", 0);
    let after_second: usize = log_lines(&world).len();

    assert_eq!(second.status.code(), Some(0));
    assert_eq!(after_second, after_first + 1, "the second call may only run the user's own command: {:?}", log_lines(&world));
}

#[test]
fn outside_a_catalog_repository_even_a_resolving_command_goes_straight_through() {
    let world = make_world();
    std::fs::remove_file(world.repository.join("rust-catalog.toml")).expect("remove the marker");
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["check"], "true", 0);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(log_lines(&world), vec!["check".to_string()]);
}

#[test]
fn a_literal_dependency_written_by_add_is_warned_about_after_cargo_succeeds_and_the_status_stays_cargos() {
    let world = make_world();
    write_lock(&world, SATISFIED);
    std::fs::write(
        world.repository.join(CRATE_DIR).join("Cargo.toml"),
        "[package]\nname = \"y\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n\n[workspace]\n",
    )
    .expect("manifest");

    let output = run_shadow(&world, &["add", "serde"], "", 0);

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stderr).contains("serde"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("workspace catalog"));
    let failed = run_shadow(&world, &["add", "serde"], "", DISTINCT_EXIT_CODE);
    assert_eq!(failed.status.code(), Some(DISTINCT_EXIT_CODE));
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("workspace catalog"), "no warning after a failed add");
}

#[test]
fn version_prints_cargos_text_then_one_line_naming_the_wrapper_and_the_real_cargo() {
    let world = make_world();

    let output = run_shadow(&world, &["--version"], "", 0);

    let expected: String = format!(
        "{FAKE_MARKER} --version\ncargo-shadow {}, shadowing cargo at {}\n",
        env!("CARGO_PKG_VERSION"),
        world.fake_directory.join("cargo").display()
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn started_under_its_own_name_it_is_not_a_shadow_and_answers_help_and_version() {
    let world = make_world();
    let own: PathBuf = world.shadow_directory.join("cargo-shadow");
    symlink(SHADOW_BINARY, &own).expect("link under its own name");

    let version = Command::new(&own).arg("--version").output().expect("start");
    let help = Command::new(&own).arg("--help").output().expect("start");

    assert!(String::from_utf8_lossy(&version.stdout).starts_with("cargo-shadow "));
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(help.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&help.stdout).contains(FAKE_MARKER));
}

#[test]
fn a_missing_real_cargo_is_reported_with_the_not_found_code() {
    let world = make_world();
    let path: String = path_of(&[&world.shadow_directory]);

    let output = Command::new(world.shadow_directory.join("cargo"))
        .arg("fmt")
        .current_dir(&world.repository)
        .env("PATH", path)
        .output()
        .expect("start the shadow");

    assert_eq!(output.status.code(), Some(127));
}
