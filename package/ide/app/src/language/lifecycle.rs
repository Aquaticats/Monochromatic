//! Document synchronization:
//!  `didOpen`,
//!  `didChange` with its fallbacks,
//!  `didSave`,
//!  and `didClose`.

/// The fixed fallback delay of the unversioned-diagnostics hold.
use super::diagnostics::HOLD_FALLBACK;
/// A reload renews what each server may be asked again for.
use super::owed::Owed;
/// Requests that follow every open and change.
use super::request;
/// The spelling Helix uses for paths below the project root.
use super::root::RootView;
/// Session records and the document as servers know it.
use super::session::OpenDocument;
/// States a server can reach through synchronization.
use super::status::{DocumentState, ServerState};
/// Commands from the interface thread.
use super::sync::{DocumentOpen, DocumentReload};
/// The worker whose state these steps change.
use super::worker::{Internal, Worker};
/// Starting and attaching servers is its own step;
///  reloads also reach servers that watch the file.
use super::{attach, forward};
/// A reload is a change of the displayed file.
use crate::change_watch::{ServerChange, ServerChangeKind};
/// What:
///  `compare_ropes` computes the edit list between two texts.
/// Why:
///  It is the fallback when a reload does not continue from the text servers were told about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compareRopes } from 'helix-core';
/// ```
use helix_core::diff::compare_ropes;
/// helix-lsp's client handle and the protocol's data types.
use helix_lsp::{Client, lsp};

/// What:
///  Build the address a document is announced under.
///  `Option<lsp::Url>` is "an address,
///       or nothing" for a path that cannot be expressed as a `file` address.
/// Why:
///  Below the project root the address uses Helix's spelling of the root,
///  so it agrees with
///      the root address Helix sends in `initialize`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function address(root: string, path: string): URL | undefined
/// ```
fn address(worker: &Worker, path: &std::path::Path) -> Option<lsp::Url> {
    // What: `match` unpacks the `Result`: a usable view respells the path, a refusal keeps it.
    // Why: With a wrong working directory no server starts anyway; the address is then unused.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spelled = view ? view.toHelix(path) : path;
    // ```
    let spelled = match RootView::discover(&worker.root) {
        Ok(view) => view.to_helix(path),
        Err(_) => path.to_path_buf(),
    };
    // `ok()` turns the `Result` into an `Option`, dropping the unit error value.
    return lsp::Url::from_file_path(spelled).ok();
}

/// What:
///  Display a file:
///  close the previous one,
///  record the new one,
///  and attach its servers.
///       `async fn` returns a future (a promise);
///  `&mut Worker` lends the worker for modification.
/// Why:
///  There is exactly one displayed document;
///  servers of other languages keep running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function open(worker: Worker, open: DocumentOpen): Promise<void>
/// ```
pub(super) async fn open(worker: &mut Worker, open: DocumentOpen) {
    close(worker);
    let Some(url) = address(worker, &open.path) else {
        tracing::warn!(path = %open.path.display(), "displayed file has no file address; language support is off for it");
        return;
    };
    worker.session.diagnostics.open(open.stamp, 0);
    // `Some(...)` is the "value present" variant of `Option`.
    worker.session.document = Some(OpenDocument {
        path: open.path,
        url,
        text: open.text,
        stamp: open.stamp,
        version: 0,
        config: None,
        // `String::new()` creates empty owned text; the language step fills it in.
        language_id: String::new(),
        state: DocumentState::NoLanguage,
        window: None,
    });
    attach::attach(worker).await;
}

/// What:
///  Send `didOpen` to one server if it is initialized,
///  serves the document,
///  and has not
///       received it yet;
///  then ask it for diagnostics and hints.
/// Why:
///  helix-lsp silently drops notifications sent before initialization,
///  so this runs both
///      when a file is displayed on a ready server and when a server reports `initialized`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sendDidOpen(worker: Worker, index: number): void
/// ```
pub(super) fn send_did_open(worker: &mut Worker, index: usize) {
    // `as_ref()` borrows the document inside the `Option` without moving it.
    let Some(document) = worker.session.document.as_ref() else {
        return;
    };
    let record = &mut worker.session.servers[index];
    if !record.attached || record.opened {
        return;
    }
    // The gate: nothing is sent to a client whose `initialize` has not completed.
    let Some(client) = record.ready() else {
        return;
    };
    client.text_document_did_open(
        // `clone` copies the address and identifier because helix-lsp takes them by value.
        document.url.clone(),
        document.version,
        &document.text,
        document.language_id.clone(),
    );
    tracing::debug!(server = %record.identity.name, version = document.version, "sent didOpen");
    record.opened = true;
    request::pull_diagnostics(worker, index, 0);
    request::hints(worker, index, 0);
}

/// True when the server's capabilities say it takes `didOpen` and `didClose`.
fn accepts_open_close(client: &Client) -> bool {
    // What: `match` on a borrowed `Option` of a tagged union; `*kind` reads the borrowed number.
    // Why: The object form states it directly; the bare number form implies it unless it is "none".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return typeof sync === 'number' ? sync !== 0 : sync?.openClose === true;
    // ```
    return match &client.capabilities().text_document_sync {
        Some(lsp::TextDocumentSyncCapability::Options(options)) => options.open_close == Some(true),
        Some(lsp::TextDocumentSyncCapability::Kind(kind)) => {
            *kind != lsp::TextDocumentSyncKind::NONE
        }
        None => false,
    };
}

