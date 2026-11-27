//! What:
//!  What one push this invocation ran itself left behind:
//!  the attempt it ended as,
//!  and the failure that stopped a push or one of its Git queries.
//! Why:
//!  The push,
//!  its queries and the retry that reads a failed attempt all answer with these two shapes,
//!  so they sit where each of those readers can name them
//!  without naming the file that starts the program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PushAttempt = { exitCode: number; signal?: number; output: string };
//! class SingleFlightError extends Error {}
//! ```

/// How to allow a push longer,
/// in the words of the program that runs it.
use crate::push_deadline_advice::limit_clause;

/// What:
///  Why a push or one of its Git queries could not finish.
/// Why:
///   Every variant is a failure the service reports,
/// and the commit stays saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SubprocessError | TypeError | NodeJS.ErrnoException
/// ```
#[derive(Debug)]
pub enum SingleFlightError {
    /// Git could not be started,
    /// or failed in a way the incumbent does not model as an exit status.
    Start(std::io::Error),
    /// A Git query exited with a status the caller did not expect.
    Git {
        /// Arguments of the query,
        /// for the diagnostic.
        arguments: String,
        /// Exit code,
        /// or nothing when a signal ended the process.
        code: Option<i32>,
        /// Complete standard error text of the query.
        stderr: String,
    },
    /// A coordination file could not be read or published.
    Io(std::io::Error),
    /// A Git query showed no sign of work for its quiet window and was stopped.
    TimedOut {
        /// Arguments of the query,
        /// for the diagnostic.
        arguments: String,
        /// The quiet window it exceeded,
        /// in whole seconds.
        seconds: u64,
    },
    /// A command of a merge step was still running at its total ceiling and was stopped.
    Ceiling {
        /// Arguments of the command,
        /// for the diagnostic.
        arguments: String,
        /// The ceiling it passed,
        /// in whole seconds.
        seconds: u64,
    },
}

impl std::fmt::Display for SingleFlightError {
    /// Writes the diagnostic text of the failure.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            Self::Start(error) => write!(formatter, "git could not be started: {error}"),
            Self::Git {
                arguments,
                code,
                stderr,
            } => write!(
                formatter,
                "git {arguments} exited with {code:?}: {}",
                stderr.trim_end()
            ),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::TimedOut {
                arguments,
                seconds,
            } => write!(
                formatter,
                "git {arguments} showed no sign of work for {seconds} seconds and was stopped; {}",
                limit_clause()
            ),
            Self::Ceiling {
                arguments,
                seconds,
            } => write!(
                formatter,
                "git {arguments} was still running at its {seconds}-second ceiling and was stopped: one command of a merge step may \
                 not run for ever, so the merge it was computing is not made. Nothing was changed: the commit is kept, no reference \
                 moved and the worktree was not touched. Hint: a merge driver that keeps working without returning does this; make \
                 it return, or set {} higher",
                crate::push_deadline_and_patience_settings::MERGE_CEILING_VARIABLE
            ),
        };
    }
}

impl std::error::Error for SingleFlightError {}

/// What:
///  One push this invocation ran itself.
/// Why:
///   `output` is Git's interleaved output with one trailing line break removed,
/// as the incumbent's
///  `nano-spawn` `output` is;
/// a successful push's output is never recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushAttempt = { exitCode: number; output: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushAttempt {
    /// Push exit code,
    /// `0` on success,
    /// and `128 + N` for a program a signal ended.
    pub exit_code: i64,
    /// The number of the signal that ended the program,
    /// and nothing when it exited by itself.
    /// Why it is carried beside the code:
    ///  `128 + N` is also a status a program can choose,
    ///  so only the signal says which of the two happened,
    ///  and the words after a failure say "was ended by signal N" for one and "exited with status N" for the other.
    pub signal: Option<i32>,
    /// Complete interleaved push output.
    pub output: String,
}
