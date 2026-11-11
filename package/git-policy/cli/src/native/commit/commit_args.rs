//! What:
//!  Argument transforms of native preparation:
//!       the normalised `commit` argument region,
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const invocation = splitCommitInvocation({ args, subcommandIndex });
//! const commitArgs = withoutAllFlag(invocation.commitArgs);
//! ```

/// Pathspec separator token,
/// shared with the escape hatch module.
/// The pathspec separator that ends option parsing in Git and in the incumbent.
const PATHSPEC_SEPARATOR: &str = "--";
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;

/// What:
///  Bring Git's own table of `git commit` long options and its abbreviation rules into this file.
/// Why:
///   Whether a token takes the next token as its value is Git's answer,
///  and every unambiguous abbreviation of a value-taking option is one of its spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { COMMIT_TABLE } from './command_commit_table.ts';
/// import { longOptionTakesNextToken } from './command_options_long_resolution.ts';
/// ```
use crate::command_commit_table::COMMIT_TABLE;
use crate::command_options_long_resolution::long_option_takes_next_token;

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

/// Short-value options whose clustered form can carry the value in the same token.
const SHORT_VALUE_OPTIONS: [&str; 6] = ["-m", "-F", "-C", "-c", "-t", "-U"];

/// Short commit options with an optional attached value (`-S[<keyid>]`,
/// `-u[<mode>]`).
const OPTIONAL_VALUE_LETTERS: [char; 2] = ['S', 'u'];

