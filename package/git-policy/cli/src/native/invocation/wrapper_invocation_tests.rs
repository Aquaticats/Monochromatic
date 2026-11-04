//! What:
//!  Removal of wrapper controls from whole invocations,
//!  by command grammar.
//! Why:
//!  A control that reaches Git fails the command;
//!  a message,
//!  option value or path that
//!      spells a control and is removed changes what the command does.
//!  Both are checked
//!      for commands with a ported Git table,
//!  without one,
//!  and with a region Git refuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(stripWrapperControls(['reset', '--', '--no-enforce-worktree']).args).toEqual(['reset', '--', '--no-enforce-worktree']);
//! ```

/// The function under test,
///  its result types and the argument builder.
use super::{
    RegionReading, StrippedInvocation, command_region, command_word, strip_wrapper_controls,
};
use crate::command_options::OptionErrorKind;
use crate::command_test_support::os_arguments;
use crate::global_arguments::GlobalOutcome;
use crate::policy_registry::PolicyId;
use crate::wrapper_controls::{CONTROL_SPELLINGS, Controls, no_controls};
use std::ffi::OsString;

/// Strip one invocation written as text.
fn strip(values: &[&str]) -> StrippedInvocation {
    return strip_wrapper_controls(os_arguments(values).as_slice());
}

/// The record of an invocation that only escaped the given policies.
fn escaping(policies: &[PolicyId]) -> Controls {
    let mut controls: Controls = no_controls();
    controls.escaped = policies.to_vec();
    return controls;
}

/// Assert the kept arguments,
///  the recorded controls and the reading of one invocation.
fn assert_stripped(values: &[&str], kept: &[&str], controls: &Controls, reading: RegionReading) {
    let found: StrippedInvocation = strip(values);
    assert_eq!(found.arguments, os_arguments(kept), "{values:?}");
    assert_eq!(&found.controls, controls, "{values:?}");
    assert_eq!(found.reading, reading, "{values:?}");
}

/// Assert that nothing is removed or recorded.
fn assert_untouched(values: &[&str], reading: RegionReading) {
    assert_stripped(values, values, &no_controls(), reading);
}

/// Each command's own hatch and every general escape are removed from option position of a ported table.
#[test]
fn table_commands_lose_controls_in_option_position() {
    let linked: Controls = escaping(&[PolicyId::LinkedWorktreeOnly]);
    let branch: Controls = escaping(&[PolicyId::BranchWorktreeOnly]);
    let add: Controls = escaping(&[PolicyId::AddExplicit]);
    for (values, kept, controls) in [
        (
            vec!["add", "--no-enforce-bulk-add", "."],
            vec!["add", "."],
            &add,
        ),
        (
            vec!["add", ".", "--no-enforce-add-explicit"],
            vec!["add", "."],
            &add,
        ),
        (
            vec![
                "add",
                "--no-enforce-bulk-add",
                "--no-enforce-add-explicit",
                "-A",
            ],
            vec!["add", "-A"],
            &add,
        ),
        (
            vec!["reset", "--no-enforce-worktree", "--hard"],
            vec!["reset", "--hard"],
            &linked,
        ),
        (
            vec!["reset", "--hard", "--no-enforce-worktree", "--", "file"],
            vec!["reset", "--hard", "--", "file"],
            &linked,
        ),
        (
            vec!["clean", "-fd", "--no-enforce-linked-worktree-only"],
            vec!["clean", "-fd"],
            &linked,
        ),
        (
            vec!["stash", "--no-enforce-worktree", "list"],
            vec!["stash", "list"],
            &linked,
        ),
        (
            vec!["stash", "push", "--no-enforce-worktree", "-m", "note"],
            vec!["stash", "push", "-m", "note"],
            &linked,
        ),
        (
            vec!["switch", "--no-enforce-worktree-branch", "-c", "topic"],
            vec!["switch", "-c", "topic"],
            &branch,
        ),
        (
            vec![
                "checkout",
                "-b",
                "topic",
                "--no-enforce-branch-worktree-only",
            ],
            vec!["checkout", "-b", "topic"],
            &branch,
        ),
        (
            vec!["branch", "--no-enforce-worktree-branch", "topic"],
            vec!["branch", "topic"],
            &branch,
        ),
        // A general escape on a command whose own hatch is another one.
        (
            vec!["add", "--no-enforce-worktree", "file"],
            vec!["add", "file"],
            &linked,
        ),
        (vec!["reset", "--no-enforce-bulk-add"], vec!["reset"], &add),
    ] {
        assert_stripped(
            values.as_slice(),
            kept.as_slice(),
            controls,
            RegionReading::Table,
        );
    }
    let mut keep_going: Controls = escaping(&[PolicyId::RequireRoot]);
    keep_going.keep_going = true;
    for (values, kept) in [
        (
            vec![
                "push",
                "--cli-git-keep-going",
                "--no-enforce-require-root",
                "origin",
            ],
            vec!["push", "origin"],
        ),
        (
            vec![
                "status",
                "--no-enforce-require-root",
                "--short",
                "--cli-git-keep-going",
            ],
            vec!["status", "--short"],
        ),
        (
            vec![
                "config",
                "--cli-git-keep-going",
                "--no-enforce-require-root",
                "list",
            ],
            vec!["config", "list"],
        ),
        (
            vec![
                "--cli-git-keep-going",
                "-C",
                "dir",
                "add",
                "--no-enforce-require-root",
                "file",
            ],
            vec!["-C", "dir", "add", "file"],
        ),
    ] {
        assert_stripped(
            values.as_slice(),
            kept.as_slice(),
            &keep_going,
            RegionReading::Table,
        );
    }
}

