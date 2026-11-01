//! The reason a displayed file has no hover,
//!  no inlay hints,
//!  or no go to definition,
//!  as a problem entry of its own.
//!
//! Nothing else on screen says why a resting pointer shows nothing or why a file has no hints:
//!  a failed pointer hover is only logged,
//!  and no surface exists for hints.
//!  The reason is therefore published as one `Information` entry in the displayed file's diagnostics,
//!  at the start of the file,
//!  where the source view already draws such entries as rows above their line.
//!  Nothing is drawn while a server is still starting,
//!  or when every feature works.

/// The entry is a diagnostic of the displayed text.
use super::diagnostics::{Diagnostic, DiagnosticsSnapshot, Freshness, Severity, SourceGroup};
/// The entry names the file generation and revision it describes.
use super::identity::{DocumentStamp, ServerIdentity};
/// What the servers of the displayed file are doing.
use super::status::{DocumentState, LanguageStatus, ServerState, ServerStatus};

/// The source named in the entry's message row,
///  for example `Information (language support): ...`.
pub const SOURCE: &str = "language support";

/// The failures kept per server process,
/// and their wording.
mod trouble;
/// The failures kept for one server process.
pub(super) use trouble::Trouble;
/// The wording of a kept failure.
use trouble::failure_text;

/// What:
///  End a sentence with a period unless it already ends with one.
///  `String` is returned owned.
/// Why:
///  Reasons come from several places;
///  some end with a sentence,
///  some with a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sentence(text: string): string
/// ```
fn sentence(text: &str) -> String {
    let trimmed = text.trim_end();
    // A text that ends with lines a server wrote is left as it is: a period would join the last of them.
    if trimmed.ends_with('.') || trimmed.contains('\n') {
        return trimmed.to_string();
    }
    return format!("{trimmed}.");
}

/// What:
///  The line that explains one server that is not usable for the displayed file,
///  or nothing for a usable or starting one.
/// Why:
///  Every state has its own cause and its own remedy.
///  The remedy for a missing program or a start that failed is to display the file again,
///  because the Language module looks programs up and starts failed servers at that moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function problem(row: ServerStatus): string | undefined
/// ```
fn problem(row: &ServerStatus) -> Option<String> {
    let server = &row.server.name;
    return match &row.state {
        ServerState::MissingExecutable { reason } => {
            Some(sentence(&format!("{server} is not available: {reason}")))
        }
        ServerState::LaunchRefused { reason } => {
            Some(sentence(&format!("{server} was not started: {reason}")))
        }
        ServerState::FailedToStart { reason } => {
            Some(sentence(&format!("{server} failed to start: {reason}")))
        }
        ServerState::Exited => Some(format!(
            "{server} stopped. Open the file again to start it again."
        )),
        ServerState::Unsynchronized => Some(format!(
            "{server} cannot follow changes made to this file outside the IDE, so it is not asked about it. Open the file again to give it the current text."
        )),
        ServerState::RootOutsideProject { root } => Some(format!(
            "{server} was not started because it would treat {}, which is outside this project, as its workspace. Open the IDE on that directory to use it.",
            root.display()
        )),
        ServerState::WrongWorkingDirectory { directory } => Some(format!(
            "{server} cannot start because the working directory {} does not contain the project. Restart the application.",
            directory.display()
        )),
        ServerState::NotStarted { reason } => {
            Some(sentence(&format!("{server} was not started: {reason}")))
        }
        ServerState::Starting | ServerState::Ready => None,
    };
}

/// Whether a state is one that the reader can end by displaying the file again.
fn retried_on_display(state: &ServerState) -> bool {
    return matches!(
        state,
        ServerState::MissingExecutable { .. }
            | ServerState::FailedToStart { .. }
            | ServerState::Exited
    );
}

/// True when some server in `ready` offers the feature that `wanted` tests for.
fn offering(ready: &[&ServerStatus], wanted: fn(&ServerStatus) -> bool) -> bool {
    for row in ready {
        if wanted(row) {
            return true;
        }
    }
    return false;
}

/// True when the server reported that it offers hover.
fn offers_hover(row: &ServerStatus) -> bool {
    return match row.features {
        Some(found) => found.hover,
        None => false,
    };
}

/// True when the server reported that it offers inlay hints.
fn offers_hints(row: &ServerStatus) -> bool {
    return match row.features {
        Some(found) => found.inlay_hints,
        None => false,
    };
}

