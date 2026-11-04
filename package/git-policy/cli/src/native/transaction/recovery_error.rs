//! What:
//!  The fail-closed failure of reading or recovering durable transaction state.
//! Why:
//!  Malformed,
//!  unsafe or contradictory state is never guessed at or deleted:
//!  the command
//!      stops with exit status 2 and a message naming the path,
//!  and the state stays for
//!      inspection (`CommitTransactionRecoveryError` in the incumbent;
//!  implementation plan,
//!      "Commit transactions and recovery").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! throw new CommitTransactionRecoveryError(`Unsafe transaction registry: ${root}`);
//! ```

/// What:
///  One recovery failure with its complete,
///  human-readable message.
///       A tuple-like `struct` holds one unnamed field.
/// Why:
///   Every site that fails closed knows what it found and where;
///  the message is the
///       whole diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommitTransactionRecoveryError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryError(pub String);

/// What:
///  The message as display text.
/// Why:
///   Diagnostics print the message unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for RecoveryError {
    /// Writes the message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

/// What:
///  A recovery failure for a filesystem step that failed.
/// Why:
///   One constructor names the step,
///  the path and the operating system's reason the same
///       way everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new CommitTransactionRecoveryError(`${operation} ${path} failed: ${error.message}`)
/// ```
pub fn io_failure(
    operation: &str,
    path: &std::path::Path,
    error: &std::io::Error,
) -> RecoveryError {
    return RecoveryError(format!("{operation} {} failed: {error}", path.display()));
}

/// Message controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_error_tests.rs"]
mod tests;
