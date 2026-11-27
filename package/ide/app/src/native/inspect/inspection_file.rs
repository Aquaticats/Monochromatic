//! The inspection file's records:
//!  one hint,
//!  one diagnostic,
//!  the delays they arrive after,
//!  and the Language module's records the file is read into.

/// A malformed inspection file stops startup with a message naming it.
use anyhow::{
    Result,
    bail,
};
/// The snapshot records a language server would produce.
use ide_app::language::{
    diagnostics::{
        Diagnostic,
        DiagnosticsSnapshot,
        Freshness,
        Severity,
        SourceGroup,
    },
    hints::{
        HintKind,
        HintsSnapshot,
        InlayHint,
    },
    identity::{
        DocumentStamp,
        ServerIdentity,
    },
};
/// What:
///  `Deserialize` lets serde build a record from JSON.
/// Why:
///  The inspection file is JSON written by the probe script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = JSON.parse(text) as InspectionFile;
/// ```
use serde::Deserialize;
/// Shared snapshot pointers.
use std::sync::Arc;

/// One hint in the inspection file.
#[derive(Deserialize)]
pub(super) struct FileHint {
    /// Character offset the label is drawn for.
    position: usize,
    /// Label text as a server would send it.
    label: String,
    /// What:
    ///  `Option<String>` is the kind word,
    ///  `type` or `parameter`,
    ///  or nothing.
    /// Why:
    ///  Real servers send both kinds;
    ///  an omitted kind is a real protocol case too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind?: string;
    /// ```
    kind: Option<String>,
}

/// One diagnostic in the inspection file.
#[derive(Deserialize)]
pub(super) struct FileProblem {
    /// Source name the problem is grouped by.
    source: String,
    /// First marked character.
    start: usize,
    /// Character after the marked range.
    end: usize,
    /// The severity word,
    ///  or nothing for an omitted severity,
    ///  which is a real protocol case worth showing.
    severity: Option<String>,
    /// Rule or error code.
    code: Option<String>,
    /// Message,
    ///  possibly several lines.
    message: String,
}

/// What:
///  Milliseconds after which hints and diagnostics arrive;
///  `u64` is an unsigned 64-bit integer
///       (sibling `u32`),
///  the type `Duration::from_millis` takes.
///  `Copy` lets the pair be passed by value.
/// Why:
///  Servers answer hints and diagnostics separately and at different times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Delays = { hints: number; diagnostics: number };
/// ```
#[derive(Clone, Copy, Deserialize)]
pub(super) struct Delays {
    /// Milliseconds until the hints arrive.
    pub(super) hints: u64,
    /// Milliseconds until the diagnostics arrive.
    pub(super) diagnostics: u64,
}

/// The whole inspection file;
///  `#[serde(default)]` lets a list be left out.
#[derive(Deserialize)]
pub(super) struct InspectionFile {
    /// Hint labels.
    #[serde(default)]
    pub(super) hints: Vec<FileHint>,
    /// Diagnostics.
    #[serde(default)]
    pub(super) diagnostics: Vec<FileProblem>,
    /// Arrival times after startup;
    ///  absent for arrival at once.
    pub(super) delays: Option<Delays>,
    /// Arrival times after each external reload;
    ///  absent when a reload leaves the text without annotations.
    pub(super) reload_delays: Option<Delays>,
}

/// Translate a severity word;
///  an unknown word is an error in the inspection file.
pub(super) fn severity(word: Option<&str>) -> Result<Option<Severity>> {
    let Some(known) = word else {
        return Ok(None);
    };
    if known == "error" {
        return Ok(Some(Severity::Error));
    }
    if known == "warning" {
        return Ok(Some(Severity::Warning));
    }
    if known == "information" {
        return Ok(Some(Severity::Information));
    }
    if known == "hint" {
        return Ok(Some(Severity::Hint));
    }
    bail!("Unknown severity {known:?} in the inspection annotation file");
}

/// Translate a hint kind word;
///  an unknown word is an error in the inspection file.
pub(super) fn kind(word: Option<&str>) -> Result<Option<HintKind>> {
    let Some(known) = word else {
        return Ok(None);
    };
    if known == "type" {
        return Ok(Some(HintKind::Type));
    }
    if known == "parameter" {
        return Ok(Some(HintKind::Parameter));
    }
    bail!("Unknown hint kind {known:?} in the inspection annotation file");
}

/// What:
///  The file's records in the Language module's shapes,
///  without a stamp:
///  hints in position order and
///       diagnostics grouped by source.
/// Why:
///  The same records are stamped for the initial text and again for every reloaded text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Records = { hints: InlayHint[]; groups: SourceGroup[] };
/// ```
pub(super) struct Records {
    /// Hints in position order.
    pub(super) hints: Vec<InlayHint>,
    /// Diagnostics grouped by source.
    pub(super) groups: Vec<SourceGroup>,
}

/// Orders hints by the character offset they are drawn before.
fn by_position(
    left: &InlayHint,
    right: &InlayHint,
) -> std::cmp::Ordering {
    return left.position.cmp(&right.position);
}

/// Convert the parsed file into the Language module's records.
pub(super) fn records(file: InspectionFile) -> Result<Records> {
    let server = ServerIdentity {
        name: "inspection".to_string(),
        instance: 0,
    };
    let mut hints = Vec::new();
    for hint in file.hints {
        hints.push(InlayHint {
            position: hint.position,
            label: hint.label,
            kind: kind(hint.kind.as_deref())?,
            padding_left: false,
            padding_right: false,
            server: server.clone(),
        });
    }
    hints.sort_by(by_position);
    let mut groups: Vec<SourceGroup> = Vec::new();
    for problem in file.diagnostics {
        let item = Diagnostic {
            start: problem.start,
            end: problem.end,
            severity: severity(problem.severity.as_deref())?,
            code: problem.code,
            message: problem.message,
            server: server.clone(),
            freshness: Freshness::Versioned,
        };
        // What: `iter_mut().find(...)` lends the group with this source for appending, if one exists.
        // Why: Problems are grouped by source, as the Language module groups them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const group = groups.find(known => known.source === problem.source);
        // ```
        let mut found_group = None;
        for (index, known) in groups.iter().enumerate() {
            if known.source == problem.source {
                found_group = Some(index);
                break;
            }
        }
        if let Some(index) = found_group {
            groups[index].items.push(item);
        } else {
            groups.push(SourceGroup {
                source: problem.source,
                items: vec![item],
            });
        }
    }
    return Ok(Records {
        hints,
        groups,
    });
}

/// The records' hints as a snapshot of the text `stamp` that answers for all of its `lines`.
pub(super) fn hint_snapshot(
    records: &Records,
    stamp: DocumentStamp,
    lines: usize,
) -> Arc<HintsSnapshot> {
    return Arc::new(HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: lines,
        hints: records.hints.clone(),
    });
}

/// The records' diagnostics as a snapshot of the text `stamp`.
pub(super) fn problem_snapshot(
    records: &Records,
    stamp: DocumentStamp,
) -> Arc<DiagnosticsSnapshot> {
    return Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: records.groups.clone(),
    });
}
