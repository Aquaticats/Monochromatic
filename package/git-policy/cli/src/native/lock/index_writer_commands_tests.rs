//! Controls for index-writer classification:
//!  each always-writing command,
//!  the option-dependent
//! forms of `restore`,
//!  `reset` and `apply`,
//!  abbreviations,
//!  separators and non-built-in routes.

use super::*;
use crate::command_test_support::{git, git_status, remove, repository_with_tracked_file};
use crate::forwarded_command::{ResolvedCommand, Route};
use std::ffi::OsString;
use std::path::PathBuf;

/// A resolved command whose subcommand is the first word,
///  reached directly.
fn direct(words: &[&str]) -> ResolvedCommand {
    return ResolvedCommand {
        arguments: words.iter().map(OsString::from).collect(),
        subcommand_index: 0,
        subcommand: words.first().map(OsString::from),
        route: Route::Direct,
        // These controls name a built-in, which no shell alias runs.
        shell_command: None,
        // A built-in is a word Git finds, so no autocorrection can reach it.
        missing_word: false,
    };
}

/// Whether a direct command written with these words writes the index.
fn writes(words: &[&str]) -> bool {
    return is_index_writer(&direct(words));
}

/// Every command in the always-writing list writes the index in every form.
#[test]
fn every_always_writing_command_writes_the_index() {
    // The registry lists each command once: a duplicate would hide the second row from the count of a set.
    let mut unique: Vec<&[u8]> = ALWAYS_WRITING_COMMANDS.to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(ALWAYS_WRITING_COMMANDS.len(), unique.len(), "each command is listed once");
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

/// Git's five mode options write one variable,
///  so the last one is the mode and a `--soft` written first does not keep the index.
#[test]
fn reset_writes_the_index_for_the_mode_git_ends_with() {
    assert!(writes(&["reset", "--soft", "--mixed"]));
    assert!(writes(&["reset", "--soft", "--hard"]));
    assert!(writes(&["reset", "--soft", "--ke"]));
    assert!(!writes(&["reset", "--hard", "--soft"]));
    assert!(!writes(&["reset", "--soft", "--so"]));
}

/// A token after a value-taking option is that option's value,
///  so a file named like a mode is no mode at all.
#[test]
fn reset_does_not_read_an_option_value_as_a_mode() {
    assert!(writes(&["reset", "--pathspec-from-file", "--soft"]));
    assert!(writes(&["reset", "--pathspec-from-file=--soft"]));
    assert!(writes(&["reset", "-U", "--soft"]));
    assert!(!writes(&["reset", "--pathspec-from-file", "--soft", "--soft"]));
    // An option with an optional value takes it only when it is attached, so this `--soft` is the mode.
    assert!(!writes(&["reset", "--recurse-submodules", "--soft"]));
    assert!(writes(&["reset", "--recurse-submodules=no", "--mixed"]));
}

/// A region Git refuses resets nothing,
///  so it never needs the coordination.
#[test]
fn reset_writes_no_index_when_git_refuses_the_region() {
    assert!(!writes(&["reset", "--soft=1"]));
    assert!(!writes(&["reset", "--unknown"]));
}

/// What:
///  The staged paths of a repository,
///  as Git reports them.
/// Why:
///   Whether the index was written is what the classification claims;
///  the staged set is the observable effect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(root, ['diff', '--cached', '--name-only']).stdout
/// ```
fn staged_paths(root: &std::path::Path) -> String {
    return String::from_utf8_lossy(&git(root, &["diff", "--cached", "--name-only"]).stdout).into_owned();
}

/// Every reading of a reset agrees with what real Git does to the index,
///  in each of these forms.
#[test]
fn every_reset_reading_matches_the_index_git_writes() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("reset-index-writer");
    // A file whose name is a mode option, for the forms that read a pathspec list instead of an argument.
    std::fs::write(root.join("--soft"), b"tracked.txt\n").expect("pathspec list named like a mode");
    for region in [
        vec!["--soft"],
        vec!["--mixed"],
        vec!["--soft", "--mixed"],
        vec!["--hard", "--soft"],
        vec!["--pathspec-from-file", "--soft"],
        vec!["--pathspec-from-file=--soft"],
        vec!["--recurse-submodules", "--soft"],
    ] {
        // Stage a change, so a reset that writes the index has something to take back.
        std::fs::write(root.join("tracked.txt"), b"staged change\n").expect("modify tracked file");
        git(root.as_path(), &["add", "--", "tracked.txt"]);
        assert_eq!(staged_paths(root.as_path()), "tracked.txt\n", "{region:?} starts staged");
        let mut words: Vec<&str> = vec!["reset", "--quiet"];
        words.extend(region.iter().copied());
        let output: std::process::Output = git_status(root.as_path(), words.as_slice());
        assert!(output.status.success(), "{region:?} failed: {}", String::from_utf8_lossy(&output.stderr));
        let written: bool = staged_paths(root.as_path()).is_empty();
        let mut command: Vec<&str> = vec!["reset"];
        command.extend(region.iter().copied());
        assert_eq!(writes(command.as_slice()), written, "{region:?} classification against real Git");
        // Restore the tracked file for the next region, whatever this one did to it.
        git(root.as_path(), &["reset", "--quiet", "--hard"]);
    }
    remove(directory.as_path());
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
        // These controls name a built-in, which no shell alias runs.
        shell_command: None,
        missing_word: false,
    };
    assert!(is_index_writer(&command));
}
