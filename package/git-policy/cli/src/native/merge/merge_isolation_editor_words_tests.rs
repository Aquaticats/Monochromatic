//! What:
//!  The words of an editor command,
//! checked against the words a POSIX shell makes of the same line.
//! Why:
//!  The wrapper starts no shell,
//! so the only way to know the person's editor receives the arguments Git would
//!  give it is to compare the words with a real shell's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(editorWords(line)).toEqual(shellWords(line));
//! ```

use super::*;
use crate::test_support::{executable, fixture, remove};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The path the message file is given as,
/// which both sides append the way `launch_editor` does.
const MESSAGE: &str = "MESSAGE";

/// A script that prints each argument it receives on its own line.
fn printing_script(parent: &Path) -> PathBuf {
    let path: PathBuf = parent.join("print-arguments");
    executable(path.as_path(), b"#!/bin/sh\nprintf '%s\\n' \"$@\"\n");
    return path;
}

/// The arguments a shell gives the editor named by `line`,
/// in the form `launch_editor` runs it.
fn a_shell_s_word(line: &str) -> String {
    let finished = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("{line} \"$@\""))
        .arg(line)
        .arg(MESSAGE)
        .output()
        .expect("the shell starts");
    assert!(finished.status.success(), "{line}: {}", String::from_utf8_lossy(finished.stderr.as_slice()));
    return String::from_utf8_lossy(finished.stdout.as_slice()).into_owned();
}

/// The arguments this command gives the editor named by `words`.
fn these_words(words: &[OsString]) -> String {
    let finished = Command::new(&words[0])
        .args(&words[1..])
        .arg(MESSAGE)
        .output()
        .expect("the editor starts");
    assert!(finished.status.success(), "{}", String::from_utf8_lossy(finished.stderr.as_slice()));
    return String::from_utf8_lossy(finished.stdout.as_slice()).into_owned();
}

/// What:
///  Every line this command splits gives its editor the arguments a shell would,
/// so `code --wait` and a quoted path arrive whole.
/// Why:
///   The whole point of splitting the line here is that the words stay Git's.
#[test]
fn the_words_are_the_ones_a_shell_makes() {
    let parent: PathBuf = fixture("merge-editor-words");
    let script: PathBuf = printing_script(parent.as_path());
    let program: String = script.to_str().expect("the script's path is text").to_owned();
    let lines: [String; 8] = [
        program.clone(),
        format!("{program} --wait"),
        format!("{program}   --wait\t-n "),
        format!("{program} 'an argument with spaces'"),
        format!("{program} \"an argument with spaces\""),
        format!("{program} a\\ b"),
        format!("{program} '' \"\""),
        format!("{program} 'a $dollar and a `backtick`' \"d\\\"quoted\""),
    ];
    for line in lines {
        let Some(words) = editor_words(line.as_str()) else { panic!("{line}: the line splits") };
        assert_eq!(these_words(words.as_slice()), a_shell_s_word(line.as_str()), "{line}");
    }
    remove(parent.as_path());
}

/// What:
///  A line that asks a shell for an expansion,
/// a tilde,
/// a glob,
/// a control operator or a comment is not split at all.
/// Why:
///   Guessing it would give the editor other arguments than Git gives it,
/// and such a merge stays native Git's
///  whole instead.
#[test]
fn a_line_that_needs_a_shell_is_not_guessed() {
    for line in [
        "editor $HOME/x",
        "editor ~/x",
        "editor \"a $variable b\"",
        "editor *.txt",
        "editor a;b",
        "editor a | b",
        "editor a && b",
        "editor (a)",
        "editor a > out",
        "editor #comment",
        "editor 'unbalanced",
        "editor \"unbalanced",
        "editor trailing\\",
        "  ",
        "",
    ] {
        assert_eq!(editor_words(line), None, "{line:?} needs a shell");
    }
}

/// A line of blanks names no program,
/// so there is nothing to start.
#[test]
fn a_blank_command_makes_no_words() {
    assert_eq!(editor_words(" "), None);
    assert_eq!(editor_words("\t\n "), None);
}

/// An empty quoted argument is a word of its own,
/// which a program can tell from no argument at all.
#[test]
fn an_empty_quoted_argument_is_a_word() {
    assert_eq!(editor_words("editor ''"), Some(vec![OsString::from("editor"), OsString::from("")]));
    assert_eq!(editor_words("editor \"\""), Some(vec![OsString::from("editor"), OsString::from("")]));
}
