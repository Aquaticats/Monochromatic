//! What:
//!  The one failure type of the linked-worktree ignored-state copy,
//!  with the complete user-facing diagnostic in its message.
//! Why:
//!  Every copy step fails with a message that names the path it could not handle
//!  and,
//! when there is one,
//! the operating system's cause.
//!  The incumbent renders the cause after the message with a fixed separator,
//!  so both wrappers print the same words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class WorktreeCopyError extends Error {
//!   constructor(message: string, cause?: unknown) { super(`${message}${causeSuffix(cause)}`); }
//! }
//! ```

/// What:
///  The generic message the lifecycle reports for a failure that is not already a copy failure.
/// Why:
///   The incumbent's `asWorktreeCopyError` wraps unknown failures with this exact sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'monochromatic-git-policy-cli: automatic ignored-state worktree copy failed.'
/// ```
pub const AUTOMATIC_FAILURE_MESSAGE: &str =
    "monochromatic-git-policy-cli: automatic ignored-state worktree copy failed. Hint: if you just created a worktree, it works without \
     the copied caches and build output; rebuild them there or copy the ones you need by hand.";

/// What:
///  One copy failure and its complete diagnostic.
/// Why:
///   Callers print the message unchanged;
/// the lifecycle decides the exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WorktreeCopyError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeCopyError(pub String);

/// What:
///  The diagnostic of one failed step:
/// its message,
/// then the cause when one exists.
/// Why:
///   A cause is appended exactly once,
/// after the separator the incumbent uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${message}${cause === undefined ? '' : ` Cause: ${caughtValueText(cause)}`}`
/// ```
pub fn failure_with_cause(
    message: &str,
    cause: Option<&dyn std::fmt::Display>,
) -> WorktreeCopyError {
    // `match` on the optional cause keeps the no-cause message free of any suffix.
    match cause {
        None => {
            return WorktreeCopyError(String::from(message));
        }
        Some(text) => {
            return WorktreeCopyError(format!("{message} Cause: {text}"));
        }
    }
}

/// What:
///  A failure from the operating system or from a process,
/// carried as its own text with no wrapper sentence.
/// Why:
///   Every step that does not own a wording of its own leaves the raw text to the nearest enclosing wrapper.
///   Messages monochromatic-git-policy-cli owns always start with `monochromatic-git-policy-cli:`,
/// so a raw text can be told apart without another flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error // a non-WorktreeCopyError propagates unchanged to the caller that wraps it
/// ```
pub fn raw_failure(cause: &dyn std::fmt::Display) -> WorktreeCopyError {
    return WorktreeCopyError(cause.to_string());
}

/// What:
///  The same as `raw_failure`,
/// for a cause passed by value,
/// so it can stand where a function path is wanted.
/// Why:
///   `map_err` hands its function the error by value;
/// this lets `.map_err(raw_failure_of)` replace a closure that only
///  borrowed the error to call `raw_failure`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rawFailureOf = (cause: unknown) => rawFailure(cause);
/// ```
pub fn raw_failure_of<Cause: std::fmt::Display>(cause: Cause) -> WorktreeCopyError {
    return raw_failure(&cause);
}

/// What:
///  Whether a failure is raw text that an enclosing wrapper still has to word.
/// Why:
///   The incumbent wraps only failures that are not already its own `WorktreeCopyError`,
/// and monochromatic-git-policy-cli's own messages always start with its prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// !(error instanceof WorktreeCopyError)
/// ```
pub fn is_raw_failure(error: &WorktreeCopyError) -> bool {
    return !error.0.starts_with("monochromatic-git-policy-cli:");
}

/// What:
///  Words a raw failure with the given message,
/// and leaves an owned failure as it is.
/// Why:
///   Each wrapping step names the input it was handling,
/// and never hides a message monochromatic-git-policy-cli already wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error instanceof WorktreeCopyError ? error : new WorktreeCopyError(message, error)
/// ```
pub fn wrap_raw(error: WorktreeCopyError, message: &str) -> WorktreeCopyError {
    // `if` keeps owned messages untouched and words only raw text.
    if is_raw_failure(&error) {
        return failure_with_cause(message, Some(&error.0));
    }
    return error;
}

/// What:
///  The lifecycle's wording for a raw failure that reached the top of a copy.
/// Why:
///   The user sees the generic sentence once,
/// with the underlying cause after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function asWorktreeCopyError(error: unknown): WorktreeCopyError;
/// ```
pub fn lifecycle_failure(error: WorktreeCopyError) -> WorktreeCopyError {
    return wrap_raw(error, AUTOMATIC_FAILURE_MESSAGE);
}

/// Error message controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_error_tests.rs"]
mod tests;
