//! On-demand start of a language's servers,
//!  after the program,
//!  the root,
//!  and the launch were checked.

/// Removal of a server that must not keep running.
use super::retire;
/// Why a configured server cannot be started.
use crate::language::config::Unavailable;
/// Identities given to started processes.
use crate::language::identity::ServerIdentity;
/// Private directories a launch needs are prepared before the process exists.
use crate::language::launch::prepare;
/// `didOpen` follows readiness.
use crate::language::lifecycle::send_did_open;
/// A new process has no kept failures yet.
use crate::language::notice::Trouble;
/// A new process owes no answers yet.
use crate::language::owed::Owed;
/// Root checks made before and after a start.
use crate::language::root::{RootRefusal, RootView};
/// Session records.
use crate::language::session::ServerRecord;
/// States reported to the interface thread.
use crate::language::status::ServerState;
/// The worker whose state these steps change.
use crate::language::worker::{Internal, Worker};
/// One language's Helix configuration.
use helix_core::syntax::config::LanguageConfiguration;
/// helix-lsp's client handle.
use helix_lsp::Client;
/// What:
///  `Arc` is a thread-safe shared pointer (siblings:
///  `Rc`,
///  `Box`);
///  `Duration` is a time span.
/// Why:
///  helix-lsp hands clients out as `Arc<Client>`,
///  and the start deadline is a timer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = T;
/// ```
use std::{sync::Arc, time::Duration};

/// Seconds added to a server's start allowance before its start is declared failed.
const START_MARGIN: u64 = 2;

/// Translate a root refusal into the state shown for a server.
fn refusal_state(refusal: &RootRefusal) -> ServerState {
    // What: `match` unpacks the tagged union; `clone` copies the path into the new state.
    // Why: The two refusals have different remedies and stay distinct states.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return refusal.kind === 'outsideProject' ? { kind: 'rootOutsideProject', root } : { kind: 'wrongWorkingDirectory', directory };
    // ```
    return match refusal {
        RootRefusal::OutsideProject(root) => ServerState::RootOutsideProject { root: root.clone() },
        RootRefusal::WrongWorkingDirectory(directory) => ServerState::WrongWorkingDirectory {
            directory: directory.clone(),
        },
    };
}

/// What:
///  Take a client helix-lsp returned into the session:
///  a new process gets a record,
///  an
///       identity,
///  and a start deadline;
///  a reused one keeps its record.
/// Why:
///  The workspace folders helix-lsp will announce are checked once more after the start;
///      a folder outside the project ends the server at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function adopt(worker: Worker, name: string, client: Client, view: RootView): Promise<void>
/// ```
async fn adopt(worker: &mut Worker, name: String, client: Arc<Client>, view: &RootView) {
    let index = match worker.session.index_of(client.id()) {
        Some(index) => index,
        None => {
            let instance = worker.session.next_instance;
            worker.session.next_instance += 1;
            worker.session.servers.push(ServerRecord {
                identity: ServerIdentity { name, instance },
                // `Some(...)` is the "value present" variant; `clone` copies the shared pointer.
                client: Some(client.clone()),
                state: ServerState::Starting,
                attached: false,
                opened: false,
                progress: Vec::new(),
                // `Owed::default()` builds the empty record: nothing was asked yet.
                owed: Owed::default(),
                trouble: Trouble::default(),
            });
            let seconds = worker.languages.start_timeout(client.name()) + START_MARGIN;
            worker.timer(
                Duration::from_secs(seconds),
                Internal::StartDeadline(client.id()),
            );
            worker.session.servers.len() - 1
        }
    };
    // What: `.await` waits for helix-lsp's folder list; `clone` copies it so the lock is released.
    // Why: The list is filled when the client is created, before `initialize` is answered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const folders = [...await client.workspaceFolders()];
    // ```
    let folders = client.workspace_folders().await.clone();
    for folder in &folders {
        // `to_file_path` returns `Result`; an address that is not a local path counts as outside.
        let inside = match folder.uri.to_file_path() {
            Ok(path) => view.contains(&path),
            Err(_) => false,
        };
        if !inside {
            tracing::error!(server = client.name(), folder = %folder.uri, "server was rooted outside the project and is stopped");
            // The worker stops it itself: what it wrote to standard error is no report.
            crate::logging::stderr_tail::forget(client.name());
            let root = folder.uri.to_file_path().unwrap_or_default();
            retire(
                worker,
                index,
                ServerState::RootOutsideProject { root },
                true,
            );
            worker.session.servers[index].attached = true;
            return;
        }
    }
    let record = &mut worker.session.servers[index];
    record.attached = true;
    if client.is_initialized() {
        record.state = ServerState::Ready;
        send_did_open(worker, index);
    }
}

