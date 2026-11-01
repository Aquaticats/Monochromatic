//! What:
//!  Branch-creation facts of `git branch`,
//!  the escape hatch positions of all three
//!       guarded commands,
//!  and the regions Git refuses.
//! Why:
//!  The branch-worktree policy is bypassed by any creating spelling the parser reads as
//!      listing,
//!  and blocks ordinary work when it reads a listing as creation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseBranchCreationRegion({ subcommand: 'branch', postSubcommandArgs: ['topic'] }).createsBranch).toBe(true);
//! ```

use super::BranchCreationCommand::{Branch, Checkout, Switch};
/// The parser,
///  its result types,
///  the shared hatch spelling and the argument builders.
use super::{
    BranchCreationCommand, BranchCreationRegion, branch_creation_command,
    parse_branch_creation_region,
};
use crate::command_options::OptionErrorKind::{
    AmbiguousOption, HelpRequested, MissingValue, SingleDashLongOption, UnexpectedValue,
    UnknownOption,
};
use crate::command_options::{OptionErrorKind, WrapperOccurrence};
use crate::command_options_query::without_tokens;
use crate::command_test_support::os_arguments;
use crate::escape_hatch::BRANCH_WORKTREE_ESCAPE_HATCH as HATCH;
use std::ffi::OsString;

/// Split a space-separated argument line;
///  the empty line is the empty region.
fn words(line: &str) -> Vec<&str> {
    return line.split_whitespace().collect();
}

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(command: BranchCreationCommand, line: &str) -> BranchCreationRegion {
    return parse_branch_creation_region(
        command,
        os_arguments(words(line).as_slice()).as_slice(),
        &[],
    )
    .expect("valid region");
}

/// Whether a `git branch` region creates a branch.
fn branch_creates(line: &str) -> bool {
    return region(Branch, line).creates_branch;
}

/// Only the three guarded words select a command;
///  `worktree add -b` is not guarded.
#[test]
fn names_the_guarded_commands() {
    assert_eq!(branch_creation_command(b"branch"), Some(Branch));
    assert_eq!(branch_creation_command(b"checkout"), Some(Checkout));
    assert_eq!(branch_creation_command(b"switch"), Some(Switch));
    let others: [&[u8]; 6] = [
        b"worktree",
        b"branches",
        b"Branch",
        b"",
        b"restore",
        b"switch\xff",
    ];
    for word in others {
        assert_eq!(branch_creation_command(word), None, "{word:?}");
    }
}

/// Ported:
///  listing patterns,
///  deletion and rename pass;
///  plain creation and copy are creation.
#[test]
fn ports_the_incumbent_branch_cases() {
    for line in [
        "--list feature/*",
        "--delete old-topic",
        "--move old-topic new-topic",
    ] {
        assert!(!branch_creates(line), "{line}");
    }
    for line in ["topic", "--cop old-topic new-topic"] {
        assert!(branch_creates(line), "{line}");
    }
    let created: BranchCreationRegion = region(Branch, "topic");
    assert_eq!(created.implicit_creation_target, None);
    assert!(created.wrapper.escape.is_empty() && created.wrapper.other.is_empty());
}

/// Ported:
///  the checkout and switch creation options,
///  in the incumbent's spellings.
#[test]
fn ports_the_incumbent_checkout_and_switch_cases() {
    for line in ["-b topic", "--orphan topic", "--track origin/topic"] {
        assert!(region(Checkout, line).creates_branch, "{line}");
    }
    for line in ["--cre topic", "--force-c topic", "--track origin/topic"] {
        assert!(region(Switch, line).creates_branch, "{line}");
    }
    let paths: BranchCreationRegion = region(Checkout, "-- file.txt");
    assert!(!paths.creates_branch);
    assert_eq!(paths.implicit_creation_target, None);
}

