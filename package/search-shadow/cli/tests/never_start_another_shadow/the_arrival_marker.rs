//! What:
//! The arrival stop:
//! the marker a shadow leaves in the environment of the program it starts,
//! the line printed when a shadow finds it was left for this very process or its direct parent,
//! and the cases that pass.
//! Why:
//! No check of a file sees what a script or a launcher with no record does,
//! so a start that a shadow makes is shown by the start itself.
//! A marker left for anyone else (a script the real tool runs,
//! a process number the kernel reused) is ignored,
//! and a shadow that was started this way prints one line and does nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const marker = 'spawn ' + process.pid + ' ' + birthOf(process.pid) + ' monochromatic-search-shadow 0.0.0';
//! assert(runBounded(grepOf(first, [first, real], { env: { MONOCHROMATIC_SHADOW_STARTED_BY: marker } })).status === 2);
//! ```

/// Imports the file-system module that writes the stand-in script and reads `/proc`.
use std::fs;

/// Imports the borrowed path type used to name the copied programs.
use std::path::Path;

/// Imports the owned path type one test keeps the shadow link in.
use std::path::PathBuf;

/// Imports the command type a test adds the marker to.
use std::process::Command;

/// Imports the fixtures,
/// the marker's variable name and the deadline this part of the rule runs on.
use super::{
    ARRIVAL_VARIABLE,
    SHADOW_EXECUTABLE,
    command_for,
    copy_of_another_version,
    copy_program,
    finished,
    link_as,
    run_bounded,
    scratch,
    write_real_script,
};

#[test]
fn a_script_that_starts_the_other_shadow_stops_on_arrival_with_one_line_and_a_non_zero_status() {
    let root = scratch();
    let (first, second, scripts) = (
        root.path().join("first"),
        root.path().join("second"),
        root.path().join("scripts"),
    );
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    link_as(&second, "grep");
    // No test of the script's bytes can tell that it leads to a shadow: only the start itself shows it.
    // A test starts this script as the real tool, so a child process writes it (issue 6527).
    crate::through_a_child_process::write_program(
        scripts.join("grep").as_path(),
        format!(
            "#!/bin/sh\nexec '{}' \"$@\"\n",
            second.join("grep").display()
        )
        .as_bytes(),
    );

    // The script comes after the first shadow on PATH and the second shadow's link is out of PATH: the script is the real tool the first one finds.
    let output = finished(run_bounded(command_for(
        &link,
        &[&first, &scripts],
        &["needle", "haystack"],
    )));

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    assert_eq!(
        stderr.trim_end().lines().count(),
        1,
        "the stop is one line: {stderr}"
    );
    assert!(stderr.contains("started by"), "stderr: {stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}

/// The start time of a process,
/// as `/proc/<pid>/stat` field 22 says it (the test reads it the way the arrival marker does).
fn birth_of(pid: u32) -> String {
    let text: String = fs::read_to_string(format!("/proc/{pid}/stat")).expect("read the stat file");
    let after_name: &str = text
        .rsplit_once(')')
        .expect("the name ends with a bracket")
        .1;
    return after_name
        .split_whitespace()
        .nth(19)
        .expect("field 22")
        .to_string();
}

#[test]
fn an_arrival_marker_left_for_the_direct_parent_stops_the_shadow_and_one_left_for_anyone_else_does_not()
 {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    write_real_script(&real, "grep");
    let own_pid: u32 = std::process::id();

    // A start by this process: the shadow's parent is the process that wrote the marker, so it was started by a shadow.
    let mut for_parent: Command = command_for(&link, &[&first, &real], &["needle", "haystack"]);
    for_parent.env(
        ARRIVAL_VARIABLE,
        format!(
            "spawn {own_pid} {} monochromatic-search-shadow 0.0.0",
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
    let mut for_someone_else: Command =
        command_for(&link, &[&first, &real], &["needle", "haystack"]);
    for_someone_else.env(
        ARRIVAL_VARIABLE,
        format!(
            "spawn {} {} monochromatic-search-shadow 0.0.0",
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
        String::from_utf8_lossy(&passed.stdout),
        "REAL grep needle haystack\n"
    );

    // The right process number with another birth identity is a reused number: ignored.
    let mut reused: Command = command_for(&link, &[&first, &real], &["needle", "haystack"]);
    reused.env(
        ARRIVAL_VARIABLE,
        format!("spawn {own_pid} 1 monochromatic-search-shadow 0.0.0"),
    );
    let passed_again = finished(run_bounded(reused));
    assert!(
        passed_again.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&passed_again.stderr)
    );
}
