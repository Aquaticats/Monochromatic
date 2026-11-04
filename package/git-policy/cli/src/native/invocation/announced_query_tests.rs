//! What:
//!  Recognition of the announced query spellings, the text built for them and the status gate.
//! Why:
//!  Only `--version`, `-v`, `--help` and `-h` at the query position are announced,
//!  the text is a promise to people and scripts that read it,
//!  and Git that did not exit with status 0 must leave the caller with Git's output alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(announcement('version', '1.2.3', '/usr/bin/git')).toBe('monochromatic-git-policy-cli 1.2.3, shadowing git at /usr/bin/git\n');
//! ```

/// Import the functions under test and the layout scan that feeds them.
use super::{
    AnnouncedQuery, WRAPPER_VERSION, announced_query, announcement, printable_after,
};
use crate::forwarding::ChildOutcome;
use crate::global_arguments::global_layout;
use std::ffi::OsString;
use std::path::Path;

/// Build owned arguments from text.
fn arguments(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// The query of these arguments, read the way the planner reads it.
fn query_of(values: &[&str]) -> Option<AnnouncedQuery> {
    let owned: Vec<OsString> = arguments(values);
    return announced_query(owned.as_slice(), &global_layout(owned.as_slice()));
}

/// The four spellings are recognized, alone and behind global options.
#[test]
fn the_four_spellings_are_recognized() {
    for (values, expected) in [
        (vec!["--version"], AnnouncedQuery::Version),
        (vec!["-v"], AnnouncedQuery::Version),
        (vec!["--help"], AnnouncedQuery::Help),
        (vec!["-h"], AnnouncedQuery::Help),
        (vec!["-C", "dir", "--version"], AnnouncedQuery::Version),
        (vec!["-c", "a.b=c", "--no-pager", "-h"], AnnouncedQuery::Help),
        (vec!["--version", "commit"], AnnouncedQuery::Version),
    ] {
        assert_eq!(query_of(values.as_slice()), Some(expected), "{values:?}");
    }
}

/// Other queries, commands, errors and look-alikes are not announced.
#[test]
fn everything_else_is_not_announced() {
    for values in [
        vec!["--exec-path"],
        vec!["--exec-path=/x", "--version-extra"],
        vec!["--html-path"],
        vec!["--man-path"],
        vec!["--info-path"],
        vec!["--list-cmds=builtins"],
        vec!["commit", "--help"],
        vec!["log", "-v"],
        vec!["help", "-h"],
        vec!["--versions"],
        vec!["-vv"],
        vec!["-C"],
        vec![],
        // The value of a separated-value option is never a query.
        vec!["-C", "--version", "status"],
    ] {
        assert_eq!(query_of(values.as_slice()), None, "{values:?}");
    }
}

/// The banner is one line with the version, then Git's path, ending in a line feed.
#[test]
fn the_banner_is_one_exact_line() {
    assert_eq!(
        announcement(AnnouncedQuery::Version, "1.2.3", Path::new("/usr/bin/git")),
        "monochromatic-git-policy-cli 1.2.3, shadowing git at /usr/bin/git\n"
    );
}

/// The help section starts with a blank line, names the wrapper and Git, and stays short.
#[test]
fn the_help_section_is_short_and_exact() {
    let section: String =
        announcement(AnnouncedQuery::Help, "1.2.3", Path::new("/usr/bin/git"));
    assert_eq!(
        section,
        "\nmonochromatic-git-policy-cli 1.2.3 sits in front of this git\n\
         It adds repository policies on commit, push and worktree commands,\n\
         the `git cli-git` management command (`git cli-git --help`),\n\
         and `git cli-git doctor`, which lists the traditional hooks a repository holds.\n\
         real git: /usr/bin/git\n"
    );
    assert!(section.lines().count() < 8, "{section}");
}

/// The version is the crate's own and is never empty.
#[test]
fn the_version_is_the_crate_version() {
    assert_eq!(WRAPPER_VERSION, env!("CARGO_PKG_VERSION"));
    assert!(!WRAPPER_VERSION.is_empty());
}

/// Only an exit code of 0 releases the text;
///  every other ending prints nothing.
#[test]
fn text_follows_only_a_successful_run() {
    assert_eq!(printable_after(ChildOutcome::Exited(0), "banner\n"), "banner\n");
    for outcome in [
        ChildOutcome::Exited(1),
        ChildOutcome::Exited(2),
        ChildOutcome::Exited(128),
        ChildOutcome::Exited(129),
        ChildOutcome::Signaled(9),
        ChildOutcome::Signaled(15),
    ] {
        assert_eq!(printable_after(outcome, "banner\n"), "", "{outcome:?}");
    }
}
