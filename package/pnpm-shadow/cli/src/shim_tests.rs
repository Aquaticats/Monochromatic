//! The unit tests of the child launch,
//! the exit-status mapping and the warning that precedes pnpm.

/// Imports the functions under test.
use crate::shim::{exit_code_of, run_program, warn_about_workspace};
/// Imports the constants the expectations derive from.
use crate::constants::{SIGNAL_EXIT_CODE_OFFSET, WORKSPACE_MARKER};
/// Imports the scratch directories.
use crate::test_scratch::scratch;

/// Imports the owned operating-system text type.
use std::ffi::OsString;
/// Imports the path type.
use std::path::Path;
/// Imports the child-process builder.
use std::process::Command;

/// The number of the termination signal on Linux and macOS,
/// written once and named.
const SIGTERM_NUMBER: i32 = 15;

/// An exit code no default maps to,
/// so a pass-through is visible.
const DISTINCT_EXIT_CODE: i32 = 7;

/// Runs `sh -c <script>` and returns its raw status.
fn shell_status(script: &str) -> std::process::ExitStatus {
    return Command::new("sh").arg("-c").arg(script).status().expect("start sh");
}

#[test]
fn a_normal_exit_passes_its_code_through() {
    let status = shell_status(&format!("exit {DISTINCT_EXIT_CODE}"));

    assert_eq!(exit_code_of(status), DISTINCT_EXIT_CODE);
}

#[test]
fn death_by_signal_becomes_the_shell_convention() {
    let status = shell_status("kill -TERM $$");

    assert_eq!(exit_code_of(status), SIGNAL_EXIT_CODE_OFFSET + SIGTERM_NUMBER);
}

#[test]
fn run_program_returns_the_childs_code_with_the_arguments_untouched() {
    let arguments: Vec<OsString> = vec![OsString::from("-c"), OsString::from(format!("exit {DISTINCT_EXIT_CODE}"))];

    let code = run_program(Path::new("/bin/sh"), &arguments, None).expect("sh runs");

    assert_eq!(code, DISTINCT_EXIT_CODE);
}

#[test]
fn run_program_reports_a_program_that_cannot_start() {
    let missing = scratch().path().join("no-such-program");

    assert!(run_program(&missing, &[], None).is_err());
}

#[test]
fn the_warning_names_a_violating_manifest_of_the_workspace_that_holds_the_directory() {
    let workspace = scratch();
    std::fs::write(workspace.path().join(WORKSPACE_MARKER), "packages: []\n").expect("write the marker");
    std::fs::create_dir_all(workspace.path().join("a")).expect("make a directory");
    std::fs::write(
        workspace.path().join("a/package.json"),
        r#"{ "dependencies": { "left-pad": "1.3.0" } }"#,
    )
    .expect("write the manifest");
    let mut sink: Vec<u8> = Vec::new();

    warn_about_workspace(&workspace.path().join("a"), &mut sink);

    let text: String = String::from_utf8(sink).expect("text");
    assert!(text.contains("a/package.json"));
    assert!(text.contains("left-pad"));
}

#[test]
fn a_clean_workspace_and_a_directory_outside_any_workspace_write_nothing() {
    let workspace = scratch();
    std::fs::write(workspace.path().join(WORKSPACE_MARKER), "packages: []\n").expect("write the marker");
    std::fs::write(workspace.path().join("package.json"), r#"{ "dependencies": { "x": "catalog:" } }"#)
        .expect("write the manifest");
    let mut clean_sink: Vec<u8> = Vec::new();
    let mut outside_sink: Vec<u8> = Vec::new();

    warn_about_workspace(workspace.path(), &mut clean_sink);
    warn_about_workspace(Path::new("/"), &mut outside_sink);

    assert!(clean_sink.is_empty());
    assert!(outside_sink.is_empty());
}
