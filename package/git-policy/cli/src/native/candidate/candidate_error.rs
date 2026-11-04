//! What:
//!  The one failure type of the candidate layer,
//!  with a closed list of causes.
//! Why:
//!  Every candidate failure stops policy evaluation (a file that could not be read
//!      was not checked),
//!  and callers branch on the cause without parsing message text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class CandidateError extends Error { readonly failure: CandidateFailure }
//! ```

/// What:
///  Why candidate listing or reading failed.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   The policy engine maps causes to engine-failure codes,
///  and tests name the
///       exact cause instead of matching diagnostic wording.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateFailure = 'git-not-started' | 'git-failed' | 'listing-malformed' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateFailure {
    /// The real Git program could not be started at all.
    GitNotStarted,
    /// A listing command ran and exited unsuccessfully.
    GitFailed,
    /// Listing output is not the NUL-delimited record format Git 2.56.0 prints.
    ListingMalformed,
    /// A listed entry has a file mode no candidate can carry.
    UnsupportedMode,
    /// A listed entry has a change status this layer does not interpret.
    UnsupportedStatus,
    /// The index holds a conflicted (unmerged) entry.
    UnmergedPath,
    /// The object reader ended,
    ///  or was closed after an earlier failure.
    ReaderEnded,
    /// An object reply does not follow the `cat-file --batch` format.
    ReplyMalformed,
    /// An object reply stopped before its declared size or terminator.
    ReplyTruncated,
    /// An object reply names a different object than the one requested.
    ReplyMismatched,
    /// Git reported the requested object as missing.
    ObjectMissing,
    /// The requested object exists with a type the caller cannot use.
    ObjectKindUnexpected,
    /// A candidate belongs to a version that was invalidated.
    StaleCandidate,
    /// The private index copy a prediction stages into could not be created or copied.
    PrivateIndexUnavailable,
}

/// What:
///  One candidate failure:
///  its cause and a complete explanation.
///       `String` is a heap-allocated,
///  growable UTF-8 buffer this struct owns.
///       Sibling the reader might expect:
///  `&str`,
///  a borrowed view that owns nothing.
/// Why:
///   The error outlives the formatted text built at the failure site,
///  so a
///       borrowed `&str` would dangle.
///  Messages never contain file content or
///       pathnames:
///  a pathname can itself be a forbidden string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateError extends Error { failure: CandidateFailure; message: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateError {
    /// Closed cause callers branch on.
    pub failure: CandidateFailure,
    /// Complete diagnostic naming the Git operation and what to check.
    pub message: String,
}

/// What:
///  `impl CandidateError { ... }` attaches functions to the struct,
///  like class statics.
/// Why:
///   Failure sites keep their condition next to its user-facing explanation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new CandidateError(failure, message);
/// ```
impl CandidateError {
    /// What:
    ///  Build an error from a cause and borrowed text.
    ///  `&str` lends the caller's
    ///       bytes;
    ///  `String::from` copies them into storage the error owns.
    /// Why:
    ///   Messages are usually `format!` temporaries that end with the caller's statement.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(failure: CandidateFailure, message: string) { super(message); }
    /// ```
    pub fn new(failure: CandidateFailure, message: &str) -> CandidateError {
        // What: `return CandidateError { ... };` builds the struct and hands it back.
        // Why:  Rust never copies a borrowed string into an owning field implicitly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { failure, message };
        // ```
        return CandidateError {
            failure,
            message: String::from(message),
        };
    }
}

/// What:
///  `impl std::fmt::Display for CandidateError` supplies Rust's "print me" interface.
///       `::` walks module paths,
///  like `std.fmt.Display` would in TS.
/// Why:
///   The executable writes the message with ordinary formatting,
///  not field access.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return this.message; }
/// ```
impl std::fmt::Display for CandidateError {
    /// What:
    ///  `&self` borrows this error read-only;
    ///  `&mut` lends the formatter for writing;
    ///       `'_` is an unnamed lifetime the compiler fills in.
    /// Why:
    ///   Formatting writes into the caller's buffer without allocating another string.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// format(out: Writer): void { out.write(this.message); }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `.as_str()` lends the owned message as a borrowed view for the writer.
        return formatter.write_str(self.message.as_str());
    }
}

/// What:
///  An empty `impl` marks the type as a standard error value.
/// Why:
///   Generic error handling can carry it without converting it to a bare string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `extends Error` already provides this in TypeScript.
/// ```
impl std::error::Error for CandidateError {}

/// Message controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_error_tests.rs"]
mod tests;
