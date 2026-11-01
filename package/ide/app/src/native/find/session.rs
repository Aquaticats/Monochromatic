//! Find bar transitions:
//!  open,
//!  edit,
//!  step to a match,
//!  and close.

/// Session state and the identity synchronization shared with the polling timer.
use super::{Find, tick};
/// Source state,
///  the window,
///  the rendering boundary,
///  and scrolling a range into view.
use crate::native::{AppWindow, State, render, viewport};
/// A selected match is not a wrap-offset caret,
/// so it stands on the default side.
use ide_app::document::ReadingAffinity;
/// A match is a source character range,
///  the same unit as the reading selection.
use ide_app::find::FindRange;
/// Enter uses matches only when they describe the displayed document and current find text.
use ide_app::find_navigation::{at_or_after, at_or_before, navigable_matches};
/// What:
///  `Rc<RefCell<State>>` is the shared,
///  borrow-checked source state of this window.
/// Why:
///  Transitions change the selection that rendering and copying read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::RefCell, rc::Rc};

/// Make one match the reading selection and scroll it into view.
/// The active match is the selection,
///  so Ctrl+C copies it and reloads map it like any selection.
pub(super) fn select(window: &AppWindow, state: &Rc<RefCell<State>>, range: FindRange) {
    // What: `borrow_mut` lends the source state mutably until `drop` below.
    // Why: The selection belongs to the document, which also owns reload correspondence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const current = state.current;
    // ```
    let mut current = state.borrow_mut();
    let mut position = current.document.position();
    position.anchor = range.start;
    position.head = range.end;
    position.affinity = ReadingAffinity::Downstream;
    current.document.select(position);
    drop(current);
    tracing::debug!(
        start = range.start,
        end = range.end,
        "selected in-file match"
    );
    viewport::reveal(window, state, range.start, range.end);
}

/// Ctrl+F:
///  show the bar,
///  focus its input with the previous find text selected,
///  and request matches.
/// Ignored while the modal search overlay is open or no source file is displayed.
pub(super) fn open(window: &AppWindow, state: &Rc<RefCell<State>>, find: &mut Find) {
    if window.get_search_open() {
        tracing::debug!("ignored in-file find request while project search is open");
        return;
    }
    if !window.get_source_available() {
        tracing::debug!("ignored in-file find request without a displayed source file");
        return;
    }
    tracing::debug!(reopened = find.open, "in-file find opened");
    find.open = true;
    // What: `None` forgets the last requested identity.
    // Why: Reopening must request matches for the remembered find text again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // find.requested = undefined;
    // ```
    find.requested = None;
    window.set_find_open(true);
    window.invoke_focus_find();
    tick::update(window, state, find);
}

/// A changed find text is a new query generation;
///  its reply selects the nearest match.
pub(super) fn edit(window: &AppWindow, state: &Rc<RefCell<State>>, find: &mut Find, raw: &str) {
    if !find.open {
        tracing::debug!("ignored find text edit while the bar is closed");
        return;
    }
    // What: `checked_add` returns `None` instead of wrapping past the largest `u64`.
    // Why: A reused query generation could accept a reply for older find text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (find.queryGeneration === Number.MAX_SAFE_INTEGER) { fail('...'); return; }
    // ```
    let Some(generation) = find.query_generation.checked_add(1) else {
        tick::fail(
            window,
            state,
            find,
            "Find text identity exhausted; restart the application to keep searching".to_string(),
        );
        return;
    };
    find.query_generation = generation;
    // What: `to_string` copies the borrowed text into storage this state owns.
    // Why: The callback's string is released when the callback returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // find.query = raw;
    // ```
    find.query = raw.to_string();
    find.seek = true;
    tracing::debug!(
        generation,
        characters = raw.chars().count(),
        "find text edited"
    );
    tick::update(window, state, find);
}

/// Enter (positive delta) or Shift+Enter (negative delta):
///  select the next or previous match,
///  wrapping.
pub(super) fn navigate(window: &AppWindow, state: &Rc<RefCell<State>>, find: &Find, delta: i32) {
    let current = state.borrow();
    let identity = tick::wanted(&current, find);
    // What: `let ... else` extracts usable matches or leaves early when there are none.
    // Why: Until the reply for the displayed document and current text arrives, Enter does nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const matches = navigableMatches(current.find, identity); if (!matches) return;
    // ```
    let Some(matches) = navigable_matches(&current.find, identity) else {
        tracing::debug!(
            ?identity,
            "ignored match navigation without current results"
        );
        return;
    };
    let position = current.document.position();
    let start = position.anchor.min(position.head);
    let end = position.anchor.max(position.head);
    let index = if delta < 0 {
        at_or_before(&matches.ranges, start)
    } else {
        at_or_after(&matches.ranges, end)
    };
    let Some(found) = index else {
        tracing::debug!("ignored match navigation without matches");
        return;
    };
    let range = matches.ranges[found];
    drop(current);
    select(window, state, range);
}

/// Escape:
///  hide the bar,
///  remove highlights,
///  cancel pending work,
///  and return focus to the source.
/// The find text and the selection (the last active match) are kept.
pub(super) fn close(window: &AppWindow, state: &Rc<RefCell<State>>, find: &mut Find) {
    if !find.open {
        return;
    }
    find.open = false;
    find.requested = None;
    find.seek = false;
    find.worker.cancel();
    state.borrow_mut().find = None;
    window.set_find_open(false);
    // What: `into` converts a string literal into the window's shared string type.
    // Why: A diagnostic for a closed bar must not reappear on the next open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.findError = '';
    // ```
    window.set_find_error("".into());
    render(window, state);
    window.invoke_focus_source();
    tracing::debug!("in-file find closed and pending work cancelled");
}
