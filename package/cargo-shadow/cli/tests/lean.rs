//! Integration tests of `cargo-shadow lean`:
//! the built binary run in a disposable repository with a `CARGO_HOME` that holds no configuration,
//! so nothing outside the fixture is read or removed.
//! Each run puts fake `cargo`,
//! `rustup`,
//! `podman` and `systemctl` programs first on `PATH` that log every start,
//! so the tests show what the tool started and what it never starts.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The built binary.
const BINARY: &str = env!("CARGO_BIN_EXE_cargo-shadow");

/// Where integration-test scratch goes.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// The programs the tool might start,
/// each replaced by a logging fake.
const FAKES: [&str; 4] = ["cargo", "rustup", "podman", "systemctl"];

/// One disposable world.
struct World {
    /// Keeps the directory alive.
    _guard: tempfile::TempDir,
    /// The repository root (a main checkout with an empty `.git` directory).
    repository: PathBuf,
    /// Where the fakes are.
    fakes: PathBuf,
    /// The log the fakes append to.
    log: PathBuf,
    /// An empty Cargo home.
    cargo_home: PathBuf,
}

/// Makes a world with a repository that holds one in-tree build with a stale unit and a current one.
fn make_world() -> World {
    std::fs::create_dir_all(SCRATCH_ROOT).expect("scratch root");
    let guard: tempfile::TempDir = tempfile::tempdir_in(SCRATCH_ROOT).expect("scratch directory");
    let repository: PathBuf = guard.path().join("repository");
    let fakes: PathBuf = guard.path().join("fakes");
    let cargo_home: PathBuf = guard.path().join("cargo-home");
    let log: PathBuf = guard.path().join("started.log");
    for directory in [repository.join(".git"), fakes.clone(), cargo_home.clone()] {
        std::fs::create_dir_all(directory).expect("directory");
    }
    for name in FAKES {
        let program: PathBuf = fakes.join(name);
        std::fs::write(&program, format!("#!/bin/sh\necho \"{name} $@\" >> \"{}\"\nexit 0\n", log.display())).expect("fake");
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    let profile: PathBuf = repository.join("package").join("x").join("y").join("target").join("debug");
    for directory in [".fingerprint", "deps", "build", "incremental"] {
        std::fs::create_dir_all(profile.join(directory)).expect("profile directory");
    }
    std::fs::write(profile.join(".cargo-build-lock"), b"").expect("lock file");
    return World { _guard: guard, repository, fakes, log, cargo_home };
}

/// Writes a unit into the fixture's profile directory.
fn add_unit(world: &World, name: &str, compiler: u64, age_seconds: u64) -> PathBuf {
    let profile: PathBuf = world.repository.join("package/x/y/target/debug");
    let fingerprint: PathBuf = profile.join(".fingerprint").join(format!("{name}-0123456789abcdef"));
    std::fs::create_dir_all(&fingerprint).expect("fingerprint");
    std::fs::write(fingerprint.join(format!("lib-{name}.json")), format!("{{\"rustc\":{compiler}}}")).expect("json");
    let stamp: PathBuf = fingerprint.join("invoked.timestamp");
    std::fs::write(&stamp, b"").expect("stamp");
    let modified: std::time::SystemTime = std::time::SystemTime::now() - std::time::Duration::from_secs(age_seconds);
    std::fs::File::open(&stamp).expect("open").set_modified(modified).expect("time");
    std::fs::write(profile.join("deps").join(format!("lib{name}-0123456789abcdef.rlib")), vec![1_u8; 4096]).expect("artifact");
    return fingerprint;
}

/// Runs the binary under its own name with the fakes first on `PATH`.
fn run(world: &World, directory: &Path, arguments: &[&str]) -> Output {
    let path: String = format!("{}:{}", world.fakes.display(), std::env::var("PATH").unwrap_or_default());
    return Command::new(BINARY)
        .args(arguments)
        .current_dir(directory)
        .env("PATH", path)
        .env("CARGO_HOME", &world.cargo_home)
        .env("HOME", world.cargo_home.parent().expect("parent"))
        .env_remove("RUSTUP_TOOLCHAIN")
        .output()
        .expect("start the binary");
}

/// What the fakes logged.
fn started(world: &World) -> String {
    return std::fs::read_to_string(&world.log).unwrap_or_default();
}

const FIFTEEN_DAYS: u64 = 15 * 86_400;
const ONE_HOUR: u64 = 3_600;

#[test]
fn a_run_removes_the_stale_unit_keeps_the_current_one_and_starts_no_other_program() {
    let world = make_world();
    let stale: PathBuf = add_unit(&world, "alpha", 111, FIFTEEN_DAYS);
    let current: PathBuf = add_unit(&world, "beta", 222, ONE_HOUR);
    let output: Output = run(&world, &world.repository, &["lean", "run", "--only", "units"]);
    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(output.status.code(), Some(0), "{stdout} {}", String::from_utf8_lossy(&output.stderr));
    assert!(!stale.exists() && current.exists());
    assert!(stdout.contains("older-compiler sweep: 1 of 1 profile directories read; removed 1 units"), "{stdout}");
    assert_eq!(started(&world), "", "the units mechanism starts no other program");
}

#[test]
fn a_dry_run_removes_nothing_and_says_so() {
    let world = make_world();
    let stale: PathBuf = add_unit(&world, "alpha", 111, FIFTEEN_DAYS);
    add_unit(&world, "beta", 222, ONE_HOUR);
    let output: Output = run(&world, &world.repository, &["lean", "run", "--dry-run", "--only", "units"]);
    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(stale.exists());
    assert!(stdout.contains("dry run (nothing is removed)") && stdout.contains("would remove 1 units"), "{stdout}");
}

#[test]
fn the_full_run_starts_only_podman_and_the_containment_probe_never_cargo_or_itself() {
    let world = make_world();
    let output: Output = run(&world, &world.repository, &["lean", "run", "--dry-run"]);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let log: String = started(&world);
    assert!(!log.lines().any(|line| { return line.starts_with("cargo "); }), "{log}");
    // The only `systemctl` calls are the bounded runner's own probe of the delegated control group (`src/bounded_run.rs`).
    for line in log.lines().filter(|line| { return line.starts_with("systemctl "); }) {
        assert!(line.contains("app.slice"), "unexpected systemctl call: {line}");
    }
    assert!(log.lines().any(|line| { return line.starts_with("podman images "); }), "the Podman listing must have been asked for: {log}");
    assert!(!log.contains("cargo-shadow"), "{log}");
}

#[test]
fn a_full_run_prints_the_cheap_mechanisms_before_the_units_pass() {
    let world = make_world();
    let output: Output = run(&world, &world.repository, &["lean", "run", "--dry-run"]);
    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(output.status.code(), Some(0), "{stdout} {}", String::from_utf8_lossy(&output.stderr));
    let at = |prefix: &str| -> usize { return stdout.lines().position(|line| { return line.starts_with(prefix); }).unwrap_or_else(|| panic!("no line starts with `{prefix}`: {stdout}")); };
    // The units pass is the one a three-hour deadline cuts off, so it must be last;
    // the passes behind it in the plan's order are the ones that starved (measured 2026-11-08).
    let cheapest_first: [&str; 5] = ["run records:", "toolchains:", "podman:", "idle trees:", "older-compiler sweep:"];
    let mut previous: usize = 0;
    for prefix in cheapest_first {
        let here: usize = at(prefix);
        assert!(here >= previous, "`{prefix}` comes before an earlier mechanism: {stdout}");
        previous = here;
    }
}

#[test]
fn outside_a_checkout_the_run_says_so_and_fails() {
    let world = make_world();
    let outside: PathBuf = world.fakes.join("not-a-checkout");
    std::fs::create_dir_all(&outside).expect("directory");
    let output: Output = run(&world, &outside, &["lean", "run", "--dry-run"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not inside a Git checkout"));
}

/// Writes the installed binary of the fixture and returns where the units go.
fn installed_binary(world: &World) -> PathBuf {
    let toolchain_bin: PathBuf = world.repository.join(".cache").join("toolchain-bin");
    std::fs::create_dir_all(&toolchain_bin).expect("toolchain bin");
    std::fs::write(toolchain_bin.join("cargo-shadow"), b"").expect("binary");
    return world.fakes.join("units");
}

#[test]
fn service_install_without_enabling_writes_the_continuous_service_and_runs_no_systemctl() {
    let world = make_world();
    let units: PathBuf = installed_binary(&world);
    let output: Output = run(&world, &world.repository, &["lean", "service-install", "--no-enable", "--unit-directory", units.to_str().expect("text")]);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let service: String = std::fs::read_to_string(units.join("cargo-shadow-lean.service")).expect("service unit");
    assert!(service.contains(&format!("ExecStart={}/.cache/toolchain-bin/cargo-shadow lean run", world.repository.display())));
    assert!(service.contains("Restart=always") && service.contains("WantedBy=default.target"));
    assert!(!units.join("cargo-shadow-lean.timer").exists(), "the install writes no timer");
    assert_eq!(started(&world), "");
}

#[test]
fn service_install_stops_and_disables_the_once_a_day_timer_before_it_starts_the_service() {
    let world = make_world();
    let units: PathBuf = installed_binary(&world);
    std::fs::create_dir_all(&units).expect("units");
    let timer: PathBuf = units.join("cargo-shadow-lean.timer");
    std::fs::write(&timer, "the unit the daily timer was installed from").expect("old timer");
    let output: Output = run(&world, &world.repository, &["lean", "service-install", "--unit-directory", units.to_str().expect("text")]);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!timer.exists(), "the file the daily timer was installed from must go");
    let log: String = started(&world);
    let at = |needle: &str| -> usize { return log.lines().position(|line| { return line.contains(needle); }).unwrap_or_else(|| panic!("no `systemctl` call contains `{needle}`: {log}")); };
    let stopped: usize = at("disable --now cargo-shadow-lean.timer");
    let reloaded: usize = at("daemon-reload");
    let enabled: usize = at("enable --now cargo-shadow-lean.service");
    assert!(stopped < reloaded && reloaded < enabled, "the old timer goes before the service starts: {log}");
}

#[test]
fn a_user_manager_that_cannot_stop_the_old_timer_stops_the_install_before_it_writes_anything() {
    let world = make_world();
    let units: PathBuf = installed_binary(&world);
    let manager: PathBuf = world.fakes.join("systemctl");
    std::fs::write(&manager, "#!/bin/sh\necho 'Failed to connect to bus: No medium found' >&2\nexit 1\n").expect("fake systemctl");
    std::fs::set_permissions(&manager, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let output: Output = run(&world, &world.repository, &["lean", "service-install", "--unit-directory", units.to_str().expect("text")]);
    assert_eq!(output.status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stdout));
    assert!(String::from_utf8_lossy(&output.stderr).contains("could not stop cargo-shadow-lean.timer"), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!units.join("cargo-shadow-lean.service").exists(), "nothing is written while the daily pass could still run");
}

#[test]
fn the_old_timer_install_name_says_what_it_does_now_and_installs_the_continuous_service() {
    let world = make_world();
    let units: PathBuf = installed_binary(&world);
    std::fs::create_dir_all(&units).expect("units");
    std::fs::write(units.join("cargo-shadow-lean.timer"), "the unit the daily timer was installed from").expect("old timer");
    let output: Output = run(&world, &world.repository, &["lean", "timer-install", "--no-enable", "--unit-directory", units.to_str().expect("text")]);
    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(output.status.code(), Some(0), "{stdout} {}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("service-install") && stdout.contains("continuous"), "the old name must say what it does now: {stdout}");
    let service: String = std::fs::read_to_string(units.join("cargo-shadow-lean.service")).expect("service unit");
    assert!(service.contains("Restart=always"), "{service}");
    assert!(!units.join("cargo-shadow-lean.timer").exists());
}
