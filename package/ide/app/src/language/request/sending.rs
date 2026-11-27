//! Sending one request to one server:
//!  the configuration feature it is routed by,
//!  the readiness gate,
//!  and the one place that names helix-lsp's request methods.

/// Offsets become server positions only through the shared converter.
use super::super::position::{
    to_lsp_position,
    to_lsp_range,
};
/// The three position requests this routes.
use super::super::reply::RequestKind;
/// The session's servers and the feature exclusions it carries.
use super::super::session::allowed;
/// Servers that could not follow a reload are not asked about positions.
use super::super::status::ServerState;
/// The worker whose state these steps change.
use super::super::worker::Worker;
/// The answer,
/// the ticket,
/// and the futures they travel in.
use super::{
    Answer,
    Ask,
    Payload,
    PayloadFuture,
    Sent,
    Ticket,
    bounded,
    erase,
};
/// The feature a language's configuration can restrict per server.
use helix_core::syntax::config::LanguageServerFeature;
/// The protocol's document identifier.
use helix_lsp::lsp;

/// The configuration feature a request is routed by.
fn feature(ask: Ask) -> LanguageServerFeature {
    return match ask {
        Ask::Position(RequestKind::Definition) => LanguageServerFeature::GotoDefinition,
        Ask::Position(RequestKind::References) => LanguageServerFeature::GotoReference,
        Ask::Position(RequestKind::Hover) => LanguageServerFeature::Hover,
        Ask::Hints {
            ..
        } => LanguageServerFeature::InlayHints,
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
pub(in crate::language) fn dispatch(
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
        return Answer {
            ticket,
            payload,
        };
    }));
    return Sent::Spawned;
}
