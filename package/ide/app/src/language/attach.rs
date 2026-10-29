//! Server processes:
//!  on-demand start,
//!  the start deadline,
//!  readiness,
//!  exit,
//!  and removal.

/// What:
///  `use` brings names from other modules into this file under their short names.
/// Why:
///  Results of an ended server are removed by its identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type ServerIdentity } from './identity';
/// ```
use super::identity::ServerIdentity;
/// `didOpen` follows readiness.
use super::lifecycle::send_did_open;
/// States reported to the interface thread.
use super::status::{DocumentState, ServerState};
/// The worker's messages to itself,
///  one of which carries the last standard-error lines of a server whose start failed.
use super::worker::Internal;
/// The worker whose state these steps change.
use super::worker::Worker;
/// helix-lsp's key for one server process.
use helix_lsp::LanguageServerId;

/// On-demand start of a language's servers,
///  after the program,
///  the root,
///  and the launch were checked.
mod start;

/// Records of a server that ended unexpectedly or did not finish starting,
///  with its last
/// standard-error lines.
mod report;

/// What:
///  Remove one server process from helix-lsp's registry and record why.
///  `usize` is the
///       record's index in the session;
///  `stop` says whether the process may still be running.
/// Why:
///  helix-lsp never removes a client by itself.
///  `remove_by_id` plus `force_shutdown` is used
///      instead of `Registry::stop`,
///  because `stop` leaves a marker that prevents every later
///      start of that server name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retire(worker: Worker, index: number, state: ServerState, stop: boolean): void
/// ```
fn retire(worker: &mut Worker, index: usize, state: ServerState, stop: bool) {
    let record = &mut worker.session.servers[index];
    // `take()` moves the client out of the record, leaving "nothing" behind.
    if let Some(client) = record.client.take() {
        if stop {
            // Sends `shutdown` and `exit` without waiting; dropping the client then kills what is left.
            client.force_shutdown();
        }
        worker.registry.remove_by_id(client.id());
    }
    record.state = state;
    record.opened = false;
    record.progress.clear();
    // `clone` copies the identity so the record borrow ends before the stores are changed.
    let identity = record.identity.clone();
    forget_results(worker, &identity);
}

/// Remove everything a server process contributed to the displayed document.
fn forget_results(worker: &mut Worker, identity: &ServerIdentity) {
    worker.session.diagnostics.server_exited(identity);
    // The entries of other servers are kept, in their order.
    let held = std::mem::take(&mut worker.session.hints);
    for entry in held {
        if entry.0 != *identity {
            worker.session.hints.push(entry);
        }
    }
}

/// What:
///  The receiver that tells the worker why a server's start failed,
///  with the server's last standard-error lines.
///  It owns the channel to the worker and the identity of the server record.
/// Why:
///  The lines arrive after the record waited for the end of the server's standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const afterEnd = (lines: string[]) => internal.send({ kind: 'endCause', identity, lines });
/// ```
struct SendEndCause {
    /// The worker's own message channel.
    internal: tokio::sync::mpsc::UnboundedSender<Internal>,
    /// The record of the server whose start failed.
    identity: ServerIdentity,
}

/// The lines go to the worker as an `EndCause` message.
impl report::AfterEnd for SendEndCause {
    /// Send the cause;
    ///  a worker that stopped meanwhile needs none.
    fn after_end(self, lines: Vec<String>) {
        if let Err(error) = self.internal.send(Internal::EndCause(self.identity, lines)) {
            tracing::debug!(%error, "language worker stopped before the cause of a failed start arrived");
        }
    }
}

/// What:
///  Attach already running servers to a document outside the project;
///  nothing is started.
/// Why:
///  A dependency or standard-library file opened from a definition belongs to no project
///      root of its own;
///  only the servers the project already runs are asked about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attachRunning(worker: Worker, names: string[]): void
/// ```
fn attach_running(worker: &mut Worker, names: &[String]) {
    for index in 0..worker.session.servers.len() {
        let record = &mut worker.session.servers[index];
        if record.client.is_none() || !names.contains(&record.identity.name) {
            continue;
        }
        record.attached = true;
        send_did_open(worker, index);
    }
}