/// What:
///  Create the private directories of every server that is about to start;
///  returns the
///       name and reason of the first failure.
/// Why:
///  A launch that cannot be prepared must not happen at all;
///  there is no unconfined fallback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareLaunches(worker: Worker, names: string[]): [string, string] | undefined
/// ```
fn prepare_launches(worker: &Worker, names: &[String]) -> Option<(String, String)> {
    for name in names {
        // A running server keeps its directories; emptying its scratch space would break it.
        let mut running = false;
        for record in worker.session.servers.iter() {
            if record.client.is_some() && &record.identity.name == name {
                running = true;
                break;
            }
        }
        if running {
            continue;
        }
        let Some(launch) = worker.languages.launch(name) else {
            continue;
        };
        if let Err(reason) = prepare(launch, &worker.root, worker.languages.state_root()) {
            return Some((name.clone(), reason));
        }
    }
    // `None` is the "nothing" variant: every launch is prepared.
    return None;
}

/// What:
///  Start or reuse the servers of the displayed document's language inside the project.
/// Why:
///  Each outcome of `Registry::get` is one of the states the interface must tell apart,
///  and
///      nothing is spawned before the executable,
///  the root,
///  and the launch were checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function start(worker: Worker, config: LanguageConfiguration, names: string[]): Promise<void>
/// ```
pub(super) async fn start(
    worker: &mut Worker,
    config: &Arc<LanguageConfiguration>,
    names: &[String],
) {
    let mut startable: Vec<String> = Vec::new();
    for name in names {
        match worker.languages.unavailable(name) {
            Some(Unavailable::Missing(reason)) => {
                let state = ServerState::MissingExecutable {
                    reason: reason.clone(),
                };
                worker.session.report(name, state);
            }
            Some(Unavailable::Refused(reason)) => {
                let state = ServerState::LaunchRefused {
                    reason: reason.clone(),
                };
                worker.session.report(name, state);
            }
            None => startable.push(name.clone()),
        }
    }
    if startable.is_empty() {
        return;
    }
    let Some(shown) = worker.session.document.as_ref() else {
        return;
    };
    let path = shown.path.clone();
    let view = match RootView::discover(&worker.root) {
        Ok(view) => view,
        Err(refusal) => {
            tracing::error!(
                ?refusal,
                "language servers are not started: the process working directory is not the project root"
            );
            for name in &startable {
                worker.session.report(name, refusal_state(&refusal));
            }
            return;
        }
    };
    let document = view.to_helix(&path);
    if let Err(refusal) = view.lsp_root(config, &document) {
        tracing::warn!(
            ?refusal,
            "language servers are not started: their root would be outside the project"
        );
        for name in &startable {
            worker.session.report(name, refusal_state(&refusal));
        }
        return;
    }
    if let Some((failed, reason)) = prepare_launches(worker, &startable) {
        tracing::error!(server = %failed, %reason, "language servers are not started: a launch could not be prepared");
        for name in &startable {
            let state = if name == &failed {
                ServerState::LaunchRefused {
                    reason: reason.clone(),
                }
            } else {
                ServerState::NotStarted {
                    reason: format!("not started because {failed} could not be launched safely"),
                }
            };
            worker.session.report(name, state);
        }
        return;
    }
    let root_directories = [view.helix().to_path_buf()];
    // What: `Registry::get` returns a lazy sequence of `(name, Result<Arc<Client>>)`; `collect()`
    //       runs it to completion into a list. It reuses a running server whose root matches,
    //       otherwise spawns the process and sends `initialize` from a task.
    // Why: Collecting first ends the registry borrow before the session is changed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const started = [...registry.get(config, document, [root], false)];
    // ```
    let started: Vec<(String, helix_lsp::Result<Arc<Client>>)> = worker
        .registry
        .get(config, Some(document.as_path()), &root_directories, false)
        .collect();
    let mut seen: Vec<String> = Vec::new();
    for (name, outcome) in started {
        seen.push(name.clone());
        match outcome {
            // Helix looked the program up itself and did not find it: it vanished since the registry was built.
            Err(helix_lsp::Error::ExecutableNotFound(error)) => {
                worker.session.report(
                    &name,
                    ServerState::MissingExecutable {
                        reason: error.to_string(),
                    },
                );
            }
            Err(error) => {
                worker.session.report(
                    &name,
                    ServerState::FailedToStart {
                        reason: error.to_string(),
                    },
                );
            }
            Ok(client) => adopt(worker, name, client, &view).await,
        }
    }
    for name in &startable {
        if !seen.contains(name) {
            let reason = "its required root files are absent from the project".to_string();
            worker
                .session
                .report(name, ServerState::NotStarted { reason });
        }
    }
    return;
}
