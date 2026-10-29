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

/// Hint windows and the latest-value snapshot.
use super::hints::{HintWindow, HintsSnapshot, InlayHint, request_lines};
/// Identities captured in every ticket.
use super::identity::{DocumentStamp, ServerIdentity};
/// Offsets become server positions only through the shared converter.
use super::position::{to_lsp_position, to_lsp_range};
/// The three position requests.
use super::reply::RequestKind;
/// Session state and configured feature exclusions.
use super::session::{Session, allowed};
/// Servers that could not follow a reload are not asked about positions.
use super::status::ServerState;
/// The worker whose state these steps change.
use super::worker::Worker;
/// What:
///  `Rope` is Helix's character-indexed text buffer;
///  `LanguageServerFeature` names the
///       features a language's configuration can restrict per server.
/// Why:
///  A ticket keeps the text of the revision it asks about;
///  routing honors the configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope, LanguageServerFeature } from 'helix-core';
/// ```
use helix_core::{Rope, syntax::config::LanguageServerFeature};
/// The column unit of one server,
///  the protocol's data types,
///  and its wire-level types.
use helix_lsp::{OffsetEncoding, jsonrpc, lsp};
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
use std::{future::Future, path::PathBuf, pin::Pin, time::Duration};

/// Classifying failures,
///  retrying,
///  and storing or replying.
mod answer;
/// Converting answers against the ticket's text.
mod convert;
/// Routing position requests and their retries.
mod route;
/// The worker loop hands finished requests to this function.
pub(super) use answer::finish;
/// The worker loop hands position requests and due retries to these functions.
pub(super) use route::{position, retry};

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
enum Sent {
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

/// The configuration feature a request is routed by.
fn feature(ask: Ask) -> LanguageServerFeature {
    return match ask {
        Ask::Position(RequestKind::Definition) => LanguageServerFeature::GotoDefinition,
        Ask::Position(RequestKind::References) => LanguageServerFeature::GotoReference,
        Ask::Position(RequestKind::Hover) => LanguageServerFeature::Hover,
        Ask::Hints { .. } => LanguageServerFeature::InlayHints,
        Ask::Pull => LanguageServerFeature::PullDiagnostics,
    };
}

/// What:
///  Send one request to one server,
///  or say why it was not sent.
///  `index` is the server's
///       record;
///  `number`,
///  `position`,
///  and `attempt` go into the ticket.
/// Why:
///  This is the single place where the application calls helix-lsp's request methods.
///  Each
///      of them panics on a client that has not initialized,
///  so the readiness gate comes first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dispatch(worker, index, number, ask, position, attempt): Sent
/// ```
fn dispatch(
    worker: &mut Worker,
    index: usize,
    number: u64,
    ask: Ask,
    position: usize,
    attempt: u32,
) -> Sent {
    let Some(document) = worker.session.document.as_ref() else {
        return Sent::Skipped;
    };
    let record = &worker.session.servers[index];
    let name = record.identity.name.as_str();
    if record.client.is_none() || !allowed(document.config.as_ref(), name, feature(ask)) {
        return Sent::Skipped;
    }
    // The gate: `ready` is `None` until `initialize` completed.
    let Some(client) = record.ready() else {
        return Sent::Starting;
    };
    if record.state == ServerState::Unsynchronized {
        return Sent::Unsynchronized;
    }
    if !record.opened {
        return Sent::Starting;
    }
    let encoding = client.offset_encoding();
    // helix-lsp only applies the longer start allowance; the request timeout is applied here.
    let seconds = worker.languages.timeout(name);
    let identifier = lsp::TextDocumentIdentifier::new(document.url.clone());
    // What: Each arm asks helix-lsp for its typed future. The methods return `None` when the
    //       server did not announce the capability; `erase(async move { ... })` wraps the typed
    //       future so its payload variant records which request it answers.
    // Why: `None` is the observable form of "unsupported capability" for a running server.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending: Promise<Payload> | undefined = client.hover(identifier, at)?.then(wrapHover);
    // ```
    let pending: Option<PayloadFuture> = match ask {
        Ask::Position(kind) => {
            let Some(at) = to_lsp_position(&document.text, position, encoding) else {
                return Sent::Skipped;
            };
            match kind {
                RequestKind::Definition => match client.goto_definition(identifier, at, None) {
                    Some(request) => Some(erase(async move {
                        return Payload::Definition(bounded(seconds, request).await);
                    })),
                    None => None,
                },
                RequestKind::References => {
                    match client.goto_reference(identifier, at, true, None) {
                        Some(request) => Some(erase(async move {
                            return Payload::References(bounded(seconds, request).await);
                        })),
                        None => None,
                    }
                }
                RequestKind::Hover => match client.text_document_hover(identifier, at, None) {
                    Some(request) => Some(erase(async move {
                        return Payload::Hover(bounded(seconds, request).await);
                    })),
                    None => None,
                },
            }
        }
        Ask::Hints {
            first_line,
            last_line,
        } => {
            // `line_to_char` gives the offset of a line's first character; the line count itself is valid.
            let start = document.text.line_to_char(first_line);
            let end = document.text.line_to_char(last_line);
            let Some(range) = to_lsp_range(&document.text, start, end, encoding) else {
                return Sent::Skipped;
            };
            match client.text_document_range_inlay_hints(identifier, range, None) {
                Some(request) => Some(erase(async move {
                    return Payload::Hints(bounded(seconds, request).await);
                })),
                None => None,
            }
        }
        Ask::Pull => {
            let previous = worker
                .session
                .diagnostics
                .previous_result_id(&record.identity);
            match client.text_document_diagnostic(identifier, previous) {
                Some(request) => Some(erase(async move {
                    return Payload::Pull(bounded(seconds, request).await);
                })),
                None => None,
            }
        }
    };
    let Some(future) = pending else {
        return Sent::Unsupported;
    };
    let ticket = Ticket {
        number,
        ask,
        position,
        stamp: document.stamp,
        // `clone` on a rope copies a small handle; the ticket keeps this revision's text alive.
        text: document.text.clone(),
        path: document.path.clone(),
        encoding,
        server: record.identity.clone(),
        attempt,
    };
    // What: `Box::pin(async move { ... })` builds a heap-stored future that owns the ticket and
    //       resolves to the answer; `push` adds it to the set the worker loop awaits.
    // Why: The loop keeps answering server requests while this answer is pending, and it sees
    //      the answer before any notification the server sent after it. No helper task and no
    //      worker state are involved until the loop takes the finished answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // worker.requests.add(pending.then(payload => ({ ticket, payload })));
    // ```
    worker.requests.push(Box::pin(async move {
        let payload = future.await;
        return Answer { ticket, payload };
    }));
    return Sent::Spawned;
}

/// What:
///  Ask one server for diagnostics of the displayed document,
///  if it offers pull diagnostics.
///       `matches!` is true when the outcome is the "request is on its way" variant.
/// Why:
///  The outcome needs no reply:
///  an unsupported or unready server simply contributes nothing.
///      Once a request is on its way,
///  the server owes nothing until that request ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pullDiagnostics(worker: Worker, index: number, attempt: number): void {
///   if (dispatch(worker, index, 0, { kind: 'pull' }, 0, attempt) === 'spawned') worker.session.servers[index].owed.pull = false;
/// }
/// ```
pub(super) fn pull_diagnostics(worker: &mut Worker, index: usize, attempt: u32) {
    if matches!(
        dispatch(worker, index, 0, Ask::Pull, 0, attempt),
        Sent::Spawned
    ) {
        worker.session.servers[index].owed.pull = false;
    }
}

/// What:
///  The hint request for the last reported window against the displayed text,
///  or nothing
///       when no file is displayed or no window was reported.
///  `&Session` lends the session
///       read-only;
///  `Option<Ask>` is "a request,
///  or nothing".
/// Why:
///  First asks,
///  retries,
///  and catch-up asks all name the lines wanted now,
///  never the lines an
///      earlier request named,
///  so a request sent again after a wait cannot bring back hints for
///      lines the reader has scrolled away from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hintAsk(session: Session): Ask | undefined {
///   const window = session.document?.window;
///   if (!window) return undefined;
///   const [firstLine, lastLine] = requestLines(window, session.document.text.lineCount);
///   return { kind: 'hints', firstLine, lastLine };
/// }
/// ```
pub(super) fn hint_ask(session: &Session) -> Option<Ask> {
    // The trailing `?` returns `None` when no document is displayed, or no window was reported.
    let document = session.document.as_ref()?;
    let window = document.window?;
    let (first_line, last_line) = request_lines(window, document.text.len_lines());
    return Some(Ask::Hints {
        first_line,
        last_line,
    });
}

/// Ask one server for hints of the last reported window,
///  if any window was reported.
pub(super) fn hints(worker: &mut Worker, index: usize, attempt: u32) {
    let Some(ask) = hint_ask(&worker.session) else {
        return;
    };
    // A request on its way means the server owes nothing until that request ends.
    if matches!(dispatch(worker, index, 0, ask, 0, attempt), Sent::Spawned) {
        worker.session.servers[index].owed.hints = false;
    }
}

/// What:
///  The server at `index` just sent something:
///  ask it again for the hints and pull
///       diagnostics it still owes for the displayed text,
///  if the catch-up rule allows.
/// Why:
///  A request that stayed unanswered through its retries is not repeated on a timer.
///  A
///      message of any kind shows that the server processes its input again,
///  which is the
///      moment a new request can succeed.
///  `Owed::catch_up` bounds how often this happens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function catchUp(worker: Worker, index: number): void {
///   const owed = worker.session.servers[index].owed.catchUp();
///   if (!owed) return;
///   if (owed.pull) pullDiagnostics(worker, index, 0);
///   if (owed.hints) hints(worker, index, 0);
/// }
/// ```
pub(super) fn catch_up(worker: &mut Worker, index: usize) {
    let Some(owed) = worker.session.servers[index].owed.catch_up() else {
        return;
    };
    tracing::debug!(server = %worker.session.servers[index].identity.name, ?owed, "the server sent a message while it owed answers; asking again");
    if owed.pull {
        pull_diagnostics(worker, index, 0);
    }
    if owed.hints {
        hints(worker, index, 0);
    }
}

/// What:
///  The interface reported the visible lines:
///  remember them and ask every server for hints.
/// Why:
///  The window is kept so hints can be asked for again after a reload and after a late start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function window(worker: Worker, stamp: DocumentStamp, window: HintWindow): void
/// ```
pub(super) fn window(worker: &mut Worker, stamp: DocumentStamp, window: HintWindow) {
    let Some(document) = worker.session.document.as_mut() else {
        return;
    };
    if document.stamp != stamp {
        tracing::debug!(
            ?stamp,
            "dropped a hint window for text that is no longer displayed"
        );
        return;
    }
    document.window = Some(window);
    for index in 0..worker.session.servers.len() {
        if worker.session.servers[index].attached {
            hints(worker, index, 0);
        }
    }
}

/// Orders inlay hints by the character offset they are drawn before.
fn by_position(left: &InlayHint, right: &InlayHint) -> std::cmp::Ordering {
    return left.position.cmp(&right.position);
}

/// What:
///  Merge the stored hints of every server into the latest-value snapshot,
///  or nothing when
///       no server has answered for the displayed text.
/// Why:
///  The interface draws one ordered list regardless of how many servers contributed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hintsSnapshot(session: Session): HintsSnapshot | undefined
/// ```
pub(super) fn hints_snapshot(session: &Session) -> Option<HintsSnapshot> {
    // The trailing `?` returns `None` when no document is displayed.
    let document = session.document.as_ref()?;
    if session.hints.is_empty() {
        return None;
    }
    let mut merged: Vec<InlayHint> = Vec::new();
    for (_, hints) in &session.hints {
        // `extend` appends copies of one server's hints.
        merged.extend(hints.iter().cloned());
    }
    // `sort_by` is stable: equal positions keep server order.
    merged.sort_by(by_position);
    return Some(HintsSnapshot {
        stamp: document.stamp,
        first_line: session.hint_lines.0,
        last_line: session.hint_lines.1,
        hints: merged,
    });
}
