//! What:
//! End-to-end tests of the rule "a shadow never starts a shadow,
//! of any version,
//! as its real `pnpm`".
//! Each test builds a disposable `PATH` of shadow copies,
//! launcher records and real programs,
//! starts the built `pnpm-shadow` under the name `pnpm`,
//! and gives it a deadline.
//! Why:
//! Two shadows at different paths used to choose each other as the real tool (measured 2026-11-07 for `search-shadow`:
//! about 105 starts per second under `strace`).
//! A test of such a loop must itself never wait without end,
//! so every run here is polled against a deadline and killed when it passes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runWithDeadline(spawn(join(first, 'pnpm'), [...]), LOOP_BOUND);
//! assert(outcome.kind === 'finished');
//! ```

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The built shadow.
const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_pnpm-shadow");

/// The file name of the shadow program inside each folder of copies.
const SHADOW_FILE: &str = "pnpm-shadow";

/// The tool the shadow shadows.
const TOOL: &str = "pnpm";

/// The arguments of a run that the shadow passes straight to the real tool.
const PASS_THROUGH_ARGUMENTS: &[&str] = &["run", "task"];

/// The name of the environment variable the arrival marker travels in (`src/shadow_identity.rs`).
const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The version the built shadow carries,
/// which the copies below change.
const OWN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long a run may take before it counts as a loop.
/// A run that ends takes a few milliseconds;
/// the loop it replaces never ends.
/// The bound is far above the first and below the patience of a test run,
/// and it only matters in the failing case.
const LOOP_BOUND: Duration = Duration::from_secs(5);

/// How often a run is polled while it has not ended.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What a bounded run did.
enum Outcome {
    /// It ended by itself.
    Finished(Output),
    /// It was still running at the bound and was killed.
    Looped,
}

/// Runs a prepared command with the deadline `LOOP_BOUND`.
fn run_bounded(mut command: Command) -> Outcome {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child: Child = command.spawn().expect("start the shadow");
    let started: Instant = Instant::now();
    loop {
        if child.try_wait().expect("poll the shadow").is_some() {
            return Outcome::Finished(child.wait_with_output().expect("collect the output"));
        }
        if started.elapsed() >= LOOP_BOUND {
            child.kill().expect("kill the looping shadow");
            child.wait().expect("reap the looping shadow");
            return Outcome::Looped;
        }
        sleep(POLL_INTERVAL);
    }
}

/// The output of a run that must have ended,
/// or a failure that says it looped.
fn finished(outcome: Outcome) -> Output {
    match outcome {
        Outcome::Finished(output) => return output,
        Outcome::Looped => panic!("the run was still going after {LOOP_BOUND:?}: the shadows are starting each other"),
    }
}

