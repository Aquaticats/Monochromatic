//! Integration tests:
//! the built binary started as `cargo` and as `cargo-shadow`,
//! against a fake cargo on PATH.

use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The built binary,
/// which Cargo builds before running these tests.
const SHADOW_BINARY: &str = env!("CARGO_BIN_EXE_cargo-shadow");

/// Where integration-test scratch goes:
/// the package's build directory,
/// never the system temporary directory.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// What the fake cargo prints first,
/// so the tests can tell its output from the shadow's.
const FAKE_MARKER: &str = "fake-cargo-ran";

/// An exit code no default maps to,
/// so a pass-through is visible.
const DISTINCT_EXIT_CODE: i32 = 9;

/// The crate root inside the catalog repository.
const CRATE_DIR: &str = "package/x/y";

/// The fake cargo:
/// `update --locked` passes on a lock whose first line says satisfied;
/// a plain `update` fixes it;
/// `metadata` prints an empty package list;
/// anything else logs its arguments and prints the marker.
/// Every call is logged.
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
case \" $* \" in\n\
  *\" --locked \"*|*\" --frozen \"*) if ! head -n 1 Cargo.lock | grep -q satisfied; then echo 'error: the lock file needs to be updated but --locked was passed' >&2; exit 101; fi ;;\n\
esac\n\
echo \"fake-cargo-ran $@\"\n\
exit \"${FAKE_CARGO_EXIT:-0}\"\n";

/// One test world:
/// the shadow linked as `cargo`,
/// a fake real cargo,
/// and a catalog repository to run in.
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
fn in_ci_a_stale_lock_warns_naming_it_then_cargo_runs_with_locked_and_stops_on_its_own_lock_error() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["check"], "true", 0);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(101), "{stderr}");
    assert!(stderr.contains(CRATE_DIR), "{stderr}");
    assert!(stderr.contains("warning"), "{stderr}");
    assert!(stderr.contains("--locked"), "the warning must say that --locked was added: {stderr}");
    assert!(!String::from_utf8_lossy(&output.stdout).contains(FAKE_MARKER));
    assert_eq!(log_lines(&world).last().cloned(), Some("check --locked".to_string()));
    assert_eq!(read_lock(&world), STALE);
}

#[test]
fn the_users_own_locked_or_frozen_option_is_not_added_twice_and_the_failure_is_cargos() {
    for option in ["--locked", "--frozen"] {
        let world = make_world();
        write_lock(&world, STALE);

        let output = run_shadow(&world, &["check", option], "", 0);

        let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
        assert_eq!(output.status.code(), Some(101), "{option}: {stderr}");
        assert!(stderr.contains(CRATE_DIR), "{option}: {stderr}");
        assert_eq!(log_lines(&world).last().cloned(), Some(format!("check {option}")), "{option}");
        assert_eq!(read_lock(&world), STALE, "{option}");
    }
}

#[test]
fn locked_goes_before_a_double_dash_so_it_stays_cargos_option_and_not_the_programs() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["run", "--", "--flag"], "true", 0);

    assert_eq!(output.status.code(), Some(101));
    assert_eq!(log_lines(&world).last().cloned(), Some("run --locked -- --flag".to_string()));
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

#[test]
fn cargo_update_is_answered_as_pnpm_update_and_cargos_own_resolver_never_sees_the_bare_command() {
    let world = make_world();
    write_lock(&world, SATISFIED);

    let output = run_shadow(&world, &["update"], "", 0);

    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let lines: Vec<String> = log_lines(&world);
    assert!(lines.iter().any(|line| { return line.contains("generate-lockfile"); }), "{lines:?}");
    assert!(!lines.contains(&"update".to_string()), "the bare cargo update must not reach cargo: {lines:?}");
}

#[test]
fn cargo_update_locked_is_pnpm_install_frozen_lockfile_and_fails_naming_a_stale_lock() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["update", "--locked"], "", 0);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains(CRATE_DIR));
    assert_eq!(read_lock(&world), STALE);
}

