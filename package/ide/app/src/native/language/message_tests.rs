//! Message sentences per server state and outcome,
//!  and hover text,
//!  without a window.

/// The sentences under test,
///  the action that names the key,
///  and hover rendering.
use super::{Action, hover_text::plain, message};
/// States,
///  outcomes,
///  and identities as the worker reports them.
use ide_app::language::{
    identity::ServerIdentity,
    reply::{HoverText, RequestFailure, RequestOutcome},
    status::{DocumentState, LanguageStatus, ServerState, ServerStatus},
};
/// Paths name refused roots and directories.
use std::path::PathBuf;

/// The scripted server's identity.
fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted-ls".to_string(),
        instance: 1,
    };
}

/// A status for a file of the `scripted` language with one server row in `state`.
fn status(document: DocumentState, state: Option<ServerState>) -> LanguageStatus {
    let mut servers = Vec::new();
    if let Some(found) = state {
        servers.push(ServerStatus {
            server: server(),
            state: found,
            features: None,
            progress: Some("Indexing".to_string()),
        });
    }
    return LanguageStatus {
        file: Some(1),
        language: Some("scripted".to_string()),
        document,
        servers,
    };
}

/// The sentence for one answer of the scripted server.
fn single(action: Action, outcome: RequestOutcome, from: &LanguageStatus) -> String {
    return message::explain(action, &[(Some(server()), outcome)], from);
}

/// Starting,
///  unsupported,
///  failed,
///  superseded,
///  and empty answers each get their own sentence.
#[test]
fn each_outcome_names_its_reason_and_remedy() {
    let ready = status(DocumentState::Attached, Some(ServerState::Ready));
    assert_eq!(
        single(Action::Definition, RequestOutcome::Starting, &ready),
        "scripted-ls is still starting (Indexing). Press Ctrl+B again in a moment."
    );
    assert_eq!(
        single(Action::Definition, RequestOutcome::Unsupported, &ready),
        "scripted-ls does not offer go to definition."
    );
    // The fixture's server reports work in progress, which an empty answer mentions.
    assert_eq!(
        single(Action::Definition, RequestOutcome::Empty, &ready),
        "No definition found. scripted-ls is still working (Indexing); press Ctrl+B again when it finishes."
    );
    let idle = status(DocumentState::Attached, None);
    assert_eq!(
        single(Action::Definition, RequestOutcome::Empty, &idle),
        "No definition found."
    );
    assert_eq!(
        single(Action::References, RequestOutcome::Empty, &idle),
        "No usages found."
    );
    assert_eq!(
        single(Action::Hover, RequestOutcome::Empty, &idle),
        "No hover information at this position."
    );
    let failure = RequestOutcome::Failed(RequestFailure::Rpc {
        code: -32603,
        message: "broken".to_string(),
    });
    assert!(single(Action::References, failure, &ready).starts_with(
        "Find references failed: scripted-ls answered with error -32603: broken. Press Ctrl+B"
    ));
    let timeout = RequestOutcome::Failed(RequestFailure::Timeout);
    assert!(
        single(Action::Hover, timeout, &ready).contains("did not answer in time. Press Ctrl+Q")
    );
    assert!(
        single(
            Action::PointerDefinition,
            RequestOutcome::Superseded,
            &ready
        )
        .ends_with("Press Ctrl+click to try again.")
    );
    assert!(
        single(Action::Definition, RequestOutcome::Unsynchronized, &ready)
            .contains("Press Ctrl+B again to give it the current text.")
    );
}

