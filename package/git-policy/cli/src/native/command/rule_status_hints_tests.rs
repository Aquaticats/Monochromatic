//! What:
//!  Every `status-hints-off.unit.test.ts` case,
//!  the global forms that run no
//!       subcommand,
//!  and a real-Git control of the injected pair.
//! Why:
//!  The injection must silence Git's hints without overriding an explicit caller choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(statusHintsOff(['status'])).toEqual(['-c', 'advice.statusHints=false', 'status']);
//! ```

/// The functions under test,
///  their result type and the real-Git fixture helpers.
use super::{QUIET_INJECTION, has_explicit_status_hints_override, status_hints_off};
use crate::command_test_support::{git_output, os_arguments, remove, repository_with_tracked_file};
use crate::rule_argument_rewrite::ArgumentRewrite;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

/// Decide for text arguments.
fn decide(values: &[&str]) -> ArgumentRewrite {
    return status_hints_off(os_arguments(values).as_slice());
}

/// Ask the override question for text arguments.
fn overridden(values: &[&str]) -> bool {
    return has_explicit_status_hints_override(os_arguments(values).as_slice());
}

/// Ported:
///  the five `hasExplicitStatusHintsOverride` cases.
#[test]
fn detects_an_explicit_override_before_the_subcommand_only() {
    assert!(overridden(&["-c", "advice.statusHints=true", "status"]));
    assert!(overridden(&["-c", "advice.statusHints", "status"]));
    assert!(overridden(&["-c", "Advice.StatusHints=true", "status"]));
    assert!(!overridden(&["status", "-c", "advice.statusHints=true"]));
    assert!(!overridden(&["status"]));
    // The question is about the global options of any command line.
    assert!(overridden(&["-c", "advice.statusHints=true", "commit"]));
    assert!(overridden(&["-c", "advice.statusHints=true"]));
    assert!(overridden(&["-c", "advice.statusHints=true", "--version"]));
    assert!(!overridden(&[]));
}

/// Ported:
///  "passes non-status commands through unchanged".
#[test]
fn passes_non_status_commands_through_unchanged() {
    for values in [
        vec!["commit", "-m", "message", "file.ts"],
        vec![],
        vec!["--version", "status"],
        vec!["-C", "status", "log"],
        vec!["--bogus", "status"],
        vec!["-c"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            ArgumentRewrite::Unchanged,
            "{values:?}"
        );
    }
}

/// Ported:
///  "injects advice override before status" and "preserves global options while
/// injecting before status".
#[test]
fn injects_the_advice_override_before_status() {
    assert_eq!(
        decide(&["status"]),
        ArgumentRewrite::Rewritten(os_arguments(&["-c", "advice.statusHints=false", "status"]))
    );
    assert_eq!(
        decide(&["-C", "/fixture/repo", "status", "--short"]),
        ArgumentRewrite::Rewritten(os_arguments(&[
            "-C",
            "/fixture/repo",
            "-c",
            "advice.statusHints=false",
            "status",
            "--short"
        ]))
    );
    assert_eq!(QUIET_INJECTION, &["-c", "advice.statusHints=false"]);
}

/// Ported:
///  "skips injection when user set advice.statusHints" and "... bare ...";
/// divergence:
///  `--config-env` sets the same key.
#[test]
fn skips_injection_when_the_caller_set_the_key() {
    for values in [
        vec!["-c", "advice.statusHints=true", "status"],
        vec!["-c", "advice.statusHints", "status"],
        vec!["--config-env=advice.statusHints=HINTS", "status"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            ArgumentRewrite::Unchanged,
            "{values:?}"
        );
    }
    // A post-subcommand token is a pathspec or status option, not configuration.
    assert_eq!(
        decide(&["status", "-c", "advice.statusHints=true"]),
        ArgumentRewrite::Rewritten(os_arguments(&[
            "-c",
            "advice.statusHints=false",
            "status",
            "-c",
            "advice.statusHints=true"
        ]))
    );
}

/// Real Git prints its `use "git add"` hint by default and not with the injected pair.
#[test]
fn the_injected_pair_silences_git_hints() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("status-hints");
    std::fs::write(root.join("tracked.txt"), b"two\n").expect("modify tracked file");
    let plain: Output = git_output(root.as_path(), os_arguments(&["status"]).as_slice());
    assert!(String::from_utf8_lossy(&plain.stdout).contains("use \"git add"));
    let arguments: Vec<OsString> = if let ArgumentRewrite::Rewritten(list) = decide(&["status"]) {
        list
    } else {
        panic!("expected a rewrite");
    };
    let quiet: Output = git_output(root.as_path(), arguments.as_slice());
    assert!(quiet.status.success());
    let text: String = String::from_utf8_lossy(&quiet.stdout).into_owned();
    assert!(
        text.contains("tracked.txt") && !text.contains("use \"git add"),
        "{text}"
    );
    remove(directory.as_path());
}
