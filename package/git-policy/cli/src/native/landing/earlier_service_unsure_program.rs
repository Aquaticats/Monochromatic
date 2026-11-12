//! What:
//!  The description of one running program that the look for the commit service of an earlier release could not rule out:
//! its
//!  process ID,
//! the program it runs (the executable path of the first word of its command line),
//! the folder its `--journal`
//!  option names,
//! and what the system said when that folder was looked up.
//! No other word of the command line is kept.
//! Why:
//!  A program whose command line carries `--journal` with a folder that cannot be resolved holds back the settle command and the
//!  fold of the earlier names,
//! and it may be a tool that has nothing to do with Git.
//! A person can act on that hold only when the
//!  message says which program it is.
//! Any other word of its command line can be a secret (`--token abc`,
//! `--password x`,
//! a key
//!  in a form no masker knows),
//! so those words are never kept,
//! shown or logged.
//! The words that are kept are masked with the
//!  shared masker (`lfs_output_mask.rs`) for URL userinfo before the text exists as a `String`,
//! and control characters are
//!  written as escapes,
//! so one program's words can neither start a second line of the message nor move the terminal's cursor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const printable = (bytes: Uint8Array): string => escapeControls(decodeLossily(maskUserinfo(bytes)))
//! const programOf = (words: Uint8Array[]): string => printable(cutAtFirstSpace(words[0] ?? ''))
//! ```

use crate::lfs_output_mask::mask_url_secrets;

/// A running program that may be an earlier commit service:
/// its `--journal` folder could not be resolved,
/// so it could not be
///  compared with the worktree's earlier journals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsureProgram {
    /// The process ID.
    pub pid: u32,
    /// The program it runs:
    /// the first word of its command line,
    /// which is the executable path or the bare name it was started
    ///  by,
    /// cut at its first space (a program that rewrites its own title puts more words there),
    /// printable (see
    ///  [`printable`]).
    /// Empty when the first word is empty.
    pub program: String,
    /// The word after its `--journal` option,
    /// as the command line spells it,
    /// printable.
    pub journal_word: String,
    /// Whether that word is a relative path,
    /// which is looked up in the working directory of the program itself.
    pub journal_word_is_relative: bool,
    /// What the system answered when the folder was looked up (`No such file or directory (os error 2)`).
    pub reason: String,
}

/// What:
///  The bytes as text that is safe to print:
/// the userinfo of every URL masked,
/// bytes that are not UTF-8 replaced by
///  U+FFFD,
/// and every control character (a newline,
/// an escape that starts a terminal sequence) written as an escape such as `\n`.
/// Why:
///   The program is named in a one-line message that goes to a terminal.
/// Another program's command line is not trusted text.
///  Masking works on the bytes,
/// before any decoding,
/// as the masker does everywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function printable(bytes: Uint8Array): string;
/// ```
pub fn printable(bytes: &[u8]) -> String {
    let masked: Vec<u8> = mask_url_secrets(bytes);
    let mut shown: String = String::with_capacity(masked.len());
    for character in String::from_utf8_lossy(masked.as_slice()).chars() {
        if character.is_control() {
            shown.extend(character.escape_default());
        } else {
            shown.push(character);
        }
    }
    return shown;
}

/// What:
///  The program a command line runs:
/// its first word,
/// cut at the first space,
/// printable.
/// Why:
///   The first word is the executable path (or the bare name) the program was started by.
/// A program that rewrites its title
///  (`nginx: master process --key value`) puts words that were never a path after it,
/// and those are as likely to be secrets as
///  any other argument,
/// so the cut keeps only what can be a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programOf(firstWord: Uint8Array | undefined): string;
/// ```
pub fn program_of(first_word: Option<&[u8]>) -> String {
    let word: &[u8] = first_word.unwrap_or_default();
    let end: usize = word.iter().position(u8::is_ascii_whitespace).unwrap_or(word.len());
    return printable(&word[..end]);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "earlier_service_unsure_program_tests.rs"]
mod tests;
