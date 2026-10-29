//! Keep the wanted find identity equal to what is displayed and typed;
//!  apply only its reply.

/// Session state and the shared selection step.
use super::{Find, session};
/// Source state,
///  the window,
///  and the single rendering boundary.
use crate::native::{AppWindow, State, render};
/// Accepted results keep their tag;
///  incremental typing selects the first match at or after the selection.
use ide_app::find_navigation::{FindResults, at_or_after};
/// Every request carries the identity its reply must still match when it returns.
use ide_app::find_worker::{FindIdentity, FindReply, FindRequest};
/// What:
///  `Rc<RefCell<State>>` is the shared,
///  borrow-checked source state of this window.
/// Why:
///  Results are stored next to the document they describe,
///  where rendering reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::RefCell, rc::Rc};

/// The tag results must carry to describe the displayed file,
///  its revision,
///  and the current find text.
pub(super) fn wanted(state: &State, find: &Find) -> FindIdentity {
    return FindIdentity {
        file: state.file_generation,
        revision: state.document.revision(),
        query: find.query_generation,
    };
}

/// Show a diagnostic in the bar and drop results that can no longer be trusted.
pub(super) fn fail(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    find: &mut Find,
    message: String,
) {
    tracing::warn!(%message, "in-file find failed");
    find.seek = false;
    // What: `borrow_mut` lends the source state mutably for this one statement.
    // Why: The loan must end before rendering borrows the same state again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.current.find = undefined;
    // ```
    state.borrow_mut().find = None;
    // What: `into` converts the owned `String` into the window's shared string type.
    // Why: Generated window setters take the toolkit's string, not Rust's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.findError = message;
    // ```
    window.set_find_error(message.into());
    render(window, state);
}

/// Store an accepted reply beside the document and,
///  after an edit,
///  select the nearest match.
fn apply(window: &AppWindow, state: &Rc<RefCell<State>>, find: &mut Find, reply: FindReply) {
    // What: `match` extracts the matches or the worker's diagnostic.
    // Why: A refused query or oversized file is shown in the bar instead of "No matches".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let matches; try { matches = reply.result; } catch (error) { fail(String(error)); return; }
    // ```
    let matches = match reply.result {
        Ok(matches) => matches,
        Err(error) => {
            fail(window, state, find, format!("{error:#}"));
            return;
        }
    };
    let mut current = state.borrow_mut();
    let position = current.document.position();
    let origin = position.anchor.min(position.head);
    // What: `None` means no selection change; `Some(range)` is the match to select.
    // Why: Only an edit of the find text moves the selection; reloads and file switches do not.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let target: FindRange | undefined;
    // ```
    let mut target = None;
    if find.seek
        && let Some(index) = at_or_after(&matches.ranges, origin)
    {
        target = Some(matches.ranges[index]);
    }
    find.seek = false;
    tracing::debug!(
        identity = ?reply.identity,
        total = matches.ranges.len(),
        truncated = matches.truncated,
        seek = target.is_some(),
        "accepted in-file find results"
    );
    current.find = Some(FindResults::new(reply.identity, matches));
    // What: `drop` ends the mutable loan now.
    // Why: Selecting and rendering borrow the same state again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // no equivalent: TS has no borrow to release
    // ```
    drop(current);
    window.set_find_error("".into());
    if let Some(range) = target {
        session::select(window, state, range);
        return;
    }
    render(window, state);
}

/// Request matches whenever the displayed file,
///  its revision,
///  or the find text changed;
///  then poll once.
pub(super) fn update(window: &AppWindow, state: &Rc<RefCell<State>>, find: &mut Find) {
    if !find.open || !find.available {
        return;
    }
    let current = state.borrow();
    let identity = wanted(&current, find);
    if find.requested != Some(identity) {
        find.requested = Some(identity);
        if find.query.is_empty() {
            drop(current);
            tracing::debug!(?identity, "empty find text; clearing matches");
            find.worker.cancel();
            find.seek = false;
            state.borrow_mut().find = None;
            window.set_find_error("".into());
            render(window, state);
        } else {
            // What: `clone` on the rope copies a small handle sharing text chunks; on the query it copies bytes.
            // Why: The request must own its snapshot before crossing to the worker thread.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const request = { identity, source: document.text, query: find.query };
            // ```
            let request = FindRequest {
                identity,
                source: current.document.text().clone(),
                query: find.query.clone(),
            };
            drop(current);
            if let Err(error) = find.worker.request(request) {
                find.available = false;
                fail(window, state, find, format!("{error:#}"));
                return;
            }
        }
    } else {
        drop(current);
    }
    match find.worker.poll() {
        Ok(Some(reply)) => {
            apply(window, state, find, reply);
        }
        Ok(None) => {}
        Err(error) => {
            find.available = false;
            fail(window, state, find, format!("{error:#}"));
        }
    }
}
