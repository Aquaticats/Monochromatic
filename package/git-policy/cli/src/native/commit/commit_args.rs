//! What:
//!  Argument transforms of native preparation:
//!       the split of global options from the commit region,
//!       the removal of `-a`/`--all`,
//!       and the arguments of the real `git commit` that run against the complete private index.
//! Why:
//!  Native `git commit` must see the policy-settled index,
//! not worktree bytes again,
//!  so selection controls and pathspecs are removed once the private index already holds them.
//!  Only a token in option position is an option,
//!  and a path is removed only where it is a path:
//!  the value of an option such as `-m` is the person's text and stays,
//!  even when it spells a path or a flag.
//!  The options end at `--` and at `--end-of-options`,
//!  and a control leaves in every spelling Git accepts of it.
//!  How the region is read as Git reads it lives in `commit_args_reading.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const invocation = splitCommitInvocation({ args, subcommandIndex });
//! const commitArgs = withoutAllFlag(invocation.commitArgs);
//! ```

/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;

/// What:
///  Bring the reading of the region and Git's own table of `git commit` long options into this file.
/// Why:
///   The transforms here remove a token only where the region reading says it is an option,
///  and every unambiguous abbreviation Git accepts is one of a control's spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { COMMIT_TABLE } from './command_commit_table.ts';
/// import { longOptionNames } from './command_options_long_resolution.ts';
/// import { boundaryIndex, takesNextToken } from './commit_args_reading.ts';
/// ```
use super::commit_args_reading::is_optional_value_letter;
use crate::command_commit_table::COMMIT_TABLE;
use crate::command_options_long_resolution::long_option_names;

/// What:
///  The reading of the region is the commit path's,
///  so callers keep reaching it through this module.
/// Why:
///   Moving it to its own file must not move a name any caller knows.
pub use super::commit_args_reading::{boundary_index, ends_the_options, normalise_commit_args, short_letter_takes_a_value, takes_next_token, value_positions};

/// The `git commit` option that stages every tracked change.
/// The private index already holds that staging,
/// so no spelling of it reaches the real commit.
const ALL: &str = "all";
/// The short letter of `git commit --all`.
const ALL_LETTER: char = 'a';
/// The `git commit` option that commits only the named paths.
/// The private index already holds that selection,
/// so no spelling of it reaches the real commit.
const ONLY: &str = "only";
/// The short letter of `git commit --only`.
const ONLY_LETTERS: [char; 1] = ['o'];

/// The `git commit` option that reads its pathspecs from a file.
/// The private index already holds that selection,
/// so no spelling of it reaches the real commit.
const PATHSPEC_FROM_FILE: &str = "pathspec-from-file";
/// The `git commit` option that reads a pathspec file as NUL-separated names.
/// It means nothing without the file option,
/// so it leaves with it.
const PATHSPEC_FILE_NUL: &str = "pathspec-file-nul";
/// The selection controls whose selection the private index already holds.
const SELECTION_CONTROLS: [&str; 5] = ["-i", "-p", "--include", "--interactive", "--patch"];
/// The long names of those controls,
/// which Git also accepts in every unambiguous abbreviation.
const SELECTION_OPTION_NAMES: [&str; 3] = ["include", "interactive", "patch"];
/// The short letters of those controls,
/// which a cluster such as `-im` carries.
const SELECTION_LETTERS: [char; 2] = ['i', 'p'];

/// Global options that redirect native Git away from the shadow repository,
/// so they are dropped.
const REDIRECTING_VALUE_OPTIONS: [&str; 3] = ["-C", "--git-dir", "--work-tree"];

/// Global options that consume the next token as their value.
const VALUE_TAKING_GLOBAL_OPTIONS: [&str; 7] = [
    "-c",
    "-C",
    "--git-dir",
    "--work-tree",
    "--namespace",
    "--super-prefix",
    "--attr-source",
];