/// What:
///  Tell one server about the new text;
///  returns false when it cannot be synchronized.
/// Why:
///  `text_document_did_change` picks full text or incremental ranges from the negotiated
///      kind and returns nothing when the server takes no changes.
///  Such a server is closed and
///      reopened with the new text if it accepts that;
///  otherwise it is out of sync for this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function synchronize(client: Client, document: OpenDocument, reload: DocumentReload): boolean
/// ```
fn synchronize(client: &Client, document: &OpenDocument, reload: &DocumentReload) -> bool {
    let versioned =
        lsp::VersionedTextDocumentIdentifier::new(document.url.clone(), document.version);
    let sent = client.text_document_did_change(
        versioned,
        &reload.previous,
        &document.text,
        &reload.changes,
    );
    let identifier = lsp::TextDocumentIdentifier::new(document.url.clone());
    if sent.is_none() {
        if !accepts_open_close(client) {
            return false;
        }
        client.text_document_did_close(identifier.clone());
        client.text_document_did_open(
            document.url.clone(),
            document.version,
            &document.text,
            document.language_id.clone(),
        );
        tracing::debug!(
            server = client.name(),
            "resynchronized by didClose and didOpen"
        );
    }
    // After an external reload the text on disk equals the new text, so `didSave` is truthful.
    // helix-lsp sends it only when the server asked for save notifications.
    if client
        .text_document_did_save(identifier, &document.text)
        .is_some()
    {
        tracing::debug!(
            server = client.name(),
            "sent didSave after an external reload"
        );
    }
    return true;
}

/// What:
///  Apply an accepted external reload:
///  advance the document,
///  synchronize every server that
///       holds it,
///  and ask again for diagnostics and hints.
/// Why:
///  Results for the previous revision are invalid from this moment on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reload(worker: Worker, reload: DocumentReload): void
/// ```
pub(super) fn reload(worker: &mut Worker, mut reload: DocumentReload) {
    // `as_mut()` borrows the document inside the `Option` for modification.
    let Some(document) = worker.session.document.as_mut() else {
        tracing::debug!("ignored a reload while no file is displayed");
        return;
    };
    if reload.file != document.stamp.file || reload.revision == document.stamp.revision {
        tracing::debug!(
            file = reload.file,
            revision = reload.revision,
            "ignored a reload for another file or the displayed revision"
        );
        return;
    }
    if reload.base_revision != document.stamp.revision {
        // A reload was missed (for example a full command queue): diff against what servers hold.
        tracing::warn!(
            base = reload.base_revision,
            held = document.stamp.revision,
            "reload does not continue the synchronized text; recomputing its changes"
        );
        let transaction = compare_ropes(&document.text, &reload.text);
        reload.previous = document.text.clone();
        reload.changes = transaction.changes().clone();
    }
    document.text = reload.text.clone();
    document.stamp.revision = reload.revision;
    document.version += 1;
    let stamp = document.stamp;
    let version = document.version;
    let path = document.path.clone();
    let mut holders = Vec::new();
    let mut synchronized = Vec::new();
    // Index loop: the body changes one record while reading the document.
    for index in 0..worker.session.servers.len() {
        let record = &mut worker.session.servers[index];
        // `Owed::default()` builds the empty record: the new text is asked about afresh, and
        // answers owed for the previous text are no longer wanted.
        record.owed = Owed::default();
        // A server still starting receives the current text through its eventual `didOpen`.
        if !record.attached || !record.opened || record.state == ServerState::Unsynchronized {
            continue;
        }
        let Some(client) = record.ready() else {
            continue;
        };
        let Some(current) = worker.session.document.as_ref() else {
            continue;
        };
        if synchronize(client, current, &reload) {
            holders.push(record.identity.clone());
            synchronized.push(index);
        } else {
            tracing::warn!(server = %record.identity.name, "server takes no changes and no reopen; it is out of sync for this file");
            record.state = ServerState::Unsynchronized;
        }
    }
    let serial = worker.session.diagnostics.reload(stamp, version, &holders);
    worker.timer(HOLD_FALLBACK, Internal::HoldExpired(serial));
    worker.session.hints.clear();
    worker.session.pending.clear();
    // Servers that registered a watcher for the displayed file hear about the reload too, even when its
    // folder is not watched for them.
    forward::receive(
        worker,
        ServerChange {
            path,
            kind: ServerChangeKind::Changed,
        },
    );
    for index in synchronized {
        request::pull_diagnostics(worker, index, 0);
        request::hints(worker, index, 0);
    }
}

/// What:
///  Stop displaying the document:
///  `didClose` to every server that received `didOpen`.
/// Why:
///  Servers stay running for the next file of their language;
///  everything derived from the
///      closed file is cleared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function close(worker: Worker): void
/// ```
pub(super) fn close(worker: &mut Worker) {
    // `take()` moves the document out of the `Option`, leaving "nothing" behind.
    let Some(document) = worker.session.document.take() else {
        return;
    };
    for record in &worker.session.servers {
        if !record.opened {
            continue;
        }
        if let Some(client) = record.ready() {
            client.text_document_did_close(lsp::TextDocumentIdentifier::new(document.url.clone()));
            tracing::debug!(server = %record.identity.name, "sent didClose");
        }
    }
    worker.session.detach();
    worker.session.diagnostics.close();
}
