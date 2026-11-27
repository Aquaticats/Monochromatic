//! Sending the five feature requests:
//!  routing by feature,
//!  the readiness gate,
//!  and bookkeeping.
//!
//! Every request captures a ticket:
//!  the stamp,
//!  the text,
//!  the column unit,
//!  and the server
//! process it was sent to.
//!  The answer is converted against that ticket,
//!  never against later state.

/// Identities captured in every ticket.
use super::identity::{
    DocumentStamp,
    ServerIdentity,
};
/// The three position requests.
use super::reply::RequestKind;
/// Helix's character-indexed text buffer,
///  which a ticket keeps for the revision it asks about.
use helix_core::Rope;
/// The column unit of one server,
///  the protocol's data types,
///  and its wire-level types.
use helix_lsp::{
    OffsetEncoding,
    jsonrpc,
    lsp,
};
/// What:
///  `Future` is Rust's promise;
///  `Pin<Box<dyn Future>>` is an owned,
///  heap-stored promise
///       whose concrete type is erased;
///  `PathBuf` is an owned filesystem path;
///  `Duration` is a
///       time span.
/// Why:
///  Each helix-lsp request method returns a differently typed future;
///  erasing the type lets
///      one place await all five.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PayloadFuture = Promise<Payload>;
/// ```
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    time::Duration,
};

/// Classifying failures,
///  retrying,
///  and storing or replying.
mod answer;
/// Converting answers against the ticket's text.
mod convert;
/// The merged hint list the interface draws.
mod hints_snapshot;
/// Routing position requests and their retries.
mod route;
/// Sending one request to one server.
mod sending;
/// The requests the worker makes on its own.
mod worker_asks;
/// The worker loop hands finished requests to this function.
pub(super) use answer::finish;
/// The merged hints of every server, or nothing while none answered.
pub(super) use hints_snapshot::hints_snapshot;
/// The worker loop hands position requests and due retries to these functions.
pub(super) use route::{
    position,
    retry,
};
/// One request sent to one server,
///  which the position routing and the retries call.
pub(super) use sending::dispatch;
/// The requests the worker sends without a caller,
///  and the ones it keeps for the displayed text.
pub(super) use worker_asks::{
    catch_up,
    hint_ask,
    hints,
    pull_diagnostics,
    window,
};

/// What a ticket asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ask = { kind: 'position'; request: RequestKind }
///          | { kind: 'hints'; firstLine: number; lastLine: number } | { kind: 'pull' };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ask {
    /// Definition,
    ///  references,
    ///  or hover at a character offset.
    Position(
        /// Which of the three position requests.
        RequestKind,
    ),
    /// Inlay hints for a line range.
    Hints {
        /// First requested line.
        first_line: usize,
        /// Line the request ends before.
        last_line: usize,
    },
    /// Pull diagnostics for the document.
    Pull,
}

/// What:
///  Everything a late answer needs to be judged and converted,
///  captured when the request
///       is sent.
///  `u32` is an unsigned 32-bit count (siblings:
///  `u64`,
///  `usize`).
/// Why:
///  An answer must be converted against the text it was asked about,
///  and tagged with the
///      revision and server process it belongs to,
///  so the interface can drop it when stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ticket = { number: number; ask: Ask; position: number; stamp: DocumentStamp; text: Rope;
///                 path: string; encoding: Encoding; server: ServerIdentity; attempt: number };
/// ```
#[derive(Clone, Debug)]
pub(super) struct Ticket {
    /// Request number of a position request;
    ///  zero for requests the worker makes on its own.
    pub(super) number: u64,
    /// What was asked.
    pub(super) ask: Ask,
    /// Character offset of a position request.
    pub(super) position: usize,
    /// File generation and revision the request was sent for.
    pub(super) stamp: DocumentStamp,
    /// Text of that revision;
    ///  a rope clone shares its chunks.
    pub(super) text: Rope,
    /// Resolved path of the document,
    ///  to recognize targets inside it.
    pub(super) path: PathBuf,
    /// Column unit of the server that was asked.
    pub(super) encoding: OffsetEncoding,
    /// Server process that was asked.
    pub(super) server: ServerIdentity,
    /// How many times this request was already sent again after being superseded.
    pub(super) attempt: u32,
}

