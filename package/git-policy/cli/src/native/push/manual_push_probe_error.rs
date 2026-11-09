//! What:
//!  Why the probe of a manual push could not establish the updates Git would push,
//! and what Git itself said when it refused.
//! Why:
//!  The probe (`manual_push_probe.rs`) and the lifecycle that reads its result share these types;
//! they sit in a file of their
//!  own so the probe stays under the file-size limit and this file needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ManualPushProbeError extends Error { refusal?: GitRefusal }
//! ```

use crate::auto_push_single_flight::SingleFlightError;
use crate::manual_push_cause::ManualPushCause;

/// What:
///  Git's own refusal of a push:
/// the exit status and the standard error it ended with.
/// Why:
///   When the dry run exits with a failure before it reports any destination,
/// Git has refused the push itself (a ref to
///  delete that the remote lacks,
/// a refspec naming nothing,
/// a remote that does not exist,
/// an option it does not know).
///  Nothing could be published,
/// so Git's words and status are the answer,
/// and the wrapper adds none of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitRefusal = { code: number; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitRefusal {
    /// The exit status Git ended with;
    /// never zero.
    pub code: i32,
    /// What Git wrote to standard error,
    /// as Git wrote it.
    pub stderr: String,
}

/// What:
///  Why the probe could not establish the updates Git would push.
/// Why:
///   Every variant stops the policy lifecycle;
/// none guesses an update.
/// A failure of the wrapper's own reading is a
///  content-unavailable engine failure,
/// and a refusal by Git carries Git's own report instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error { refusal?: GitRefusal }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushProbeError {
    /// Diagnostic naming what could not be established.
    pub message: String,
    /// Git's refusal when Git itself ended the dry run before it reported any destination;
    /// `None` for a failure of the
    ///  wrapper's own reading.
    pub refusal: Option<GitRefusal>,
    /// What the person can do about it,
    /// which the hint is chosen by.
    pub cause: ManualPushCause,
}

impl std::fmt::Display for ManualPushProbeError {
    /// Writes the diagnostic text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

impl std::error::Error for ManualPushProbeError {}

/// Build a probe error from text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ManualPushProbeError(cause, message)
/// ```
pub(crate) fn probe_error(cause: ManualPushCause, message: String) -> ManualPushProbeError {
    return ManualPushProbeError { message, refusal: None, cause };
}

/// What:
///  The cause of a Git query that could not produce an exit status.
/// Why:
///   A query that never started means the Git the wrapper runs is missing;
/// one stopped at its deadline already says in its
///  message which setting changes the deadline;
/// anything else Git did not explain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error instanceof StartError ? 'git-not-started' : error instanceof TimedOut ? 'self-explained' : 'git-silent'
/// ```
pub(crate) fn cause_of_query(error: &SingleFlightError) -> ManualPushCause {
    return match error {
        SingleFlightError::Start(_) => ManualPushCause::GitNotStarted,
        SingleFlightError::TimedOut { .. } | SingleFlightError::Ceiling { .. } => ManualPushCause::SelfExplained,
        SingleFlightError::Git { .. } | SingleFlightError::Io(_) => ManualPushCause::GitSilent,
    };
}
