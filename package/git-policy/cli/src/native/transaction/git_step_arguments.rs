//! What:
//!  The arguments of one Git step that ran long,
//!  named so that a slow record explains itself,
//!  with every word the person typed left out.
//! Why:
//!  A record that says `git for-each-ref` took 26 seconds names no question,
//! and the next reader cannot tell
//!  the whole listing of every ref from one ref's value.
//! On 2026-11-12 four commits took 6 to 99 seconds,
//! and each record named the child by
//!  its program and subcommand alone
//! (`doc/troubleshooting/cli-git-one-child-of-a-commit-takes-seconds.md`).
//!  What the record may carry is bounded by what a Git step is asked:
//! an option says which question the step asked and is kept,
//!  an object name or a ref name is the repository's own and is kept,
//! and every other word (a path,
//! a message,
//! an editor's file) is the person's and is replaced by
//!  `an argument`.
//! A value that option carries is never kept,
//! whatever it is,
//! so `--message` and `--file` name their value by the same placeholder.
//! The list is bounded (`NAMED_WORDS_LIMIT`),
//! so a step asked about ten thousand paths says how many words it left out
//!  rather than writing them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! namedArguments(['commit', '--message', 'secret', 'a.txt']) // 'commit --message an argument an argument'
//! ```

/// The options before the subcommand that take a value,
/// shared with the step's own name.
use crate::add_timing::option_takes_value;
/// Owned operating-system text of one argument.
use std::ffi::OsString;

/// The most words of one step's arguments a record names.
///
/// Measured on 2026-11-12 over the 29,053 Git steps of this repository's own timing records:
/// 496 of them ran a second or longer (1.7 percent),
/// and the longest arguments among those are a `commit` of a handful of words and
///  a `for-each-ref` of one;
/// 12 words name the question every one of them asked.
/// The rest is a count,
/// so a step asked about ten thousand
///  paths adds the same few bytes as one asked about two.
pub const NAMED_WORDS_LIMIT: usize = 12;

/// The word that stands for a word the person typed.
pub const AN_ARGUMENT: &str = "an argument";

/// The options whose value is text the person wrote,
/// which is never named.
///  `-S` and `--gpg-sign` name a signing key rather than the person,
/// and they are here because a key names its owner.
const TEXT_OPTIONS: [&str; 12] = [
    "-m",
    "--message",
    "-F",
    "--file",
    "-C",
    "--reuse-message",
    "-c",
    "--reedit-message",
    "--author",
    "--template",
    "-S",
    "--gpg-sign",
];

/// The option's own name,
/// which is all of it before an `=` that carries its value.
fn option_name(word: &str) -> &str {
    return match word.split_once('=') {
        Some((before, _value)) => before,
        None => word,
    };
}

/// Whether `word` is one of the options whose value is never named,
///  written either as `--message` with its value on the next word or as `-mtext` on this one.
fn carries_the_person_s_text(word: &str) -> bool {
    let name: &str = option_name(word);
    for known in TEXT_OPTIONS {
        if name == known {
            return true;
        }
        // A short option's value can be written onto its own word (`-msecret`).
        if known.len() == 2 && word.starts_with(known) {
            return true;
        }
    }
    return false;
}

/// Whether `word` is an object name,
/// which is the repository's own name for a thing.
fn is_an_object_name(word: &str) -> bool {
    return (4..=64).contains(&word.len()) && word.bytes().all(is_lower_hex_digit);
}

/// Whether `byte` is a digit or a lower-case hexadecimal letter,
/// which is every byte an object name holds.
fn is_lower_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
}

/// Whether `word` is a ref name,
/// which is the repository's own name for a thing.
fn is_a_ref_name(word: &str) -> bool {
    return word == "HEAD" || word.starts_with("refs/");
}

/// What:
///  One word of a step's arguments as the record may name it,
///  or nothing when the word is the person's and is replaced.
/// Why:
///   The decision is per word and stands here once,
/// so a control reads it in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedWord = (word) => objectName(word) || refName(word) ? word : undefined;
/// ```
fn named_word(word: &str) -> Option<&str> {
    if is_an_object_name(word) || is_a_ref_name(word) {
        return Some(word);
    }
    return None;
}

/// What:
///  Keeps one word the record may name,
///  or counts it as one it left out.
/// Why:
///   The bound and the count are one decision,
/// and every word goes through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keep = (word) => named.length < LIMIT ? named.push(word) : (leftOut += 1);
/// ```
fn keep(named: &mut Vec<String>, left_out: &mut usize, word: &str) {
    if named.len() >= NAMED_WORDS_LIMIT {
        *left_out = left_out.saturating_add(1);
        return;
    }
    named.push(String::from(word));
}

/// What:
///  The arguments of one Git step,
///  as one line of text the record carries.
/// Why:
///   A step whose arguments are all left out still says how many it was given,
/// so a record never reads like a missing field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = namedArguments(arguments); // 'for-each-ref --format=%(refname)%00...'
/// ```
pub fn named_arguments(arguments: &[OsString]) -> String {
    let mut named: Vec<String> = Vec::new();
    let mut left_out: usize = 0;
    // Whether the subcommand has been named: every word after it is a question's own argument.
    let mut named_the_subcommand: bool = false;
    // `mut` lets one word take the value that follows it to itself.
    let mut index: usize = 0;
    while index < arguments.len() {
        let text: String = arguments[index].to_string_lossy().into_owned();
        let word: &str = text.as_str();
        if option_takes_value(arguments[index].as_os_str()) {
            // A global option's value is a directory of the wrapper's own and names no question.
            index = index.saturating_add(2);
            continue;
        }
        if carries_the_person_s_text(word) {
            // The option is named,
            // and a placeholder says a value stood here without saying what it was.
            keep(&mut named, &mut left_out, option_name(word));
            keep(&mut named,
            &mut left_out,
            AN_ARGUMENT);
            index = if word.contains('=') { index.saturating_add(1) } else { index.saturating_add(2) };
            continue;
        }
        if word.starts_with('-') {
            keep(&mut named, &mut left_out, word);
            index = index.saturating_add(1);
            continue;
        }
        // The first word that is not an option is the subcommand, which is the program's own verb and never the person's text.
        if !named_the_subcommand {
            keep(&mut named, &mut left_out, word);
            named_the_subcommand = true;
            index = index.saturating_add(1);
            continue;
        }
        match named_word(word) {
            Some(thing) => keep(&mut named, &mut left_out, thing),
            None => keep(&mut named, &mut left_out, AN_ARGUMENT),
        }
        index = index.saturating_add(1);
    }
    if left_out > 0 {
        named.push(format!("and {left_out} more"));
    }
    return named.join(" ");
}

/// What the arguments namer decided is a control in the test module.
#[cfg(test)]
#[path = "git_step_arguments_tests.rs"]
mod tests;
