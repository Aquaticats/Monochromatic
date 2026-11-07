//! What:
//!  The description of one running program that the look for the commit service of an earlier release could not rule out: its
//!  process ID, its command line in a form that is safe to print, the folder its `--journal` option names, and what the system
//!  said when that folder was looked up.
//! Why:
//!  A program whose command line carries `--journal` with a folder that cannot be resolved holds back the settle command and the
//!  fold of the earlier names, and it may be a tool that has nothing to do with Git. A person can act on that hold only when the
//!  message says which program it is. A command line can carry a URL with credentials, so the userinfo of every URL is masked
//!  with the shared masker (`lfs_output_mask.rs`) before the text exists as a `String`, and control characters are written as
//!  escapes, so one program's words can neither start a second line of the message nor move the terminal's cursor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const printable = (bytes: Uint8Array): string => escapeControls(decodeLossily(maskUserinfo(bytes)))
//! ```

use crate::lfs_output_mask::mask_userinfo;

/// A running program that may be an earlier commit service: its `--journal` folder could not be resolved, so it could not be
///  compared with the worktree's earlier journals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsureProgram {
    /// The process ID.
    pub pid: u32,
    /// The words of its command line, separated by one space, printable (see [`printable`]).
    pub command_line: String,
    /// The word after its `--journal` option, as the command line spells it, printable.
    pub journal_word: String,
    /// Whether that word is a relative path, which is looked up in the working directory of the program itself.
    pub journal_word_is_relative: bool,
    /// What the system answered when the folder was looked up (`No such file or directory (os error 2)`).
    pub reason: String,
}

/// What:
///  The bytes as text that is safe to print: the userinfo of every URL masked, bytes that are not UTF-8 replaced by
///  U+FFFD, and every control character (a newline, an escape that starts a terminal sequence) written as an escape such as `\n`.
/// Why:
///   The program is named in a one-line message that goes to a terminal. Another program's command line is not trusted text.
///  Masking works on the bytes, before any decoding, as the masker does everywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function printable(bytes: Uint8Array): string;
/// ```
pub fn printable(bytes: &[u8]) -> String {
    let masked: Vec<u8> = mask_userinfo(bytes);
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "earlier_service_unsure_program_tests.rs"]
mod tests;
