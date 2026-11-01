//! Inlay hints for the displayed lines:
//!  which range to ask for and how labels are shaped.

/// Hints name the text and the server process they were computed for.
use super::identity::{DocumentStamp, ServerIdentity};
/// Hint positions pass through the shared converter with its line-bound check.
use super::position::from_lsp_position;
/// What:
///  `Rope` is Helix's character-indexed text buffer.
/// Why:
///  Hint positions are converted against the text of the revision that was asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The protocol's hint record and the column unit of the answering server.
use helix_lsp::{OffsetEncoding, lsp};

/// What kind of information a hint adds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HintKind = 'type' | 'parameter';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintKind {
    /// An inferred type,
    ///  for example `: u32`.
    Type,
    /// A parameter name at a call site,
    ///  for example `width:`.
    Parameter,
}

/// One hint,
///  reduced to what a read-only view draws.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InlayHint = { position: number; label: string; kind?: HintKind;
///                    paddingLeft: boolean; paddingRight: boolean; server: ServerIdentity };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlayHint {
    /// Character offset the hint is drawn before.
    ///  `usize` is the index type Helix ropes use.
    pub position: usize,
    /// Label text;
    ///  label parts are joined,
    ///  and their commands and locations are not followed.
    pub label: String,
    /// What:
    ///  `Option<HintKind>` is "a kind,
    ///  or nothing".
    /// Why:
    ///  The protocol lets a server omit the kind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind?: HintKind;
    /// ```
    pub kind: Option<HintKind>,
    /// The server asks for a space before the label.
    pub padding_left: bool,
    /// The server asks for a space after the label.
    pub padding_right: bool,
    /// Which server process produced the hint.
    pub server: ServerIdentity,
}

/// The visible part of the file,
///  as the interface thread reports it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HintWindow = { firstLine: number; visibleLines: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HintWindow {
    /// Zero-based first visible source line.
    pub first_line: usize,
    /// Number of source lines the view shows.
    pub visible_lines: usize,
}

/// The latest-value hints the interface thread polls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HintsSnapshot = { stamp: DocumentStamp; firstLine: number; lastLine: number; hints: InlayHint[] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HintsSnapshot {
    /// The displayed text every position in this snapshot refers to.
    pub stamp: DocumentStamp,
    /// First line the hints were requested for.
    pub first_line: usize,
    /// Line the request ended before;
    ///  it may equal the number of lines.
    pub last_line: usize,
    /// What:
    ///  `Vec<InlayHint>` is a growable list (siblings:
    ///  fixed `[T; N]`,
    ///  borrowed `&[T]`).
    /// Why:
    ///  Hints of every answering server are merged and ordered by position.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hints: InlayHint[];
    /// ```
    pub hints: Vec<InlayHint>,
}

/// What:
///  Compute the line range to request:
///  one view height before the first visible line and
///       two view heights after it,
///  limited to the text.
///  The parentheses form a tuple.
/// Why:
///  This is Helix's own rule (`helix-term/src/commands/lsp.rs`):
///  some scrolling stays
///      covered while the request remains far smaller than the whole file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requestLines(window: HintWindow, lineCount: number): [number, number] {
///   const first = Math.max(0, window.firstLine - window.visibleLines);
///   const last = Math.min(lineCount, window.firstLine + 2 * window.visibleLines);
///   return [Math.min(first, last), last];
/// }
/// ```
pub fn request_lines(window: HintWindow, line_count: usize) -> (usize, usize) {
    // What: `saturating_sub` subtracts and stops at zero instead of wrapping below it;
    //       `saturating_add` and `saturating_mul` stop at the largest value instead of overflowing.
    // Why: Unsigned integers cannot be negative, and a huge window must not overflow.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = Math.max(0, window.firstLine - window.visibleLines);
    // ```
    let first = window.first_line.saturating_sub(window.visible_lines);
    let last = window
        .first_line
        .saturating_add(window.visible_lines.saturating_mul(2))
        .min(line_count);
    return (first.min(last), last);
}

/// Translate the protocol's numeric hint kind into the application's names.
fn kind(raw: Option<lsp::InlayHintKind>) -> Option<HintKind> {
    // The trailing `?` returns `None` when the server sent no kind.
    let value = raw?;
    if value == lsp::InlayHintKind::TYPE {
        // `Some(...)` is the "value present" variant of `Option`.
        return Some(HintKind::Type);
    }
    if value == lsp::InlayHintKind::PARAMETER {
        return Some(HintKind::Parameter);
    }
    // `None` is the "nothing" variant: an unknown kind is treated like an omitted one.
    return None;
}

/// What:
///  Flatten a label into plain text.
///  `match` unpacks the protocol's "string or parts" union.
/// Why:
///  A reader shows the text only;
///  part commands,
///  locations,
///  and tooltips are not followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function label(raw: string | LabelPart[]): string {
///   return typeof raw === 'string' ? raw : raw.map(part => part.value).join('');
/// }
/// ```
fn label(raw: lsp::InlayHintLabel) -> String {
    return match raw {
        lsp::InlayHintLabel::String(text) => text,
        lsp::InlayHintLabel::LabelParts(parts) => {
            // `String::new()` creates an empty owned text buffer; `mut` allows appending to it.
            let mut joined = String::new();
            for part in parts {
                // `&part.value` lends the part's text to the append without moving it.
                joined.push_str(&part.value);
            }
            joined
        }
    };
}

/// What:
///  Shape a server's answer for drawing.
///  `raw` is moved in (the caller gives it up);
///       `&Rope` and `&ServerIdentity` are lent read-only.
/// Why:
///  Hints on a line the text does not have are dropped,
///  because Helix's converter would
///      otherwise place them at the end of the file;
///  text edits and resolve data are ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shape(raw: lsp.InlayHint[], text: Rope, encoding: Encoding, server: ServerIdentity): InlayHint[]
/// ```
pub fn shape(
    raw: Vec<lsp::InlayHint>,
    text: &Rope,
    encoding: OffsetEncoding,
    server: &ServerIdentity,
) -> Vec<InlayHint> {
    let mut shaped: Vec<InlayHint> = Vec::new();
    for hint in raw {
        // `let Some(x) = option else { ... }` binds the converted offset or skips this hint.
        let Some(position) = from_lsp_position(text, hint.position, encoding) else {
            tracing::debug!(line = hint.position.line, server = %server.name, "dropped an inlay hint outside the displayed text");
            continue;
        };
        shaped.push(InlayHint {
            position,
            label: label(hint.label),
            kind: kind(hint.kind),
            // `unwrap_or(false)` substitutes "no padding" when the server omitted the flag.
            padding_left: hint.padding_left.unwrap_or(false),
            padding_right: hint.padding_right.unwrap_or(false),
            // `clone` copies the identity so each hint owns its own value.
            server: server.clone(),
        });
    }
    return shaped;
}

/// Request ranges and label shaping are exercised without a server.
#[cfg(test)]
#[path = "hints_tests.rs"]
mod tests;
