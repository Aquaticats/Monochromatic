//! What:
//!  Explicit creation and the remote-guess candidate of `git checkout` and `git switch`
//!       regions,
//!  for every argument shape Git 2.56.0 distinguishes.
//! Why:
//!  `git checkout topic` silently creates a branch when one remote has `topic`;
//!  the
//!      policy can only probe the remote for the token this parser names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseBranchCreationRegion({ subcommand: 'checkout', postSubcommandArgs: ['topic'] }).implicitCreationTarget).toBe('topic');
//! ```

/// The parser,
///  the command names and the argument builders.
use super::BranchCreationCommand::{Checkout, Switch};
use super::{BranchCreationCommand, BranchCreationRegion, parse_branch_creation_region};
use crate::command_test_support::os_arguments;

/// Parse a space-separated region Git accepts,
///  with no other wrapper flags.
fn region(command: BranchCreationCommand, line: &str) -> BranchCreationRegion {
    let values: Vec<&str> = line.split_whitespace().collect();
    return parse_branch_creation_region(command, os_arguments(values.as_slice()).as_slice(), &[])
        .expect("valid region");
}

/// Require that a region neither creates explicitly nor names another candidate.
fn assert_targets(command: BranchCreationCommand, cases: &[(&str, Option<usize>)]) {
    for (line, target) in cases {
        let found: BranchCreationRegion = region(command, line);
        assert!(!found.creates_branch, "{command:?} {line}");
        assert_eq!(
            found.implicit_creation_target, *target,
            "{command:?} {line}"
        );
    }
}

/// Require explicit creation,
///  which leaves no candidate to probe.
fn assert_creates(command: BranchCreationCommand, lines: &[&str]) {
    for line in lines {
        let found: BranchCreationRegion = region(command, line);
        assert!(found.creates_branch, "{command:?} {line}");
        assert_eq!(found.implicit_creation_target, None, "{command:?} {line}");
    }
}

/// Git guesses for `git checkout <name>` and `git checkout <name> --` only.
#[test]
fn checkout_names_the_single_reference_argument() {
    assert_targets(
        Checkout,
        &[
            ("topic", Some(0)),
            ("topic --", Some(0)),
            ("-q topic", Some(1)),
            ("topic -q", Some(0)),
            ("-q topic -f --", Some(1)),
            ("--end-of-options topic", Some(1)),
            ("--end-of-options topic --", Some(1)),
            ("--conflict merge topic", Some(2)),
            ("--conflict=merge topic", Some(1)),
            ("--recurse-submodules topic", Some(1)),
            ("-l topic", Some(1)),
            ("", None),
            ("--", None),
            ("-- topic", None),
            ("--end-of-options -- topic", None),
            ("topic file", None),
            ("topic -- file", None),
            ("topic other --", None),
            ("-", None),
            ("- --", None),
        ],
    );
}

/// Without `--` a glob character makes the argument a pathspec;
///  with `--` it is a name.
#[test]
fn checkout_does_not_guess_for_an_unseparated_wildcard() {
    assert_targets(
        Checkout,
        &[
            ("top*", None),
            ("to?ic", None),
            ("to[ic", None),
            ("to\\ic", None),
            ("top* --", Some(0)),
            ("to?ic --", Some(0)),
        ],
    );
}

/// The options that switch guessing off,
///  by final state where Git keeps a final state.
#[test]
fn checkout_options_stop_guessing() {
    assert_targets(
        Checkout,
        &[
            ("--no-guess topic", None),
            ("--guess --no-guess topic", None),
            ("--no-guess --guess topic", Some(2)),
            ("--guess topic", Some(1)),
            ("-p topic", None),
            ("--patch topic", None),
            ("-p --no-patch topic", Some(2)),
            ("-d topic", None),
            ("--detach topic", None),
            ("--detach --no-detach topic", Some(2)),
            ("-2 topic", None),
            ("-3 topic", None),
            ("--ours topic", None),
            ("--theirs topic", None),
            ("--overlay topic", None),
            ("--no-overlay topic", None),
            ("-qd topic", None),
        ],
    );
}

/// Divergence:
///  a pathspec file does not stop Git from guessing.
#[test]
fn checkout_still_guesses_with_a_pathspec_file() {
    assert_targets(
        Checkout,
        &[
            ("--pathspec-from-file=list topic", Some(1)),
            ("--pathspec-from-file list topic", Some(2)),
        ],
    );
}

/// Every spelling of explicit creation;
///  `--no-track` also names a new branch.
#[test]
fn checkout_creates_explicitly() {
    assert_creates(
        Checkout,
        &[
            "-b new",
            "-bnew",
            "-B new",
            "-qb new",
            "-b new origin/topic",
            "--orphan new",
            "--orphan=new",
            "--orp new",
            "-t origin/topic",
            "--track origin/topic",
            "--track=inherit origin/topic",
            "--tr origin/topic",
            "--no-track origin/topic",
            "--no-orphan -b new",
            "-b new --no-guess",
        ],
    );
    // `--no-orphan` clears the name `--orphan` wrote, so Git guesses again.
    assert_targets(Checkout, &[("--orphan new --no-orphan topic", Some(3))]);
}

/// `git switch` takes one reference and no paths,
///  so `--` does not change the candidate.
#[test]
fn switch_names_the_single_reference_argument() {
    assert_targets(
        Switch,
        &[
            ("topic", Some(0)),
            ("-- topic", Some(1)),
            ("--end-of-options topic", Some(1)),
            ("-q topic", Some(1)),
            ("--discard-changes topic", Some(1)),
            ("top*", Some(0)),
            ("", None),
            ("--", None),
            ("topic other", None),
            ("topic -- other", None),
            ("-", None),
            ("--no-guess topic", None),
            ("--no-guess --guess topic", Some(2)),
            ("-d topic", None),
            ("--detach topic", None),
            ("--create new --no-create topic", Some(3)),
        ],
    );
}

/// Every spelling of explicit creation that `git switch` accepts.
#[test]
fn switch_creates_explicitly() {
    assert_creates(
        Switch,
        &[
            "-c new",
            "-cnew",
            "-C new",
            "-qc new",
            "--create new",
            "--create=new",
            "--cre new",
            "--force-create new",
            "--force-c new",
            "--orphan new",
            "-t origin/topic",
            "--track origin/topic",
            "--no-track origin/topic",
            "-c new --no-guess",
        ],
    );
}

/// Names are read as bytes:
///  a non-UTF-8 name is a candidate like any other.
#[cfg(unix)]
#[test]
fn names_a_non_utf8_candidate() {
    use crate::command_test_support::byte_argument;
    for command in [Checkout, Switch] {
        let found: BranchCreationRegion = parse_branch_creation_region(
            command,
            &[byte_argument(b"-q"), byte_argument(b"top\xffic")],
            &[],
        )
        .expect("valid region");
        assert_eq!(found.implicit_creation_target, Some(1), "{command:?}");
        let created: BranchCreationRegion =
            parse_branch_creation_region(command, &[byte_argument(b"--orphan=\xff")], &[])
                .expect("valid region");
        assert!(created.creates_branch, "{command:?}");
    }
}
