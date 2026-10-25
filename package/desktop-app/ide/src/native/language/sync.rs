//! Telling the worker about the displayed file:
//!  opens,
//!  reloads,
//!  and the visible lines for hints.
//!
//! A command the worker's queue cannot take now (`Ok(false)`) is sent again on the next tick.
//! Requests are sent only after the worker was told about the displayed text,
//!  because the worker
//! drops,
//!  without any reply,
//!  a request for text it does not hold.

/// The language state and the surface helper that reads the displayed stamp.
use super::{Language, surface::displayed};
/// The window reports the scroll offset and the viewport height.
use crate::native::{AppWindow, State};
/// What:
///  `anyhow::Result<bool>` is "told the worker" (true),
///  "try again" (false),
///  or an error.
/// Why:
///  An error means the worker stopped;
///  the caller then turns language support off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Result<T> = T; // throws when the worker stopped
/// ```
use anyhow::Result;
/// Commands and the identities they carry.
use ide_app::language::{
    hints::{HintWindow, request_lines},
    identity::DocumentStamp,
    sync::DocumentOpen,
};
/// One code row is the least a line takes,
///  which bounds how many lines the view can show.
use ide_app::row_map::CODE_ROW;
/// What:
///  `Rc<RefCell<State>>` is the window's shared source state;
///  `Duration` and `Instant`
///       measure how long the visible lines stayed the same.
/// Why:
///  The reload record is taken out of the source state once the worker accepted it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// How long the visible lines must stay the same before hints are asked for them.
///  Chosen,
///  not
/// measured:
///  long enough that a wheel scroll asks once at its end,
///  short enough to feel prompt.
const HINT_SETTLE: Duration = Duration::from_millis(200);

/// What:
///  The visible lines last reported and the lines seen most recently with their time.
///       `Option<(A, B)>` is "a pair,
///  or nothing".
/// Why:
///  Hints are asked for once per settled view and once per newly displayed text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HintLines = { sent?: [DocumentStamp, HintWindow]; seen?: [HintWindow, number] };
/// ```
#[derive(Debug, Default)]
pub(super) struct HintLines {
    /// Stamp and lines the worker was last told,
    ///  or last skipped because stored answers covered them.
    sent: Option<(DocumentStamp, HintWindow)>,
    /// Lines seen on the latest tick and since when they have not changed.
    seen: Option<(HintWindow, Instant)>,
}

/// What:
///  Make the worker hold the displayed text.
///  `Ok(true)` once it does.
/// Why:
///  A new file generation is an open;
///  a new revision of the same file is the reload the
///      reload path recorded;
///  an explicit action can ask for the file to be displayed again so
///      the worker re-resolves programs and restarts failed servers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function update(language: Language, source: Shared<State>): boolean
/// ```
pub(super) fn update(language: &mut Language, source: &Rc<RefCell<State>>) -> Result<bool> {
    // `let Some(worker) = ... else` leaves when language support is off.
    let Some(worker) = language.worker.as_mut() else {
        return Ok(false);
    };
    let current = source.borrow();
    // `clone` copies the path; without a displayed file there is nothing to tell.
    let Some(path) = current.file_path.clone() else {
        return Ok(false);
    };
    let stamp = displayed(&current);
    if language.synced == Some(stamp) && !language.reopen {
        return Ok(true);
    }
    let same_file = match language.synced {
        Some(synced) => synced.file == stamp.file,
        None => false,
    };
    // A reload record for exactly the displayed revision continues what the worker holds.
    if same_file
        && !language.reopen
        && let Some(reload) = current.language_reload.as_ref()
        && reload.file == stamp.file
        && reload.revision == stamp.revision
    {
        let command = reload.clone();
        drop(current);
        if !worker.reload(command)? {
            return Ok(false);
        }
        tracing::debug!(?stamp, "told the language worker about a reload");
        language.synced = Some(stamp);
        source.borrow_mut().language_reload = None;
        return Ok(true);
    }
    // `text().clone()` copies the rope handle; its chunks are shared, not copied.
    let open = DocumentOpen {
        path: path.clone(),
        text: current.document.text().clone(),
        stamp,
    };
    drop(current);
    if !worker.open(open)? {
        return Ok(false);
    }
    tracing::debug!(?stamp, path = %path.display(), reopened = language.reopen, "told the language worker about the displayed file");
    language.synced = Some(stamp);
    language.reopen = false;
    // An older reload record is covered by the full text just sent.
    let mut stored = source.borrow_mut();
    stored.language_reload = None;
    // A worker that opened the file again holds no earlier hint request, so no lines count as asked about.
    stored.annotations.forget_answered();
    return Ok(true);
}

/// What:
///  Report the visible lines once they settled,
///  or at once for newly displayed text.
/// Why:
///  Hints are asked for the lines in view and some around them;
///  scrolling asks only when it stops,
///  and not at all when stored answers of the same text revision already asked about every line the request would ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hints(language: Language, window: AppWindow, source: Shared<State>, stamp: DocumentStamp): void
/// ```
pub(super) fn hints(
    language: &mut Language,
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    stamp: DocumentStamp,
) -> Result<()> {
    let Some(worker) = language.worker.as_mut() else {
        return Ok(());
    };
    // The vertical mapping names the line at the top edge of the view. Every line takes at least one code
    // row, so the view height in code rows is the most lines it can show; `as usize` truncates that count.
    let first_line = source
        .borrow()
        .row_map
        .line_at((-window.get_scroll_y()).max(0.0));
    let visible_lines = ((window.get_viewport_height() / CODE_ROW).ceil() as usize).max(1);
    let wanted = HintWindow {
        first_line,
        visible_lines,
    };
    if language.hints.sent == Some((stamp, wanted)) {
        return Ok(());
    }
    let new_text = match language.hints.sent {
        Some((sent, _)) => sent != stamp,
        None => true,
    };
    // The lines of the previous sighting, if any.
    let mut seen_lines = None;
    if let Some((lines, _)) = language.hints.seen {
        seen_lines = Some(lines);
    }
    if seen_lines != Some(wanted) {
        language.hints.seen = Some((wanted, Instant::now()));
    }
    let settled = match language.hints.seen {
        Some((_, since)) => since.elapsed() >= HINT_SETTLE,
        None => false,
    };
    if !new_text && !settled {
        return Ok(());
    }
    // What: The lines the request for `wanted` would ask, as the worker computes them from the text.
    // Why: When stored answers of this revision asked about all of them, the store holds their hints already.
    //      The view is then recorded as handled; the worker keeps the window it was last told.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [first, last] = requestLines(wanted, lineCount);
    // if (annotations.answeredCovers(stamp, first, last)) return;
    // ```
    let (first, last) = request_lines(wanted, source.borrow().document.text().len_lines());
    let covered = source
        .borrow()
        .annotations
        .answered_covers(stamp, first..last);
    if covered {
        tracing::debug!(
            ?stamp,
            first_line,
            visible_lines,
            "skipped the hint request: stored answers already cover its lines"
        );
        language.hints.sent = Some((stamp, wanted));
        return Ok(());
    }
    if worker.request_hints(stamp, wanted)? {
        tracing::debug!(
            ?stamp,
            first_line,
            visible_lines,
            "reported visible lines for inlay hints"
        );
        language.hints.sent = Some((stamp, wanted));
    }
    return Ok(());
}