#[test]
fn cargo_generate_lockfile_keeps_locked_versions_by_running_the_install_step_instead() {
    let world = make_world();
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["generate-lockfile"], "", 0);

    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(read_lock(&world), SATISFIED);
    assert!(!log_lines(&world).iter().any(|line| { return line.contains("generate-lockfile"); }));
}

#[test]
fn precise_for_an_indirect_dependency_is_refused_as_pnpm_refuses_it() {
    let world = make_world();
    write_lock(&world, SATISFIED);

    let output = run_shadow(&world, &["update", "serde", "--precise", "1.0.0"], "", 0);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a direct dependency"));
    assert!(!log_lines(&world).iter().any(|line| { return line.contains("--precise"); }));
}

#[test]
fn outside_a_catalog_repository_cargo_update_goes_to_cargo_untouched() {
    let world = make_world();
    std::fs::remove_file(world.repository.join("rust-catalog.toml")).expect("remove the marker");
    write_lock(&world, STALE);

    let output = run_shadow(&world, &["update", "-p", "serde"], "", 0);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(log_lines(&world), vec!["update -p serde".to_string()]);
}

/// A launcher:
/// a `cargo` that runs the shadow again,
/// as the toolchain plan's launcher-backed links do.
/// It carries a nesting
/// counter in the environment and gives up (exit 99) at the fourth nested level,
/// so a regression of the loop guard fails fast
/// with a small process tree instead of looping;
/// it also counts its runs in a file.
fn write_launcher(world: &World) -> (PathBuf, PathBuf) {
    let directory: PathBuf = world.shadow_directory.parent().expect("a parent").join("launcher");
    std::fs::create_dir_all(&directory).expect("make the launcher directory");
    let count: PathBuf = directory.join("count");
    let script: String = format!(
        "#!/bin/sh\necho run >> '{count}'\ndepth=$((${{LAUNCHER_DEPTH:-0}} + 1))\nif [ \"$depth\" -gt 3 ]; then echo 'launcher nested too deep' >&2; exit 99; fi\nexport LAUNCHER_DEPTH=$depth\nexec '{shadow}' \"$@\"\n",
        count = count.display(),
        shadow = world.shadow_directory.join("cargo").display()
    );
    let launcher: PathBuf = directory.join("cargo");
    std::fs::write(&launcher, script).expect("write the launcher");
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).expect("make it executable");
    return (directory, count);
}

