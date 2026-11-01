//! Server-to-client traffic:
//!  what `helix-term` does around helix-lsp in the terminal editor.
//!
//! helix-lsp frames,
//!  matches replies,
//!  and delivers;
//!  everything a server sends on its own
//! arrives here and is answered,
//!  stored,
//!  logged,
//!  or ignored.

/// Server lifecycle steps triggered by synthetic notifications.
use super::attach;
/// The reply policy for server requests.
use super::incoming::{ClientView, Effect, decide};
/// Diagnostics are pulled again when a server asks for a refresh.
use super::request;
/// Configured feature exclusions apply to pushed diagnostics too.
use super::session::allowed;
/// Servers that could not follow a reload are not listened to.
use super::status::ServerState;
/// The target validator resolves the address a server names.
use super::target::{Classified, classify};
/// The worker whose state these steps change.
use super::worker::Worker;
/// The feature name pushed diagnostics are filed under in Helix's configuration.
use helix_core::syntax::config::LanguageServerFeature;
/// What:
///  `Call` is one server-to-client message;
///  `Notification` is helix-lsp's parsed form of
///       the notifications it models;
///  `jsonrpc` holds the wire-level types;
///  `lsp` the protocol's
///       data types.
/// Why:
///  Messages arrive unparsed so the embedder decides what each means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Call, Notification, jsonrpc, lsp } from 'helix-lsp';
/// ```
use helix_lsp::{Call, Client, LanguageServerId, Notification, jsonrpc, lsp};
/// What:
///  `Arc` is a thread-safe shared pointer (siblings:
///  `Rc`,
///  `Box`).
/// Why:
///  File-watcher registrations keep a weak pointer made from the shared client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = T;
/// ```
use std::sync::Arc;

/// Longest server message copied into the log,
///  in characters.
const LOGGED_MESSAGE_CHARS: usize = 300;

/// True for helix-lsp's synthetic notification that a server process ended.
pub(super) fn is_exit(call: &Call) -> bool {
    // What: `matches!` is a macro that is true when the value fits the pattern, including the
    //       `if` condition after it.
    // Why: Shutdown counts ended processes without parsing other traffic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return call.kind === 'notification' && call.method === 'exit';
    // ```
    return matches!(call, Call::Notification(notification) if notification.method == "exit");
}

/// Shorten a server message for the log.
fn shortened(message: &str) -> String {
    // `chars().take(n).collect()` keeps at most `n` characters in a new owned `String`.
    return message.chars().take(LOGGED_MESSAGE_CHARS).collect();
}

/// What:
///  Answer one request a server sent,
///  then apply its side effect.
/// Why:
///  The decision is made by the pure policy;
///  this function only supplies the client's
///      settings and folders and carries out what the policy asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function onRequest(worker, client, server, method, params, id): Promise<void>
/// ```
async fn on_request(
    worker: &mut Worker,
    client: &Arc<Client>,
    server: LanguageServerId,
    call: jsonrpc::MethodCall,
) {
    // `clone` copies the folder list so helix-lsp's lock is released before the reply is built.
    let folders = client.workspace_folders().await.clone();
    let view = ClientView {
        settings: client.config(),
        folders: &folders,
    };
    let decision = decide(&call.method, call.params, &view);
    tracing::debug!(server = client.name(), method = %call.method, policy = decision.policy, "answered a server request");
    // What: `Client::reply` queues the response and returns `Result`.
    // Why: A failed send means the server is gone; that is logged, not ignored.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { client.reply(id, reply); } catch (error) { log.warn(error); }
    // ```
    if let Err(error) = client.reply(call.id, decision.reply) {
        tracing::warn!(server = client.name(), %error, "cannot answer a server request");
    }
    match decision.effect {
        Effect::None => {}
        Effect::RefreshDiagnostics => {
            if let Some(index) = worker.session.index_of(server) {
                request::pull_diagnostics(worker, index, 0);
            }
        }
        // The registrations are kept here, not in helix-lsp's handler, which drops relative patterns
        // and kinds (`watched_files.rs`).
        Effect::RegisterWatchers(watchers) => {
            for (identifier, options) in watchers {
                worker.watched.register(server, identifier, options);
            }
        }
        Effect::UnregisterWatchers(identifiers) => {
            for identifier in identifiers {
                worker.watched.unregister(server, &identifier);
            }
        }
    }
    return;
}