/// What:
///  Finds the first value-taking short option letter inside a cluster token.
/// Why:
///   A cluster such as `-am` carries `m`,
/// whose value is the following token or the rest of the cluster.
/// A letter with an attached value (`-S<keyid>`,
/// `-u<mode>`) ends the search instead:
/// what follows it is its value,
/// so a key ID or mode that spells `m`,
/// `F`,
/// `C`,
/// `c`,
/// `t` or `U` is never read as flags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// findValueOptionIndex(token)
/// ```
fn find_value_option_index(letters: &[char]) -> Option<usize> {
    for (index, letter) in letters.iter().enumerate().skip(1) {
        if OPTIONAL_VALUE_LETTERS.contains(letter) {
            return None;
        }
        let option: String = format!("-{letter}");
        if SHORT_VALUE_OPTIONS.contains(&option.as_str()) {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  Splits one short cluster that carries its value inline into the separated form.
/// Why:
///   `-amhello` becomes `-a`,
/// `-m`,
/// `hello`,
/// so the value is a token of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normaliseInlineShort('-amhello') // ['-a', '-m', 'hello']
/// ```
fn normalise_inline_short(token: &str) -> Vec<String> {
    if !token.starts_with('-') || token.starts_with("--") || token.len() <= 2 {
        return vec![String::from(token)];
    }
    let letters: Vec<char> = token.chars().collect();
    let Some(value_index) = find_value_option_index(letters.as_slice()) else {
        return vec![String::from(token)];
    };
    let mut tokens: Vec<String> = Vec::new();
    for letter in &letters[1..value_index] {
        tokens.push(format!("-{letter}"));
    }
    tokens.push(format!("-{}", letters[value_index]));
    let inline: String = letters[value_index + 1..].iter().collect();
    if !inline.is_empty() {
        tokens.push(inline);
    }
    return tokens;
}

/// What:
///  Whether a commit option token takes the next token as its value.
/// Why:
///   A value can be spelled like the separator (`-m --`),
/// and only a token in option position can end the options.
///  A short cluster takes the next token when its value letter ends it;
/// `-S` and `-u` take their value attached only;
///  a long option is matched as Git matches it,
/// so every unambiguous abbreviation of a value-taking option takes one too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takesNextToken('-m'); // true
/// takesNextToken('-qm'); // true
/// takesNextToken('-mhello'); // false
/// takesNextToken('--mess'); // true
/// ```
pub(crate) fn takes_next_token(token: &str) -> bool {
    if token.starts_with("--") {
        return long_option_takes_next_token(token.as_bytes(), COMMIT_TABLE);
    }
    if !token.starts_with('-') || token.len() < 2 {
        return false;
    }
    let letters: Vec<char> = token.chars().skip(1).collect();
    for (index, letter) in letters.iter().enumerate() {
        if OPTIONAL_VALUE_LETTERS.contains(letter) {
            return false;
        }
        if SHORT_VALUE_OPTIONS.contains(&format!("-{letter}").as_str()) {
            return index + 1 == letters.len();
        }
    }
    return false;
}

/// What:
///  The index of the `--` that ends the options of a commit region,
/// or nothing when the person typed none.
/// Why:
///   Everything after it is a path,
/// so a wrapper option must never follow it;
/// a `--` that is the value of an option such as
///  `-m` is a message and not the separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// separatorIndex(['-m', '--', '--', 'a.txt']); // 2
/// ```
pub fn separator_index(region: &[String]) -> Option<usize> {
    let mut index: usize = 0;
    while index < region.len() {
        let token: &str = region[index].as_str();
        if token == PATHSPEC_SEPARATOR {
            return Some(index);
        }
        if takes_next_token(token) {
            index += 1;
        }
        index += 1;
    }
    return None;
}

/// What:
///  Normalises the post-`commit` argument region,
/// keeping the pathspec tail verbatim.
/// Why:
///   Later transforms recognize separated values only,
/// and the pathspec tail after `--` must not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normaliseCommitArgs(['-amhello', 'file.ts']); // ['-a', '-m', 'hello', 'file.ts']
/// ```
pub fn normalise_commit_args(args: &[String]) -> Vec<String> {
    let separator: Option<usize> = separator_index(args);
    let (region, tail): (&[String], &[String]) = match separator {
        Some(index) => (&args[..index], &args[index..]),
        None => (args, &[]),
    };
    let mut normalised: Vec<String> = Vec::new();
    let mut value_next: bool = false;
    for token in region {
        // The value of an option is the person's text and is never read as a cluster of options.
        if value_next {
            normalised.push(token.clone());
            value_next = false;
            continue;
        }
        normalised.extend(normalise_inline_short(token));
        value_next = takes_next_token(token);
    }
    normalised.extend(tail.iter().cloned());
    return normalised;
}

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
/// keeping clustered short options.
/// Why:
///   The private index already holds every tracked change,
/// so native commit must not restage worktree bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutAllFlag(['-am', 'x']); // ['-m', 'x']
/// ```
pub fn without_all_flag(commit_args: &[String]) -> Vec<String> {
    let normalised: Vec<String> = normalise_commit_args(commit_args);
    let mut kept: Vec<String> = Vec::new();
    let mut value_next: bool = false;
    let mut after_separator: bool = false;
    for token in normalised {
        if after_separator || value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        if token == PATHSPEC_SEPARATOR {
            after_separator = true;
            kept.push(token);
            continue;
        }
        if token == "--all" {
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
                if *letter != 'a' {
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
///  The pathspec removal takes the last path of each pathspec,
///  as `lastIndexOf` found it in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateExplicitCommitArgs({ args, pathspecs });
/// ```
pub fn private_explicit_commit_args(args: &[String], pathspecs: &[String]) -> Vec<String> {
    let person_values: Vec<usize> = value_positions(args);
    let mut without_file: Vec<String> = Vec::new();
    let mut index: usize = 0;
    while index < args.len() {
        let token: &str = args[index].as_str();
        // A value is the person's text, never the control it spells.
        if person_values.contains(&index) {
            without_file.push(String::from(token));
            index += 1;
            continue;
        }
        if token == "--pathspec-from-file" {
            index += 2;
            continue;
        }
        if token.starts_with("--pathspec-from-file=") || token == "--pathspec-file-nul" {
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
    // Only the separator and the `-o` and `--only` the person typed in option position go; the same text as the value of an option
    //  (`-m --`) is the person's and stays.
    let separator: Option<usize> = separator_index(without_pathspecs.as_slice());
    let mut kept: Vec<String> = Vec::new();
    let mut value_next: bool = false;
    for (position, token) in without_pathspecs.into_iter().enumerate() {
        let past_separator: bool = match separator {
            Some(at) => position > at,
            None => false,
        };
        if past_separator {
            kept.push(token);
            continue;
        }
        if separator == Some(position) {
            continue;
        }
        if value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        value_next = takes_next_token(token.as_str());
        if token == "-o" || token == "--only" {
            continue;
        }
        kept.push(token);
    }
    return kept;
}

/// What:
///  The indexes whose token is the value of the option before it,
///  up to the separator.
/// Why:
///   The value of an option is the person's text,
/// so no transform may read it as a path or as a flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// valuePositions(['a.txt', '-m', 'a.txt']) // [2]
/// ```
fn value_positions(tokens: &[String]) -> Vec<usize> {
    let limit: usize = separator_index(tokens).unwrap_or(tokens.len());
    let mut positions: Vec<usize> = Vec::new();
    let mut index: usize = 0;
    while index < limit {
        let token: &str = tokens[index].as_str();
        index += 1;
        if takes_next_token(token) && index < limit {
            positions.push(index);
            index += 1;
        }
    }
    return positions;
}

/// What:
///  Removes the selection controls once the private index holds the exact user choice.
/// Why:
///   `-i`,
/// `-p`,
/// `--include`,
/// `--interactive` and `--patch` already ran against the private index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateSelectionCommitArgs({ args, pathspecs });
/// ```
pub fn private_selection_commit_args(args: &[String], pathspecs: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for token in private_explicit_commit_args(args, pathspecs) {
        if !["-i", "-p", "--include", "--interactive", "--patch"].contains(&token.as_str()) {
            kept.push(token);
        }
    }
    return kept;
}

/// What:
///  Whether a letter of a short-option cluster takes an optional value.
///  `&char` borrows the letter,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOptionalValueLetter = (letter: string) => OPTIONAL_VALUE_LETTERS.includes(letter);
/// ```
fn is_optional_value_letter(letter: &char) -> bool {
    return OPTIONAL_VALUE_LETTERS.contains(letter);
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
