//! Apply background disk updates without suspending selection or blocking source input.

/// What:
///  Borrow UI-thread state while the worker only receives owned snapshots.
/// Why:
///  Filesystem reads and diff computation must never hold the UI RefCell borrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { state, render } from '../native';
/// ```
use super::{AppWindow, State, render, rows};
/// Worker creation failure must surface rather than silently disabling external refresh.
use anyhow::Result;
/// A dropped timer read counts as a write still in progress.
use ide_app::change_watch::SourceChange;
/// An accepted reload is copied for the Language module before the document consumes it.
use ide_app::language::sync::DocumentReload;
/// The quiet period a timer read requires before it accepts the file's bytes.
use ide_app::refresh_policy::WRITE_QUIET;
/// Background replies retain the file generation and source base revision.
use ide_app::reload_worker::{ReloadReply, ReloadRequest, ReloadWorker, SyntaxReply};
/// Reset source classifications without mutating a snapshot shared with the previous frame.
use ide_app::source_style::SourceStyles;
/// Timer callbacks and weak window references belong to the toolkit event loop.
use slint::{ComponentHandle, Timer, TimerMode};
/// Rc/RefCell stay UI-local;
///  Instant schedules reads without changing wall-clock state.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// Record a changed read diagnostic while retaining the last readable source.
fn read_failed(window: &AppWindow, state: &Rc<RefCell<State>>, message: String) {
    let mut current = state.borrow_mut();
    if current.file_error.as_ref() == Some(&message) {
        return;
    }
    tracing::warn!(%message, "source refresh failed; retaining displayed text");
    current.file_error = Some(message);
    drop(current);
    render(window, state);
}

/// Accept classifications only for the installed source revision,
///  without hiding parser failures.
pub(super) fn apply_syntax(current: &mut State, reply: SyntaxReply) -> bool {
    if reply.revision != current.document.revision() {
        tracing::debug!(
            revision = reply.revision,
            "discarding syntax for an obsolete source revision"
        );
        return false;
    }
    current.syntax_revision = Some(reply.revision);
    // New styles change the shaped rows of every line, and the row cache key does not include them.
    current.shaper.forget_rows();
    match reply.result {
        Ok(Some(styles)) => {
            current.styles = styles;
            current.syntax_error = None;
        }
        Ok(None) => {
            current.styles = SourceStyles::from([]);
            current.syntax_error = None;
        }
        Err(error) => {
            tracing::warn!(%error, "highlighting unavailable; retaining readable source");
            current.styles = SourceStyles::from([]);
            current.syntax_error = Some(format!("{error:#}"));
        }
    }
    return true;
}