/// What:
///  Judge and store diagnostics a server pushed.
/// Why:
///  Only the displayed file's diagnostics are kept,
///  compared by resolved path because
///      servers spell the same path differently,
///  and only from a server that holds the
///      document open and in sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onPublished(worker: Worker, server: ServerId, published: PublishDiagnosticsParams): void
/// ```
fn on_published(
    worker: &mut Worker,
    server: LanguageServerId,
    published: lsp::PublishDiagnosticsParams,
) {
    let Some(index) = worker.session.index_of(server) else {
        return;
    };
    let Some(document) = worker.session.document.as_ref() else {
        return;
    };
    let record = &worker.session.servers[index];
    if !record.opened || record.state == ServerState::Unsynchronized {
        return;
    }
    let name = record.identity.name.as_str();
    if !allowed(
        document.config.as_ref(),
        name,
        LanguageServerFeature::Diagnostics,
    ) {
        return;
    }
    let Some(client) = record.ready() else {
        return;
    };
    // An equal address is the common case; otherwise the address is resolved like any target.
    let displayed = published.uri == document.url
        || match classify(&published.uri, &worker.root) {
            Classified::InsideProject(path) | Classified::OutsideProject(path) => {
                path == document.path
            }
            Classified::Refused(_) => false,
        };
    if !displayed {
        return;
    }
    let verdict = worker.session.diagnostics.push(
        &record.identity,
        client.offset_encoding(),
        published.version,
        published.diagnostics,
        document.text.len_lines(),
    );
    tracing::debug!(server = name, ?verdict, version = ?published.version, "judged pushed diagnostics");
}

/// What:
///  Track which work a server reports as running.
///  `&mut Worker` lends the worker for modification.
/// Why:
///  rust-analyzer sends hundreds of progress notifications during one start;
///  only the begin
///      and end of each token change what the status shows,
///  so reports are dropped here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onProgress(worker: Worker, server: ServerId, progress: ProgressParams): void
/// ```
fn on_progress(worker: &mut Worker, server: LanguageServerId, progress: lsp::ProgressParams) {
    let Some(index) = worker.session.index_of(server) else {
        return;
    };
    let record = &mut worker.session.servers[index];
    let lsp::ProgressParamsValue::WorkDone(work) = progress.value;
    match work {
        lsp::WorkDoneProgress::Begin(begin) => {
            let running = std::mem::take(&mut record.progress);
            for entry in running {
                if entry.0 != progress.token {
                    record.progress.push(entry);
                }
            }
            record.progress.push((progress.token, begin.title));
        }
        lsp::WorkDoneProgress::End(_) => {
            let running = std::mem::take(&mut record.progress);
            for entry in running {
                if entry.0 != progress.token {
                    record.progress.push(entry);
                }
            }
        }
        lsp::WorkDoneProgress::Report(_) => {}
    }
}

/// Handle one notification,
///  including the two helix-lsp injects itself.
fn on_notification(
    worker: &mut Worker,
    client: &Arc<Client>,
    server: LanguageServerId,
    notification: jsonrpc::Notification,
) {
    // What: `Notification::parse` returns `Result`: a typed notification or an error for methods
    //       helix-lsp does not model and for malformed parameters.
    // Why: Unknown notifications are ignored; they need no reply.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let parsed; try { parsed = Notification.parse(method, params); } catch { return; }
    // ```
    let parsed = match Notification::parse(&notification.method, notification.params) {
        Ok(parsed) => parsed,
        Err(error) => {
            tracing::debug!(server = client.name(), method = %notification.method, %error, "ignored a server notification");
            return;
        }
    };
    match parsed {
        Notification::Initialized => attach::initialized(worker, server),
        Notification::Exit => attach::exited(worker, server),
        Notification::PublishDiagnostics(published) => on_published(worker, server, published),
        Notification::ShowMessage(message) => {
            tracing::info!(server = client.name(), message = %shortened(&message.message), "language server message");
        }
        Notification::LogMessage(message) => {
            tracing::debug!(server = client.name(), message = %shortened(&message.message), "language server log");
        }
        Notification::ProgressMessage(progress) => on_progress(worker, server, progress),
    }
}

/// What:
///  Handle one message from the merged `Registry::incoming` stream.
/// Why:
///  Every request gets a reply and every lifecycle signal updates the session;
///  a message
///      from a server that was already removed is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function onCall(worker: Worker, server: ServerId, call: Call): Promise<void>
/// ```
pub(super) async fn on_call(worker: &mut Worker, server: LanguageServerId, call: Call) {
    // What: `get_by_id` returns `Option<&Arc<Client>>`; `cloned()` copies the shared pointer.
    // Why: An owned pointer does not keep the registry borrowed while the session is changed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const client = registry.getById(server); if (!client) return;
    // ```
    let Some(client) = worker.registry.get_by_id(server).cloned() else {
        tracing::debug!("dropped a message from a removed language server");
        return;
    };
    match call {
        Call::Notification(notification) => {
            on_notification(worker, &client, server, notification);
        }
        Call::MethodCall(request) => {
            on_request(worker, &client, server, request).await;
        }
        Call::Invalid { id } => {
            tracing::warn!(
                server = client.name(),
                ?id,
                "language server sent an invalid message"
            );
        }
    }
    // What: `index_of` finds the record of the server that sent the message, if it still has one.
    // Why: A message of any kind shows that the server processes its input again. Hints and
    //      pull diagnostics it left unanswered through their retries are asked for once more
    //      now, which `request::catch_up` bounds per displayed text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = worker.session.indexOf(server); if (index !== undefined) catchUp(worker, index);
    // ```
    if let Some(index) = worker.session.index_of(server) {
        request::catch_up(worker, index);
    }
    return;
}
