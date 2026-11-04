//! What:
//!  The one message shape the worktree commands give when a step on a path fails: `<path> <what> (<error>)`.
//! Why:
//!  Dozens of steps word their refusal this way. A named function that takes the path and the words stands where a closure
//!  that captured them would, and it builds the message only when the step failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = pathFailure(await tryStat(path), path, 'cannot be examined'); // throws `${path} cannot be examined (${error})`
//! ```

use std::fmt::Display;
use std::path::Path;

/// What:
///  A step's outcome with its failure turned into the message `<path> <what> (<error>)`.
///  `Result<T, E>` is Rust's "a value or an error"; `E: Display` means any error that can print itself.
/// Why:
///   The message is made only in the failing case, as a closure passed to `map_err` was; a step that succeeds pays nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathFailure<T>(outcome: () => T, path: string, what: string): T {
///   try { return outcome(); } catch (error) { throw `${path} ${what} (${error})`; }
/// }
/// ```
pub fn path_failure<T, E: Display>(outcome: Result<T, E>, path: &Path, what: &str) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{} {what} ({error})", path.display())),
    };
}

/// What:
///  A step's outcome with its failure turned into the message `<subject> <what> (<error>)`, for a subject that is a name or
///  a phrase and not a path. `&dyn Display` is a borrowed value of any type that can print itself.
/// Why:
///   Some steps name a trash directory or the snapshot, not a path, in the same shape of message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function subjectFailure<T>(outcome: () => T, subject: string, what: string): T {
///   try { return outcome(); } catch (error) { throw `${subject} ${what} (${error})`; }
/// }
/// ```
pub fn subject_failure<T, E: Display>(outcome: Result<T, E>, subject: &dyn Display, what: &str) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{subject} {what} ({error})")),
    };
}

/// Controls for the message shape stay out of the release executable.
#[cfg(test)]
#[path = "worktree_path_failure_message_tests.rs"]
mod tests;
