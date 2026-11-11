//! What:
//!  Where a `git remote` region's subcommand starts,
//!  and which spellings Git accepts before it.
//! Why:
//!  `git remote -v set-url origin <url>` writes the same setting as the form without `-v`,
//!  so the
//!       reading must skip exactly the options Git skips and stop where Git stops;
//!  the
//!       subcommand's own options belong to the subcommand and must not be read as this table's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseRemoteRegion(['-v', 'set-url']).subcommand).toBe('set-url');
//! ```

/// The reader,
///  its table and the tokenizer's refusal names.
use super::{parse_remote_region, RemoteRegion};
use crate::command_options::OptionErrorKind::{UnexpectedValue, UnknownOption};
use crate::command_options::OptionError;
use crate::command_remote_table::REMOTE_TABLE;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, fixture, git, os_arguments, remove, repository,
};
use std::path::PathBuf;

/// The subcommand of a space-separated region,
///  or nothing.
fn subcommand(line: &str) -> Option<String> {
    let values: Vec<&str> = line.split_whitespace().collect();
    let parsed: RemoteRegion = parse_remote_region(os_arguments(values.as_slice()).as_slice())
        .expect("valid region");
    return parsed
        .subcommand
        .map(|word| { return word.to_string_lossy().into_owned(); });
}

/// The options Git reads before a subcommand are skipped,
///  and the first argument names the subcommand.
#[test]
fn the_subcommand_is_the_first_token_that_is_not_one_of_gits_own_options() {
    for (line, expected) in [
        ("", None),
        ("-v", None),
        ("--verbose", None),
        ("-vv", None),
        ("--no-verbose", None),
        ("add beta https://example.invalid/beta.git", Some("add")),
        ("-v set-url origin https://example.invalid/two.git", Some("set-url")),
        ("--verbose add beta https://example.invalid/beta.git", Some("add")),
        ("-v -v add beta https://example.invalid/beta.git", Some("add")),
        // Git accepts a unique abbreviation of `--verbose`, so the reading must too.
        ("--verb add beta https://example.invalid/beta.git", Some("add")),
        ("--no-verbos rm beta", Some("rm")),
        ("show origin", Some("show")),
        ("get-url origin", Some("get-url")),
        // The subcommand's own options are the subcommand's, not this table's.
        ("add -t main beta https://example.invalid/beta.git", Some("add")),
        ("rename --progress origin two", Some("rename")),
        // `git remote -- add ...` names no subcommand Git can dispatch to.
        ("-- add beta https://example.invalid/beta.git", None),
    ] {
        assert_eq!(subcommand(line).as_deref(), expected, "{line}");
    }
}

/// A region Git refuses is reported,
///  never guessed,
///  with the token index inside the option words before the subcommand.
#[test]
fn a_region_git_refuses_is_reported() {
    for (line, kind, token) in [
        ("--bogus add beta", UnknownOption, 0),
        ("-vx add beta", UnknownOption, 0),
        ("-v --bogus add beta", UnknownOption, 1),
        ("--verbose=1 add beta", UnexpectedValue, 0),
    ] {
        let values: Vec<&str> = line.split_whitespace().collect();
        let refused: OptionError =
            parse_remote_region(os_arguments(values.as_slice()).as_slice()).expect_err("refused");
        assert_eq!((refused.kind, refused.token), (kind, token), "{line}");
    }
}

/// The table holds exactly the long options the binary lists,
/// and the subcommand words beside them are not options.
#[test]
fn the_table_holds_exactly_the_options_git_lists() {
    let directory: PathBuf = fixture("remote-table");
    let root: PathBuf = repository(directory.as_path(), "repository");
    let completion: String = git_completion(root.as_path(), &["remote"]);
    // The one listing holds the options and the subcommand words; only the options belong in this table.
    let mut listed: Vec<String> = Vec::<String>::new();
    for word in completion.split(' ') {
        if word.starts_with("--") {
            listed.push(String::from(word));
        }
    }
    assert_table_invariants(REMOTE_TABLE);
    assert_eq!(render_completion(REMOTE_TABLE), listed.join(" "));
    assert!(completion.contains(" set-url "), "the listing names the subcommands: {completion}");
    // A positive control: this Git accepts the spellings the table declares, abbreviations included.
    git(root.as_path(), &["remote", "--verbose", "add", "beta", "https://example.invalid/beta.git"]);
    git(root.as_path(), &["remote", "--verb", "add", "gamma", "https://example.invalid/gamma.git"]);
    git(root.as_path(), &["remote", "--no-verbos", "add", "delta", "https://example.invalid/delta.git"]);
    let refused: std::process::Output =
        crate::command_test_support::git_status(root.as_path(), &["remote", "set-u", "origin", "x"]);
    assert!(!refused.status.success(), "Git accepted an abbreviated subcommand");
    remove(directory.as_path());
}
