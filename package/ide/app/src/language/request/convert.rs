//! Conversion of server answers into the application's replies,
//!  against the ticket's text.

/// The ticket names the text,
///  the column unit,
///  and the document an answer belongs to.
use super::Ticket;
/// Server ranges become character offsets only through the shared converter.
use crate::language::position::from_lsp_range;
/// Replies and their outcomes.
use crate::language::reply::{HoverText, OpenTarget, RequestOutcome, Target};
/// Locations are validated before any file is read.
use crate::language::target::{Classified, classify};
/// What:
///  `Rope` is Helix's character-indexed text buffer.
/// Why:
///  A target in another file is converted against that file's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The protocol's data types.
use helix_lsp::lsp;
/// What:
///  `Path` is a borrowed filesystem path.
/// Why:
///  Targets are judged against the project root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::Path;

/// Largest target file read to convert a location,
///  in bytes.
///  `u64` matches file sizes.
const MAX_TARGET_BYTES: u64 = 16 * 1024 * 1024;

/// What:
///  Read a target file's text,
///  or nothing when it is too large or not readable as text.
///       `Option<Rope>` is "a text,
///  or nothing".
/// Why:
///  A location in another file is given in that file's lines and columns;
///  without its text
///      only the line number can be reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTarget(path: string): Rope | undefined
/// ```
fn read_target(path: &Path) -> Option<Rope> {
    // `ok()?` turns a failed metadata read into "nothing" and returns it.
    let size = std::fs::metadata(path).ok()?.len();
    if size > MAX_TARGET_BYTES {
        tracing::debug!(path = %path.display(), size, "target file is too large to convert its range");
        // `None` is the "nothing" variant of `Option`.
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    // `Some(...)` is the "value present" variant.
    return Some(Rope::from_str(&content));
}

/// What:
///  Validate and convert one location a server returned.
/// Why:
///  Refused addresses are reported as unavailable,
///  never read and never dropped;
///  openable
///      ones carry character offsets of the text they were converted against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function target(ticket: Ticket, root: string, uri: URL, range: Range): Target
/// ```
fn target(ticket: &Ticket, root: &Path, uri: &lsp::Url, range: lsp::Range) -> Target {
    let (path, outside_project) = match classify(uri, root) {
        Classified::InsideProject(path) => (path, false),
        Classified::OutsideProject(path) => (path, true),
        Classified::Refused(refusal) => {
            return Target::Unavailable {
                uri: uri.to_string(),
                refusal,
            };
        }
    };
    // Documents are compared by resolved path, never by address text.
    let same_document = path == ticket.path;
    let text = if same_document {
        // The open document's server-side text is the ticket's revision, not the file on disk.
        Some(ticket.text.clone())
    } else {
        read_target(&path)
    };
    // `and_then` continues into the conversion only when a text is available.
    let converted = match text {
        Some(found) => from_lsp_range(&found, range, ticket.encoding),
        None => None,
    };
    return Target::Open(OpenTarget {
        path,
        outside_project,
        same_document,
        // `try_from` converts the protocol's `u32`; `unwrap_or(usize::MAX)` cannot occur on 64-bit hosts.
        line: usize::try_from(range.start.line).unwrap_or(usize::MAX),
        range: converted,
    });
}

/// Reduce the three definition result shapes to targets.
pub(super) fn definition_outcome(
    ticket: &Ticket,
    root: &Path,
    found: Option<lsp::GotoDefinitionResponse>,
) -> RequestOutcome {
    let mut targets = Vec::new();
    match found {
        None => {}
        Some(lsp::GotoDefinitionResponse::Scalar(location)) => {
            targets.push(target(ticket, root, &location.uri, location.range));
        }
        Some(lsp::GotoDefinitionResponse::Array(locations)) => {
            for location in &locations {
                targets.push(target(ticket, root, &location.uri, location.range));
            }
        }
        Some(lsp::GotoDefinitionResponse::Link(links)) => {
            for link in &links {
                targets.push(target(
                    ticket,
                    root,
                    &link.target_uri,
                    link.target_selection_range,
                ));
            }
        }
    }
    if targets.is_empty() {
        return RequestOutcome::Empty;
    }
    return RequestOutcome::Locations(targets);
}

/// Convert a references answer;
///  `null` and an empty list are both an empty successful result.
pub(super) fn references_outcome(
    ticket: &Ticket,
    root: &Path,
    found: Option<Vec<lsp::Location>>,
) -> RequestOutcome {
    let mut targets = Vec::new();
    // `unwrap_or_default()` substitutes an empty list for `null`.
    for location in &found.unwrap_or_default() {
        targets.push(target(ticket, root, &location.uri, location.range));
    }
    if targets.is_empty() {
        return RequestOutcome::Empty;
    }
    return RequestOutcome::Locations(targets);
}

/// Render one legacy hover part;
///  a language-tagged part becomes a fenced code block.
fn marked(part: lsp::MarkedString) -> String {
    return match part {
        lsp::MarkedString::String(text) => text,
        lsp::MarkedString::LanguageString(code) => {
            format!("```{}\n{}\n```", code.language, code.value)
        }
    };
}

/// What:
///  Convert a hover answer.
///  The three content shapes become one text plus a Markdown flag.
/// Why:
///  The interface shows text;
///  it does not interpret the protocol's legacy shapes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hoverOutcome(ticket: Ticket, found: Hover | null): RequestOutcome
/// ```
pub(super) fn hover_outcome(ticket: &Ticket, found: Option<lsp::Hover>) -> RequestOutcome {
    let Some(hover) = found else {
        return RequestOutcome::Empty;
    };
    let (text, markdown) = match hover.contents {
        lsp::HoverContents::Markup(content) => {
            (content.value, content.kind == lsp::MarkupKind::Markdown)
        }
        lsp::HoverContents::Scalar(part) => (marked(part), true),
        lsp::HoverContents::Array(parts) => {
            let mut rendered: Vec<String> = Vec::new();
            for part in parts {
                rendered.push(marked(part));
            }
            // `join` concatenates the parts with a blank line between them.
            (rendered.join("\n\n"), true)
        }
    };
    if text.trim().is_empty() {
        return RequestOutcome::Empty;
    }
    let range = match hover.range {
        Some(found_range) => from_lsp_range(&ticket.text, found_range, ticket.encoding),
        None => None,
    };
    return RequestOutcome::Hover(HoverText {
        text,
        markdown,
        range,
    });
}