/// What:
///  The lines of the entry for the displayed file,
///  or none when nothing is wrong or the servers are still starting.
///  `&[(ServerIdentity, Trouble)]` pairs each server process with its kept failures.
/// Why:
///  This is the whole decision;
///  the text of each line is the only wording the reader gets for the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lines(status: LanguageStatus, troubles: [ServerIdentity, Trouble][]): string[]
/// ```
pub(super) fn lines(
    status: &LanguageStatus,
    troubles: &[(ServerIdentity, Trouble)],
) -> Vec<String> {
    if status.document != DocumentState::Attached || status.servers.is_empty() {
        return Vec::new();
    }
    let mut ready: Vec<&ServerStatus> = Vec::new();
    let mut starting = false;
    for row in &status.servers {
        if row.state == ServerState::Ready {
            ready.push(row);
        }
        if row.state == ServerState::Starting {
            starting = true;
        }
    }
    let mut lines: Vec<String> = Vec::new();
    if ready.is_empty() {
        // A server that is still starting may yet provide everything; the entry waits for its outcome.
        if starting {
            return lines;
        }
        lines.push(
            "Hover, inlay hints, and go to definition are not available for this file.".to_string(),
        );
        for row in &status.servers {
            lines.extend(problem(row));
        }
        let mut retried = false;
        for row in &status.servers {
            if retried_on_display(&row.state) {
                retried = true;
                break;
            }
        }
        if retried {
            lines.push(
                "Servers are looked up again when a file is displayed, so open the file again after fixing this."
                    .to_string(),
            );
        }
        return lines;
    }
    // Some server runs: only the features that no running server offers, and failures, are reported.
    let mut name_list: Vec<&str> = Vec::new();
    for row in &ready {
        name_list.push(row.server.name.as_str());
    }
    let names = name_list.join(", ");
    if !offering(&ready, offers_hover) {
        lines.push(format!(
            "Hover is not offered by {names}, so none is shown."
        ));
    }
    if !offering(&ready, offers_hints) {
        lines.push(format!(
            "Inlay hints are not offered by {names}, so none are shown."
        ));
    }
    for row in &ready {
        let mut found_trouble = None;
        for (known, trouble) in troubles {
            if known == &row.server {
                found_trouble = Some(trouble);
                break;
            }
        }
        let Some(trouble) = found_trouble else {
            continue;
        };
        if let Some(failure) = &trouble.hints {
            lines.push(sentence(&format!(
                "The inlay hint request to {} failed: {}",
                row.server.name,
                failure_text(failure)
            )));
        }
        if let Some(failure) = &trouble.hover {
            lines.push(sentence(&format!(
                "The hover request to {} failed: {}",
                row.server.name,
                failure_text(failure)
            )));
        }
    }
    // The servers that are not usable explain what the running ones do not cover.
    if !lines.is_empty() {
        for row in &status.servers {
            lines.extend(problem(row));
        }
    }
    return lines;
}

/// What:
///  Add the entry to the displayed file's diagnostics,
///  or return them unchanged when there is nothing to say.
///  `Option<DiagnosticsSnapshot>` is "a snapshot,
///  or nothing".
/// Why:
///  The source view draws a snapshot's entries and the Language module publishes one value;
///  the entry joins the servers' own diagnostics so no second channel exists.
///  It stands at the first character with an empty range,
///  so nothing of the text is underlined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withNotice(snapshot: DiagnosticsSnapshot | undefined, stamp: DocumentStamp,
///                     status: LanguageStatus, troubles: [ServerIdentity, Trouble][], advisory: string[]): DiagnosticsSnapshot | undefined
/// ```
pub(super) fn with_notice(
    snapshot: Option<DiagnosticsSnapshot>,
    stamp: DocumentStamp,
    status: &LanguageStatus,
    troubles: &[(ServerIdentity, Trouble)],
    advisory: &[String],
) -> Option<DiagnosticsSnapshot> {
    // The advisory (the toolchain lookup could not be used) belongs to every file that has servers, working or not.
    // It comes first: the display keeps only the first dozen lines of a long notice,
    //  and the reasons of the servers that are missing (a shim for each) would push it out of sight.
    let mut text: Vec<String> = Vec::new();
    if status.document == DocumentState::Attached && !status.servers.is_empty() {
        text.extend(advisory.iter().cloned());
    }
    text.extend(lines(status, troubles));
    if text.is_empty() {
        return snapshot;
    }
    let mut result = snapshot.unwrap_or(DiagnosticsSnapshot {
        stamp,
        groups: Vec::new(),
    });
    let server = match status.servers.first() {
        Some(row) => row.server.clone(),
        None => ServerIdentity {
            name: SOURCE.to_string(),
            instance: 0,
        },
    };
    result.groups.push(SourceGroup {
        source: SOURCE.to_string(),
        items: vec![Diagnostic {
            start: 0,
            end: 0,
            severity: Some(Severity::Information),
            code: None,
            message: text.join("\n"),
            server,
            freshness: Freshness::Versioned,
        }],
    });
    return Some(result);
}

/// The wording of every state,
///  checked without a server.
#[cfg(test)]
#[path = "notice_tests.rs"]
mod tests;