/// `--no-enforce-only` is removed only from option position of `git commit`,
///  and is remembered separately.
#[test]
fn commit_hatch_is_removed_by_position() {
    let mut escaped: Controls = no_controls();
    escaped.commit_only_escaped = true;
    assert_stripped(
        &["commit", "--no-enforce-only", "-m", "message", "file"],
        &["commit", "-m", "message", "file"],
        &escaped,
        RegionReading::Table,
    );
    assert_stripped(
        &["commit", "-m", "--no-enforce-only", "--no-enforce-only"],
        &["commit", "-m", "--no-enforce-only"],
        &escaped,
        RegionReading::Table,
    );
    // The hatch as a message, as an attached value and as a path is the caller's text.
    for values in [
        vec!["commit", "-m", "--no-enforce-only", "--", "file"],
        vec!["commit", "--message=--no-enforce-only", "file"],
        vec!["commit", "-m", "x", "--", "--no-enforce-only"],
        vec!["commit", "-F", "--no-enforce-only"],
    ] {
        assert_untouched(values.as_slice(), RegionReading::Table);
    }
    // On another command the commit hatch is not a control at all: Git refuses it.
    let other: StrippedInvocation = strip(&["add", "--no-enforce-only", "file"]);
    assert_eq!(
        other.arguments,
        os_arguments(&["add", "--no-enforce-only", "file"])
    );
    assert!(
        matches!(other.reading, RegionReading::Refused(_)),
        "{other:?}"
    );
}

/// A control spelled as an option value,
///  as a message or after `--` is never removed.
#[test]
fn values_and_paths_spelling_a_control_are_kept() {
    for values in [
        vec!["reset", "--", "--no-enforce-worktree"],
        vec![
            "reset",
            "--hard",
            "--",
            "--no-enforce-worktree",
            "--cli-git-keep-going",
        ],
        vec!["add", "--", "--no-enforce-bulk-add"],
        vec!["add", "--pathspec-from-file", "--no-enforce-bulk-add"],
        vec!["add", "--pathspec-from-file=--cli-git-keep-going"],
        vec!["commit", "-m", "--cli-git-keep-going", "file"],
        vec!["push", "-o", "--cli-git-keep-going"],
        vec!["push", "--repo", "--no-enforce-require-root"],
        vec!["clean", "-e", "--no-enforce-worktree"],
        vec!["stash", "push", "-m", "--no-enforce-worktree"],
        vec!["checkout", "-b", "--no-enforce-worktree-branch"],
        vec!["status", "--", "--cli-git-keep-going"],
        vec!["-C", "--cli-git-keep-going", "add", "file"],
    ] {
        assert_untouched(values.as_slice(), RegionReading::Table);
    }
}

