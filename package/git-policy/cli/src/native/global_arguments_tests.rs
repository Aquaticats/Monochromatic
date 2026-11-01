//! What:
//!  Native Git argument boundaries and opaque-value preservation.
//! Why:
//!  A wrapper cannot locate commands by splitting strings or treating option values as flags.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Inspect immutable argv and compare query/error controls with the selected native Git release.
//! ```

/// Import the actual parser and native argument/process types.
use super::{GlobalLayout, GlobalOutcome, command_tokens, global_layout};
use std::ffi::OsString;
use std::process::Command;

/// Own a fixture vector without changing any supplied argument's spelling.
fn arguments(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// Separated values,
///  inline forms and post-command flags have different boundaries.
#[test]
fn preserves_complete_global_prefixes_and_values() {
    for (values, expected) in [
        (vec!["-C", "", "-C", "link/..", "-c", "x.y=-h", "status"], 6),
        (
            vec![
                "--config-env",
                "x.y=ENV",
                "--attr-source=HEAD",
                "show",
                "-C",
            ],
            3,
        ),
        (vec!["--git-dir", "-h", "status"], 2),
        (
            vec!["--no-pager", "--no-advice", "--exec-path=/tools", "log"],
            3,
        ),
    ] {
        let input: Vec<OsString> = arguments(values.as_slice());
        let snapshot: Vec<OsString> = input.clone();
        assert_eq!(
            global_layout(input.as_slice()),
            GlobalLayout {
                prefix_len: expected,
                outcome: GlobalOutcome::Command
            }
        );
        assert_eq!(input, snapshot);
    }
}

/// Native query flags terminate before later tokens can be mistaken for commands.
#[test]
fn query_and_missing_value_boundaries_are_distinct() {
    for flag in [
        "--help",
        "-h",
        "--version",
        "-v",
        "--exec-path",
        "--exec-path-extra",
        "--html-path",
        "--man-path",
        "--info-path",
        "--list-cmds=parseopt",
    ] {
        let input: Vec<OsString> = arguments(&[flag, "commit"]);
        assert_eq!(
            global_layout(input.as_slice()).outcome,
            GlobalOutcome::Query,
            "{flag}"
        );
    }
    for flag in [
        "-C",
        "-c",
        "--config-env",
        "--attr-source",
        "--git-dir",
        "--namespace",
        "--work-tree",
        "--shallow-file",
    ] {
        assert_eq!(
            global_layout(arguments(&[flag]).as_slice()).outcome,
            GlobalOutcome::MissingValue
        );
    }
    assert_eq!(global_layout(&[]).outcome, GlobalOutcome::NoCommand);
    assert_eq!(
        global_layout(arguments(&["--bare"]).as_slice()).prefix_len,
        1
    );
}

/// Old or invented global spellings are left to Git,
///  not accepted through compatibility heuristics.
#[test]
fn unsupported_globals_do_not_gain_a_wrapper_interpretation() {
    for flag in [
        "--",
        "--super-prefix",
        "-C/fixture",
        "-cx.y=z",
        "--shallow-file=shallow",
        "--future-flag",
    ] {
        assert_eq!(
            global_layout(arguments(&[flag, "status"]).as_slice()).outcome,
            GlobalOutcome::InvalidOption,
            "{flag}"
        );
    }
}

/// The selected native Git executable confirms representative query and error forms.
#[test]
fn native_git_256_controls_confirm_the_option_boundary() {
    let version = Command::new("/usr/bin/git")
        .arg("--version")
        .output()
        .expect("Git fixture");
    assert!(version.status.success());
    assert_eq!(version.stdout, b"git version 2.56.0\n");
    let query = Command::new("/usr/bin/git")
        .arg("--exec-path-extra")
        .output()
        .expect("native prefix query");
    assert!(query.status.success());
    for flag in ["-C/fixture", "--super-prefix", "--shallow-file=unused"] {
        let rejected = Command::new("/usr/bin/git")
            .args([flag, "status"])
            .output()
            .expect("native option error");
        assert!(!rejected.status.success(), "{flag}");
        assert!(
            String::from_utf8_lossy(&rejected.stderr).contains("unknown option"),
            "{flag}"
        );
    }
}

/// Candidate path bytes are never decoded while recognizing surrounding ASCII flags.
#[cfg(unix)]
#[test]
fn native_non_utf8_values_remain_opaque() {
    use std::os::unix::ffi::OsStringExt;
    let input: Vec<OsString> = vec![
        OsString::from("-C"),
        OsString::from_vec(b"folder-\xff".to_vec()),
        OsString::from("status"),
    ];
    let snapshot: Vec<OsString> = input.clone();
    assert_eq!(global_layout(input.as_slice()).prefix_len, 2);
    assert_eq!(input, snapshot);
}

/// The command word and the tokens after it are split after the global options;
///  without a command there is no split.
#[test]
fn command_tokens_split_after_the_global_options() {
    for (values, word, region) in [
        (vec!["status"], "status", vec![]),
        (vec!["reset", "--hard"], "reset", vec!["--hard"]),
        (
            vec!["-C", "dir", "--no-pager", "add", "-A", "--", "file"],
            "add",
            vec!["-A", "--", "file"],
        ),
        (
            vec!["-c", "a=b", "switch", "switch"],
            "switch",
            vec!["switch"],
        ),
        (vec!["-C", "commit", "log", "-C"], "log", vec!["-C"]),
    ] {
        let owned: Vec<OsString> = arguments(values.as_slice());
        let (found_word, found_region) =
            command_tokens(owned.as_slice()).expect("the arguments name a command");
        assert_eq!(found_word, word, "{values:?}");
        assert_eq!(
            found_region,
            arguments(region.as_slice()).as_slice(),
            "{values:?}"
        );
    }
    for values in [
        vec![],
        vec!["--no-pager"],
        vec!["-C", "dir"],
        vec!["--version", "status"],
        vec!["--no-such-option", "status"],
        vec!["-C"],
    ] {
        assert_eq!(
            command_tokens(arguments(values.as_slice()).as_slice()),
            None,
            "{values:?}"
        );
    }
}
