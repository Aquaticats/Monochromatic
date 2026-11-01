//! The window's side of the background measurement of the whole text:
//!  what the measurements are valid for,
//!  posting a pass to the tile worker,
//!  and applying what a pass measured.

/// Starting the worker and its poll when a pass is posted.
use super::{start_poll, warm};
/// Applying a pass renders again when a count changed.
use crate::native::render::render;
/// The vertical mapping's side of the window:
///  measured rows and keeping the view still.
use crate::native::rows;
/// What a background measurement is valid for,
///  what it is given,
///  and what it produced.
use crate::native::tile_pass::{PassJob, PassKey, PassOutput};
/// Parent state and the window.
use crate::native::{AppWindow, State};
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;
/// The window's display scale for the measurement key.
use slint::ComponentHandle;
/// The shared window state,
///  and the snapshots a pass job owns.
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// What:
///  What the measurements of the window's text at its current scale and wrap width are valid for.
///  A window without a wrap width yet has wrap bits of zero.
/// Why:
///  A finished tile or pass output is used only when its key is this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentKey(current: State, scale: number): PassKey;
/// ```
pub(in crate::native) fn current_key(current: &State, scale: f32) -> PassKey {
    let wrap = if has_wrap_width(current.width) {
        current.width.to_bits()
    } else {
        0
    };
    return PassKey {
        file: current.file_generation,
        revision: current.document.revision(),
        scale: scale.to_bits(),
        wrap,
    };
}

/// What:
///  Ask the worker to measure the whole text at the current wrap width when it has not been asked yet for this
///  text,
/// scale,
/// width,
/// and syntax snapshot.
/// Why:
///  The pass is what makes the scroll extent exact without any frame waiting for it;
///  a resize or a new text starts it again,
/// and lines the worker kept are only broken again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncPass(window: AppWindow, state: Shared<State>, current: State, scale: number): void;
/// ```
pub(in crate::native) fn sync_pass(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    current: &mut State,
    scale: f32,
) {
    if !has_wrap_width(current.width) {
        return;
    }
    let key = current_key(current, scale);
    // The address of the syntax snapshot tells a new snapshot from the one the pass already used.
    let styles = Arc::as_ptr(&current.styles).cast::<()>() as usize;
    if current.pass_posted == Some((key, styles)) {
        return;
    }
    if let Err(error) = warm(window, state, current) {
        tracing::error!(%error, "source tile worker could not start");
        return;
    }
    let job = PassJob {
        key,
        // The worker handle numbers the post.
        generation: 0,
        document: current.document.clone(),
        styles: Arc::clone(&current.styles),
        typography: current.shaper.typography().clone(),
        scale,
        wrap: current.width,
        focus: current.shown.first,
    };
    let Some(worker) = current.tile_worker.as_mut() else {
        return;
    };
    match worker.post_pass(job) {
        Ok(()) => {
            tracing::debug!(
                wrap = current.width,
                focus = current.shown.first,
                "posted the measurement pass"
            );
            current.pass_posted = Some((key, styles));
            start_poll(window, state, current);
        }
        Err(error) => tracing::error!(%error, "source measurement pass request failed"),
    }
}

/// What:
///  Apply what the pass measured:
/// the visual rows of lines outside the displayed tile go into the window's table,
///  and the window renders when any count changed.
/// Why:
///  The lines of the displayed tile keep the heights they were painted with;
///  a pass result must never move the rows of the frame on screen.
///  A result for another text,
/// scale,
/// or width is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyPass(window: AppWindow, state: Shared<State>, output: PassOutput): void;
/// ```
pub(super) fn apply_pass(window: &AppWindow, state: &Rc<RefCell<State>>, output: PassOutput) {
    let factor = window.window().scale_factor();
    let mut current = state.borrow_mut();
    if output.key != current_key(&current, factor) {
        tracing::debug!(
            lines = output.lines.len(),
            "dropped a measurement for another text or width"
        );
        return;
    }
    let held = current.shown.first..current.shown.first + current.shown.count;
    let count = output.lines.len();
    let mut unheld = Vec::new();
    // The lines of the displayed tile are never applied; one whose rows in the table differ from what the pass
    // measured is a height the map has wrong, so it is named in the log.
    let mut held_differs = Vec::new();
    for (line, rows) in output.lines {
        if !held.contains(&line) {
            unheld.push((line, rows));
        } else if current.wrap_rows.rows(line) != rows && held_differs.len() < 8 {
            held_differs.push((line, current.wrap_rows.rows(line), rows));
        }
    }
    if !held_differs.is_empty() {
        tracing::debug!(
            ?held,
            ?held_differs,
            shown_width = current.shown.width,
            width = current.width,
            "the measurement found rows in the displayed tile that differ from the map (line, map rows, measured rows)"
        );
    }
    let changed = rows::absorb(&mut current, unheld);
    #[cfg(debug_assertions)]
    {
        current.pass_applied += 1;
        current.pass_lines += count as u64;
        current.pass_most_lines = current.pass_most_lines.max(count);
        current.pass_shaped += output.shaped;
        current.pass_rebroken += output.rebroken;
    }
    tracing::debug!(count, changed, done = output.done, "applied a measurement");
    drop(current);
    if changed {
        render(window, state);
    }
}
