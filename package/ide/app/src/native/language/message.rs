//! Sentences for language requests that cannot be satisfied.
//!
//! A message appears only after the user asked for something;
//!  each names what failed and what
//! to do.
//!  Which server answered,
//!  and the latest status,
//!  decide between starting,
//!  missing,
//! unsupported,
//!  failed,
//!  and empty.

/// The action names the feature and the key that repeats it.
use super::Action;
/// Replies,
///  states,
///  and the identity naming the server.
use ide_app::language::{
    identity::ServerIdentity,
    reply::{RequestFailure, RequestKind, RequestOutcome},
    status::{DocumentState, LanguageStatus, ServerState, ServerStatus},
};

/// The feature a request kind provides,
///  as a message names it.
fn feature(kind: RequestKind) -> &'static str {
    return match kind {
        RequestKind::Definition => "go to definition",
        RequestKind::References => "find references",
        RequestKind::Hover => "hover information",
    };
}

/// The feature a request kind provides,
///  capitalized to start a sentence.
fn heading(kind: RequestKind) -> &'static str {
    return match kind {
        RequestKind::Definition => "Go to definition",
        RequestKind::References => "Find references",
        RequestKind::Hover => "Hover information",
    };
}

/// What:
///  The name of the server that gave `outcome`,
///  or a generic one.
///       `Option<&ServerIdentity>` is "a borrowed identity,
///  or nothing".
/// Why:
///  Replies without a server (no process could be asked) still need a subject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (server?: ServerIdentity) => server?.name ?? 'The language server';
/// ```
fn name(server: Option<&ServerIdentity>) -> String {
    // The fallback stands for nothing; otherwise the server's own name.
    return match server {
        Some(found) => found.name.clone(),
        None => "The language server".to_string(),
    };
}

/// The work a server reports,
///  as " (title)",
///  or nothing.
fn progress(status: &LanguageStatus, server: Option<&ServerIdentity>) -> String {
    for row in &status.servers {
        // The borrowed identity is tested only when one is present.
        let same_server = match server {
            Some(found) => found.name == row.server.name,
            None => false,
        };
        if same_server && let Some(title) = &row.progress {
            return format!(" ({title})");
        }
    }
    return String::new();
}

/// What:
///  Explain a server state that keeps the action from being served.
///       `&ServerStatus` lends one status row.
/// Why:
///  A request answered "no server" is explained by why its server is not running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stateMessage(action: Action, row: ServerStatus): string | undefined
/// ```
fn state(action: Action, row: &ServerStatus) -> Option<String> {
    let server = &row.server.name;
    let key = action.key();
    let title = heading(action.kind());
    let wanted = feature(action.kind());
    // `Some(format!(...))` returns the sentence for a state that needs one.
    return match &row.state {
        ServerState::MissingExecutable { reason } => Some(format!(
            "{title} needs {server}, which is not available: {reason}. After installing it, press {key} again."
        )),
        // Every reason the launch policy gives ends with its remedy, so the reason comes last.
        ServerState::LaunchRefused { reason } => Some(format!(
            "{server} was not started, so {wanted} is not available for this file: {reason}."
        )),
        ServerState::FailedToStart { reason } => Some(format!(
            "{server} failed to start: {reason}. Press {key} again to start it again."
        )),
        ServerState::Unsynchronized => Some(unsynchronized(server, key)),
        ServerState::Exited => Some(format!(
            "{server} stopped. Press {key} again to start it again."
        )),
        ServerState::RootOutsideProject { root } => Some(format!(
            "{server} was not started because it would treat {}, which is outside this project, as its workspace. Open the IDE on that directory to use it.",
            root.display()
        )),
        ServerState::WrongWorkingDirectory { directory } => Some(format!(
            "Language servers cannot start because the working directory {} does not contain the project. Restart the application.",
            directory.display()
        )),
        ServerState::NotStarted { reason } => Some(format!("{server} was not started: {reason}.")),
        ServerState::Starting => Some(format!(
            "{server} is still starting. Try again in a moment."
        )),
        // `None`: a ready server explains nothing by its state.
        ServerState::Ready => None,
    };
}