/// Without a ported table only the tokens directly after the subcommand are examined.
#[test]
fn untabled_commands_lose_leading_controls_only() {
    let mut keep_going: Controls = escaping(&[PolicyId::RequireRoot]);
    keep_going.keep_going = true;
    assert_stripped(
        &[
            "merge",
            "--no-enforce-require-root",
            "--cli-git-keep-going",
            "topic",
        ],
        &["merge", "topic"],
        &keep_going,
        RegionReading::LeadingOnly,
    );
    for spelling in CONTROL_SPELLINGS {
        let found: StrippedInvocation = strip(&["fetch", spelling.flag, "origin"]);
        assert_eq!(
            found.arguments,
            os_arguments(&["fetch", "origin"]),
            "{}",
            spelling.flag
        );
    }
    // Behind any other token the grammar is unknown, so the token stays and Git judges it.
    for values in [
        vec!["log", "-n", "--cli-git-keep-going"],
        vec!["log", "--grep", "--no-enforce-require-root"],
        vec!["merge", "topic", "--cli-git-keep-going"],
        vec!["rm", "--", "--no-enforce-require-root"],
        vec!["fetch"],
    ] {
        assert_untouched(values.as_slice(), RegionReading::LeadingOnly);
    }
}

/// `git worktree add` and `git worktree move` also lose the controls directly after their second word.
#[test]
fn worktree_opt_out_is_removed_after_either_word() {
    let mut skipped: Controls = no_controls();
    skipped.skip_worktree_copy = true;
    for (values, kept) in [
        (
            vec!["worktree", "--no-worktree-copy", "add", "../topic"],
            vec!["worktree", "add", "../topic"],
        ),
        (
            vec![
                "worktree",
                "add",
                "--no-worktree-copy",
                "-b",
                "topic",
                "../topic",
            ],
            vec!["worktree", "add", "-b", "topic", "../topic"],
        ),
        (
            vec!["worktree", "move", "--no-worktree-copy", "a", "b"],
            vec!["worktree", "move", "a", "b"],
        ),
        (
            vec![
                "worktree",
                "--no-worktree-copy",
                "add",
                "--no-worktree-copy",
                "../topic",
            ],
            vec!["worktree", "add", "../topic"],
        ),
        (
            vec!["--no-worktree-copy", "worktree", "add", "../topic"],
            vec!["worktree", "add", "../topic"],
        ),
    ] {
        assert_stripped(
            values.as_slice(),
            kept.as_slice(),
            &skipped,
            RegionReading::LeadingOnly,
        );
    }
    // Behind an option of `add`, behind another second word, and on another command's second word it stays.
    for values in [
        vec!["worktree", "add", "-b", "--no-worktree-copy", "../topic"],
        vec!["worktree", "add", "../topic", "--no-worktree-copy"],
        vec!["worktree", "remove", "--no-worktree-copy", "../topic"],
        vec!["worktree", "list", "add", "--no-worktree-copy"],
        vec!["remote", "add", "--no-worktree-copy", "origin"],
        vec!["worktree"],
        vec!["worktree", "add"],
    ] {
        assert_untouched(values.as_slice(), RegionReading::LeadingOnly);
    }
}

