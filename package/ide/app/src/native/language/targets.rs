//! Places a definition or reference names,
//!  and how each one is opened.

/// The open path for files other than the displayed one,
///  the window,
///  source state,
///  and scrolling.
use crate::native::{
    AppWindow, State,
    navigation::{Navigation, request_jump},
    ui::ReferenceEntry,
    viewport,
};
/// A jump target is placed on the default side of a wrap offset.
use ide_app::document::ReadingAffinity;
/// Validated locations and their refusals.
use ide_app::language::reply::Target;
/// Generated list rows carry toolkit strings.
use slint::SharedString;
/// What:
///  `Rc<RefCell<T>>` is the shared,
///  borrow-checked owner;
///  `Path`/`PathBuf` are paths.
/// Why:
///  Placing the caret changes the shared source state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

/// What:
///  One openable place:
///  a canonical file,
///  whether it lies outside the project,
///  the
///       zero-based line,
///  and character offsets when the target text could be read.
/// Why:
///  The file-open path keeps it until the file is installed,
///  then puts the caret there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Jump = { path: string; outside: boolean; line: number; range?: [number, number] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::native) struct Jump {
    /// Canonical path of an existing regular file.
    pub(in crate::native) path: PathBuf,
    /// The file lies outside the project root.
    pub(in crate::native) outside: bool,
    /// Zero-based line the server named.
    pub(in crate::native) line: usize,
    /// Character offsets in that file's text,
    ///  when known.
    pub(in crate::native) range: Option<(usize, usize)>,
}

/// What:
///  Put the caret at the start of `jump` in the displayed document and scroll it into view.
///       `&Jump` lends the target.
/// Why:
///  Offsets older than the displayed text are clamped to it;
///  without offsets the line start
///      is used,
///  so a target always lands on its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function place(window: AppWindow, source: Shared<State>, jump: Jump): void
/// ```
pub(in crate::native) fn place(window: &AppWindow, source: &Rc<RefCell<State>>, jump: &Jump) {
    let mut current = source.borrow_mut();
    let text = current.document.text();
    let length = text.len_chars();
    let line = jump.line.min(text.len_lines().saturating_sub(1));
    // A let-chain: the offsets are used only when present and inside the displayed text.
    let (start, end) = if let Some((first, last)) = jump.range
        && first <= last
        && last <= length
    {
        (first, last)
    } else {
        let at = text.line_to_char(line);
        (at, at)
    };
    let mut position = current.document.position();
    position.anchor = start;
    position.head = start;
    position.affinity = ReadingAffinity::Downstream;
    current.document.select(position);
    // `drop` ends the mutable loan before scrolling borrows the state again.
    drop(current);
    tracing::debug!(path = %jump.path.display(), start, end, "placed the caret at a language target");
    viewport::reveal(window, source, start, end);
}

/// What:
///  Open one target.
///  `Err(String)` is the sentence to show for a target that cannot be
///       opened;
///  `navigation` opens a file other than the displayed one.
/// Why:
///  Unavailable locations are explained,
///  never dropped;
///  the open path itself reports read failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function open(window, source, navigation, target: Target): void // throws the sentence
/// ```
pub(super) fn open(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    target: &Target,
) -> Result<(), String> {
    let found = match target {
        Target::Open(found) => found,
        Target::Unavailable { uri, refusal } => {
            // `Err(...)` hands the sentence back for the caller to show.
            return Err(super::message::unavailable(uri, &refusal.to_string()));
        }
    };
    let jump = Jump {
        path: found.path.clone(),
        outside: found.outside_project,
        line: found.line,
        range: found.range,
    };
    // A navigation callback holding the state finishes first; the target is then not opened.
    let Ok(mut held) = navigation.try_borrow_mut() else {
        return Err("The project is busy opening another file. Try again.".to_string());
    };
    if let Err(error) = request_jump(window, source, &mut held, jump) {
        tracing::warn!(?error, "cannot open a language target");
        return Err(format!("{error:#}"));
    }
    // `Ok(())` reports success without a value.
    return Ok(());
}

/// What:
///  List rows for targets:
///  a path relative to the project and a one-based line,
///  or the
///       server's address;
///  the detail marks outside-project and unopenable entries.
/// Why:
///  The list shows editord's `path:line` labels;
///  special entries say why they are special.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entries(targets: Target[], root: string): ReferenceEntry[]
/// ```
pub(super) fn entries(targets: &[Target], root: &Path) -> Vec<ReferenceEntry> {
    let mut rows = Vec::new();
    for target in targets {
        let (label, detail) = match target {
            Target::Open(found) => {
                // `strip_prefix` shortens project paths; outside paths stay whole.
                let shown = found.path.strip_prefix(root).unwrap_or(&found.path);
                let detail = if found.outside_project {
                    "Outside project"
                } else {
                    ""
                };
                (format!("{}:{}", shown.display(), found.line + 1), detail)
            }
            Target::Unavailable { uri, .. } => (uri.clone(), "Cannot open"),
        };
        rows.push(ReferenceEntry {
            label: SharedString::from(label),
            detail: SharedString::from(detail),
        });
    }
    return rows;
}
