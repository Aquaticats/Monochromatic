//! Controls for index-writer classification:
//!  each always-writing command,
//!  the option-dependent
//! forms of `restore`,
//!  `reset` and `apply`,
//!  abbreviations,
//!  separators and non-built-in routes.

use super::*;
use crate::forwarded_command::{ResolvedCommand, Route};
use std::ffi::OsString;

/// A resolved command whose subcommand is the first word,
///  reached directly.
fn direct(words: &[&str]) -> ResolvedCommand {
    return ResolvedCommand {
        arguments: words.iter().map(OsString::from).collect(),
        subcommand_index: 0,
        subcommand: words.first().map(OsString::from),
        route: Route::Direct,
    };
}

/// Whether a direct command written with these words writes the index.
fn writes(words: &[&str]) -> bool {
    return is_index_writer(&direct(words));
}

/// Every command in the always-writing list writes the index in every form.
#[test]
fn every_always_writing_command_writes_the_index() {
    assert_eq!(ALWAYS_WRITING_COMMANDS.len(), 15);
    for name in ALWAYS_WRITING_COMMANDS {
        let word: &str = std::str::from_utf8(name).expect("ASCII command name");
        assert!(writes(&[word]), "{word} must write the index");
        assert!(writes(&[word, "--no-such-option", "file"]), "{word} with options must write the index");
    }
}

/// Commands that never write the real index,
///  including `commit` whose transaction coordinates itself.
#[test]
fn readers_and_the_commit_path_do_not_coordinate_as_index_writers() {
    assert!(!writes(&["status"]));
    assert!(!writes(&["log", "--oneline"]));
    assert!(!writes(&["diff", "--cached"]));
    assert!(!writes(&["commit", "-m", "x"]));
    assert!(!writes(&["fetch"]));
    assert!(!writes(&["clean", "-fd"]));
}

/// `restore` writes the index only with `--staged` or `-S`,
///  in full,
///  abbreviated or clustered form.
#[test]
fn restore_writes_the_index_only_when_staged() {
    assert!(writes(&["restore", "--staged", "x"]));
    assert!(writes(&["restore", "--stag", "x"]));
    assert!(writes(&["restore", "--staged=yes", "x"]));
    assert!(writes(&["restore", "-S", "x"]));
    assert!(writes(&["restore", "-WS", "x"]));
    assert!(!writes(&["restore", "x"]));
    assert!(!writes(&["restore", "--s", "x"]));
    assert!(!writes(&["restore", "-sS", "x"]));
    assert!(!writes(&["restore", "--", "--staged"]));
}

/// `reset` writes the index unless `--soft` is named,
///  whether in full or as an accepted prefix.
#[test]
fn reset_writes_the_index_except_when_soft() {
    assert!(writes(&["reset"]));
    assert!(writes(&["reset", "--hard", "HEAD~1"]));
    assert!(writes(&["reset", "--mixed"]));
    assert!(!writes(&["reset", "--soft", "HEAD~1"]));
    assert!(!writes(&["reset", "--so"]));
    assert!(!writes(&["reset", "--s"]));
}

/// `apply` writes the index with `--cached` or `--index`;
///  an ambiguous prefix does not count.
#[test]
fn apply_writes_the_index_only_when_cached_or_indexed() {
    assert!(!writes(&["apply", "x.patch"]));
    assert!(writes(&["apply", "--cached", "x.patch"]));
    assert!(writes(&["apply", "--cac", "x.patch"]));
    assert!(!writes(&["apply", "--c", "x.patch"]));
    assert!(!writes(&["apply", "--check", "x.patch"]));
    assert!(writes(&["apply", "--index", "x.patch"]));
    assert!(writes(&["apply", "--ind", "x.patch"]));
    assert!(!writes(&["apply", "--in", "x.patch"]));
}

/// Options after `--` are file names,
///  so they never change a classification.
#[test]
fn options_after_the_separator_are_not_options() {
    assert!(!writes(&["restore", "--", "-S"]));
    assert!(writes(&["reset", "--", "--soft"]));
    assert!(!writes(&["apply", "--", "--cached"]));
}

/// A lone `-` is a positional word,
///  not an option.
#[test]
fn a_lone_dash_is_not_an_option() {
    let command: ResolvedCommand = direct(&["restore", "-", "x"]);
    assert_eq!(option_tokens(&command), Vec::<&[u8]>::new());
    assert!(!is_index_writer(&command));
}

/// A long option matches by its full name or an accepted prefix,
///  with an attached value ignored.
#[test]
fn long_options_match_full_names_and_accepted_prefixes() {
    assert!(matches_long(b"--staged", RESTORE_STAGED));
    assert!(matches_long(b"--stag", RESTORE_STAGED));
    assert!(matches_long(b"--staged=1", RESTORE_STAGED));
    assert!(matches_long(b"--sta", RESTORE_STAGED));
    assert!(matches_long(b"--st", RESTORE_STAGED));
    assert!(!matches_long(b"--s", RESTORE_STAGED));
    assert!(!matches_long(b"--stagedx", RESTORE_STAGED));
    assert!(!matches_long(b"--other", RESTORE_STAGED));
}

/// A short-option cluster sets a letter only before a value-taking letter.
#[test]
fn clusters_set_letters_until_a_value_letter() {
    assert!(cluster_has(b"-S", b'S', b"s"));
    assert!(cluster_has(b"-WS", b'S', b"s"));
    assert!(!cluster_has(b"-sS", b'S', b"s"));
    assert!(!cluster_has(b"--S", b'S', b"s"));
    assert!(!cluster_has(b"S", b'S', b"s"));
}

/// Only a built-in route can write the index:
///  aliases to a built-in do,
///  shell and unresolved routes never.
#[test]
fn only_builtin_routes_can_write_the_index() {
    let alias: ResolvedCommand = ResolvedCommand {
        route: Route::Alias,
        ..direct(&["add", "x"])
    };
    let shell: ResolvedCommand = ResolvedCommand {
        route: Route::ShellAlias,
        ..direct(&["add", "x"])
    };
    let unresolved: ResolvedCommand = ResolvedCommand {
        route: Route::Unresolved,
        ..direct(&["add", "x"])
    };
    let query: ResolvedCommand = ResolvedCommand {
        subcommand: None,
        ..direct(&["--version"])
    };
    assert!(is_index_writer(&alias));
    assert!(!is_index_writer(&shell));
    assert!(!is_index_writer(&unresolved));
    assert!(!is_index_writer(&query));
}

/// Global options before the subcommand are not part of the option list,
///  and the subcommand is still classified.
#[test]
fn global_options_before_the_subcommand_do_not_hide_it() {
    let command: ResolvedCommand = ResolvedCommand {
        arguments: ["-C", "/repo", "rm", "x"].iter().map(OsString::from).collect(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("rm")),
        route: Route::Direct,
    };
    assert!(is_index_writer(&command));
}
