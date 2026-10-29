//! Inspection-only annotation injection:
//!  debug builds started with `IDE_INSPECT_ANNOTATIONS` naming a JSON file
//! install its hints and diagnostics for the displayed text,
//!  so nested-compositor frames can show them
//! without a language server.
//!  Release builds do not contain this module,
//!  and without the variable it does nothing.
//! A window started this way runs without the language worker (`servers_wanted` in the `start` module):
//!  a worker
//! without servers still publishes "no problems" for the displayed text,
//!  which would replace the file's diagnostics.
//!
//! The file holds `{ "hints": [{ "position", "label", "kind" }], "diagnostics": [{ "source", "start", "end",
//! "severity", "code", "message" }], "delays": { "hints", "diagnostics" }, "reload_delays": { "hints",
//! "diagnostics" } }` with character offsets of the initial file;
//!  `severity` is `error`,
//!  `warning`,
//! `information`,
//!  `hint`,
//!  or absent,
//!  and `kind` is `type`,
//!  `parameter`,
//!  or absent.
//!
//! Without `delays` and `reload_delays` both snapshots are installed at once for the initial text;
//!  an external
//! change or a file switch makes them stale,
//!  and they disappear exactly as stale server snapshots do.
//! `delays` gives milliseconds after startup at which hints and diagnostics arrive separately,
//!  as from a server
//! that answers late.
//!  `reload_delays` makes them arrive again,
//!  that many milliseconds after each external
//! reload,
//!  stamped for the new text;
//!  the offsets are reused,
//!  so the change must not move the annotated text.