/// The sentence for a server that cannot follow external changes to the displayed file.
fn unsynchronized(server: &str, key: &str) -> String {
    return format!(
        "{server} cannot follow changes made to this file outside the IDE, so its answer would point at the wrong place. Press {key} again to give it the current text."
    );
}

/// What:
///  Explain why no server answered,
///  from the latest status.
/// Why:
///  "No server" alone gives no remedy;
///  the document state and each server's state do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noServer(action: Action, status: LanguageStatus): string
/// ```
fn no_server(action: Action, status: &LanguageStatus) -> String {
    let wanted = feature(action.kind());
    // `as_deref` lends the language name as `&str`; `unwrap_or` names an unknown one.
    let language = status.language.as_deref().unwrap_or("this language");
    match status.document {
        DocumentState::Closed => return "No file is displayed.".to_string(),
        DocumentState::NoLanguage => {
            return format!(
                "No language is recognized for this file, so {wanted} is not available here."
            );
        }
        DocumentState::NoServerConfigured => {
            return format!(
                "The {language} language has no language server here, so {wanted} is not available for this file."
            );
        }
        DocumentState::OutsideProject | DocumentState::Attached => {}
    }
    for row in &status.servers {
        if let Some(sentence) = state(action, row) {
            return sentence;
        }
    }
    if status.document == DocumentState::OutsideProject {
        return format!(
            "This file is outside the project, and no {language} language server of the project is running. Open a {language} file of the project, then return here and press {} again.",
            action.key()
        );
    }
    return format!("No language server answered, so {wanted} is not available for this file.");
}

/// Explain a failed request.
fn failed(action: Action, server: Option<&ServerIdentity>, failure: &RequestFailure) -> String {
    let title = heading(action.kind());
    let who = name(server);
    let key = action.key();
    return match failure {
        RequestFailure::Rpc { code, message } => format!(
            "{title} failed: {who} answered with error {code}: {message}. Press {key} to try again; if it keeps failing, restart the application."
        ),
        RequestFailure::Timeout => format!(
            "{title} failed: {who} did not answer in time. Press {key} to try again when it is less busy."
        ),
        RequestFailure::StreamClosed => format!(
            "{title} failed: {who} stopped while answering. Press {key} again to start it again."
        ),
        RequestFailure::Other(text) => {
            format!("{title} failed: {text}. Press {key} to try again.")
        }
    };
}

/// The answer with the lowest rank,
///  or nothing without answers.
fn most_telling(
    outcomes: &[(Option<ServerIdentity>, RequestOutcome)],
) -> Option<&(Option<ServerIdentity>, RequestOutcome)> {
    let mut best: Option<&(Option<ServerIdentity>, RequestOutcome)> = None;
    for candidate in outcomes {
        let better = match best {
            Some(held) => rank(&candidate.1) < rank(&held.1),
            None => true,
        };
        if better {
            best = Some(candidate);
        }
    }
    return best;
}

/// What:
///  Rank outcomes by how much they tell the user;
///  lower is more telling.
/// Why:
///  With several servers,
///  the most actionable answer explains the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rank = (outcome: RequestOutcome): number => ({ failed: 0, unsynchronized: 1, ... })[outcome.kind];
/// ```
fn rank(outcome: &RequestOutcome) -> u8 {
    return match outcome {
        RequestOutcome::Failed(_) => 0,
        RequestOutcome::Unsynchronized => 1,
        RequestOutcome::Starting => 2,
        RequestOutcome::Superseded => 3,
        RequestOutcome::Empty => 4,
        RequestOutcome::Unsupported => 5,
        RequestOutcome::NoServer => 6,
        // Results are not failures and are never explained.
        RequestOutcome::Locations(_) | RequestOutcome::Hover(_) => 7,
    };
}

/// What:
///  Whether the answer to a hover is about the language support itself,
///  not about this position:
///  a failed request,
///  a server that cannot follow the file,
///  a feature the server does not offer,
///  or no running server for a file that has servers configured.
///  `&[(Option<ServerIdentity>, RequestOutcome)]` lends the answers received.
/// Why:
///  An empty answer,
/// a request set aside,
/// and a server still starting are properties of one position or of the
///  moment and are not worth interrupting a resting pointer for;
///  the others hold for every position of the file until something is installed or fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statesSupportProblem(outcomes: [ServerIdentity | undefined, RequestOutcome][], status: LanguageStatus): boolean
/// ```
pub(super) fn states_support_problem(
    outcomes: &[(Option<ServerIdentity>, RequestOutcome)],
    status: &LanguageStatus,
) -> bool {
    if status.document != DocumentState::Attached {
        return false;
    }
    // The most telling answer decides, as it does for the explanation itself.
    let Some((_, outcome)) = most_telling(outcomes) else {
        return true;
    };
    return matches!(
        outcome,
        RequestOutcome::Failed(_)
            | RequestOutcome::Unsynchronized
            | RequestOutcome::Unsupported
            | RequestOutcome::NoServer
    );
}

