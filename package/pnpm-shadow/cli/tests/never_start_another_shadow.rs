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
/// Includes the bounded-run deadline from its own file in this test's folder.
#[path = "never_start_another_shadow/bounded_run.rs"]
mod bounded_run;
/// Includes the `PATH` fixture builders from their own file in this test's folder.
#[path = "never_start_another_shadow/fixtures.rs"]
mod fixtures;

/// Imports the deadline helpers.
use crate::bounded_run::{
    finished,
    run_bounded,
};
/// Imports the `PATH` fixture builders and the names they use.
use crate::fixtures::{
    ARRIVAL_VARIABLE,
    OWN_VERSION,
    SHADOW_EXECUTABLE,
    SHADOW_FILE,
    TOOL,
    birth_of,
    command_for,
    copy_of_another_version,
    copy_program,
    expected_first_line,
    link_as_tool,
    scratch,
    write_real_script,
};
/// Imports the file helpers the tests use to build records and scripts.
use std::fs;
/// Imports the executable-mode helper.
use std::os::unix::fs::PermissionsExt;
/// Imports the path types.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the child-process builder.
use std::process::Command;

#[test]
fn two_copies_of_different_versions_that_each_come_first_for_the_other_end_at_once_and_the_real_tool_is_started_by_the_first_only()
 {
    let root = scratch();
    let (first, second, real) = (
        root.path().join("first"),
        root.path().join("second"),
        root.path().join("real"),
    );
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    link_as_tool(&second);
    write_real_script(&real);

    let output = finished(run_bounded(command_for(&link, &[&first, &second, &real])));

    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout.lines().next(),
        Some(expected_first_line().as_str()),
        "stdout: {stdout}"
    );
    // The marker names the shadow that started the real tool: the first, never the second copy (which must not have been started).
    let marker: &str = stdout.lines().nth(1).unwrap_or_default();
    assert!(
        marker.ends_with(&format!(" {OWN_VERSION}")),
        "the real tool was not started by the first shadow alone: {marker}"
    );
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

    assert_eq!(
        output.status.code(),
        Some(127),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("no real"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_launcher_record_that_names_another_shadow_version_is_skipped() {
    let root = scratch();
    let (first, second, launchers, real) = (
        root.path().join("first"),
        root.path().join("second"),
        root.path().join("launchers"),
        root.path().join("real"),
    );
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    // The launcher is a program of its own with a record beside it that names the program it starts; only its record leads to the other shadow.
    copy_program(Path::new("/usr/bin/true"), &launchers.join(TOOL));
    fs::write(
        launchers.join(format!("{TOOL}.launch")),
        format!(
            r#"{{"toolId":"t","lockFile":"/dev/null","real":"{}","scope":"repo"}}"#,
            second.join(SHADOW_FILE).display()
        ),
    )
    .expect("write the record");
    write_real_script(&real);

    let output = finished(run_bounded(command_for(
        &link,
        &[&first, &launchers, &real],
    )));

    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout.lines().next(),
        Some(expected_first_line().as_str()),
        "the launcher that leads to a shadow was started: {stdout}"
    );
}

#[test]
fn a_launcher_record_that_names_a_real_tool_is_not_skipped() {
    let root = scratch();
    let (first, launchers) = (root.path().join("first"), root.path().join("launchers"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    copy_program(Path::new("/usr/bin/true"), &launchers.join(TOOL));
    fs::write(
        launchers.join(format!("{TOOL}.launch")),
        r#"{"toolId":"t","lockFile":"/dev/null","real":"/usr/bin/true","scope":"repo"}"#,
    )
    .expect("write the record");

    let output = finished(run_bounded(command_for(&link, &[&first, &launchers])));

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_script_that_starts_the_other_shadow_stops_on_arrival_with_a_message_and_a_non_zero_status() {
    let root = scratch();
    let (first, second, scripts) = (
        root.path().join("first"),
        root.path().join("second"),
        root.path().join("scripts"),
    );
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    copy_of_another_version(&second.join(SHADOW_FILE));
    link_as_tool(&second);
    let link: PathBuf = link_as_tool(&first);
    // No test of the script's bytes can tell that it leads to a shadow: only the start itself shows it.
    fs::create_dir_all(&scripts).expect("make the folder");
    fs::write(
        scripts.join(TOOL),
        format!("#!/bin/sh\nexec '{}' \"$@\"\n", second.join(TOOL).display()),
    )
    .expect("write the script");
    fs::set_permissions(scripts.join(TOOL), fs::Permissions::from_mode(0o755))
        .expect("make it executable");

    // The script comes after the first shadow on PATH, and the second shadow's link is not on PATH: the script is the real tool the first one finds.
    let output = finished(run_bounded(command_for(&link, &[&first, &scripts])));

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert!(stderr.contains("started by"), "stderr: {stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}

/// The start time of a process,
#[test]
fn an_arrival_marker_left_for_the_direct_parent_stops_the_shadow_and_one_left_for_anyone_else_does_not()
 {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join(SHADOW_FILE));
    let link: PathBuf = link_as_tool(&first);
    write_real_script(&real);
    let own_pid: u32 = std::process::id();

    // A start by this process: the shadow's parent is the process that wrote the marker, so it was started by a shadow.
    let mut for_parent: Command = command_for(&link, &[&first, &real]);
    for_parent.env(
        ARRIVAL_VARIABLE,
        format!(
            "spawn {own_pid} {} monochromatic-other-shadow 0.0.0",
            birth_of(own_pid)
        ),
    );
    let stopped = finished(run_bounded(for_parent));
    assert_eq!(
        stopped.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&stopped.stderr)
    );

    // A marker that names another process, as one a real tool's own script inherits: ignored, the real tool runs.
    let mut for_someone_else: Command = command_for(&link, &[&first, &real]);
    for_someone_else.env(
        ARRIVAL_VARIABLE,
        format!(
            "spawn {} {} monochromatic-other-shadow 0.0.0",
            own_pid + 1,
            birth_of(own_pid)
        ),
    );
    let passed = finished(run_bounded(for_someone_else));
    assert!(
        passed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&passed.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&passed.stdout).lines().next(),
        Some(expected_first_line().as_str())
    );

    // The right process number with another birth identity is a reused number: ignored.
    let mut reused: Command = command_for(&link, &[&first, &real]);
    reused.env(
        ARRIVAL_VARIABLE,
        format!("spawn {own_pid} 1 monochromatic-other-shadow 0.0.0"),
    );
    let passed_again = finished(run_bounded(reused));
    assert!(
        passed_again.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&passed_again.stderr)
    );
}
