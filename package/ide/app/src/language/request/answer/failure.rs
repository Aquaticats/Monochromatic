//! What a failed request means:
//!  the server's state moved under a request,
//!  or the request failed with a class to report.

/// The class reported to the interface.
use crate::language::reply::RequestFailure;

/// What:
///  How a failed request is treated.
///  An `enum` with data is a tagged union.
/// Why:
///  Codes `-32801` (content modified) and `-32800` (request cancelled) mean the server's
///      state moved under the request;
///  that is not a failure to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failure = { kind: 'superseded' } | { kind: 'failed'; failure: RequestFailure };
/// ```
pub(super) enum Failure {
    /// The server gave up because its own state changed.
    Superseded,
    /// A real failure,
    ///  with its class.
    Failed(
        /// The class reported to the interface.
        RequestFailure,
    ),
}

/// Name the class of a request error.
pub(super) fn classify_error(error: &helix_lsp::Error) -> Failure {
    // What: `match` on the borrowed error; `rpc.code.code()` reads the numeric protocol code.
    // Why: A protocol error, a timeout, and an ended server lead to different states.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (error instanceof RpcError) return [-32801, -32800].includes(error.code) ? superseded : failed(error);
    // ```
    return match error {
        helix_lsp::Error::Rpc(rpc) => {
            let code = rpc.code.code();
            if code == -32801 || code == -32800 {
                Failure::Superseded
            } else {
                Failure::Failed(RequestFailure::Rpc {
                    code,
                    // `clone` copies the server's message into the reply.
                    message: rpc.message.clone(),
                })
            }
        }
        helix_lsp::Error::Timeout(_) => Failure::Failed(RequestFailure::Timeout),
        helix_lsp::Error::StreamClosed => Failure::Failed(RequestFailure::StreamClosed),
        other => Failure::Failed(RequestFailure::Other(other.to_string())),
    };
}

/// What:
///  Whether a request error,
///  borrowed,
///  counts as the server's own answer;
///  `matches!` tests the error's variant.
/// Why:
///  A protocol error is still an answer;
///  silence and a closed stream are not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = (error?: Error) => !error || error instanceof RpcError;
/// ```
pub(super) fn answered(error: Option<&helix_lsp::Error>) -> bool {
    return match error {
        Some(found) => matches!(found, helix_lsp::Error::Rpc(_)),
        None => true,
    };
}
