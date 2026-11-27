//! The window's viewport callback:
//!  every report of the viewport's offset,
//!  height,
//!  and width,
//!  and what follows from it.

/// The window,
///  the source state,
///  the GPU notifier,
///  the minimap's picture,
///  the render that follows a step,
///  the row clock,
///  the tile poll,
///  and the wheel ease.
use super::super::{
    AppWindow,
    State,
    gpu_tile,
    minimap,
    render,
    rows,
    tile,
    wheel,
};
/// The scrolled placement each report ends in.
use super::place_scrolled;
/// Weak window handles,
///  and the display scale a report carries.
use slint::ComponentHandle;
/// Checked shared ownership remains confined to the UI thread,
///  and the clock that tells a scroll step from a report.
use std::{
    cell::RefCell,
    rc::Rc,
    time::Instant,
};

/// Retain pixel scrolling;
///  update the raster only at viewport tile boundaries and when the wrap width changes.
pub(in crate::native) fn bind_viewport(
    owner: &AppWindow,
    shared: &Rc<RefCell<State>>,
) {
    // The code view is drawn on the GPU, by the window's one rendering notifier.
    gpu_tile::bind(owner, &shared.borrow().gpu);
    // What: `Rc::clone` copies the pointer to the shared state; `as_weak` does not keep the window alive.
    // Why: The stored callback outlives this binding function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const state = shared; const weak = new WeakRef(owner);
    // ```
    let flick_state = Rc::clone(shared);
    let flick_weak = owner.as_weak();
    // The source scroll reports every wheel turn or pan it takes; the turn the window noted names the end of the ease.
    owner.on_source_flicked(move || {
        if let Some(window) = flick_weak.upgrade() {
            wheel::flicked(&window, &flick_state);
        }
    });
    let state = Rc::clone(shared);
    let weak = owner.as_weak();
    // What: `move |...|` transfers the captured handles into the stored callback.
    // Why: Borrowed locals would not outlive this function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // owner.onViewportChanged((offset, width, height) => { ... });
    // ```
    owner.on_viewport_changed(move |offset, viewport_width, height| {
        // What: `borrow_mut` lends the state mutably; `&mut current` passes that loan to `place`.
        // Why: The loan must end before rendering borrows the state again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const changed = place(state.current, offset, width, height);
        // ```
        let mut reported = None;
        if let Some(window) = weak.upgrade() {
            reported = Some(window.window().scale_factor());
        }
        tracing::debug!(
            target: "monochromatic_ide::native::frame",
            viewport_width,
            height,
            "the view reported its size"
        );
        let mut current = state.borrow_mut();
        if let Some(factor) = reported {
            // The size limit of the tile depends on the display scale.
            current.scale = factor;
        }
        // An offset nobody announced is the reader scrolling: by wheel, by touch, or by a drag past an edge.
        let stepped = offset != current.offset;
        if stepped {
            // `Some(...)` records this moment as the latest scroll.
            current.scrolled_at = Some(rows::quiet_now());
            current.stepped_at = Some(Instant::now());
        }
        let changed = place_scrolled(&mut current, offset, viewport_width, height);
        drop(current);
        // What: `upgrade` returns `Some(window)` only while the window still exists.
        // Why: Closing the app must not keep a hidden window alive through callbacks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const window = weak.deref(); if (window) render(window, state);
        // ```
        let Some(window) = weak.upgrade() else {
            return;
        };
        if changed {
            render(&window, &state);
        } else {
            // The tile stays, but the minimap's picture slides with the scroll and may need other bands.
            minimap::note(&window, &state);
        }
        if stepped {
            // What: A tile that finished since the last step is swapped in here, after the step's own work.
            // Why: The poll of the tile worker leaves it for this step, so the setters that swap it in run in
            //      the frame this step draws, not in a frame of their own. State and window agree here, as
            //      they do when the poll's timer fires.
            tile::poll(&window, &state);
        }
    });
}