/// What:
///  Splits a transformed argument list into the kept global options and the commit region.
/// Why:
///   Redirecting options (`-C`,
/// `--git-dir`,
/// `--work-tree`,
/// `--bare`) would send native commit elsewhere,
///  so they are dropped with their values,
/// and every other global option is kept in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitCommitInvocation({ args, subcommandIndex });
/// ```
pub fn split_commit_invocation(args: &[String], subcommand_index: usize) -> (Vec<String>, Vec<String>) {
    let globals: &[String] = &args[..subcommand_index];
    let mut kept: Vec<String> = Vec::new();
    let mut index: usize = 0;
    while index < globals.len() {
        let token: &str = globals[index].as_str();
        let name: &str = if token.starts_with("--") && token.contains('=') {
            &token[..token.find('=').unwrap_or(token.len())]
        } else {
            token
        };
        let separated: bool = VALUE_TAKING_GLOBAL_OPTIONS.contains(&token);
        if REDIRECTING_VALUE_OPTIONS.contains(&name) || token == "--bare" {
            if separated {
                index += 1;
            }
            index += 1;
            continue;
        }
        kept.push(String::from(token));
        if separated {
            if let Some(value) = globals.get(index + 1) {
                kept.push(value.clone());
            }
            index += 1;
        }
        index += 1;
    }
    return (kept, args[subcommand_index + 1..].to_vec());
}

/// What:
///  Removes `-a`/`--all` from the commit region,
/// keeping clustered short options and every path after the options end.
/// Why:
///   The private index already holds every tracked change,
/// so native commit must not restage worktree bytes,
///  and a path named `-a` or `--all` is the person's path and keeps its spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutAllFlag(['-am', 'x']); // ['-m', 'x']
/// ```
pub fn without_all_flag(commit_args: &[String]) -> Vec<String> {
    let normalised: Vec<String> = normalise_commit_args(commit_args);
    let mut kept: Vec<String> = Vec::new();
    let mut value_next: bool = false;
    let mut after_the_options: bool = false;
    for token in normalised {
        if after_the_options || value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        if ends_the_options(token.as_str()) {
            after_the_options = true;
            kept.push(token);
            continue;
        }
        if spells_a_flag(token.as_str(), ALL) {
            continue;
        }
        if token.starts_with('-') && !token.starts_with("--") && token.len() > 1 {
            let letters: Vec<char> = token[1..].chars().collect();
            let value_start: Option<usize> = letters.iter().position(is_optional_value_letter);
            let flags: &[char] = match value_start {
                Some(index) => &letters[..index],
                None => &letters[..],
            };
            let mut cluster: String = String::from("-");
            for letter in flags {
                if *letter != ALL_LETTER {
                    cluster.push(*letter);
                }
            }
            if let Some(index) = value_start {
                for letter in &letters[index..] {
                    cluster.push(*letter);
                }
            }
            if cluster == "-" {
                continue;
            }
            value_next = takes_next_token(cluster.as_str());
            kept.push(cluster);
            continue;
        }
        value_next = takes_next_token(token.as_str());
        kept.push(token);
    }
    return kept;
}

