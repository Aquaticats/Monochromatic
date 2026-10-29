//! Inlay hints and diagnostics enter the window here.
//!  The renderer paints only snapshots stamped with the
//! displayed file generation and revision,
//!  and only their part inside the materialized rows.
//!
//! Snapshots live in `State::annotations`,
//!  an `ide_app::annotation::Annotations`.
//!  There are two ways in:
//!
//! - A poll that already holds the state mutably stores each polled snapshot with
//!   `annotations.accept_hints(displayed, text, snapshot)` or `accept_diagnostics(displayed, text, snapshot)`,
//!   and calls `render` once after releasing the state when either answered `true`.
//!   Accepting draws nothing by itself.
//! - [`set_annotations`] replaces both snapshots at once and renders;
//!    tests and the inspection path use it.
//!
//! Either way the render first brings the vertical mapping up to date (`rows::refresh`),
//!  keeping the view still
//! where rows appeared above it,
//!  then takes the visible part;
//!  the frame stamp repaints the source image only
//! when that part or its inks changed.
//!  Reloads and file switches need no call:
//!  a held snapshot stops matching
//! the displayed stamp and is no longer painted.

/// The parent's rendering entry point,
///  used only by `set_annotations`,
///  which exists only in tests and
/// debug builds;
///  release builds would report the import as unused.
#[cfg(any(test, debug_assertions))]
use super::render;
/// The parent's window and state.
use super::{AppWindow, State};
/// The visible subset and the text of one problem.
use ide_app::annotation::{Visible, describe};
/// Annotation inks and the positioned frame the raster paints.
use ide_app::annotation_layout::AnnotationColors;
/// The stamp naming the displayed text.
use ide_app::language::identity::DocumentStamp;
/// Snapshot records,
///  taken only by `set_annotations` (tests and debug builds).
#[cfg(any(test, debug_assertions))]
use ide_app::language::{diagnostics::DiagnosticsSnapshot, hints::HintsSnapshot};
/// Window property access.
use slint::SharedString;
/// What:
///  `Arc` is the shared snapshot pointer.
/// Why:
///  The Language module hands snapshots out as `Arc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::sync::Arc;
/// What:
///  `Rc<RefCell<State>>` is the UI thread's shared source state,
///  taken only by `set_annotations`.
/// Why:
///  The window state is single-threaded;
///  tests and debug builds alone call that function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
#[cfg(any(test, debug_assertions))]
use std::{cell::RefCell, rc::Rc};

/// What:
///  At most this many problems are named in the accessible description;
///  `usize` is the count type
///       (siblings `u32`,
///  `u64`).
/// Why:
///  A pathological pile of diagnostics at one character must not produce a description without end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DESCRIBED_PROBLEMS = 8;
/// ```
const DESCRIBED_PROBLEMS: usize = 8;

/// What:
///  Replace both snapshots at once and render;
///  `Option<Arc<...>>` is a shared snapshot or nothing.
///       Snapshots for another file generation or revision are kept but never painted.
/// Why:
///  Tests and the inspection path install a complete set in one call.
///  Handing back the same snapshots
///      does nothing;
///  a change repaints only when the visible annotations differ,
///  which the frame stamp decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setAnnotations(window: AppWindow, state: Shared<State>,
///   hints?: HintsSnapshot, diagnostics?: DiagnosticsSnapshot): void;
/// ```
///
/// Built only for tests and debug builds (`#[cfg(any(test, debug_assertions))]`),
///  because its only
/// production caller,
///  the inspection path,
///  is itself debug-only;
///  a release build would otherwise
/// report the function as unused.
#[cfg(any(test, debug_assertions))]
pub(super) fn set_annotations(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    hints: Option<Arc<HintsSnapshot>>,
    diagnostics: Option<Arc<DiagnosticsSnapshot>>,
) {
    let mut current = state.borrow_mut();
    if current.annotations.holds(&hints, &diagnostics) {
        return;
    }
    // What: `as_ref().map(...)` reads a field of the snapshot when there is one; `?stamp` logs the debug form.
    // Why: Which text a snapshot names explains why it is or is not painted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // logger.debug('annotation snapshots installed', { hints: hints?.stamp, diagnostics: diagnostics?.stamp });
    // ```
    let mut hint_stamp = None;
    if let Some(snapshot) = hints.as_ref() {
        hint_stamp = Some(snapshot.stamp);
    }
    let mut diagnostic_stamp = None;
    if let Some(snapshot) = diagnostics.as_ref() {
        diagnostic_stamp = Some(snapshot.stamp);
    }
    let shown = displayed(&current);
    tracing::debug!(
        hints = ?hint_stamp,
        diagnostics = ?diagnostic_stamp,
        displayed = ?shown,
        "annotation snapshots installed"
    );
    // What: Destructuring `&mut State` lends two fields separately.
    // Why: The store is replaced while the displayed text is only read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { annotations, document } = current;
    // ```
    let State {
        annotations,
        document,
        ..
    } = &mut *current;
    annotations.replace(shown, document.text(), hints, diagnostics);
    drop(current);
    render(window, state);
}