/// Install a matching file/revision result using the current caret and selection.
fn apply(window: &AppWindow, state: &Rc<RefCell<State>>, reply: ReloadReply) {
    if reply.generation != state.borrow().file_generation {
        tracing::debug!(
            generation = reply.generation,
            "discarding reload for a previously displayed file"
        );
        return;
    }
    // A timer read met a file written within the quiet period: wait for the writer like a notification would.
    if reply.recent_write {
        tracing::debug!("timer read met a recent write; waiting for the writer to go quiet");
        state
            .borrow_mut()
            .refresh
            .changed(SourceChange::Unsettled, Instant::now());
        return;
    }
    let update = match reply.result {
        Ok(update) => update,
        Err(error) => {
            read_failed(
                window,
                state,
                format!(
                    "{error:#}. Showing the last readable source. Restore a readable UTF-8 file at this path or open another file."
                ),
            );
            return;
        }
    };
    let offset = (-window.get_scroll_y()).max(0.0);
    let mut current = state.borrow_mut();
    // How far the view's top edge lies below the top of the top line's code row; negative inside its virtual rows.
    let top_line = current.row_map.line_at(offset);
    let within = offset - current.row_map.code_top(top_line);
    let mut redraw = current.file_error.is_some();
    let mut mapped_viewport = None;
    if let Some(reload) = update {
        // Language servers need both texts and the edits between them, which `apply_reload`
        // consumes; the copy is handed over only when the document accepted the reload.
        let language_reload = DocumentReload::from_reload(current.file_generation, &reload);
        // Where the old text's virtual rows end up in the new text; their space is held open there.
        let carried = rows::carried(&current, reload.changes());
        if !current.document.apply_reload(reload) {
            return;
        }
        // Only the latest is kept: the worker recomputes the edits when it missed a revision.
        current.language_reload = Some(language_reload);
        let position = current.document.position();
        let first = current.document.text().char_to_line(position.viewport);
        current.first = first.saturating_sub(1);
        // Cleared styles change the shaped rows of every line, and the row cache key does not include them.
        current.shaper.forget_rows();
        current.styles = SourceStyles::from([]);
        current.syntax_revision = None;
        current.syntax_error = None;
        // The new text has its own vertical mapping, with the old rows' space held open until annotations of
        // the new text arrive; the line the view started in keeps its place in the view.
        rows::hold(&mut current, carried, window.window().scale_factor());
        let target = (current.row_map.code_top(first) + within).max(0.0);
        // The window's next offset report is this mapping, not the reader scrolling.
        current.offset = target;
        mapped_viewport = Some((target, current.row_map.height()));
        redraw = true;
    }
    current.file_error = None;
    if let Some(syntax) = reply.syntax {
        redraw = apply_syntax(&mut current, syntax) || redraw;
    }
    drop(current);
    if let Some((target, extent)) = mapped_viewport {
        // Update extent before offset so the old height cannot clamp a mapped viewport.
        window.set_content_extent(extent);
        window.set_scroll_y(-target);
    }
    if redraw {
        render(window, state);
    }
}

/// Poll completed work on the UI thread;
///  start a disk read when `State::refresh` says one is due:
/// on a change notification for the displayed file,
///  every 250 ms while its directory is unwatched,
/// or on the safety sweep.
///  Missing highlighting is requested again after the 100 ms reread gap.
/// The returned timer owns the worker and must remain alive until the window closes.
pub(super) fn bind(window: &AppWindow, shared: &Rc<RefCell<State>>) -> Result<Timer> {
    let mut worker = ReloadWorker::new()?;
    let state = Rc::clone(shared);
    let weak = window.as_weak();
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
        let Some(active_window) = weak.upgrade() else {
            return;
        };
        // Space held open for rows of a replaced text is given up when its time has passed, and rows that
        // waited for scrolling to stop are shown; neither comes with an event of its own.
        let due = rows::due(
            &mut state.borrow_mut(),
            active_window.window().scale_factor(),
        );
        if due {
            render(&active_window, &state);
        }
        match worker.try_take() {
            Ok(Some(reply)) => {
                apply(&active_window, &state, reply);
            }
            Ok(None) => {}
            Err(error) => {
                read_failed(&active_window, &state, format!("{error:#}"));
                return;
            }
        }
        let now = Instant::now();
        let current = state.borrow();
        let Some(path) = &current.file_path else {
            return;
        };
        let highlight = current.syntax_revision != Some(current.document.revision());
        // Missing highlighting asks again without a notification, after the schedule's reread gap.
        if !current.refresh.due(now, highlight) {
            return;
        }
        // What: `if ... { None } else { Some(WRITE_QUIET) }` is an expression choosing the quiet requirement.
        // Why: A read no notification asked for can start in the middle of a save, before that save's
        //      notification arrives, whatever the order of timers; it accepts only a file quiet for 50 ms.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const requireQuiet = refresh.hasUnreadChange() ? undefined : WRITE_QUIET;
        // ```
        let require_quiet = if current.refresh.has_unread_change() {
            None
        } else {
            Some(WRITE_QUIET)
        };
        let requested = worker.request(ReloadRequest {
            path: path.clone(),
            snapshot: current.document.clone(),
            generation: current.file_generation,
            highlight_unchanged: highlight,
            require_quiet,
        });
        drop(current);
        match requested {
            Ok(true) => {
                state.borrow_mut().refresh.requested(now);
            }
            Ok(false) => {}
            Err(error) => {
                read_failed(&active_window, &state, format!("{error:#}"));
            }
        }
    });
    return Ok(timer);
}