/// What:
///  Removes the pathspec-file controls and the explicit pathspecs,
///  then the `--` separator and `-o`/`--only`,
/// from the commit region.
/// Why:
///   The private index already holds the selected paths,
/// so the real commit names none of them.
///  Only a token in option position is removed:
/// the value of an option is the person's text,
///  so a message that spells a path,
/// a path command or a flag never leaves as that token.
///  A control is removed in every spelling Git accepts,
/// so an abbreviation such as `--pathspec-fr` cannot hand the real commit a second selection.
///  The pathspec removal takes the last path of each pathspec,
///  as `lastIndexOf` found it in the incumbent.
///  Everything after `--` or `--end-of-options` is a path and nothing else,
///  so a path that spells `--pathspec-file-nul`,
/// `-o` or `--all` is committed as the path it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateExplicitCommitArgs({ args, pathspecs });
/// ```
pub fn private_explicit_commit_args(args: &[String], pathspecs: &[String]) -> Vec<String> {
    let person_values: Vec<usize> = value_positions(args);
    let option_region_end: Option<usize> = boundary_index(args);
    let mut without_file: Vec<String> = Vec::new();
    let mut index: usize = 0;
    while index < args.len() {
        let token: &str = args[index].as_str();
        // A value is the person's text, never the control it spells, and a token past the end of the options is a path.
        let option_position: bool = match option_region_end {
            Some(at) => index < at,
            None => true,
        };
        if !option_position || person_values.contains(&index) {
            without_file.push(String::from(token));
            index += 1;
            continue;
        }
        if long_option_names(token.as_bytes(), PATHSPEC_FROM_FILE, COMMIT_TABLE) {
            // The name after it is the value the preparation already read, so it goes with the option.
            index += if takes_next_token(token) { 2 } else { 1 };
            continue;
        }
        // A value on this flag is a spelling Git refuses, and the refusal stays Git's.
        if spells_a_flag(token, PATHSPEC_FILE_NUL) {
            index += 1;
            continue;
        }
        without_file.push(String::from(token));
        index += 1;
    }
    let values: Vec<usize> = value_positions(without_file.as_slice());
    let mut removed: Vec<bool> = vec![false; without_file.len()];
    for pathspec in pathspecs.iter().rev() {
        // The last token equal to the pathspec that is a path goes,
        // as `.rposition(...)` found it.
        let mut last_position: Option<usize> = None;
        for (position, token) in without_file.iter().enumerate() {
            if removed[position] || values.contains(&position) {
                continue;
            }
            if token == pathspec {
                last_position = Some(position);
            }
        }
        if let Some(position) = last_position {
            removed[position] = true;
        }
    }
    let mut without_pathspecs: Vec<String> = Vec::with_capacity(without_file.len());
    for (position, token) in without_file.into_iter().enumerate() {
        if !removed[position] {
            without_pathspecs.push(token);
        }
    }
    // Only the end of the options and the `-o` and `--only` the person typed in option position go; the same text as the value
    //  of an option (`-m --`) is the person's and stays.
    let boundary: Option<usize> = boundary_index(without_pathspecs.as_slice());
    let mut kept: Vec<String> = Vec::new();
    let mut value_next: bool = false;
    for (position, token) in without_pathspecs.into_iter().enumerate() {
        let past_the_options: bool = match boundary {
            Some(at) => position > at,
            None => false,
        };
        if past_the_options {
            kept.push(token);
            continue;
        }
        if boundary == Some(position) {
            continue;
        }
        if value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        value_next = takes_next_token(token.as_str());
        if spells_a_flag(token.as_str(), ONLY) {
            continue;
        }
        // A cluster carries `-o` beside the letters that stay, and a letter that takes a value ends it.
        match cluster_without_letters(token.as_str(), ONLY_LETTERS.as_slice()) {
            Some(cluster) => {
                if cluster != "-" {
                    kept.push(cluster);
                }
            }
            None => kept.push(token),
        }
    }
    return kept;
}

/// What:
///  Whether one token is a flag of `git commit` in a spelling Git accepts,
///  written without a value.
/// Why:
///   A flag takes no value,
/// so a value attached to it (`--only=x`) is a spelling Git refuses,
///  and that refusal stays Git's rather than being removed with the flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spellsAFlag('--on', 'only') // true
/// spellsAFlag('--only=x', 'only') // false
/// ```
fn spells_a_flag(token: &str, name: &str) -> bool {
    return long_option_names(token.as_bytes(), name, COMMIT_TABLE) && !token.contains('=');
}

