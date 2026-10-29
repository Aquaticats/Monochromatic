//! Diagnostics for the displayed file:
//!  what the interface thread sees,
//!  grouped by source.

/// Each stored set names the text and the server process it was accepted for.
use super::identity::{DocumentStamp, ServerIdentity};
/// Server ranges become character offsets only through the shared converter.
use super::position::from_lsp_range;
/// What:
///  `Rope` is Helix's character-indexed text buffer.
/// Why:
///  Ranges are converted against the text of the revision the set is stamped with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The protocol's diagnostic record and the column unit of the server that sent it.
use helix_lsp::{OffsetEncoding, lsp};
/// What:
///  `Duration` is a time span.
/// Why:
///  The unversioned-diagnostics hold has a fixed fallback delay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Duration = number; // milliseconds
/// ```
use std::time::Duration;

/// How long an unversioned-diagnostics hold lasts for a server that answers no request after a
/// reload.
///  Chosen,
///  not measured:
///  longer than the answers observed in the integration spike for
/// the first request after a change,
///  short enough that diagnostics do not stay hidden.
///  Public so
/// tests can tell whether an observation fell inside the hold.
pub const HOLD_FALLBACK: Duration = Duration::from_secs(2);

/// The store that decides which pushed and pulled sets are current.
mod store;
/// The worker owns one store.
pub(crate) use store::DiagnosticStore;

/// What:
///  A closed set of four names,
///  mirroring the protocol's numeric severities.
/// Why:
///  The reader styles a problem by severity and must not depend on protocol numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Severity = 'error' | 'warning' | 'information' | 'hint';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// The code is wrong.
    Error,
    /// The code is suspicious.
    Warning,
    /// A neutral remark.
    Information,
    /// A suggestion.
    Hint,
}

/// How a diagnostic set was shown to be about the displayed revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Freshness = 'versioned' | 'unversioned' | 'pulled';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    /// Pushed with a document version equal to the displayed revision's version.
    Versioned,
    /// Pushed without a version;
    ///  accepted by the conservative hold rule and never version-checked.
    Unversioned,
    /// Answer to a request this application sent for the displayed revision.
    Pulled,
}

/// One problem,
///  in character offsets of the snapshot's revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Diagnostic = { start: number; end: number; severity?: Severity; code?: string;
///                     message: string; server: ServerIdentity; freshness: Freshness };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// First character of the marked range.
    ///  `usize` is the index type Helix ropes use.
    pub start: usize,
    /// Character after the marked range;
    ///  equal to `start` for a point.
    pub end: usize,
    /// What:
    ///  `Option<Severity>` is "a severity,
    ///  or nothing".
    /// Why:
    ///  The protocol lets a server omit the severity.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// severity?: Severity;
    /// ```
    pub severity: Option<Severity>,
    /// The server's rule or error code as text,
    ///  for example `E0308` or `2322`.
    pub code: Option<String>,
    /// The server's message.
    ///  `String` owns its text (sibling:
    ///  borrowed `&str`).
    pub message: String,
    /// Which server process reported it.
    pub server: ServerIdentity,
    /// Whether the set was version-checked,
    ///  pulled,
    ///  or accepted unversioned.
    pub freshness: Freshness,
}

/// All problems one source reported,
///  for example `rustc` or `ts`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceGroup = { source: string; items: Diagnostic[] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceGroup {
    /// The `source` the server named;
    ///  empty when it named none.
    pub source: String,
    /// What:
    ///  `Vec<Diagnostic>` is a growable list (siblings:
    ///  fixed `[T; N]`,
    ///  borrowed `&[T]`).
    /// Why:
    ///  The number of problems is known only at run time.
    ///  Items are ordered by position.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// items: Diagnostic[];
    /// ```
    pub items: Vec<Diagnostic>,
}

/// The latest-value diagnostics the interface thread polls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DiagnosticsSnapshot = { stamp: DocumentStamp; groups: SourceGroup[] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsSnapshot {
    /// The displayed text every offset in this snapshot refers to.
    pub stamp: DocumentStamp,
    /// Groups ordered by source name.
    pub groups: Vec<SourceGroup>,
}

/// Translate the protocol's numeric severity into the application's names.
fn severity(raw: Option<lsp::DiagnosticSeverity>) -> Option<Severity> {
    // What: The trailing `?` returns `None` from this function when the server sent no severity.
    // Why: An absent severity stays absent instead of being guessed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (raw === undefined) return undefined;
    // ```
    let value = raw?;
    if value == lsp::DiagnosticSeverity::ERROR {
        // `Some(...)` is the "value present" variant of `Option`.
        return Some(Severity::Error);
    }
    if value == lsp::DiagnosticSeverity::WARNING {
        return Some(Severity::Warning);
    }
    if value == lsp::DiagnosticSeverity::INFORMATION {
        return Some(Severity::Information);
    }
    if value == lsp::DiagnosticSeverity::HINT {
        return Some(Severity::Hint);
    }
    // `None` is the "nothing" variant: an unknown number is treated like an omitted severity.
    return None;
}

/// Render the protocol's number-or-text code as text.
fn code(raw: Option<&lsp::NumberOrString>) -> Option<String> {
    // What: `match` unpacks the tagged union; `to_string` and `clone` produce an owned `String`.
    // Why: The reader shows the code and never calculates with it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return raw === undefined ? undefined : String(raw);
    // ```
    return match raw? {
        lsp::NumberOrString::Number(number) => Some(number.to_string()),
        lsp::NumberOrString::String(text) => Some(text.clone()),
    };
}

/// What:
///  Convert one protocol diagnostic against `text`.
///  `&` parameters lend their values
///       read-only;
///  the result is nothing when the range starts on a line the text lacks.
/// Why:
///  A range that does not exist in the displayed text cannot be drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convert(raw, text, encoding, server, freshness): Diagnostic | undefined
/// ```
fn convert(
    raw: &lsp::Diagnostic,
    text: &Rope,
    encoding: OffsetEncoding,
    server: &ServerIdentity,
    freshness: Freshness,
) -> Option<Diagnostic> {
    let (start, end) = from_lsp_range(text, raw.range, encoding)?;
    return Some(Diagnostic {
        start,
        end,
        severity: severity(raw.severity),
        code: code(raw.code.as_ref()),
        message: raw.message.clone(),
        server: server.clone(),
        freshness,
    });
}

/// What:
///  Append one converted problem to the group of its source,
///  creating the group on first use.
///       `&mut Vec<SourceGroup>` lends the list for modification.
/// Why:
///  The reader aggregates by `source`,
///  so `rustc` and `rust-analyzer` stay apart even when
///      one server reports both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addToGroup(groups: SourceGroup[], source: string, item: Diagnostic): void {
///   let group = groups.find(known => known.source === source);
///   if (!group) { group = { source, items: [] }; groups.push(group); }
///   group.items.push(item);
/// }
/// ```
fn add_to_group(groups: &mut Vec<SourceGroup>, source: &str, item: Diagnostic) {
    // `iter_mut` walks the list handing out modifiable borrows of each group.
    for group in groups.iter_mut() {
        if group.source == source {
            group.items.push(item);
            return;
        }
    }
    groups.push(SourceGroup {
        source: source.to_string(),
        items: vec![item],
    });
}

/// Versioning,
///  the unversioned hold,
///  invalidation,
///  and aggregation are exercised without a server.
#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
