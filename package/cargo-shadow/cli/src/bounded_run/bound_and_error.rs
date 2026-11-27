//! What:
//! What one bounded call is allowed and how it fails:
//! the bound it must finish within,
//! the name of what it runs and the command that runs it again,
//! and the error that says which of those was not met.
//! Why:
//! The owner's rule is that nothing waits forever,
//! so every call carries the time it may take,
//! and a failure after the program started must never read as an ordinary error:
//! `NotQuiescent` is the case a caller must not roll back under.
//! Split out of `bounded_run.rs`;
//! the two types are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded_run_bound_and_error.ts: interface Bound, type RunError
//! ```

/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// One variant of the error carries the I/O failure that ended the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::time::Duration;` imports a span of time.
/// Why:
/// The bound is a span,
/// and the error names it when the call ran past it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// The bound of one run and the words an error uses.
#[derive(Clone, Debug)]
pub struct Bound {
    /// The longest the child may run.
    pub limit: Duration,
    /// What is running,
    /// for the error.
    pub what: String,
    /// The command to run again,
    /// for the error.
    pub rerun: String,
}

/// Why a bounded run did not produce output.
#[derive(Debug)]
pub enum RunError {
    /// The child could not be started or a system call failed.
    Io(
        /// The system error.
        io::Error,
    ),
    /// The bound passed;
    /// the child and everything it started were stopped.
    TimedOut {
        /// What was running.
        what: String,
        /// The bound.
        limit: Duration,
        /// The command to run again.
        rerun: String,
    },
    /// Something the child started could not be stopped or shown to be gone.
    NotQuiescent {
        /// What was running.
        what: String,
        /// What the stop could not show:
        /// how long it waited and what it was waiting for,
        /// with the state of every member that was left (issue 6528).
        shortfall: String,
    },
    /// The output could not be read to its end.
    IncompleteOutput {
        /// What was running.
        what: String,
    },
    /// The input could not be delivered to the end.
    IncompleteInput {
        /// What was running.
        what: String,
    },
}

/// How the error reads to a person.
impl std::fmt::Display for RunError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the error with what was running and what it means for the files.
    /// Why:
    /// The texts say only what is established:
    /// the command may have changed files before it was stopped.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            RunError::Io(error) => write!(formatter, "{error}"),
            RunError::TimedOut {
                what,
                limit,
                rerun,
            } => write!(
                formatter,
                "{what} did not finish within {limit:?}; it and every process it started were stopped. It may have changed files before it was stopped; this tool did not restore or remove anything because of that. Run {rerun} again when it can finish"
            ),
            RunError::NotQuiescent {
                what,
                shortfall,
            } => write!(
                formatter,
                "{what} started a process that could not be stopped or shown to be gone, so files it may still write cannot be trusted; {shortfall}; this tool changed no file because of that and the journal of the run was kept"
            ),
            RunError::IncompleteOutput {
                what,
            } => write!(
                formatter,
                "the output of {what} could not be read to its end; the partial output was discarded"
            ),
            RunError::IncompleteInput {
                what,
            } => write!(
                formatter,
                "{what} did not take all of its input, so its answer is not trusted"
            ),
        };
    }
}