/// What:
///  Resolve the displayed document's language and attach its servers.
/// Why:
///  This is the single path by which a server comes to serve a document,
///  used when a file
///      is displayed and when an exited server is started again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attach(worker: Worker): Promise<void>
/// ```
pub(super) async fn attach(worker: &mut Worker) {
    let Some(shown) = worker.session.document.as_ref() else {
        return;
    };
    let path = shown.path.clone();
    // The inner block ends the registry guard's borrow before the worker is changed again.
    let recognized = {
        let loader = worker.languages.loader.load();
        let mut language_id = None;
        if let Some(found) = loader.language_for_filename(&path) {
            language_id = Some(loader.language(found).config().language_id.clone());
        }
        language_id
    };
    let Some(language) = recognized else {
        return;
    };
    if let Err(error) = worker.languages.refresh(&language, Some(&path)) {
        tracing::error!(
            ?error,
            "cannot rebuild the language registry; keeping the previous one"
        );
    }
    let configured = {
        let loader = worker.languages.loader.load();
        let mut language_config = None;
        if let Some(found) = loader.language_for_filename(&path) {
            language_config = Some(loader.language(found).config().clone());
        }
        language_config
    };
    let Some(config) = configured else {
        return;
    };
    // `to_vec` copies the borrowed list so the registry tables are free to change.
    let names = worker.languages.configured(&language).to_vec();
    let outside = !path.starts_with(&worker.root);
    if let Some(document) = worker.session.document.as_mut() {
        // The server-facing identifier falls back to Helix's language name.
        document.language_id = match config.language_server_language_id.clone() {
            Some(server_language) => server_language,
            None => config.language_id.clone(),
        };
        document.config = Some(config.clone());
        document.state = if names.is_empty() {
            DocumentState::NoServerConfigured
        } else if outside {
            DocumentState::OutsideProject
        } else {
            DocumentState::Attached
        };
    }
    if outside {
        attach_running(worker, &names);
    } else {
        start::start(worker, &config, &names).await;
    }
}

/// What:
///  Start exited servers of the displayed document again;
///  returns true when anything was tried.
/// Why:
///  A server that ended is started again only by an explicit open or position request,
///      never in a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function revive(worker: Worker): Promise<boolean>
/// ```
pub(super) async fn revive(worker: &mut Worker) -> bool {
    let mut exited = false;
    for record in worker.session.servers.iter() {
        if record.attached && record.state == ServerState::Exited {
            exited = true;
            break;
        }
    }
    if !exited {
        return false;
    }
    // Rows without a process are rebuilt by the attach step.
    let rows = std::mem::take(&mut worker.session.servers);
    for record in rows {
        if record.client.is_some() {
            worker.session.servers.push(record);
        }
    }
    attach(worker).await;
    return true;
}

/// What:
///  A server finished `initialize`:
///  send its settings,
///  mark it ready,
///  and open the document.
/// Why:
///  helix-lsp injects a synthetic `initialized` notification for exactly this;
///  Helix's
///      editor sends `workspace/didChangeConfiguration` at the same point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initialized(worker: Worker, server: ServerId): void
/// ```
pub(super) fn initialized(worker: &mut Worker, server: LanguageServerId) {
    let Some(index) = worker.session.index_of(server) else {
        return;
    };
    let record = &mut worker.session.servers[index];
    let Some(client) = record.ready() else {
        return;
    };
    if let Some(settings) = client.config() {
        client.did_change_configuration(settings.clone());
    }
    tracing::info!(server = %record.identity.name, instance = record.identity.instance, encoding = ?client.offset_encoding(), "language server is ready");
    record.state = ServerState::Ready;
    send_did_open(worker, index);
}