/// What:
///  Removes the selection controls once the private index holds the exact user choice.
/// Why:
///   `-i`,
/// `-p`,
/// `--include`,
/// `--interactive` and `--patch` already ran against the private index.
///  A value that spells one of them is the person's text and stays,
/// so the real commit is never handed an option whose value was removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateSelectionCommitArgs({ args, pathspecs });
/// ```
pub fn private_selection_commit_args(args: &[String], pathspecs: &[String]) -> Vec<String> {
    let without_explicit: Vec<String> = private_explicit_commit_args(args, pathspecs);
    let person_values: Vec<usize> = value_positions(without_explicit.as_slice());
    let mut kept: Vec<String> = Vec::new();
    for (position, token) in without_explicit.into_iter().enumerate() {
        if person_values.contains(&position) {
            kept.push(token);
            continue;
        }
        if names_selection_control(token.as_str()) {
            continue;
        }
        // A cluster carries its selection letter beside the other flags, as `-im` does.
        match without_selection_letters(token.as_str()) {
            Some(cluster) => {
                if cluster != "-" {
                    kept.push(cluster);
                }
            }
            None => kept.push(token),
        }
    }
    return kept;
}

/// What:
///  Whether one token names a selection control in any spelling Git accepts.
/// Why:
///   Git reads every unambiguous prefix of a long option (`--patc` for `--patch`),
/// and a control the real commit runs again asks the person a second time.
fn names_selection_control(token: &str) -> bool {
    if SELECTION_CONTROLS.contains(&token) {
        return true;
    }
    for name in SELECTION_OPTION_NAMES {
        if long_option_names(token.as_bytes(), name, COMMIT_TABLE) {
            return true;
        }
    }
    return false;
}

/// What:
///  The short cluster without the selection letters,
/// or nothing when the token is no cluster or carries none.
/// Why:
///   `-im` and `-ip` carry `-i` or `-p` beside other flags,
/// and those already ran against the private index.
///  A letter that takes a value (`-m<text>`,
///  `-S<keyid>`) ends the search:
/// what follows it is the person's value and is kept whole,
/// so a message or key that spells `i` or `p` never disappears.
fn without_selection_letters(token: &str) -> Option<String> {
    return cluster_without_letters(token, SELECTION_LETTERS.as_slice());
}

/// What:
///  The short cluster of one token without the letters named,
/// or nothing when the token is no cluster or carries none of them.
/// Why:
///   A control spelled as a short letter sits beside other flags in one token (`-om`),
/// and only the letter goes.
///  A letter that takes a value ends the search:
/// what follows it is the person's value and is kept whole,
/// so a message or key that spells one of the removed letters never disappears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clusterWithoutLetters('-om', ['o']) // Some('-m')
/// ```
fn cluster_without_letters(token: &str, removed: &[char]) -> Option<String> {
    if !token.starts_with('-') || token.starts_with("--") || token.len() < 2 {
        return None;
    }
    let letters: Vec<char> = token.chars().skip(1).collect();
    let mut cluster: String = String::from("-");
    let mut removed_one: bool = false;
    let mut index: usize = 0;
    while index < letters.len() {
        let letter: char = letters[index];
        // A letter that takes a value ends the flags: what follows it is the person's value and stays whole,
        // so a key or a message that spells a removed letter never disappears.
        if short_letter_takes_a_value(letter) {
            break;
        }
        if removed.contains(&letter) {
            removed_one = true;
        } else {
            cluster.push(letter);
        }
        index += 1;
    }
    if !removed_one {
        return None;
    }
    for letter in &letters[index..] {
        cluster.push(*letter);
    }
    return Some(cluster);
}

/// What:
///  Chooses the arguments of the real `git commit` for the commit mode.
/// Why:
///   Explicit-path commits drop their pathspecs;
/// index commits drop selection controls only
///  when selection ran privately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolvePrivateCommitArgs({ args, pathspecs, mode, selectedPrivately });
/// ```
pub fn resolve_private_commit_args(
    args: &[String],
    pathspecs: &[String],
    mode: TransactionMode,
    selected_privately: bool,
) -> Vec<String> {
    if mode == TransactionMode::ExplicitPath {
        return private_explicit_commit_args(args, pathspecs);
    }
    if selected_privately {
        return private_selection_commit_args(args, pathspecs);
    }
    return args.to_vec();
}

/// Argument transforms stay out of the release executable.
#[cfg(test)]
#[path = "commit_args_tests.rs"]
mod tests;