/// What:
///  The sentence for a finished request that produced nothing to show.
///       `&[...]` borrows the list of every server's answer.
/// Why:
///  Exactly one sentence is shown,
///  chosen from the most telling answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explain(action: Action, outcomes: [ServerIdentity?, RequestOutcome][], status: LanguageStatus): string
/// ```
pub(super) fn explain(
    action: Action,
    outcomes: &[(Option<ServerIdentity>, RequestOutcome)],
    status: &LanguageStatus,
) -> String {
    // The answer with the lowest rank decides; `None` means there was no answer.
    let Some((server, outcome)) = most_telling(outcomes) else {
        return no_server(action, status);
    };
    let who = name(server.as_ref());
    let wanted = feature(action.kind());
    let key = action.key();
    return match outcome {
        RequestOutcome::Failed(failure) => failed(action, server.as_ref(), failure),
        RequestOutcome::Unsynchronized => unsynchronized(&who, key),
        RequestOutcome::Starting => format!(
            "{who} is still starting{}. Press {key} again in a moment.",
            progress(status, server.as_ref())
        ),
        RequestOutcome::Superseded => format!(
            "{who} set the request aside because its analysis kept changing. Press {key} to try again."
        ),
        RequestOutcome::Empty => {
            let sentence = match action.kind() {
                RequestKind::Definition => "No definition found.",
                RequestKind::References => "No usages found.",
                RequestKind::Hover => "No hover information at this position.",
            };
            // A server still analyzing the project may answer empty for now.
            let busy = progress(status, server.as_ref());
            if busy.is_empty() {
                sentence.to_string()
            } else {
                format!(
                    "{sentence} {who} is still working{busy}; press {key} again when it finishes."
                )
            }
        }
        RequestOutcome::Unsupported => format!("{who} does not offer {wanted}."),
        RequestOutcome::NoServer | RequestOutcome::Locations(_) | RequestOutcome::Hover(_) => {
            no_server(action, status)
        }
    };
}

/// The sentence for a request whose text changed on disk before the answer arrived.
pub(super) fn changed(action: Action) -> String {
    return format!(
        "The file changed on disk before {} arrived. Press {} again.",
        match action.kind() {
            RequestKind::Definition => "the definition",
            RequestKind::References => "the references",
            RequestKind::Hover => "the hover information",
        },
        action.key()
    );
}

/// The sentence for a location the server named that cannot be opened.
pub(super) fn unavailable(address: &str, reason: &str) -> String {
    return format!("Cannot open {address}: {reason}.");
}

/// The sentence once the Language module stopped;
///  `reason` is the handle's own explanation.
pub(super) fn stopped(reason: &str) -> String {
    return format!("{reason}.");
}

/// What:
///  Whether displaying the file again could let a server serve it.
/// Why:
///  The worker re-resolves programs and restarts failed servers only when a file is
///      displayed,
///  so an explicit action displays it again when the status shows such a state
///      and no server would answer as things are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function needsReopen(status: LanguageStatus): boolean
/// ```
pub(super) fn needs_reopen(status: &LanguageStatus) -> bool {
    let mut ready = false;
    let mut fixable = false;
    for row in &status.servers {
        // `matches!` tests a value against a pattern without extracting anything.
        if row.state == ServerState::Ready {
            ready = true;
        }
        if row.state == ServerState::Unsynchronized {
            return true;
        }
        if matches!(
            row.state,
            ServerState::MissingExecutable { .. } | ServerState::FailedToStart { .. }
        ) {
            fixable = true;
        }
    }
    if ready {
        return false;
    }
    return fixable || status.document == DocumentState::OutsideProject;
}
