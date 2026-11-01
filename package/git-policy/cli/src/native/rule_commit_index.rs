//! What:
//!  The pure core of the index-against-`HEAD` check used by the commit-only transform.
//! Why:
//!  The caller runs real Git;
//!  this module only says which command to run and what its
//!      exit status means,
//!  so the decision is testable without a process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // indexStateFromExit(exitCode): 'differs' | 'matches' | 'unknown'
//! ```

/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Comparison of staged content with `HEAD`.
///  An `enum` is a closed set of alternatives.
/// Why:
///   `Unknown` means Git could not answer (unborn `HEAD`,
///  no repository);
///  the transform
///       then defers to real Git instead of rejecting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexVsHeadState = 'differs' | 'matches' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexVsHead {
    /// Staged changes exist.
    Differs,
    /// The index equals `HEAD`.
    Matches,
    /// Git could not compare.
    Unknown,
}

/// The query whose exit status answers the question:
///  0 for equal,
///  1 for staged changes.
pub const INDEX_QUERY: &[&str] = &["diff-index", "--quiet", "--cached", "HEAD", "--"];

/// What:
///  Build the real-Git argument list:
///  the caller's global options,
///  then the query.
///       `&[OsString]` borrows the global prefix;
///  `Vec<OsString>` is the owned result.
/// Why:
///   The check must inspect the same repository the commit will run in,
///  so `-C`,
///       `--git-dir` and the other global options are forwarded unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...preSubcommandArgs, 'diff-index', '--quiet', '--cached', 'HEAD', '--'];
/// ```
pub fn index_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into a list this function owns.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    // `for token in INDEX_QUERY` borrows each spelling; `OsString::from` copies it.
    for token in INDEX_QUERY {
        arguments.push(OsString::from(token));
    }
    return arguments;
}

/// What:
///  Interpret the query's exit status.
///  `Option<i32>` is "an exit code,
///  or nothing when
///       a signal ended the process";
///  `i32` is the platform's exit-code integer type.
/// Why:
///   Only the two documented codes are answers;
///  everything else is "Git cannot say".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return exitCode === 0 ? 'matches' : exitCode === 1 ? 'differs' : 'unknown';
/// ```
pub fn index_state_from_exit(exit_code: Option<i32>) -> IndexVsHead {
    // `Some(0)` is "the process exited with code 0".
    if exit_code == Some(0) {
        return IndexVsHead::Matches;
    }
    if exit_code == Some(1) {
        return IndexVsHead::Differs;
    }
    return IndexVsHead::Unknown;
}

/// Exit-status mapping and a real-Git control of the query.
#[cfg(test)]
#[path = "rule_commit_index_tests.rs"]
mod tests;
