//! What:
//!  The words one pathspec element is read as:
//!  an exact file name,
//!  a directory or a pattern,
//!  or a subtraction that applies to every element of the list.
//! Why:
//!  Native Git commits the tracked and already-staged paths a directory or a pattern matches,
//!  and never a file nothing staged,
//!  while an exact path the person named is taken as the wrapper's own deliberate difference.
//!  The elements that subtract paths (`:(exclude)`,
//!  `:!`,
//!  `:^`) apply to every element of the selection,
//!  the exact file names included,
//!  because that is how native Git reads one list:
//!  `a.txt ':(exclude)a.txt'` takes no path,
//!  and one `git add` per element would stage `a.txt` anyway.
//!  A glob character is a pattern character only in a mode that reads it as one,
//!  so the mode the caller wrote has to be read here and not assumed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const excludes = (token) => token.startsWith(':!') || token.startsWith(':^') || token.startsWith(':(exclude');
//! ```

/// The Git context,
///  whose global options and overlay decide whether a glob character is ordinary text.
use crate::transaction_git::GitContext;
/// `OsStr` borrows operating-system text without copying it,
/// and `OsString` owns a whole element.
use std::ffi::{
    OsStr,
    OsString,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Whether one byte is a pattern character of the glob mode:
/// `*`,
/// `?` or `[`.
fn is_pattern_character(byte: &u8) -> bool {
    return matches!(*byte, b'*' | b'?' | b'[');
}

/// What:
///  Whether the pathspec modes in effect make a glob character ordinary text,
/// so `st*r.txt` names the file of that name.
/// Why:
///   The elements the person named decide both which paths this stage takes from the worktree and which of them an
/// ignore rule can refuse,
///  so both ask one owner rather than each reading the modes again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const literal = globCharactersAreLiteral(modesInEffect(globalPrefix, overlay));
/// ```
pub(crate) fn pattern_characters_are_literal(context: &GitContext) -> bool {
    return crate::conflict_side_modes::glob_characters_are_literal(
        crate::conflict_side_modes::modes_in_effect(
            context.global_prefix.as_slice(),
            context.overlay.as_slice(),
        )
        .as_slice(),
    );
}

/// What:
///  Whether one pathspec token names one file of the worktree rather than a directory or a pattern.
/// Why:
///   An exact name is the only shape step 3 of `SPEC.md` "Explicit-path commit" selects from the worktree,
/// and a directory or a pattern must take what native Git takes.
///  Pathspec magic selects a set and never one name,
/// and a glob character is a pattern character only in a mode that reads it as one:
///  `--literal-pathspecs` and `--noglob-pathspecs` make `st*r.txt` the file of that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesOneFile = (cwd, token, literal) => !token.startsWith(':') && (literal || !/[*?[]/.test(token)) && lstat(join(cwd, token)).isFile();
/// ```
pub(crate) fn names_one_file(
    cwd: &Path,
    token: &OsStr,
    literal: bool,
) -> bool {
    let bytes: &[u8] = token.as_encoded_bytes();
    if bytes.first() == Some(&b':') {
        return false;
    }
    // A backslash is Git's escape and not a pattern character here: `a\\b.txt` names the file of that name when one is
    // there, and a token whose literal pathname is not there is left to Git as the pattern it may be.
    if !literal && bytes.iter().any(is_pattern_character) {
        return false;
    }
    let Ok(held) = std::fs::symlink_metadata(cwd.join(token)) else {
        return false;
    };
    let kind = held.file_type();
    return kind.is_file() || kind.is_symlink();
}

/// What:
///  Whether one pathspec token subtracts the paths it names from the rest of the selection:
///  the long-form word `exclude` and the short forms `:!` and `:^`.
/// Why:
///   A subtracting element applies to the whole selection,
/// so the exact files a commit stages need it in the same `git add` start:
///  `a.txt ':(exclude)a.txt'` is no path to native Git,
/// and one start per element would stage the file the caller excluded.
///  The grammar is Git's own (`pathspec.c`,
/// `parse_short_magic` and `parse_long_magic`):
///  the long form holds words between the parentheses,
/// which Git matches by their whole spelling and only in lower case,
/// and the short form is a run of `!`,
/// `^` and
/// `/`,
/// of which only the first two subtract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const excludesPaths = (token) => token.startsWith(':(exclude') || /^:[!^]/.test(token);
/// ```
pub(crate) fn excludes_paths(token: &OsStr) -> bool {
    let bytes: &[u8] = token.as_encoded_bytes();
    let Some(magic) = bytes.strip_prefix(b":") else {
        return false;
    };
    if let Some(long) = magic.strip_prefix(b"(") {
        let Some(close) = long.iter().position(is_close_paren) else {
            return false;
        };
        return long
            .get(..close)
            .unwrap_or(long)
            .split(is_comma)
            .any(is_exclude_word);
    }
    let mut at: usize = 0;
    while let Some(byte) = magic.get(at) {
        match *byte {
            b'!' | b'^' => return true,
            b'/' => at += 1,
            _ => return false,
        }
    }
    return false;
}

/// Whether one borrowed element subtracts the paths it names.
/// Why:
///   The element lists below are read by path rather than by a closure,
/// so each question is one named function (`rust/avoidable-closure`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subtractsPaths = (element) => excludesPaths(element);
/// ```
pub(crate) fn subtracts_paths(element: &&OsString) -> bool {
    return excludes_paths(element.as_os_str());
}

/// Whether one borrowed element names paths the selection takes.
/// Why:
///   The complement of `subtracts_paths`,
/// read the same way:
/// a pattern list that holds no naming element reads as every path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesAPath = (element) => !subtractsPaths(element);
/// ```
pub(crate) fn names_a_path(element: &&OsString) -> bool {
    return !subtracts_paths(element);
}

/// Whether one long-form magic word is the subtraction word.
/// Why:
///   Git matches the word by its whole spelling in lower case,
/// so `EXCLUDE` and `exclude-` are other magic and stay naming elements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isExcludeWord = (word) => word === 'exclude';
/// ```
fn is_exclude_word(word: &[u8]) -> bool {
    return word == b"exclude";
}

/// Whether one byte closes the long form of pathspec magic.
fn is_close_paren(byte: &u8) -> bool {
    return *byte == b')';
}

/// Whether one byte separates the words of long-form pathspec magic.
fn is_comma(byte: &u8) -> bool {
    return *byte == b',';
}