/// The stamp of the text the window displays:
///  its file generation and content revision.
pub(super) fn displayed(current: &State) -> DocumentStamp {
    return DocumentStamp {
        file: current.file_generation,
        revision: current.document.revision(),
    };
}

/// Convert a toolkit color to raster bytes without losing alpha.
fn rgba(color: slint::Color) -> [u8; 4] {
    return [color.red(), color.green(), color.blue(), color.alpha()];
}

/// Annotation inks of the current scheme,
///  read from the window so every painter agrees.
pub(super) fn colors(window: &AppWindow) -> AnnotationColors {
    return AnnotationColors {
        hint: rgba(window.get_hint_ink().color()),
        error: rgba(window.get_error_ink()),
        warning: rgba(window.get_warning_ink()),
        information: rgba(window.get_information_ink()),
        suggestion: rgba(window.get_suggestion_ink()),
    };
}

/// The index of the first block that stands at or below line `first`,
///  found by binary search over blocks sorted by line.
fn first_block_from(blocks: &[Arc<ide_app::virtual_row::Block>], first: usize) -> usize {
    let mut low = 0;
    let mut high = blocks.len();
    while low < high {
        let middle = low + (high - low) / 2;
        if blocks[middle].line < first {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low;
}

/// What:
///  The annotations of the materialized rows:
///  their blocks of virtual rows and the diagnostics touching
///       them,
///  or nothing for stale snapshots;
///  `Arc` lets the frame stamp and the layout share one list.
/// Why:
///  Computed on every render with binary searches,
///  so an unchanged visible part reuses the image.
///      `State::blocks` must be current,
///  which `rows::refresh` sees to at the start of every render.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function visible(current: State, bounds: TileBounds): Visible;
/// ```
pub(super) fn visible(current: &State, bounds: super::tile::TileBounds) -> Arc<Visible> {
    let last = bounds.first.saturating_add(bounds.count);
    // A binary search finds the first block at or below the first materialized line.
    let from = first_block_from(&current.blocks, bounds.first);
    let mut blocks = Vec::new();
    for block in &current.blocks[from..] {
        if block.line >= last {
            break;
        }
        // `Arc::clone` copies the pointer to the shared block, not its rows.
        blocks.push(Arc::clone(block));
    }
    let marks = current.annotations.marks(
        displayed(current),
        current.document.text(),
        bounds.first,
        last,
    );
    return Arc::new(Visible { blocks, marks });
}

/// What:
///  The problems at the caret as one text,
///  worst first,
///  each spelled out in full;
///  empty when the caret
///       touches no diagnostic.
/// Why:
///  The source view's accessible description starts with it,
///  so assistive technology reads every problem
///      at the caret completely,
///  also one whose message rows are cut.
///  Every render recomputes it,
///  because any
///      caret movement can enter or leave a range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function problems(current: State): string;
/// ```
pub(super) fn problems(current: &State) -> String {
    let head = current.document.position().head;
    let found = current.annotations.at(displayed(current), head);
    let mut lines = Vec::new();
    for problem in found.iter().take(DESCRIBED_PROBLEMS) {
        lines.push(describe(problem));
    }
    if found.len() > DESCRIBED_PROBLEMS {
        lines.push(format!("{} more", found.len() - DESCRIBED_PROBLEMS));
    }
    return lines.join("\n");
}

/// Hand the caret's problems to the window;
///  an unchanged text leaves the toolkit property alone.
pub(super) fn present_problems(window: &AppWindow, text: String) {
    let shared = SharedString::from(text);
    if window.get_caret_problems() != shared {
        window.set_caret_problems(shared);
    }
}