/// Runs a command with a deadline,
/// in its own process group so a runaway tree can be killed whole;
/// `None` on a timeout.
fn output_within(command: &mut Command, seconds: u64) -> Option<Output> {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let mut child = command.process_group(0).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("start the command");
    let started: Instant = Instant::now();
    loop {
        if child.try_wait().expect("poll").is_some() {
            return Some(child.wait_with_output().expect("collect the output"));
        }
        if started.elapsed() > Duration::from_secs(seconds) {
            let _ = Command::new("kill").arg("-KILL").arg(format!("-{}", child.id())).status();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn an_unrecorded_launcher_that_leads_back_stops_the_second_start_at_once_and_its_message_fixes_the_problem_when_copied() {
    let world = make_world();
    write_lock(&world, STALE);
    let (launcher_directory, count) = write_launcher(&world);
    let launcher: PathBuf = launcher_directory.join("cargo");
    let path: String = path_of(&[&world.shadow_directory, &launcher_directory, &world.fake_directory]);
    let run = || -> Output {
        let mut command = Command::new(world.shadow_directory.join("cargo"));
        command
            .args(["check", "-q"])
            .current_dir(world.repository.join(CRATE_DIR))
            .env("PATH", &path)
            .env("CI", "")
            .env("FAKE_CARGO_LOG", &world.log);
        return output_within(&mut command, 20).expect("the command timed out: the shadow keeps leading back into the launcher");
    };

    let output = run();

    // The second start stops before doing any work, with status 2: no lockfile was touched, and no cargo ran.
    let said: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(2), "{said}");
    let runs: usize = std::fs::read_to_string(&count).unwrap_or_default().lines().count();
    // The first shadow reaches the launcher once for each real-cargo start it makes (the install step's calls, then the command itself);
    // every one of those starts is a second cargo-shadow that stopped at once, so no cargo ran and nothing was changed.
    assert!(runs >= 1, "the launcher never ran, so the schedule was not built");
    assert_eq!(read_lock(&world), STALE, "the second start touched the lockfile");
    assert!(log_lines(&world).is_empty(), "a cargo ran: {:?}", log_lines(&world));
    assert!(said.matches("snapshotted").count() <= 1, "a second start began an install step: {said}");
    // The message names what is real for this fixture: the launcher it chose, the shadow it is, the PATH it saw, and the real cargo.
    assert!(said.contains(&launcher.display().to_string()), "the launcher is not named: {said}");
    assert!(said.contains(&path), "the PATH it saw is not shown: {said}");
    assert!(said.contains(&world.fake_directory.join("cargo").display().to_string()), "the real cargo is not listed: {said}");
    assert!(said.contains("cargo --version") && said.contains("Fix A") && said.contains("Fix B"), "{said}");

    // Fix A, copied as printed: the export line puts the real cargo's folder first, and the command then reaches the real cargo.
    let export: &str = said.lines().map(str::trim).find(|line| { return line.starts_with("export PATH="); }).expect("an export line");
    let fixed_path: String = {
        let shown = Command::new("/bin/sh").arg("-c").arg(format!("{export}; printf %s \"$PATH\"")).env("PATH", &path).output().expect("runs the export line");
        assert!(shown.status.success(), "the export line does not run as printed: {export}");
        String::from_utf8_lossy(&shown.stdout).into_owned()
    };
    assert!(fixed_path.starts_with(&world.fake_directory.display().to_string()), "the export line did not put the real cargo's folder first: {fixed_path}");
    let mut fixed = Command::new(world.shadow_directory.join("cargo"));
    fixed.args(["check", "-q"]).current_dir(world.repository.join(CRATE_DIR)).env("PATH", &fixed_path).env("CI", "").env("FAKE_CARGO_LOG", &world.log);
    let after_a = output_within(&mut fixed, 20).expect("the fixed command timed out");
    assert_eq!(after_a.status.code(), Some(0), "{}", String::from_utf8_lossy(&after_a.stderr));
    assert_eq!(String::from_utf8_lossy(&after_a.stdout).trim_end(), format!("{FAKE_MARKER} check -q"));

    // Fix B, copied as printed: the record command writes the launcher's `.launch` file, after which the original PATH works.
    let record_command: &str = said.lines().map(str::trim).find(|line| { return line.starts_with("printf %s '{"); }).expect("a record line");
    let written = Command::new("/bin/sh").arg("-c").arg(record_command).output().expect("runs the record line");
    assert!(written.status.success(), "the record line does not run as printed: {record_command}: {}", String::from_utf8_lossy(&written.stderr));
    assert!(launcher_directory.join("cargo.launch").exists(), "the record line did not write the record beside the launcher");
    std::fs::remove_file(&world.log).ok();
    write_lock(&world, STALE);
    let after_b = run();
    assert_eq!(after_b.status.code(), Some(0), "{}", String::from_utf8_lossy(&after_b.stderr));
    assert_eq!(String::from_utf8_lossy(&after_b.stdout).trim_end(), format!("{FAKE_MARKER} check -q"));
}

/// Review finding (recursion guard kind):
/// `cargo add` and `cargo remove` are started as a child
/// (the shadow stays alive to warn afterwards),
/// so the guard it leaves must be the spawn kind.
/// With the exec kind a launcher that execs the shadow again has a different process id than the
/// guard names,
/// the guard is ignored,
/// the hop count restarts,
/// and the chain never reaches the cap.
#[test]
fn an_unrecorded_launcher_that_leads_back_stops_the_second_start_of_add_with_status_2() {
    let world = make_world();
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    write_lock(&world, SATISFIED);
    // Borrow `&world`: lends the value without copying or giving it away.
    let (launcher_directory, _count) = write_launcher(&world);
    // Borrow `&[`: lends the value without copying or giving it away.
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let path: String = path_of(&[&world.shadow_directory, &launcher_directory, &world.fake_directory]);
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    let mut command = Command::new(world.shadow_directory.join("cargo"));
    command
        .args(["add", "serde"])
        .current_dir(world.repository.join(CRATE_DIR))
        .env("PATH", path)
        .env("CI", "")
        // Borrow `&world.log`: lends the value without copying or giving it away.
        .env("FAKE_CARGO_LOG", &world.log);

    // Borrow `&mut command`: lends the value without copying or giving it away.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    let output = output_within(&mut command, 20).expect("the command timed out: the shadow keeps leading back into the launcher");

    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // Borrow `&output.stderr`: lends the value without copying or giving it away.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    // The shadow started `cargo add` as a child of itself (it stays alive to warn afterwards), so the second start is a spawn guard:
    // it stops at once with status 2 and the message, and the real cargo never ran (owner decision of 2026-11-04, option A).
    assert_eq!(output.status.code(), Some(2), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("because cargo-shadow was started by cargo-shadow"), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(log_lines(&world).is_empty(), "the real cargo ran after the second start: {:?}", log_lines(&world));
}

/// A toolchain-style launcher (certification of 0.1.0 behind the repository's launcher, 2026-11-04):
/// the link directory holds `cargo`, a symlink to an executable in a launchers directory that starts the shadow through the `current`
/// link of the tool's tree and has a `<name>.launch` record beside it naming that real binary, as `mpm-shadow` writes it.
/// The launcher appends a line to a file for each of its runs.
/// Returns the directory with the `cargo` link and the count file.
fn write_recorded_launcher(world: &World) -> (PathBuf, PathBuf) {
    let root: PathBuf = world.shadow_directory.parent().expect("a parent").join("recorded");
    let launchers: PathBuf = root.join("toolchain-launchers");
    let tree: PathBuf = root.join("tree");
    let links: PathBuf = root.join("cargo-shadow-bin");
    for directory in [&launchers, &tree.join("0.1.0").join("bin"), &links] {
        std::fs::create_dir_all(directory).expect("make a directory");
    }
    symlink(SHADOW_BINARY, tree.join("0.1.0").join("bin").join("cargo-shadow")).expect("the real binary of the tree");
    symlink("0.1.0", tree.join("current")).expect("the current link");
    let real: PathBuf = tree.join("current").join("bin").join("cargo-shadow");
    let count: PathBuf = root.join("count");
    // The launcher keeps the name it was started under for the program it starts (`exec -a`), as the toolchain launcher does.
    let script: String = format!(
        "#!/bin/bash\necho run >> '{count}'\nexec -a cargo '{real}' \"$@\"\n",
        count = count.display(),
        real = real.display()
    );
    let launcher: PathBuf = launchers.join("cargo-shadow");
    std::fs::write(&launcher, script).expect("write the launcher");
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).expect("make it executable");
    // Built field by field because the `json!` macro unwraps a conversion that clippy.toml disallows.
    let mut fields = serde_json::Map::new();
    fields.insert("toolId".to_string(), serde_json::Value::String("cargo:monochromatic-cargo-shadow".to_string()));
    fields.insert("lockFile".to_string(), serde_json::Value::String(root.join("toolchain.lock.json").to_string_lossy().to_string()));
    fields.insert("real".to_string(), serde_json::Value::String(real.to_string_lossy().to_string()));
    fields.insert("scope".to_string(), serde_json::Value::String("repo".to_string()));
    let record: String = serde_json::Value::Object(fields).to_string();
    std::fs::write(launchers.join("cargo-shadow.launch"), record).expect("write the record");
    symlink(&launcher, links.join("cargo")).expect("link the launcher as cargo");
    return (links, count);
}

/// The shadow started through a launcher that execs it runs once per command and the real cargo runs once for the command:
/// no PATH entry that leads back to the shadow (its own binary, a launcher whose record names it, a link to either) is taken for the
/// real cargo, so there is no second run to stop with a marker.
#[test]
fn started_through_a_launcher_the_shadow_runs_once_and_never_takes_the_launcher_for_the_real_cargo() {
    let world = make_world();
    write_lock(&world, SATISFIED);
    let (links, count) = write_recorded_launcher(&world);
    let path: String = path_of(&[&links, &world.fake_directory]);
    let mut command = Command::new(links.join("cargo"));
    command.args(["check", "-q"]).current_dir(world.repository.join(CRATE_DIR)).env("PATH", path).env("CI", "").env("FAKE_CARGO_LOG", &world.log);

    let output = output_within(&mut command, 20).expect("the command timed out");

    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim_end(), format!("{FAKE_MARKER} check -q"));
    let runs: usize = std::fs::read_to_string(&count).expect("the launcher counted its runs").lines().count();
    assert_eq!(runs, 1, "the launcher ran {runs} times: the shadow took it for the real cargo and ran itself again");
    let real_runs: usize = log_lines(&world).iter().filter(|line| { return line.starts_with("check"); }).count();
    assert_eq!(real_runs, 1, "the real cargo ran {real_runs} times for one command: {:?}", log_lines(&world));
}

/// Acceptance row K1 (certification, third pass): the warning about kept files is on the command's real standard error on every
/// completed run past the count, with the count, the combined size and the central folder, and the completed runs delete none of the
/// kept files and replace none of them (content and inode are compared, not only the names).
/// The central folder and the lockfile's directory are populated; each run is a stale lockfile that the shadow updates.
#[test]
fn the_shadow_prints_the_kept_files_warning_on_stderr_every_completed_run_and_keeps_every_kept_file_itself() {
    use std::os::unix::fs::MetadataExt;
    // The count past which the warning is printed (`KEPT_FILES_WARNING_COUNT`) and the central folder (`QUARANTINE_DIR`), read from
    // the library so the test cannot drift from them.
    let count: usize = cargo_shadow::journal::KEPT_FILES_WARNING_COUNT;
    let world = make_world();
    let central: PathBuf = world.repository.join(cargo_shadow::journal::QUARANTINE_DIR);
    std::fs::create_dir_all(&central).expect("the central folder");
    let crate_root: PathBuf = world.repository.join(CRATE_DIR);
    for index in 0..count {
        std::fs::write(central.join(format!("Cargo.lock.displaced.{index}")), format!("kept {index}")).expect("a central kept file");
    }
    std::fs::write(crate_root.join("Cargo.lock.refused"), "refused text").expect("a kept copy beside the lockfile");
    std::fs::write(crate_root.join(".Cargo.lock.shadow-abc.tmp"), "stranded").expect("a stranded temporary");
    // Every kept file: path, content, inode. A directory that cannot be listed fails the test instead of reading as empty.
    let inventory = || -> Vec<(PathBuf, Vec<u8>, u64)> {
        let mut found: Vec<(PathBuf, Vec<u8>, u64)> = Vec::new();
        for directory in [&central, &crate_root] {
            for listed in std::fs::read_dir(directory).expect("lists the directory") {
                let entry = listed.expect("reads an entry");
                let name: String = entry.file_name().to_string_lossy().into_owned();
                let kept_beside: bool = name.starts_with("Cargo.lock.displaced") || name.starts_with("Cargo.lock.refused") || (name.starts_with(".Cargo.lock.shadow-") && name.ends_with(".tmp"));
                if directory == &central || kept_beside {
                    found.push((entry.path(), std::fs::read(entry.path()).expect("reads a kept file"), entry.metadata().expect("examines").ino()));
                }
            }
        }
        found.sort();
        return found;
    };

    for run in 0..3 {
        write_lock(&world, STALE);
        let before = inventory();

        let output = run_shadow(&world, &["check", "-q"], "", 0);

        assert_eq!(output.status.code(), Some(0), "run {run}: {}", String::from_utf8_lossy(&output.stderr));
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let bytes: usize = before.iter().map(|(_, content, _)| { return content.len(); }).sum();
        let notices: Vec<&str> = stderr.lines().filter(|line| { return line.contains("They were moved aside instead of deleted"); }).collect();
        assert_eq!(notices.len(), 1, "run {run}: the warning was not printed exactly once on stderr: {stderr}");
        assert!(notices[0].contains(&format!("{} files, {bytes} bytes in all", before.len())) && notices[0].contains(&central.display().to_string()), "run {run}: {}", notices[0]);
        // No kept file was deleted, replaced or changed by the completed run (a file with the same name but another inode or content fails).
        let after = inventory();
        for kept in &before {
            assert!(after.contains(kept), "run {run}: the kept file {} was deleted, replaced or changed", kept.0.display());
        }
    }
}

/// A catalog repository that is a real git repository, with `ignored/` ignored by git, a tracked crate and, in the ignored directory, a
/// staged copy of that crate (as a gate's disposable build context is). Both lockfiles are stale, so a command that the shadow takes on
/// updates the lock to satisfied, and one it leaves alone does not.
fn make_git_world() -> World {
    let world = make_world();
    let git = |arguments: &[&str]| {
        let status = Command::new("/usr/bin/git").args(arguments).current_dir(&world.repository).env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_SYSTEM", "/dev/null").status().expect("git runs");
        assert!(status.success(), "git {arguments:?} failed");
    };
    std::fs::write(world.repository.join(".gitignore"), "ignored/\n").expect("gitignore");
    write_lock(&world, STALE);
    git(&["init", "--quiet"]);
    git(&["add", "--", ".gitignore", "rust-catalog.toml", "package"]);
    git(&["-c", "user.name=t", "-c", "user.email=t@example.com", "-c", "commit.gpgsign=false", "commit", "--quiet", "--message", "base"]);
    let staged: PathBuf = world.repository.join("ignored").join("context").join(CRATE_DIR);
    std::fs::create_dir_all(&staged).expect("the staged crate");
    for name in ["Cargo.toml", "Cargo.lock"] {
        std::fs::copy(world.repository.join(CRATE_DIR).join(name), staged.join(name)).expect("copy the crate");
    }
    return world;
}

/// A cargo command whose manifest is a staged copy under a directory that git ignores is not the repository's lockfile: the shadow does
/// not take it on (no update of the staged lock, only the user's command reaches cargo), where the same command on the tracked crate is
/// taken on. Found by `cargo vendor --manifest-path` of the git-policy gate, which counted every locked version of the staged copy as
/// newly locked (no committed lockfile at its path) and refused versions nobody added.
#[test]
fn a_staged_copy_in_an_ignored_directory_is_left_to_cargo_and_the_tracked_crate_is_not() {
    let world = make_git_world();
    let staged_manifest: PathBuf = world.repository.join("ignored/context").join(CRATE_DIR).join("Cargo.toml");
    let staged_lock: PathBuf = world.repository.join("ignored/context").join(CRATE_DIR).join("Cargo.lock");
    // The system directories are on PATH so that the shadow finds `git`, as it does outside this test.
    let path: String = path_of(&[&world.shadow_directory, &world.fake_directory, Path::new("/usr/bin")]);
    let run = |manifest: &Path| -> Output {
        return Command::new(world.shadow_directory.join("cargo"))
            .args(["vendor", "--manifest-path"])
            .arg(manifest)
            .arg("--offline")
            .current_dir(&world.repository)
            .env("PATH", &path)
            .env("CI", "")
            .env("FAKE_CARGO_LOG", &world.log)
            .output()
            .expect("start the shadow");
    };

    let staged = run(&staged_manifest);

    assert_eq!(staged.status.code(), Some(0), "{}", String::from_utf8_lossy(&staged.stderr));
    assert_eq!(std::fs::read_to_string(&staged_lock).expect("the staged lock"), STALE, "the shadow took on a staged copy in an ignored directory");
    let calls: Vec<String> = log_lines(&world);
    assert_eq!(calls.len(), 1, "cargo was asked more than the user's command for the staged copy: {calls:?}");
    assert!(calls[0].starts_with("vendor"), "{calls:?}");
    assert!(!String::from_utf8_lossy(&staged.stderr).contains("comparing lockfiles"), "the shadow compared the lockfiles of a staged copy: {}", String::from_utf8_lossy(&staged.stderr));

    // The control: the same command on the tracked crate is taken on (its stale lock is updated before the command).
    let tracked = run(&world.repository.join(CRATE_DIR).join("Cargo.toml"));
    assert_eq!(tracked.status.code(), Some(0), "{}", String::from_utf8_lossy(&tracked.stderr));
    assert_eq!(read_lock(&world), SATISFIED, "the tracked crate was not taken on, so the staged copy's passing proves nothing");
}