/// The parent's state,
///  the production setter,
///  and the production repaint.
use super::{AppWindow, State, annotate, render};
/// A malformed inspection file stops startup with a message naming it.
use anyhow::{Result, bail};
/// The snapshot records a language server would produce.
use ide_app::language::{
    diagnostics::{Diagnostic, DiagnosticsSnapshot, Freshness, Severity, SourceGroup},
    hints::{HintKind, HintsSnapshot, InlayHint},
    identity::{DocumentStamp, ServerIdentity},
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
/// Timer callbacks and weak window references belong to the toolkit event loop.
use slint::{ComponentHandle, Timer, TimerMode};
/// Shared UI state,
///  shared snapshot pointers,
///  and the clock delayed arrivals are measured on.
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

/// One hint in the inspection file.
#[derive(Deserialize)]
struct FileHint {
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
struct FileProblem {
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
struct Delays {
    /// Milliseconds until the hints arrive.
    hints: u64,
    /// Milliseconds until the diagnostics arrive.
    diagnostics: u64,
}

/// The whole inspection file;
///  `#[serde(default)]` lets a list be left out.
#[derive(Deserialize)]
struct InspectionFile {
    /// Hint labels.
    #[serde(default)]
    hints: Vec<FileHint>,
    /// Diagnostics.
    #[serde(default)]
    diagnostics: Vec<FileProblem>,
    /// Arrival times after startup;
    ///  absent for arrival at once.
    delays: Option<Delays>,
    /// Arrival times after each external reload;
    ///  absent when a reload leaves the text without annotations.
    reload_delays: Option<Delays>,
}

/// Translate a severity word;
///  an unknown word is an error in the inspection file.
fn severity(word: Option<&str>) -> Result<Option<Severity>> {
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
fn kind(word: Option<&str>) -> Result<Option<HintKind>> {
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
struct Records {
    /// Hints in position order.
    hints: Vec<InlayHint>,
    /// Diagnostics grouped by source.
    groups: Vec<SourceGroup>,
}

/// Orders hints by the character offset they are drawn before.
fn by_position(left: &InlayHint, right: &InlayHint) -> std::cmp::Ordering {
    return left.position.cmp(&right.position);
}

/// Convert the parsed file into the Language module's records.
fn records(file: InspectionFile) -> Result<Records> {
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
    return Ok(Records { hints, groups });
}

/// The records' hints as a snapshot of the text `stamp` that answers for all of its `lines`.
fn hint_snapshot(records: &Records, stamp: DocumentStamp, lines: usize) -> Arc<HintsSnapshot> {
    return Arc::new(HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: lines,
        hints: records.hints.clone(),
    });
}

/// The records' diagnostics as a snapshot of the text `stamp`.
fn problem_snapshot(records: &Records, stamp: DocumentStamp) -> Arc<DiagnosticsSnapshot> {
    return Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: records.groups.clone(),
    });
}

/// What:
///  When the next hints and diagnostics are to arrive for the text `seen`;
///  `Option<Instant>` is a moment
///       or nothing once they arrived.
/// Why:
///  The timer installs each snapshot once,
///  when its moment has come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Schedule = { seen: DocumentStamp; hints?: number; diagnostics?: number };
/// ```
struct Schedule {
    /// The displayed text the moments belong to.
    seen: DocumentStamp,
    /// When the hints arrive.
    hints: Option<Instant>,
    /// When the diagnostics arrive.
    diagnostics: Option<Instant>,
}

/// The moments `delays` after now.
fn moments(delays: Delays) -> (Option<Instant>, Option<Instant>) {
    let now = Instant::now();
    // `Some(...)` records each arrival as pending.
    return (
        Some(now + Duration::from_millis(delays.hints)),
        Some(now + Duration::from_millis(delays.diagnostics)),
    );
}

/// What:
///  One timer tick:
///  schedule arrivals for a reloaded text,
///  and install what is due through the entry
///       points the language poll uses;
///  the answer says whether the window must render.
/// Why:
///  Hints and diagnostics arrive separately and late,
///  exactly as the poll would deliver them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tick(current: State, records: Records, schedule: Schedule, reload?: Delays): boolean;
/// ```
fn tick(
    current: &mut State,
    records: &Records,
    schedule: &mut Schedule,
    reload: Option<Delays>,
) -> bool {
    let stamp = annotate::displayed(current);
    if stamp != schedule.seen {
        schedule.seen = stamp;
        schedule.hints = None;
        schedule.diagnostics = None;
        if let Some(delays) = reload {
            (schedule.hints, schedule.diagnostics) = moments(delays);
        }
    }
    let now = Instant::now();
    let lines = current.document.text().len_lines();
    // Destructuring lends the store for change while the displayed text is only read.
    let State {
        annotations,
        document,
        ..
    } = current;
    let mut accepted = false;
    // False once the hints arrived; otherwise whether their moment has come.
    let hints_due = match schedule.hints {
        Some(due) => now >= due,
        None => false,
    };
    if hints_due {
        schedule.hints = None;
        let snapshot = hint_snapshot(records, stamp, lines);
        accepted = annotations.accept_hints(stamp, document.text(), snapshot) || accepted;
    }
    let diagnostics_due = match schedule.diagnostics {
        Some(due) => now >= due,
        None => false,
    };
    if diagnostics_due {
        schedule.diagnostics = None;
        let snapshot = problem_snapshot(records, stamp);
        accepted = annotations.accept_diagnostics(stamp, document.text(), snapshot) || accepted;
    }
    return accepted;
}

/// What:
///  Read `IDE_INSPECT_ANNOTATIONS` and install its snapshots:
///  at once through the production setter,
///  or
///       by a timer when the file asks for delays.
///  `Result<Option<Timer>>` is the timer to keep alive,
///  nothing
///       when none is needed,
///  or an error naming the file.
/// Why:
///  Nested-compositor frames need hints and diagnostics on a fixture without a real server,
///  also arriving
///      late and returning after a reload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inject(window: AppWindow, state: Shared<State>): Timer | undefined;
/// ```
pub(super) fn inject(window: &AppWindow, state: &Rc<RefCell<State>>) -> Result<Option<Timer>> {
    // What: `var_os` reads the variable without requiring UTF-8; `let ... else` returns when it is unset.
    // Why: Without the variable the application behaves exactly as without this module.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const path = process.env.IDE_INSPECT_ANNOTATIONS; if (path === undefined) return;
    // ```
    let Some(path) = std::env::var_os("IDE_INSPECT_ANNOTATIONS") else {
        return Ok(None);
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(read) => read,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot read the inspection annotation file {}",
                path.display()
            )));
        }
    };
    let file: InspectionFile = match serde_json::from_str(&text) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot parse the inspection annotation file {}",
                path.display()
            )));
        }
    };
    let delays = file.delays;
    let reload = file.reload_delays;
    let records = records(file)?;
    let stamp = annotate::displayed(&state.borrow());
    let lines = state.borrow().document.text().len_lines();
    tracing::info!(
        hints = records.hints.len(),
        groups = records.groups.len(),
        ?stamp,
        delayed = delays.is_some(),
        follows_reloads = reload.is_some(),
        "inspection annotations read"
    );
    let mut schedule = Schedule {
        seen: stamp,
        hints: None,
        diagnostics: None,
    };
    if let Some(wanted) = delays {
        (schedule.hints, schedule.diagnostics) = moments(wanted);
    } else {
        annotate::set_annotations(
            window,
            state,
            Some(hint_snapshot(&records, stamp, lines)),
            Some(problem_snapshot(&records, stamp)),
        );
    }
    if delays.is_none() && reload.is_none() {
        return Ok(None);
    }
    let shared = Rc::clone(state);
    let weak = window.as_weak();
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
        let Some(active) = weak.upgrade() else {
            return;
        };
        let accepted = tick(&mut shared.borrow_mut(), &records, &mut schedule, reload);
        if accepted {
            render(&active, &shared);
        }
    });
    return Ok(Some(timer));
}