/// One or two names create when no action option applies;
///  display options are not actions.
#[test]
fn branch_names_create_without_an_action() {
    for line in [
        "topic",
        "topic main",
        "-- topic",
        "--end-of-options topic",
        "-v topic",
        "-vv -q topic",
        "--color topic",
        "--no-color topic",
        "--abbrev topic",
        "--column topic",
        "--format x topic",
        "--sort refname topic",
        "-i topic",
        "--omit-empty topic",
        "--create-reflog topic",
        "-f topic main",
        "-t topic",
        "--track=inherit topic",
        "--no-track topic",
        "--recurse-submodules topic",
        "-l --no-list topic",
        "-d --no-delete topic",
        "-m --no-move topic",
        "-c --no-copy topic",
        "-u up --no-set-upstream-to topic",
        "--points-at=main --no-points-at topic",
        "--dry-run --no-dry-run topic",
        "--show-current --no-show-current topic",
        "--edit-description --no-edit-description topic",
        "--unset-upstream --no-unset-upstream topic",
    ] {
        assert!(branch_creates(line), "{line}");
    }
    for line in ["", "-v", "topic main extra", "-- a b c", "--color=always"] {
        assert!(!branch_creates(line), "{line}");
    }
}

/// Every action option,
///  every filter and the kind options stop plain creation.
#[test]
fn branch_actions_and_filters_do_not_create() {
    for line in [
        "-l topic",
        "--list topic",
        "-d topic",
        "-D topic",
        "-D --no-delete topic",
        "-m old new",
        "-M old new",
        "-M --no-move old new",
        "-u up topic",
        "--set-upstream-to=up topic",
        "--unset-upstream topic",
        "--show-current topic",
        "--edit-description topic",
        "--delete-merged=x topic",
        "--delete-merged x topic",
        "--contains main topic",
        "--contains topic",
        "topic --contains",
        "--no-contains main topic",
        "--with main topic",
        "--without main topic",
        "--merged main topic",
        "--no-merged main topic",
        "topic --merged",
        "--forked=main topic",
        "--points-at=main topic",
        "--no-points-at --points-at main topic",
        "-a topic",
        "-r topic",
        "--all topic",
        "--remotes topic",
        "--dry-run topic",
    ] {
        assert!(!branch_creates(line), "{line}");
    }
}

/// A copy creates its target with one or two names,
///  whatever `-a` or the tracking mode say.
#[test]
fn branch_copy_creates_its_target() {
    for line in [
        "-c new",
        "-c old new",
        "-C old new",
        "--copy old new",
        "-C --no-copy old new",
        "--no-copy -c old new",
        "-c -f old new",
        "-c -a old new",
        "-c --set-upstream old new",
        "-qc old new",
    ] {
        assert!(branch_creates(line), "{line}");
    }
    for line in [
        "-c",
        "-c a b c",
        "-c -d old new",
        "-c -m old new",
        "-c -l old new",
        "-c -u up new",
        "-c --contains main new",
        "-c --show-current new",
        "-c --delete-merged=x new",
        "-c --dry-run old new",
        "-c --recurse-submodules old new",
        "-d -m old new",
    ] {
        assert!(!branch_creates(line), "{line}");
    }
}

/// The last writer of the tracking mode decides whether Git refuses `--set-upstream`.
#[test]
fn branch_tracking_mode_is_last_writer_wins() {
    for line in [
        "--set-upstream topic",
        "--track --set-upstream topic",
        "--no-track --set-upstream topic",
        "--no-set-upstream --set-upstream topic",
    ] {
        assert!(!branch_creates(line), "{line}");
    }
    for line in [
        "--set-upstream --track topic",
        "--set-upstream -t topic",
        "--set-upstream --no-track topic",
        "--set-upstream --no-set-upstream topic",
    ] {
        assert!(branch_creates(line), "{line}");
    }
}

