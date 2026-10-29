//! The failures the worker keeps per server process,
//!  and how a failure is worded.

/// A failed request is described in the words the interface uses for it.
use crate::language::reply::RequestFailure;

/// What:
///  The last failure of each request the worker makes on its own,
///  per server process.
///  `Option<RequestFailure>` is "a failure,
///  or nothing";
///  `Default` builds the value with no failures.
/// Why:
///  A server that is running but answers hover or inlay hint requests with errors,
///  or never answers,
///  looks like a server without those features unless the failure is kept.
///  A later answer clears it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Trouble = { hints?: RequestFailure; hover?: RequestFailure };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::language) struct Trouble {
    /// The last inlay hint request that failed and was not asked again.
    pub(in crate::language) hints: Option<RequestFailure>,
    /// The last hover request that failed.
    pub(in crate::language) hover: Option<RequestFailure>,
}

/// What:
///  Describe a failed request in words that follow "answer" or "request".
///  `&RequestFailure` is a borrowed value.
/// Why:
///  The same failure is named the same way wherever it is shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failureText(failure: RequestFailure): string
/// ```
pub(super) fn failure_text(failure: &RequestFailure) -> String {
    return match failure {
        RequestFailure::Rpc { code, message } => {
            format!("the server answered with error {code}: {message}")
        }
        RequestFailure::Timeout => "the server did not answer in time".to_string(),
        RequestFailure::StreamClosed => "the server stopped while answering".to_string(),
        RequestFailure::Other(text) => text.clone(),
    };
}