/// Copies a program and keeps it executable.
fn copy_program(source: &Path, target: &Path) {
    fs::create_dir_all(target.parent().expect("a parent folder")).expect("make the folder");
    fs::copy(source, target).expect("copy the program");
    fs::set_permissions(target, fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// A copy of the shadow that names another version:
/// the version text inside it changes in place,
/// so the copy is a different file with a different identity text,
/// as an older release would be.
fn copy_of_another_version(target: &Path) {
    let mut bytes: Vec<u8> = fs::read(SHADOW_EXECUTABLE).expect("read the shadow");
    let needle: &[u8] = OWN_VERSION.as_bytes();
    let mut other: Vec<u8> = needle.to_vec();
    let last: usize = other.len() - 1;
    other[last] = if other[last] == b'9' { b'8' } else { b'9' };
    let mut index: usize = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            bytes[index..index + needle.len()].copy_from_slice(&other);
            index += needle.len();
        } else {
            index += 1;
        }
    }
    fs::create_dir_all(target.parent().expect("a parent folder")).expect("make the folder");
    fs::write(target, bytes).expect("write the copy");
    fs::set_permissions(target, fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// A shadow link:
/// a symlink named after the tool,
/// to the shadow program of its folder.
fn link_as_tool(folder: &Path) -> PathBuf {
    let link: PathBuf = folder.join(TOOL);
    std::os::unix::fs::symlink(folder.join(SHADOW_FILE), &link).expect("make the shadow link");
    return link;
}

/// A script standing in for the real tool:
/// prints which tool answered and the arrival marker it was started with.
fn write_real_script(folder: &Path) {
    fs::create_dir_all(folder).expect("make the folder");
    fs::write(folder.join(TOOL), format!("#!/bin/sh\nprintf 'REAL {TOOL} %s\\n' \"$*\"\nprintf 'MARKER %s\\n' \"${ARRIVAL_VARIABLE}\"\n")).expect("write the script");
    fs::set_permissions(folder.join(TOOL), fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// A scratch folder beside the build output.
fn scratch() -> tempfile::TempDir {
    std::fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).expect("make the scratch root");
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
}

/// The command that starts a shadow link with a bare environment and the given `PATH`.
fn command_for(link: &Path, path_folders: &[&Path]) -> Command {
    let path_variable: String = path_folders.iter().map(|folder| { return folder.display().to_string(); }).collect::<Vec<String>>().join(":");
    let mut command: Command = Command::new(link);
    command.args(PASS_THROUGH_ARGUMENTS);
    command.current_dir(env!("CARGO_TARGET_TMPDIR"));
    command.env_clear();
    command.env("PATH", path_variable);
    return command;
}

/// The two lines the real script prints:
/// what ran and the marker it saw.
fn expected_first_line() -> String {
    return format!("REAL {TOOL} {}", PASS_THROUGH_ARGUMENTS.join(" "));
}

#[test]
fn two_copies_of_different_versions_that_each_come_first_for_the_other_end_at_once_and_the_real_tool_is_started_by_the_first_only() {
    let root = scratch();
    let (first, second, real) = (root.path().join("first"), root.path().join("second"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    link_as_tool(&second);
    write_real_script(&real);

    let output = finished(run_bounded(command_for(&link, &[&first, &second, &real])));

    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(stdout.lines().next(), Some(expected_first_line().as_str()), "stdout: {stdout}");
    // The marker names the shadow that started the real tool: the first, never the second copy (which must not have been started).
    let marker: &str = stdout.lines().nth(1).unwrap_or_default();
    assert!(marker.ends_with(&format!(" {OWN_VERSION}")), "the real tool was not started by the first shadow alone: {marker}");
}

#[test]
fn two_copies_with_no_real_tool_behind_them_end_at_once_with_a_not_found_status() {
    let root = scratch();
    let (first, second) = (root.path().join("first"), root.path().join("second"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    link_as_tool(&second);

    let output = finished(run_bounded(command_for(&link, &[&first, &second])));

    assert_eq!(output.status.code(), Some(127), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no real"), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn a_launcher_record_that_names_another_shadow_version_is_skipped() {
    let root = scratch();
    let (first, second, launchers, real) = (root.path().join("first"), root.path().join("second"), root.path().join("launchers"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    // The launcher is a program of its own with a record beside it that names the program it starts; only its record leads to the other shadow.
    copy_program(Path::new("/usr/bin/true"), &launchers.join(TOOL));
    fs::write(
        launchers.join(format!("{TOOL}.launch")),
        format!(r#"{{"toolId":"t","lockFile":"/dev/null","real":"{}","scope":"repo"}}"#, second.join(SHADOW_FILE).display()),
    )
    .expect("write the record");
    write_real_script(&real);

    let output = finished(run_bounded(command_for(&link, &[&first, &launchers, &real])));

    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(stdout.lines().next(), Some(expected_first_line().as_str()), "the launcher that leads to a shadow was started: {stdout}");
}

#[test]
fn a_launcher_record_that_names_a_real_tool_is_not_skipped() {
    let root = scratch();
    let (first, launchers) = (root.path().join("first"), root.path().join("launchers"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    copy_program(Path::new("/usr/bin/true"), &launchers.join(TOOL));
    fs::write(launchers.join(format!("{TOOL}.launch")), r#"{"toolId":"t","lockFile":"/dev/null","real":"/usr/bin/true","scope":"repo"}"#).expect("write the record");

    let output = finished(run_bounded(command_for(&link, &[&first, &launchers])));

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn a_compiled_real_tool_still_runs_behind_the_shadow() {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    // A genuine compiled program without the shadow's identity, standing in for the real tool.
    copy_program(Path::new("/usr/bin/true"), &real.join(TOOL));

    let output = finished(run_bounded(command_for(&link, &[&first, &real])));

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn a_script_that_starts_the_other_shadow_stops_on_arrival_with_a_message_and_a_non_zero_status() {
    let root = scratch();
    let (first, second, scripts) = (root.path().join("first"), root.path().join("second"), root.path().join("scripts"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    link_as_tool(&second);
    let link: PathBuf = link_as_tool(&first);
    // No test of the script's bytes can tell that it leads to a shadow: only the start itself shows it.
    fs::create_dir_all(&scripts).expect("make the folder");
    fs::write(scripts.join(TOOL), format!("#!/bin/sh\nexec '{}' \"$@\"\n", second.join(TOOL).display())).expect("write the script");
    fs::set_permissions(scripts.join(TOOL), fs::Permissions::from_mode(0o755)).expect("make it executable");

    // The script comes after the first shadow on PATH, and the second shadow's link is not on PATH: the script is the real tool the first one finds.
    let output = finished(run_bounded(command_for(&link, &[&first, &scripts])));

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert!(stderr.contains("started by"), "stderr: {stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}

/// The start time of a process,
/// as `/proc/<pid>/stat` field 22 says it.
fn birth_of(pid: u32) -> String {
    let text: String = fs::read_to_string(format!("/proc/{pid}/stat")).expect("read the stat file");
    let after_name: &str = text.rsplit_once(')').expect("the name ends with a bracket").1;
    return after_name.split_whitespace().nth(19).expect("field 22").to_string();
}

#[test]
fn an_arrival_marker_left_for_the_direct_parent_stops_the_shadow_and_one_left_for_anyone_else_does_not() {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    write_real_script(&real);
    let own_pid: u32 = std::process::id();

    // A start by this process: the shadow's parent is the process that wrote the marker, so it was started by a shadow.
    let mut for_parent: Command = command_for(&link, &[&first, &real]);
    for_parent.env(ARRIVAL_VARIABLE, format!("spawn {own_pid} {} monochromatic-other-shadow 0.0.0", birth_of(own_pid)));
    let stopped = finished(run_bounded(for_parent));
    assert_eq!(stopped.status.code(), Some(2), "stderr: {}", String::from_utf8_lossy(&stopped.stderr));

    // A marker that names another process, as one a real tool's own script inherits: ignored, the real tool runs.
    let mut for_someone_else: Command = command_for(&link, &[&first, &real]);
    for_someone_else.env(ARRIVAL_VARIABLE, format!("spawn {} {} monochromatic-other-shadow 0.0.0", own_pid + 1, birth_of(own_pid)));
    let passed = finished(run_bounded(for_someone_else));
    assert!(passed.status.success(), "stderr: {}", String::from_utf8_lossy(&passed.stderr));
    assert_eq!(String::from_utf8_lossy(&passed.stdout).lines().next(), Some(expected_first_line().as_str()));

    // The right process number with another birth identity is a reused number: ignored.
    let mut reused: Command = command_for(&link, &[&first, &real]);
    reused.env(ARRIVAL_VARIABLE, format!("spawn {own_pid} 1 monochromatic-other-shadow 0.0.0"));
    let passed_again = finished(run_bounded(reused));
    assert!(passed_again.status.success(), "stderr: {}", String::from_utf8_lossy(&passed_again.stderr));
}
