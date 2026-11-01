//! The reasons the entry gives,
//!  state by state,
//!  without a server process.

use super::{SOURCE, Trouble, lines, with_notice};
use crate::language::{
    diagnostics::{Diagnostic, DiagnosticsSnapshot, Freshness, Severity, SourceGroup},
    identity::{DocumentStamp, ServerIdentity},
    reply::RequestFailure,
    status::{DocumentState, Features, LanguageStatus, ServerState, ServerStatus},
};
use std::path::PathBuf;

const STAMP: DocumentStamp = DocumentStamp {
    file: 1,
    revision: 0,
};

fn identity(name: &str) -> ServerIdentity {
    return ServerIdentity {
        name: name.to_string(),
        instance: 1,
    };
}

/// Whether some line contains every one of the texts.
fn has_line_with(lines: &[String], needles: &[&str]) -> bool {
    for line in lines {
        let mut contains_all = true;
        for needle in needles {
            if !line.contains(needle) {
                contains_all = false;
            }
        }
        if contains_all {
            return true;
        }
    }
    return false;
}

/// Whether some line begins with the text.
fn has_line_starting(lines: &[String], prefix: &str) -> bool {
    for line in lines {
        if line.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

fn row(name: &str, state: ServerState, features: Option<Features>) -> ServerStatus {
    return ServerStatus {
        server: identity(name),
        state,
        features,
        progress: None,
    };
}

fn everything() -> Features {
    return Features {
        definition: true,
        references: true,
        hover: true,
        inlay_hints: true,
        pull_diagnostics: true,
    };
}

fn status(servers: Vec<ServerStatus>) -> LanguageStatus {
    return LanguageStatus {
        file: Some(1),
        language: Some("toml".to_string()),
        document: DocumentState::Attached,
        servers,
    };
}

#[test]
fn servers_missing_from_path_are_named_with_the_remedy() {
    let shown = lines(
        &status(vec![
            row(
                "taplo",
                ServerState::MissingExecutable {
                    reason: "the program 'taplo' for taplo was not found on PATH".to_string(),
                },
                None,
            ),
            row(
                "tombi",
                ServerState::MissingExecutable {
                    reason: "the program 'tombi' for tombi was not found on PATH".to_string(),
                },
                None,
            ),
        ]),
        &[],
    );
    assert_eq!(
        shown,
        [
            "Hover, inlay hints, and go to definition are not available for this file.",
            "taplo is not available: the program 'taplo' for taplo was not found on PATH.",
            "tombi is not available: the program 'tombi' for tombi was not found on PATH.",
            "Servers are looked up again when a file is displayed, so open the file again after fixing this.",
        ]
    );
}

#[test]
fn a_failed_start_and_a_refused_launch_each_give_their_cause() {
    let failed = lines(
        &status(vec![row(
            "rust-analyzer",
            ServerState::FailedToStart {
                reason: "the server process ended before it finished starting".to_string(),
            },
            None,
        )]),
        &[],
    );
    assert!(
        has_line_starting(
            &failed,
            "rust-analyzer failed to start: the server process ended"
        ),
        "{failed:?}"
    );
    let refused = lines(
        &status(vec![row(
            "typescript-native",
            ServerState::LaunchRefused {
                reason: "bubblewrap is not installed".to_string(),
            },
            None,
        )]),
        &[],
    );
    assert!(
        refused.contains(
            &"typescript-native was not started: bubblewrap is not installed.".to_string()
        ),
        "{refused:?}"
    );
    // A refused launch is not retried by displaying the file again, so no such advice is given.
    assert!(
        !has_line_with(&refused, &["open the file again"]),
        "{refused:?}"
    );
}

#[test]
fn a_reason_that_ends_with_server_lines_gets_no_period_added() {
    let reason =
        "the server process ended before it finished starting; the server wrote this to its standard error:\nmise ERROR not trusted"
            .to_string();
    let shown = lines(
        &status(vec![row(
            "rust-analyzer",
            ServerState::FailedToStart { reason },
            None,
        )]),
        &[],
    );
    assert_eq!(
        shown[1],
        "rust-analyzer failed to start: the server process ended before it finished starting; the server wrote this to its standard error:\nmise ERROR not trusted"
    );
}

#[test]
fn a_server_that_is_still_starting_gives_no_entry() {
    let shown = lines(
        &status(vec![row("typescript-native", ServerState::Starting, None)]),
        &[],
    );
    assert!(
        shown.is_empty(),
        "an entry was shown while the server was still starting: {shown:?}"
    );
}

#[test]
fn a_server_that_offers_everything_gives_no_entry() {
    let shown = lines(
        &status(vec![row(
            "typescript-native",
            ServerState::Ready,
            Some(everything()),
        )]),
        &[(identity("typescript-native"), Trouble::default())],
    );
    assert!(shown.is_empty(), "{shown:?}");
}

#[test]
fn a_running_server_without_hints_is_named() {
    let shown = lines(
        &status(vec![row(
            "slint-lsp",
            ServerState::Ready,
            Some(Features {
                inlay_hints: false,
                ..everything()
            }),
        )]),
        &[],
    );
    assert_eq!(
        shown,
        ["Inlay hints are not offered by slint-lsp, so none are shown."]
    );
}

#[test]
fn failed_hover_and_hint_requests_name_the_failure() {
    let trouble = Trouble {
        hints: Some(RequestFailure::Timeout),
        hover: Some(RequestFailure::Rpc {
            code: -32603,
            message: "internal error".to_string(),
        }),
    };
    let shown = lines(
        &status(vec![row(
            "typescript-native",
            ServerState::Ready,
            Some(everything()),
        )]),
        &[(identity("typescript-native"), trouble)],
    );
    assert_eq!(
        shown,
        [
            "The inlay hint request to typescript-native failed: the server did not answer in time.",
            "The hover request to typescript-native failed: the server answered with error -32603: internal error."
        ]
    );
}

#[test]
fn a_failure_of_another_server_process_is_not_shown() {
    let trouble = Trouble {
        hints: Some(RequestFailure::StreamClosed),
        hover: None,
    };
    let stale = ServerIdentity {
        name: "typescript-native".to_string(),
        instance: 0,
    };
    let shown = lines(
        &status(vec![row(
            "typescript-native",
            ServerState::Ready,
            Some(everything()),
        )]),
        &[(stale, trouble)],
    );
    assert!(shown.is_empty(), "{shown:?}");
}

#[test]
fn files_without_a_language_server_give_no_entry() {
    for document in [
        DocumentState::Closed,
        DocumentState::NoLanguage,
        DocumentState::NoServerConfigured,
        DocumentState::OutsideProject,
    ] {
        let mut idle = status(Vec::new());
        idle.document = document;
        assert!(lines(&idle, &[]).is_empty(), "{document:?}");
    }
}

#[test]
fn a_server_rooted_outside_the_project_names_that_directory() {
    let shown = lines(
        &status(vec![row(
            "typescript-native",
            ServerState::RootOutsideProject {
                root: PathBuf::from("/srv/enclosing"),
            },
            None,
        )]),
        &[],
    );
    assert!(
        has_line_with(
            &shown,
            &["/srv/enclosing", "Open the IDE on that directory"]
        ),
        "{shown:?}"
    );
}

#[test]
fn the_entry_joins_the_servers_diagnostics_at_the_first_character() {
    let own = DiagnosticsSnapshot {
        stamp: STAMP,
        groups: vec![SourceGroup {
            source: "ts".to_string(),
            items: vec![Diagnostic {
                start: 4,
                end: 9,
                severity: Some(Severity::Error),
                code: Some("2322".to_string()),
                message: "wrong".to_string(),
                server: identity("typescript-native"),
                freshness: Freshness::Pulled,
            }],
        }],
    };
    let missing = status(vec![row(
        "taplo",
        ServerState::MissingExecutable {
            reason: "not found".to_string(),
        },
        None,
    )]);
    let joined = with_notice(Some(own.clone()), STAMP, &missing, &[], &[]).expect("snapshot");
    assert_eq!(joined.stamp, STAMP);
    assert_eq!(joined.groups.len(), 2);
    assert_eq!(joined.groups[0], own.groups[0]);
    let entry = &joined.groups[1];
    assert_eq!(entry.source, SOURCE);
    assert_eq!(entry.items.len(), 1);
    let item = &entry.items[0];
    assert_eq!((item.start, item.end), (0, 0));
    assert_eq!(item.severity, Some(Severity::Information));
    assert!(item.message.contains("taplo is not available: not found."));
    // With no snapshot of its own the entry still gets one, stamped for the displayed text.
    let alone = with_notice(None, STAMP, &missing, &[], &[]).expect("snapshot");
    assert_eq!(alone.stamp, STAMP);
    assert_eq!(alone.groups.len(), 1);
    // Nothing to say leaves the snapshot, or its absence, as it was.
    let fine = status(vec![row("taplo", ServerState::Ready, Some(everything()))]);
    assert_eq!(
        with_notice(Some(own.clone()), STAMP, &fine, &[], &[]),
        Some(own)
    );
    assert_eq!(with_notice(None, STAMP, &fine, &[], &[]), None);
}

fn advisory() -> Vec<String> {
    return vec![
        "The toolchain lookup for /work/project could not be used, so language servers are looked up on PATH only."
            .to_string(),
        "Fix what the lookup reports (run `mpm-shadow bin --json` at the worktree root in a terminal to see it yourself; a worktree whose toolchain is not installed needs `mise run toolchain:install` first), then display a file again."
            .to_string(),
        "the toolchain lookup said: Error: toolchain.lock.json is unreadable".to_string(),
    ];
}

#[test]
fn a_toolchain_failure_is_told_even_when_every_server_works() {
    let fine = status(vec![row("taplo", ServerState::Ready, Some(everything()))]);
    let entry = with_notice(None, STAMP, &fine, &[], &advisory()).expect("snapshot");
    let message = &entry.groups[0].items[0].message;
    assert_eq!(entry.groups[0].source, SOURCE);
    assert_eq!(message, &advisory().join("\n"));
}

#[test]
fn the_toolchain_failure_comes_before_the_reasons_of_the_servers_that_do_not_work() {
    let missing = status(vec![row(
        "taplo",
        ServerState::MissingExecutable {
            reason: "the program 'taplo' for taplo was not found on PATH".to_string(),
        },
        None,
    )]);
    let entry = with_notice(None, STAMP, &missing, &[], &advisory()).expect("snapshot");
    let message = &entry.groups[0].items[0].message;
    let reason = message
        .find("taplo is not available")
        .expect("reason of the server");
    let lookup = message
        .find("the toolchain lookup said:")
        .expect("the lookup's words");
    assert!(lookup < reason, "{message}");
    assert!(
        message.contains("mpm-shadow bin --json"),
        "{message}"
    );
}

#[test]
fn files_without_servers_get_no_toolchain_notice() {
    for document in [
        DocumentState::Closed,
        DocumentState::NoLanguage,
        DocumentState::NoServerConfigured,
        DocumentState::OutsideProject,
    ] {
        let mut plain = status(Vec::new());
        plain.document = document;
        assert_eq!(
            with_notice(None, STAMP, &plain, &[], &advisory()),
            None,
            "{document:?}"
        );
    }
    let no_rows = status(Vec::new());
    assert_eq!(with_notice(None, STAMP, &no_rows, &[], &advisory()), None);
}