/// A region Git refuses keeps its tokens except leading controls,
///  and carries Git's refusal.
#[test]
fn refused_regions_report_the_refusal() {
    let found: StrippedInvocation = strip(&[
        "add",
        "--cli-git-keep-going",
        "--no-such-option",
        "--no-enforce-bulk-add",
        ".",
    ]);
    assert_eq!(
        found.arguments,
        os_arguments(&["add", "--no-such-option", "--no-enforce-bulk-add", "."])
    );
    assert!(found.controls.keep_going);
    assert_eq!(found.controls.escaped, Vec::<PolicyId>::new());
    match found.reading {
        RegionReading::Refused(error) => {
            assert_eq!(error.kind, OptionErrorKind::UnknownOption);
            assert_eq!(error.token, 1);
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    let help: StrippedInvocation = strip(&["commit", "--help"]);
    assert_eq!(help.arguments, os_arguments(&["commit", "--help"]));
    match help.reading {
        RegionReading::Refused(error) => assert_eq!(error.kind, OptionErrorKind::HelpRequested),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// Without a subcommand there is no region;
///  the layout describes the kept arguments.
#[test]
fn invocations_without_a_command_have_no_region() {
    for (values, kept, outcome, prefix_len) in [
        (vec![], vec![], GlobalOutcome::NoCommand, 0),
        (
            vec!["--cli-git-keep-going"],
            vec![],
            GlobalOutcome::NoCommand,
            0,
        ),
        (
            vec!["--cli-git-keep-going", "--version"],
            vec!["--version"],
            GlobalOutcome::Query,
            0,
        ),
        (
            vec!["--no-pager", "--no-such-option", "add"],
            vec!["--no-pager", "--no-such-option", "add"],
            GlobalOutcome::InvalidOption,
            1,
        ),
        (
            vec!["--cli-git-keep-going", "-C"],
            vec!["-C"],
            GlobalOutcome::MissingValue,
            0,
        ),
    ] {
        let found: StrippedInvocation = strip(values.as_slice());
        assert_eq!(found.arguments, os_arguments(kept.as_slice()), "{values:?}");
        assert_eq!(found.layout.outcome, outcome, "{values:?}");
        assert_eq!(found.layout.prefix_len, prefix_len, "{values:?}");
        assert_eq!(found.reading, RegionReading::NoCommand, "{values:?}");
    }
    // With a subcommand the layout points at it in the kept arguments.
    let found: StrippedInvocation = strip(&[
        "--cli-git-keep-going",
        "-C",
        "dir",
        "status",
        "--cli-git-keep-going",
    ]);
    assert_eq!(found.arguments, os_arguments(&["-C", "dir", "status"]));
    assert_eq!(found.layout.outcome, GlobalOutcome::Command);
    assert_eq!(found.layout.prefix_len, 2);
}

/// Arguments that are not UTF-8 keep their bytes and positions around removed controls.
#[cfg(unix)]
#[test]
fn undecodable_arguments_are_kept_unchanged() {
    use std::os::unix::ffi::OsStringExt;
    let path: OsString = OsString::from_vec(vec![b'f', 0xfe, 0xff]);
    let arguments: Vec<OsString> = vec![
        OsString::from("add"),
        OsString::from("--no-enforce-bulk-add"),
        path.clone(),
        OsString::from("--"),
        path.clone(),
    ];
    let found: StrippedInvocation = strip_wrapper_controls(arguments.as_slice());
    assert_eq!(
        found.arguments,
        vec![
            OsString::from("add"),
            path.clone(),
            OsString::from("--"),
            path,
        ]
    );
    assert_eq!(found.controls.escaped, vec![PolicyId::AddExplicit]);
}

/// The command word and its region are read after the global options,
///  without the word itself.
#[test]
fn the_command_word_and_its_region_follow_the_global_options() {
    for (values, word, region) in [
        (vec!["status"], "status", vec![]),
        (
            vec!["status", "--short", "file"],
            "status",
            vec!["--short", "file"],
        ),
        (
            vec!["-C", "dir", "--no-pager", "commit", "-m", "x"],
            "commit",
            vec!["-m", "x"],
        ),
        (
            vec!["--cli-git-keep-going", "worktree", "add", "../topic"],
            "worktree",
            vec!["add", "../topic"],
        ),
        (
            vec!["-C", "dir", "reset", "--no-enforce-worktree", "--hard"],
            "reset",
            vec!["--hard"],
        ),
    ] {
        let stripped: StrippedInvocation = strip(values.as_slice());
        assert_eq!(command_word(&stripped), word.as_bytes(), "{values:?}");
        assert_eq!(
            command_region(&stripped),
            os_arguments(region.as_slice()).as_slice(),
            "{values:?}"
        );
    }
}
