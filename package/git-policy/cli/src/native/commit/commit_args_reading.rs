//! What:
//!  The `commit` argument region as Git reads it:
//!       which token takes the next one as its value,
//!       where the pathspec separator is,
//!       and the region with every short cluster's value separated.
//! Why:
//!  Every argument transform asks these questions,
//!  and only Git's own table answers whether a token is an option,
//! an abbreviation of one,
//! or its value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const separator = separatorIndex(args);
//! const normalised = normaliseCommitArgs(args);
//! ```

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

/// Pathspec separator token,
/// shared with the escape hatch module.
/// The pathspec separator that ends option parsing in Git and in the incumbent.
pub(super) const PATHSPEC_SEPARATOR: &str = "--";

/// Short-value options whose clustered form can carry the value in the same token.
pub(super) const SHORT_VALUE_OPTIONS: [&str; 6] = ["-m", "-F", "-C", "-c", "-t", "-U"];

/// Short commit options with an optional attached value (`-S[<keyid>]`,
/// `-u[<mode>]`).
pub(super) const OPTIONAL_VALUE_LETTERS: [char; 2] = ['S', 'u'];

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
pub fn takes_next_token(token: &str) -> bool {
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
pub(super) fn is_optional_value_letter(letter: &char) -> bool {
    return OPTIONAL_VALUE_LETTERS.contains(letter);
}
