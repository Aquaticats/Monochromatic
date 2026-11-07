//! What:
//! End-to-end tests of the rule "a shadow never starts a shadow,
//! of any version,
//! as its real tool".
//! Each test builds a disposable `PATH` of shadow copies,
//! launcher records and real programs,
//! starts the built `search-shadow` under the name `grep` or `rg`,
//! and gives it a deadline.
//! Why:
//! Two copies of the shadow at different paths used to pick each other as the real `grep`
//! and replace themselves with each other without end (measured 2026-11-07:
//! about 105 starts per second under `strace`,
//! at a quarter of one CPU).
//! A test of that loop must itself never wait without end,
//! so every run here is polled against a deadline and killed when it passes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runWithDeadline(spawn(join(a, 'grep'), ['x', file], { env: { PATH } }), LOOP_BOUND);
//! assert(outcome.kind === 'finished');
//! ```

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The built shadow.
const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_search-shadow");

/// The name of the environment variable the arrival marker travels in (`src/shadow_identity.rs`).
const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The version the built shadow carries,
/// which the copies below change.
const OWN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long a run may take before it counts as a loop.
/// A run that ends takes a few milliseconds (a start,
/// a path lookup and one more start);
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
            // A replaced process keeps its id, so killing this one ends the whole chain.
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
fn link_as(folder: &Path, name: &str) -> PathBuf {
    let link: PathBuf = folder.join(name);
    std::os::unix::fs::symlink(folder.join("search-shadow"), &link).expect("make the shadow link");
    return link;
}

