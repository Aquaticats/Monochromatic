//! What:
//!  Plans of the four announced queries and of the queries that stay an exact process replacement.
//! Why:
//!  `--version`, `-v`, `--help` and `-h` run Git as a child and add text after its answer;
//!  `--exec-path`, `--html-path`, `--man-path`, `--info-path` and `--list-cmds=...` must still
//!      become Git, so a script that parses their output sees only Git's bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(planInvocation(['--version'], env, inputs).kind).toBe('forward-announced');
//! ```
#![cfg(unix)]

/// Import the decision under test and the fixtures shared with the forwarding controls.
use super::plan_invocation;
use super::tests::{forward, inputs, text};
use crate::action::Action;
use crate::real_git::ResolutionInputs;
use crate::test_support::{REAL_GIT, fixture, remove};
use std::ffi::OsString;
use std::path::PathBuf;

/// The crate version, read where the test is compiled.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The announced plan for these fixtures:
///  real Git as a child, the same overlay as a plain forward, and this text after Git's answer.
fn announced(arguments: Vec<OsString>, announcement: String) -> Action {
    let Action::Forward {
        real_git,
        overlay,
        stderr,
        ..
    } = forward(arguments.clone())
    else {
        panic!("the fixture is a plain forward");
    };
    return Action::ForwardAnnounced {
        real_git,
        arguments,
        overlay,
        announcement,
        stderr,
    };
}

/// The line `--version` and `-v` add, written out in full.
fn banner() -> String {
    return format!("monochromatic-git-policy-cli {VERSION}, shadowing git at {REAL_GIT}\n");
}

/// The section `--help` and `-h` add, written out in full.
fn help_section() -> String {
    return format!(
        "\nmonochromatic-git-policy-cli {VERSION} sits in front of this git\n\
         It adds repository policies on commit, push and worktree commands,\n\
         the `git cli-git` management command (`git cli-git --help`),\n\
         and `git cli-git doctor`, which lists the traditional hooks a repository holds.\n\
         real git: {REAL_GIT}\n"
    );
}

/// Each of the four spellings plans a child run of Git with the unchanged arguments.
#[test]
fn the_four_spellings_plan_an_announced_child_run() {
    let root: PathBuf = fixture("entry-announced");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for (spelling, text_after) in [
        ("--version", banner()),
        ("-v", banner()),
        ("--help", help_section()),
        ("-h", help_section()),
    ] {
        assert_eq!(
            plan_invocation(text(&[spelling]).as_slice(), &[], &resolution),
            announced(text(&[spelling]), text_after),
            "{spelling}"
        );
    }
    remove(root.as_path());
}

/// Global options before the query keep the query announced and reach Git untouched,
///  and a wrapper control before it is removed as for every other command.
#[test]
fn a_global_prefix_keeps_the_query_announced() {
    let root: PathBuf = fixture("entry-announced-prefix");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for (arguments, expected, text_after) in [
        (
            vec!["-C", "somewhere", "--version"],
            vec!["-C", "somewhere", "--version"],
            banner(),
        ),
        (
            vec!["-c", "x.y=z", "--no-pager", "-h"],
            vec!["-c", "x.y=z", "--no-pager", "-h"],
            help_section(),
        ),
        (
            vec!["--cli-git-keep-going", "--version"],
            vec!["--version"],
            banner(),
        ),
        // A word after the query is an ordinary argument that Git ignores.
        (vec!["--help", "cli-git"], vec!["--help", "cli-git"], help_section()),
    ] {
        assert_eq!(
            plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution),
            announced(text(expected.as_slice()), text_after),
            "{arguments:?}"
        );
    }
    remove(root.as_path());
}

/// Every other query keeps the exact process replacement.
#[test]
fn the_other_queries_stay_an_exact_replacement() {
    let root: PathBuf = fixture("entry-exact-queries");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for arguments in [
        vec!["--exec-path"],
        vec!["--exec-path=/somewhere"],
        vec!["--html-path"],
        vec!["--man-path"],
        vec!["--info-path"],
        vec!["--list-cmds=builtins"],
        vec!["-C", "somewhere", "--html-path"],
        // A spelling that only starts like an announced one is not one.
        vec!["--version-extra"],
        vec!["--helpful"],
    ] {
        let planned: Action = plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution);
        assert!(
            !matches!(planned, Action::ForwardAnnounced { .. }),
            "{arguments:?} must not be announced"
        );
    }
    for arguments in [
        vec!["--exec-path"],
        vec!["--html-path"],
        vec!["--man-path"],
        vec!["--info-path"],
        vec!["--list-cmds=builtins"],
        vec!["-C", "somewhere", "--html-path"],
    ] {
        assert_eq!(
            plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution),
            forward(text(arguments.as_slice())),
            "{arguments:?}"
        );
    }
    remove(root.as_path());
}

/// A subcommand's own help and version flags are not top-level queries.
#[test]
fn subcommand_help_is_not_announced() {
    let root: PathBuf = fixture("entry-subcommand-help");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for arguments in [
        vec!["commit", "--help"],
        vec!["log", "-h"],
        vec!["help", "--version"],
        vec!["version", "--build-options"],
    ] {
        let planned: Action = plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution);
        assert!(
            !matches!(planned, Action::ForwardAnnounced { .. }),
            "{arguments:?} must not be announced"
        );
    }
    remove(root.as_path());
}
