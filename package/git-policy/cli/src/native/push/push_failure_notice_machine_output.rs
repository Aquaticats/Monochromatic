//! What:
//!  Whether a Git command's output is read by a program, so that no failure notice may be shown to it or taken from it.
//! Why:
//!  A prompt, an editor or a script runs `git status --porcelain` or `git rev-parse` often and reads every byte; the notice
//!  is meant for the person. The rule sits in a file of its own so `push_failure_notice.rs` stays under the file-size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isMachineRead(commandWord, region)
//! ```

use std::ffi::OsString;

/// Commands that a program reads the output of, or that carry a protocol on their streams.
const MACHINE_COMMANDS: &[&[u8]] = &[
    b"cat-file", b"check-attr", b"check-ignore", b"check-ref-format", b"commit-tree", b"config", b"credential", b"diff-files",
    b"diff-index", b"diff-tree", b"fast-export", b"fast-import", b"for-each-ref", b"hash-object", b"index-pack", b"ls-files",
    b"ls-tree", b"merge-base", b"mktree", b"name-rev", b"pack-objects", b"read-tree", b"receive-pack", b"rev-list",
    b"rev-parse", b"show-ref", b"symbolic-ref", b"unpack-objects", b"update-ref", b"upload-archive", b"upload-pack",
    b"var", b"write-tree", b"update-index", b"verify-pack",
];

/// Option spellings that ask a command for output made to be read by a program.
fn asks_for_machine_output(option: &[u8]) -> bool {
    return option == b"-z"
        || option == b"--null"
        || option.starts_with(b"--porcelain")
        || option.starts_with(b"--format")
        || option.starts_with(b"--pretty");
}

/// What:
///  Whether a command's output is read by a program, so that no record may be shown to it or taken from it.
/// Why:
///   A prompt, an editor or a script runs `git status --porcelain` or `git rev-parse` often and reads every byte; the
///  record is meant for the person, and the first such call would otherwise take it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isMachineRead(commandWord, region)
/// ```
pub fn is_machine_read(command: &[u8], region: &[OsString]) -> bool {
    if MACHINE_COMMANDS.contains(&command) {
        return true;
    }
    for argument in region {
        let bytes: &[u8] = argument.as_encoded_bytes();
        // Everything after the separator is a name, not an option.
        if bytes == b"--" {
            break;
        }
        if asks_for_machine_output(bytes) {
            return true;
        }
    }
    return false;
}