/// What:
///  A server process ended:
///  remove it from helix-lsp's registry and from the session.
/// Why:
///  Without removal the registry would hand the dead client out again,
///  and a kept client
///      would leave the ended process unreaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exited(worker: Worker, server: ServerId): void
/// ```
pub(super) fn exited(worker: &mut Worker, server: LanguageServerId) {
    let Some(index) = worker.session.index_of(server) else {
        worker.registry.remove_by_id(server);
        return;
    };
    let was_ready = worker.session.servers[index].ready().is_some();
    let state = if was_ready {
        ServerState::Exited
    } else {
        ServerState::FailedToStart {
            reason: "the server process ended before it finished starting".to_string(),
        }
    };
    // The record waits briefly for the server's last standard-error lines; the state does not.
    // A start that failed is given those lines afterwards, so its reason can say what the server wrote.
    let identity = worker.session.servers[index].identity.clone();
    if was_ready {
        report::ended_unexpectedly(identity.name.clone(), true);
    } else {
        let internal = worker.internal.clone();
        let name = identity.name.clone();
        report::ended_unexpectedly_then(name, false, SendEndCause { internal, identity });
    }
    retire(worker, index, state, false);
    if !worker.session.servers[index].attached {
        worker.session.servers.remove(index);
    }
}

/// What:
///  A starting server's deadline passed:
///  if it still has not initialized,
///  stop it.
/// Why:
///  helix-lsp reports neither an error answer to `initialize` nor a missing answer;
///  time is
///      the only signal that the start failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startDeadline(worker: Worker, server: ServerId): void
/// ```
pub(super) fn start_deadline(worker: &mut Worker, server: LanguageServerId) {
    let Some(index) = worker.session.index_of(server) else {
        return;
    };
    if worker.session.servers[index].ready().is_some() {
        return;
    }
    let name = worker.session.servers[index].identity.name.clone();
    let seconds = worker.languages.start_timeout(&name);
    tracing::error!(server = %name, seconds, "language server did not answer initialize in time and is stopped");
    let summary = format!("{name} did not answer initialize within {seconds} seconds");
    let lines = report::stopped_during_start(&name);
    let reason = with_lines(summary, &lines);
    retire(worker, index, ServerState::FailedToStart { reason }, true);
    if !worker.session.servers[index].attached {
        worker.session.servers.remove(index);
    }
}

/// How many of a server's last standard-error lines a reason carries.
const CAUSE_LINES: usize = 4;

/// How many bytes of one such line a reason carries.
const CAUSE_LINE_BYTES: usize = 200;

/// What:
///  Append the last lines a server wrote to standard error to the reason its start failed.
///  `&[String]` lends the lines,
///  oldest first.
/// Why:
///  "The server process ended before it finished starting" names no cause;
///  the server usually states it on standard error,
///  and the log is not where the reader of the window looks.
///  Only the last lines,
///  each cut,
///  are carried;
///  the log keeps the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withLines(reason: string, lines: string[]): string
/// ```
fn with_lines(reason: String, lines: &[String]) -> String {
    if lines.is_empty() {
        return reason;
    }
    let first = lines.len().saturating_sub(CAUSE_LINES);
    let mut text = format!("{reason}; the server wrote this to its standard error:");
    for line in &lines[first..] {
        text.push('\n');
        // The cut stays on a character boundary so the text remains valid.
        let mut end = line.len().min(CAUSE_LINE_BYTES);
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        text.push_str(&line[..end]);
        if end < line.len() {
            text.push_str(" [cut]");
        }
    }
    return text;
}

/// What:
///  The last standard-error lines of a server whose start failed arrived:
///  state them in the reason.
/// Why:
///  They are complete only after the server's standard error ended or `SETTLE` passed,
///  which is after the state was first published.
///  A server that was started again meanwhile is another process and keeps its own state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endCause(worker: Worker, server: ServerIdentity, lines: string[]): void
/// ```
pub(super) fn end_cause(worker: &mut Worker, server: &ServerIdentity, lines: Vec<String>) {
    let Some(index) = worker.session.index_of_identity(server) else {
        tracing::debug!(server = %server.name, "the record of a failed start was removed before its cause arrived");
        return;
    };
    let record = &mut worker.session.servers[index];
    let ServerState::FailedToStart { reason } = &record.state else {
        return;
    };
    if lines.is_empty() {
        return;
    }
    let updated = with_lines(reason.clone(), &lines);
    tracing::debug!(server = %server.name, lines = lines.len(), "stated the cause of a failed start");
    record.state = ServerState::FailedToStart { reason: updated };
}

/// The reason text a failed start carries is built without a server.
#[cfg(test)]
#[path = "attach/cause_tests.rs"]
mod cause_tests;