/// A script standing in for a real tool:
/// prints which tool answered.
fn write_real_script(folder: &Path, name: &str) {
    fs::create_dir_all(folder).expect("make the folder");
    fs::write(folder.join(name), format!("#!/bin/sh\nprintf 'REAL {name} %s\\n' \"$*\"\n")).expect("write the script");
    fs::set_permissions(folder.join(name), fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// A scratch folder beside the build output.
fn scratch() -> tempfile::TempDir {
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
}

/// The command that starts a shadow link with a bare environment and the given `PATH`.
fn command_for(link: &Path, path_folders: &[&Path], arguments: &[&str]) -> Command {
    let path_variable: String = path_folders.iter().map(|folder| { return folder.display().to_string(); }).collect::<Vec<String>>().join(":");
    let mut command: Command = Command::new(link);
    command.args(arguments);
    command.env_clear();
    command.env("PATH", path_variable);
    return command;
}

#[test]
fn two_copies_of_different_versions_that_each_come_first_for_the_other_end_at_once_and_the_real_tool_runs() {
    let root = scratch();
    let (first, second, real) = (root.path().join("first"), root.path().join("second"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    link_as(&second, "grep");
    write_real_script(&real, "grep");

    let output = finished(run_bounded(command_for(&link, &[&first, &second, &real], &["needle", "haystack"])));

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "REAL grep needle haystack\n");
}

#[test]
fn two_copies_with_no_real_tool_behind_them_end_at_once_with_the_not_found_message() {
    let root = scratch();
    let (first, second) = (root.path().join("first"), root.path().join("second"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    link_as(&second, "grep");

    let output = finished(run_bounded(command_for(&link, &[&first, &second], &["needle", "haystack"])));

    assert_eq!(output.status.code(), Some(127), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no real `grep`"), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn a_launcher_record_that_names_another_shadow_version_is_skipped() {
    let root = scratch();
    let (first, second, launchers, real) = (root.path().join("first"), root.path().join("second"), root.path().join("launchers"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    // The launcher of the toolchain is a program of its own with a record beside it that names the program it starts.
    // Its own bytes are not the shadow's; only its record leads to the other shadow.
    copy_program(Path::new("/usr/bin/true"), &launchers.join("grep"));
    fs::write(
        launchers.join("grep.launch"),
        format!(r#"{{"toolId":"cargo:monochromatic-search-shadow","lockFile":"/dev/null","real":"{}","scope":"repo"}}"#, second.join("search-shadow").display()),
    )
    .expect("write the record");
    write_real_script(&real, "grep");

    let output = finished(run_bounded(command_for(&link, &[&first, &launchers, &real], &["needle", "haystack"])));

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "REAL grep needle haystack\n", "the launcher that leads to a shadow was started");
}

#[test]
fn a_launcher_record_that_names_a_real_tool_is_not_skipped() {
    let root = scratch();
    let (first, launchers) = (root.path().join("first"), root.path().join("launchers"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    copy_program(Path::new("/usr/bin/grep"), &launchers.join("grep"));
    fs::write(
        launchers.join("grep.launch"),
        r#"{"toolId":"grep","lockFile":"/dev/null","real":"/usr/bin/grep","scope":"repo"}"#,
    )
    .expect("write the record");
    let file = root.path().join("haystack.txt");
    fs::write(&file, "one needle here\nand none here\n").expect("write the haystack");

    let output = finished(run_bounded(command_for(&link, &[&first, &launchers], &["--count", "needle", &file.display().to_string()])));

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}

#[test]
fn a_compiled_real_grep_and_a_compiled_real_rg_still_run_behind_the_shadow() {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    let grep_link: PathBuf = link_as(&first, "grep");
    let rg_link: PathBuf = link_as(&first, "rg");
    // Genuine compiled programs without the shadow's identity: the system's grep, and `true` standing in for ripgrep.
    copy_program(Path::new("/usr/bin/grep"), &real.join("grep"));
    copy_program(Path::new("/usr/bin/true"), &real.join("rg"));
    let file = root.path().join("haystack.txt");
    fs::write(&file, "one needle here\nand none here\n").expect("write the haystack");

    let grep_output = finished(run_bounded(command_for(&grep_link, &[&first, &real], &["--count", "needle", &file.display().to_string()])));
    let rg_output = finished(run_bounded(command_for(&rg_link, &[&first, &real], &["needle", &file.display().to_string()])));

    assert!(grep_output.status.success(), "stderr: {}", String::from_utf8_lossy(&grep_output.stderr));
    assert_eq!(String::from_utf8_lossy(&grep_output.stdout), "1\n");
    assert!(rg_output.status.success(), "stderr: {}", String::from_utf8_lossy(&rg_output.stderr));
}

#[test]
fn a_script_that_starts_the_other_shadow_stops_on_arrival_with_one_line_and_a_non_zero_status() {
    let root = scratch();
    let (first, second, scripts) = (root.path().join("first"), root.path().join("second"), root.path().join("scripts"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    link_as(&second, "grep");
    // No test of the script's bytes can tell that it leads to a shadow: only the start itself shows it.
    fs::create_dir_all(&scripts).expect("make the folder");
    fs::write(scripts.join("grep"), format!("#!/bin/sh\nexec '{}' \"$@\"\n", second.join("grep").display())).expect("write the script");
    fs::set_permissions(scripts.join("grep"), fs::Permissions::from_mode(0o755)).expect("make it executable");

    // The script comes after the first shadow on PATH and the second shadow's link is out of PATH: the script is the real tool the first one finds.
    let output = finished(run_bounded(command_for(&link, &[&first, &scripts], &["needle", "haystack"])));

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert_eq!(stderr.trim_end().lines().count(), 1, "the stop is one line: {stderr}");
    assert!(stderr.contains("started by"), "stderr: {stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}

/// The start time of a process,
/// as `/proc/<pid>/stat` field 22 says it (the test reads it the way the arrival marker does).
fn birth_of(pid: u32) -> String {
    let text: String = fs::read_to_string(format!("/proc/{pid}/stat")).expect("read the stat file");
    let after_name: &str = text.rsplit_once(')').expect("the name ends with a bracket").1;
    return after_name.split_whitespace().nth(19).expect("field 22").to_string();
}

#[test]
fn an_arrival_marker_left_for_the_direct_parent_stops_the_shadow_and_one_left_for_anyone_else_does_not() {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    write_real_script(&real, "grep");
    let own_pid: u32 = std::process::id();

    // A start by this process: the shadow's parent is the process that wrote the marker, so it was started by a shadow.
    let mut for_parent: Command = command_for(&link, &[&first, &real], &["needle", "haystack"]);
    for_parent.env(ARRIVAL_VARIABLE, format!("spawn {own_pid} {} monochromatic-search-shadow 0.0.0", birth_of(own_pid)));
    let stopped = finished(run_bounded(for_parent));
    assert_eq!(stopped.status.code(), Some(2), "stderr: {}", String::from_utf8_lossy(&stopped.stderr));

    // A marker that names another process, as one a real tool's own script inherits: ignored, the real tool runs.
    let mut for_someone_else: Command = command_for(&link, &[&first, &real], &["needle", "haystack"]);
    for_someone_else.env(ARRIVAL_VARIABLE, format!("spawn {} {} monochromatic-search-shadow 0.0.0", own_pid + 1, birth_of(own_pid)));
    let passed = finished(run_bounded(for_someone_else));
    assert!(passed.status.success(), "stderr: {}", String::from_utf8_lossy(&passed.stderr));
    assert_eq!(String::from_utf8_lossy(&passed.stdout), "REAL grep needle haystack\n");

    // The right process number with another birth identity is a reused number: ignored.
    let mut reused: Command = command_for(&link, &[&first, &real], &["needle", "haystack"]);
    reused.env(ARRIVAL_VARIABLE, format!("spawn {own_pid} 1 monochromatic-search-shadow 0.0.0"));
    let passed_again = finished(run_bounded(reused));
    assert!(passed_again.status.success(), "stderr: {}", String::from_utf8_lossy(&passed_again.stderr));
}

#[test]
fn link_does_not_take_the_link_of_another_shadow_version_for_a_real_tool() {
    let root = scratch();
    let (second, links) = (root.path().join("second"), root.path().join("links"));
    copy_of_another_version(&second.join("search-shadow"));
    link_as(&second, "grep");

    // The only `grep` on PATH is the link of another shadow: there is no real tool, so no link is made.
    let output = Command::new(SHADOW_EXECUTABLE).arg("link").arg(&links).env_clear().env("PATH", &second).output().expect("run link");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(links.join("grep").symlink_metadata().is_err(), "a link was made for a tool whose only candidate is a shadow");
}