/// The typed result of one request,
///  as helix-lsp delivers it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Payload = { kind: 'definition'; result: Result<DefinitionResponse | null> } | /* ... */;
/// ```
pub(super) enum Payload {
    /// Answer to `textDocument/definition`.
    Definition(
        /// Locations,
        ///  `null`,
        ///  or the request error.
        helix_lsp::Result<Option<lsp::GotoDefinitionResponse>>,
    ),
    /// Answer to `textDocument/references`.
    References(
        /// Locations,
        ///  `null`,
        ///  or the request error.
        helix_lsp::Result<Option<Vec<lsp::Location>>>,
    ),
    /// Answer to `textDocument/hover`.
    Hover(
        /// Hover content,
        ///  `null`,
        ///  or the request error.
        helix_lsp::Result<Option<lsp::Hover>>,
    ),
    /// Answer to `textDocument/inlayHint`.
    Hints(
        /// Hints,
        ///  `null`,
        ///  or the request error.
        helix_lsp::Result<Option<Vec<lsp::InlayHint>>>,
    ),
    /// Answer to `textDocument/diagnostic`.
    Pull(
        /// A diagnostic report or the request error.
        helix_lsp::Result<lsp::DocumentDiagnosticReportResult>,
    ),
}

/// A finished request:
///  its ticket and what the server said.
pub(super) struct Answer {
    /// What was asked,
    ///  of whom,
    ///  about which text.
    pub(super) ticket: Ticket,
    /// The server's result or the failure.
    pub(super) payload: Payload,
}

/// The erased future every request is converted to before it is awaited.
type PayloadFuture = Pin<Box<dyn Future<Output = Payload> + Send>>;

/// A pending request as the worker loop awaits it:
///  it resolves to the answer with its ticket.
pub(super) type AnswerFuture = Pin<Box<dyn Future<Output = Answer> + Send>>;

/// What:
///  Store any future that yields a payload on the heap behind the erased type.
///  `F` is one
///       type parameter:
///  "some future that yields a `Payload`,
///  may move between threads,
///  and
///       borrows nothing".
/// Why:
///  Five differently typed futures then fit one variable and one place that awaits them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function erase(future: Promise<Payload>): Promise<Payload> { return future; }
/// ```
fn erase<F>(future: F) -> PayloadFuture
where
    F: Future<Output = Payload> + Send + 'static,
{
    return Box::pin(future);
}

/// What:
///  Await one helix-lsp request for at most `seconds`;
///  a request still unanswered then
///       fails as timed out.
///  `T` is one type parameter:
///  whatever the request yields.
///       `tokio::time::timeout` returns `Err` when the time passed first;
///  `jsonrpc::Id::Null`
///       stands for the request number,
///  which helix-lsp does not hand out.
/// Why:
///  helix-lsp is given the server's start allowance,
///  a multiple of the request timeout,
///  so
///      that a slow `initialize` is still accepted;
///  every other request keeps the configured
///      request timeout through this bound,
///  and fails exactly as a helix-lsp timeout does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function bounded<T>(seconds: number, request: Promise<T>): Promise<T> {
///   return Promise.race([request, sleep(seconds * 1000).then(() => { throw new TimeoutError(); })]);
/// }
/// ```
async fn bounded<T>(
    seconds: u64,
    request: impl Future<Output = helix_lsp::Result<T>>,
) -> helix_lsp::Result<T> {
    return match tokio::time::timeout(Duration::from_secs(seconds), request).await {
        Ok(result) => result,
        Err(elapsed) => {
            tracing::debug!(%elapsed, seconds, "a language server request timed out");
            Err(helix_lsp::Error::Timeout(jsonrpc::Id::Null))
        }
    };
}

/// What happened when a request was to be sent to one server.
pub(in crate::language) enum Sent {
    /// The request is on its way;
    ///  the worker loop awaits its answer.
    Spawned,
    /// The server has not finished `initialize`.
    Starting,
    /// The server could not follow the latest reload.
    Unsynchronized,
    /// The server does not offer the feature.
    Unsupported,
    /// The server is not asked:
    ///  no process,
    ///  feature excluded by configuration,
    ///  or no valid position.
    Skipped,
}