/// The hatch counts in option position only,
///  and never as a branch name.
#[test]
fn reports_the_escape_hatch_by_position() {
    for (command, values, escape, creates) in [
        // Ported from the wrapper-level test: `checkout --no-enforce-worktree-branch -b side`.
        (Checkout, vec![HATCH, "-b", "side"], vec![0], true),
        (Checkout, vec!["-b", HATCH, HATCH], vec![2], true),
        (Checkout, vec!["-b", HATCH], vec![], true),
        (Checkout, vec!["--orphan", HATCH], vec![], true),
        (Checkout, vec!["--", HATCH], vec![], false),
        (Switch, vec!["-c", "topic", HATCH], vec![2], true),
        (Switch, vec!["--create", HATCH], vec![], true),
        (Switch, vec!["--end-of-options", HATCH], vec![], false),
        (Branch, vec!["topic", HATCH], vec![1], true),
        (Branch, vec![HATCH, "topic", HATCH], vec![0, 2], true),
        (Branch, vec![HATCH], vec![0], false),
        (Branch, vec!["-u", HATCH], vec![], false),
        (Branch, vec!["--contains", HATCH], vec![], false),
        // `--color` takes an attached value only, so the next token is still an option.
        (Branch, vec!["--color", HATCH, "topic"], vec![1], true),
    ] {
        let found: BranchCreationRegion =
            parse_branch_creation_region(command, os_arguments(values.as_slice()).as_slice(), &[])
                .expect("valid region");
        assert_eq!(found.wrapper.escape, escape, "{values:?}");
        assert_eq!(found.creates_branch, creates, "{values:?}");
    }
}

/// Ported:
///  removing the hatch keeps an equal-looking option value.
#[test]
fn removes_the_escape_hatch_but_keeps_an_equal_value() {
    let arguments: Vec<OsString> = os_arguments(&["checkout", "-b", HATCH, HATCH]);
    let found: BranchCreationRegion =
        parse_branch_creation_region(Checkout, &arguments[1..], &[]).expect("valid region");
    assert_eq!(
        without_tokens(arguments.as_slice(), 1, found.wrapper.escape.as_slice()),
        os_arguments(&["checkout", "-b", HATCH])
    );
}

/// The caller's own wrapper flags are reported separately,
///  by their index in its list.
#[test]
fn reports_other_wrapper_flags() {
    let found: BranchCreationRegion = parse_branch_creation_region(
        Switch,
        os_arguments(&["--no-enforce-worktree", "-c", "topic", HATCH]).as_slice(),
        &[b"--no-enforce-worktree"],
    )
    .expect("valid region");
    assert_eq!(found.wrapper.escape, vec![3]);
    assert_eq!(
        found.wrapper.other,
        vec![WrapperOccurrence { flag: 0, token: 0 }]
    );
    assert!(found.creates_branch);
}

/// A region Git refuses is reported as refused,
///  per command table.
#[test]
fn reports_what_git_refuses() {
    for (command, line, kind) in [
        (Branch, "--bogus", UnknownOption),
        (Branch, "-x topic", UnknownOption),
        (Branch, "--no-remotes", UnknownOption),
        (Branch, "--no-contains=x --no-all", UnknownOption),
        (Branch, "-u", MissingValue),
        (Branch, "--delete-merged", MissingValue),
        (Branch, "--delete=1", UnexpectedValue),
        (Branch, "--s", AmbiguousOption),
        (Branch, "-list", SingleDashLongOption),
        (Branch, "-h", HelpRequested),
        (Checkout, "-c topic", UnknownOption),
        (Checkout, "--create topic", UnknownOption),
        (Checkout, "--no-ours", UnknownOption),
        (Checkout, "-b", MissingValue),
        (Checkout, "--orphan", MissingValue),
        (Switch, "-b topic", UnknownOption),
        (Switch, "--ours", UnknownOption),
        (Switch, "-p", UnknownOption),
        (Switch, "--f", AmbiguousOption),
        (Switch, "--create", MissingValue),
    ] {
        let refused: OptionErrorKind = parse_branch_creation_region(
            command,
            os_arguments(words(line).as_slice()).as_slice(),
            &[],
        )
        .expect_err("refused")
        .kind;
        assert_eq!(refused, kind, "{command:?} {line}");
    }
}