/// A server that could not be started safely names what is unavailable,
///  then the cause,
///  and
/// ends with the remedy the launch policy gave.
#[test]
fn refused_launch_names_the_unavailable_feature_and_ends_with_the_remedy() {
    let refused = status(
        DocumentState::Attached,
        Some(ServerState::LaunchRefused {
            reason: "cannot create /state/scripted-ls: Not a directory (os error 20). Make that path creatable as a directory, then restart the application".to_string(),
        }),
    );
    assert_eq!(
        single(Action::Definition, RequestOutcome::NoServer, &refused),
        "scripted-ls was not started, so go to definition is not available for this file: cannot create /state/scripted-ls: Not a directory (os error 20). Make that path creatable as a directory, then restart the application."
    );
    assert_eq!(
        message::explain(Action::Hover, &[], &refused),
        "scripted-ls was not started, so hover information is not available for this file: cannot create /state/scripted-ls: Not a directory (os error 20). Make that path creatable as a directory, then restart the application."
    );
    // Pressing the key again cannot help; the remedy is outside the application.
    assert!(!message::needs_reopen(&refused));
}

/// "No server" is explained by the document state or the server's own state.
#[test]
fn no_server_is_explained_by_the_status() {
    let missing = status(
        DocumentState::Attached,
        Some(ServerState::MissingExecutable {
            reason: "the program 'x' for scripted-ls was not found on PATH".to_string(),
        }),
    );
    assert_eq!(
        single(Action::Definition, RequestOutcome::NoServer, &missing),
        "Go to definition needs scripted-ls, which is not available: the program 'x' for scripted-ls was not found on PATH. After installing it, press Ctrl+B again."
    );
    assert!(message::needs_reopen(&missing));
    let none = status(DocumentState::NoLanguage, None);
    assert_eq!(
        single(Action::Hover, RequestOutcome::NoServer, &none),
        "No language is recognized for this file, so hover information is not available here."
    );
    let unconfigured = status(DocumentState::NoServerConfigured, None);
    assert!(
        single(Action::Definition, RequestOutcome::NoServer, &unconfigured)
            .starts_with("The scripted language has no language server here")
    );
    let outside = status(DocumentState::OutsideProject, None);
    assert!(
        single(Action::Definition, RequestOutcome::NoServer, &outside)
            .starts_with("This file is outside the project, and no scripted language server")
    );
    assert!(message::needs_reopen(&outside));
    let refused = status(
        DocumentState::Attached,
        Some(ServerState::RootOutsideProject {
            root: PathBuf::from("/enclosing"),
        }),
    );
    assert!(
        single(Action::Definition, RequestOutcome::NoServer, &refused)
            .contains("would treat /enclosing, which is outside this project, as its workspace")
    );
    let failed = status(
        DocumentState::Attached,
        Some(ServerState::FailedToStart {
            reason: "it exited".to_string(),
        }),
    );
    assert_eq!(
        message::explain(Action::Hover, &[], &failed),
        "scripted-ls failed to start: it exited. Press Ctrl+Q again to start it again."
    );
    let ready = status(DocumentState::Attached, Some(ServerState::Ready));
    assert!(!message::needs_reopen(&ready));
}

/// With several answers,
///  the most telling one is explained.
#[test]
fn the_most_telling_answer_is_explained() {
    let ready = status(DocumentState::Attached, Some(ServerState::Ready));
    let outcomes = [
        (Some(server()), RequestOutcome::Unsupported),
        (Some(server()), RequestOutcome::Empty),
        (Some(server()), RequestOutcome::Starting),
    ];
    assert!(message::explain(Action::Definition, &outcomes, &ready).contains("still starting"));
}

/// Fence lines are dropped,
///  blank runs collapse,
///  and plain text is kept as it is.
#[test]
fn hover_markdown_is_shown_as_plain_text_without_fences() {
    let markdown = HoverText {
        text: "```rust\npub fn area(width: u32) -> u32\n```\n\n\n\n---\nMultiply *width*.\n"
            .to_string(),
        markdown: true,
        range: None,
    };
    assert_eq!(
        plain(&markdown),
        "pub fn area(width: u32) -> u32\n\n---\nMultiply *width*."
    );
    let text = HoverText {
        text: "```not a fence in plain text".to_string(),
        markdown: false,
        range: None,
    };
    assert_eq!(plain(&text), "```not a fence in plain text");
    let long = HoverText {
        text: "x".repeat(9000),
        markdown: false,
        range: None,
    };
    assert_eq!(plain(&long).chars().count(), 8002);
}
