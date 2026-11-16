//! What:
//!  Reading a file this program keeps beside its sockets, without a bound the file can hold.
//! Why:
//!  A path in the runtime folder can be a named pipe instead of a file: the folder is this account's and a program of it,
//!  or a person, can put one there. A read of a pipe waits for a writer that may never come, and a command that reads it waits
//!  for ever, which the repository's rule forbids ("Nothing may wait for ever", and the review of `e347be3115`, part one,
//!  finding 3: a pipe at the note held `service stop` and `git worktree remove` for ever, and one at the log held every commit).
//!  So every read here opens with `O_NONBLOCK` and answers nothing for a path that is not a regular file: the caller then
//!  treats the file as absent, which is a state it already handles, and says what it found where a person must act.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = readRegularFile(path); // undefined unless it is a regular file, opened O_NONBLOCK
//! ```

use std::io::Read;
use std::path::Path;

/// What:
///  The text of `path`, or nothing when it is not a regular file or cannot be read.
/// Why:
///  The file is opened non-blocking and its type is read from the handle it opened, so nothing about the path can make the
/// read wait: a pipe answers end-of-file at once with no writer, and everything that is not a regular file answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readRegularFile('/run/user/1000/monochromatic-git-policy-cli/abc.pid') // string | undefined
/// ```
pub fn read_regular_file(path: &Path) -> Option<String> {
    use std::os::unix::fs::OpenOptionsExt;
    // `O_NONBLOCK` on a regular file changes nothing; on a pipe it makes the open return at once.
    let mut file: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW).open(path).ok()?;
    if !file.metadata().ok()?.file_type().is_file() {
        return None;
    }
    let mut text: String = String::new();
    match file.read_to_string(&mut text) {
        Ok(_read) => Some(text),
        Err(_cannot_read) => None,
    }
}

/// What:
///  Whether `path` holds a regular file, which a path a program keeps may not.
/// Why:
///  A file this program writes must not be made to write into a pipe: the write would either fail or block, and the state it
///  would leave is one no reader of this program expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isRegularFile('/run/user/1000/monochromatic-git-policy-cli/abc.log') // false for a pipe
/// ```
pub fn is_regular_file(path: &Path) -> bool {
    return matches!(std::fs::symlink_metadata(path), Ok(metadata) if metadata.file_type().is_file());
}

/// Controls of the reading stay out of the release executable.
#[cfg(test)]
#[path = "read_a_regular_file_tests.rs"]
mod tests;
