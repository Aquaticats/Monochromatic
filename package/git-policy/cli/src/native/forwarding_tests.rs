//! What:
//!  Process controls for starting real Git and reporting how it ended.
//! Why:
//!  Arguments must arrive byte for byte,
//!  and exit codes and terminating signals
//!      must be reported the way Git itself reports its children.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(exitCode({ signaled: 15 })).toBe(143);
//! ```
#![cfg(unix)]

/// Import the forwarding functions under test and shared fixtures.
use super::{ChildOutcome, exit_code, git_command, replace_process_with_real_git, run_real_git};
use crate::test_support::{executable, fixture, remove};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Exit codes pass through;
///  a terminating signal becomes 128 plus its number,
///  as in Git's run-command.
#[test]
fn exit_codes_follow_git_child_convention() {
    for (outcome, expected) in [
        (ChildOutcome::Exited(0), 0),
        (ChildOutcome::Exited(1), 1),
        (ChildOutcome::Exited(128), 128),
        (ChildOutcome::Exited(255), 255),
        (ChildOutcome::Signaled(2), 130),
        (ChildOutcome::Signaled(9), 137),
        (ChildOutcome::Signaled(13), 141),
        (ChildOutcome::Signaled(15), 143),
    ] {
        assert_eq!(exit_code(outcome), expected, "{outcome:?}");
    }
}

/// The command carries the exact program,
///  each argument as its own unchanged element,
///  and only the overlay.
#[test]
fn command_preserves_program_arguments_and_overlay() {
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        OsString::from_vec(b"dir-\xff\xfe".to_vec()),
        OsString::new(),
        OsString::from("a b"),
        OsString::from("--"),
        OsString::from("$(touch x); 'q' \"d\" \\ \n*"),
        OsString::from("--"),
    ];
    let overlay: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_CONFIG_COUNT"), OsString::from("1")),
        (
            OsString::from("MARKER"),
            OsString::from_vec(b"/opt/\xff/git".to_vec()),
        ),
    ];
    let command: Command = git_command(
        Path::new("/opt/real git/git"),
        arguments.as_slice(),
        overlay.as_slice(),
    );
    assert_eq!(command.get_program(), OsStr::new("/opt/real git/git"));
    let mut seen_arguments: Vec<OsString> = Vec::<OsString>::new();
    for argument in command.get_args() {
        seen_arguments.push(argument.to_os_string());
    }
    // The hooks-off option is added only once the process has switched it on (`hooks_off::activate`).
    assert_eq!(seen_arguments, arguments);
    let mut seen_environment: Vec<(OsString, OsString)> = Vec::<(OsString, OsString)>::new();
    for (name, value) in command.get_envs() {
        seen_environment.push((
            name.to_os_string(),
            value.expect("overlay sets, never removes").to_os_string(),
        ));
    }
    seen_environment.sort();
    assert_eq!(seen_environment, overlay);
    // No overlay means the child environment is exactly the inherited one.
    assert_eq!(
        git_command(Path::new("git"), &[], &[]).get_envs().count(),
        0
    );
}

/// A child's own exit code and its terminating signal are both observed exactly.
#[test]
fn run_reports_exit_codes_and_signals() {
    let root: PathBuf = fixture("forward-run");
    for (name, script, expected) in [
        ("zero", "#!/bin/sh\nexit 0\n", ChildOutcome::Exited(0)),
        ("seven", "#!/bin/sh\nexit 7\n", ChildOutcome::Exited(7)),
        (
            "git-128",
            "#!/bin/sh\nexit 128\n",
            ChildOutcome::Exited(128),
        ),
        (
            "term",
            "#!/bin/sh\nkill -TERM $$\n",
            ChildOutcome::Signaled(15),
        ),
        (
            "kill",
            "#!/bin/sh\nkill -KILL $$\n",
            ChildOutcome::Signaled(9),
        ),
        // The child sees each argument unchanged and the overlay variable.
        (
            "argv",
            "#!/bin/sh\n[ \"$#\" = 3 ] && [ \"$1\" = 'a b' ] && [ \"$2\" = '' ] && [ \"$3\" = '--' ] && [ \"$MARKER\" = 'm v' ] && exit 42\nexit 1\n",
            ChildOutcome::Exited(42),
        ),
    ] {
        let program: PathBuf = root.join(name);
        executable(program.as_path(), script.as_bytes());
        let outcome: ChildOutcome = run_real_git(
            program.as_path(),
            &[OsString::from("a b"), OsString::new(), OsString::from("--")],
            &[(OsString::from("MARKER"), OsString::from("m v"))],
        )
        .expect("child starts");
        assert_eq!(outcome, expected, "{name}");
    }
    remove(root.as_path());
}

/// A program that cannot be started is an error for both forwarding forms,
///  never a fake exit code.
#[test]
fn unstartable_program_is_an_error() {
    let missing: &Path = Path::new("/nonexistent-directory/git");
    assert_eq!(
        run_real_git(missing, &[], &[])
            .expect_err("missing program")
            .kind(),
        std::io::ErrorKind::NotFound
    );
    // Process replacement returns only on failure, leaving this test process running.
    assert_eq!(
        replace_process_with_real_git(missing, &[], &[]).kind(),
        std::io::ErrorKind::NotFound
    );
}
