//! What:
//!  The words of an editor command,
//! made the way a POSIX shell makes them,
//! with no shell started to do it.
//! Why:
//!  `git var GIT_EDITOR` answers a command line (`code --wait`,
//! `emacsclient -a "" -nw`),
//! and Git's `launch_editor` hands that line to a shell.
//!  The wrapper starts no shell and writes no shell text
//! (`doc/planning/cli-git-no-self-invocation.md`,
//! `source_guard/no_self_invocation_tests.rs`),
//! so the words are made here instead:
//! blanks separate them,
//! single quotes hold their bytes,
//! double quotes hold theirs,
//! and a backslash escapes the next byte.
//!  A line that asks for more than that (an expansion,
//! a tilde,
//! a glob,
//! a control operator,
//! or an
//!  unbalanced quote) is not guessed:
//! the merge is native Git's whole,
//! which is where it stood before the wrapper opened the
//!  editor at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function editorWords(editor: string): string[] | undefined; // undefined: a shell would be needed
//! ```

/// Owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  The characters a shell would read as more than a word of their own.
/// Why:
///   An expansion,
/// a tilde,
/// a glob,
/// a control operator or a comment would give the command a meaning
///  this reader does not reproduce,
/// and Git's own `use_shell` decision is taken for those lines by whoever
///  needs a shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHELL_SPECIALS = '$`~|&;<>()*?[]#';
/// ```
const SHELL_SPECIALS: &str = "$`~|&;<>()*?[]#";

/// What:
///  Whether the byte separates words outside quotes,
/// as the shell's default `IFS` does.
/// Why:
///   Only blanks separate words;
/// a byte order mark or a non-breaking space is part of a word to the shell too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isBlank = (c: string) => c === ' ' || c === '\t' || c === '\n';
/// ```
fn is_blank(character: char) -> bool {
    return character == ' ' || character == '\t' || character == '\n';
}

/// What:
///  The words of an editor command,
/// or nothing when making them would need a shell.
/// Why:
///   Each word is one argument the person's editor receives,
/// and the file the message is written in is added after them by
///  the caller,
/// as `launch_editor` adds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editorWords(editor: string): OsString[] | undefined;
/// ```
pub fn editor_words(editor: &str) -> Option<Vec<OsString>> {
    let mut words: Vec<OsString> = Vec::new();
    let mut word: String = String::new();
    let mut started: bool = false;
    let mut characters = editor.chars();
    while let Some(character) = characters.next() {
        if is_blank(character) {
            if started {
                words.push(OsString::from(word.as_str()));
                word.clear();
                started = false;
            }
            continue;
        }
        if character == '\'' {
            // Single quotes hold every byte literally, expansions included, until their pair.
            let mut closed: bool = false;
            for held in characters.by_ref() {
                if held == '\'' {
                    closed = true;
                    break;
                }
                word.push(held);
            }
            if !closed {
                return None;
            }
            // `''` is one empty argument, so the word has begun even when nothing was written.
            started = true;
            continue;
        }
        if character == '"' {
            let held: String = double_quoted(&mut characters)?;
            word.push_str(held.as_str());
            started = true;
            continue;
        }
        if character == '\\' {
            // A backslash outside quotes holds the next byte, and a final one is the shell's wait for a line.
            let held: char = characters.next()?;
            if held == '\n' {
                continue;
            }
            word.push(held);
            started = true;
            continue;
        }
        if SHELL_SPECIALS.contains(character) {
            return None;
        }
        word.push(character);
        started = true;
    }
    if started {
        words.push(OsString::from(word.as_str()));
    }
    if words.is_empty() {
        return None;
    }
    return Some(words);
}

/// What:
///  The bytes a double-quoted run adds,
/// read up to its closing quote,
/// or nothing when the run asks for an expansion or never closes.
/// Why:
///   Inside double quotes a backslash holds only `$`,
/// a backtick,
/// a double quote,
/// a backslash and a newline;
///  an expansion asks for a shell,
/// and an unbalanced quote is not guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function doubleQuoted(chars): string | undefined;
/// ```
fn double_quoted(characters: &mut std::str::Chars<'_>) -> Option<String> {
    let mut held: String = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => return Some(held),
            '$' | '`' => return None,
            '\\' => {
                let next: char = characters.next()?;
                match next {
                    '"' | '\\' | '$' | '`' => held.push(next),
                    '\n' => {}
                    other => {
                        // The shell keeps a backslash that holds any other byte.
                        held.push('\\');
                        held.push(other);
                    }
                }
            }
            other => held.push(other),
        }
    }
    return None;
}

/// The words of an editor command and their controls.
#[cfg(test)]
#[path = "merge_isolation_editor_words_tests.rs"]
mod tests;
