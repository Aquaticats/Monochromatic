//! Rendering the hosted window into the nested winit framebuffer.
//!
//! One `redraw` call binds the winit backend's framebuffer,
//!  composites the space
//! (the one hosted window) into it with Smithay's `render_output` helper,
//!  submits the
//! frame,
//!  and sends frame-callbacks so the client draws its next frame.

/// What:
///      `use std::time::{Duration, Instant};`.
///  `Duration` is a span of time;
///  `Instant` is
///           a monotonic timestamp (sibling:
///  the adjustable wall clock `SystemTime`).
/// Why:
///       Frame callbacks report elapsed time;
///  `Duration::ZERO` is the throttle hint.
///           Each presentation to the parent is timestamped for stall detection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Duration ~ a millisecond count; Instant ~ performance.now().
/// ```
use std::time::{Duration, Instant};

/// What:
///      Grouped `use` of the render-element type,
///  the GLES renderer,
///  the
///           `render_output` helper,
///  and the `Rectangle` geometry type.
/// Why:
///       Everything `redraw` references.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { WaylandSurfaceRenderElement, GlesRenderer, renderOutput, Rectangle } from "smithay";
/// ```
use smithay::{
    backend::renderer::{element::surface::WaylandSurfaceRenderElement, gles::GlesRenderer},
    desktop::space::render_output,
    utils::Rectangle,
};

/// What:
///      `use crate::state::Compositor;`.
///  Our state type.
/// Why:
///       `redraw` operates on `&mut Compositor`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Compositor } from "./state";
/// ```
use crate::state::Compositor;

/// Context names the failed frame step in the error that stops the session.
use anyhow::{Context, Result};

/// Dark grey clear colour (RGBA,
///  0..=1) behind the hosted window.
///
/// What:
///      `const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];`.
///  A fixed-size array
///           of four 32-bit floats (`f32`;
///  sibling `f64` is 64-bit).
///  Order is
///           red,
///  green,
///  blue,
///  alpha.
/// Why:
///       A neutral background makes the hosted window's own drawing obvious in
///           screenshots;
///  the app is fullscreen so it is usually fully covered anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLEAR_COLOR = [0.1, 0.1, 0.1, 1.0];
/// ```
pub const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

/// Composite the hosted window into the nested framebuffer and present one frame.
///
/// What:
///      `pub fn redraw(state: &mut Compositor) -> Result<()>`.
///  Mutably borrows the whole state;
///           internally it borrows several disjoint fields (backend,
///  output,
///  space,
///           damage tracker) at once,
///  which Rust allows because they are distinct
///           fields.
/// Why:
///       The single place that turns committed client buffers into a presented
///           frame and asks the client for its next one.
///
/// Errors:
///       Binding the nested window's framebuffer,
///  rendering into it,
///  or presenting it fails (for example the parent display or its GPU context is lost).
///  The caller ends the session with that error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function redraw(state) { ... } // throws when the frame cannot be drawn or presented
/// ```
///
/// @example
/// ```ts
/// redraw(state); // called on each WinitEvent.Redraw
/// ```
pub fn redraw(state: &mut Compositor) -> Result<()> {
    // What:     `if state.recorder.is_some() { return; }`. Skip the live present while a
    //           recording is running.
    // Why:      During a 60fps recording the recorder's timer drives rendering + readback
    //           (capture only, no `submit`), so suppressing the visible present avoids
    //           double-rendering and keeps capture cadence off the parent's vsync. The
    //           visible window is intentionally frozen while recording; `stop` requests a
    //           redraw to unfreeze it.
    if state.recorder.is_some() {
        return Ok(());
    }

    // What:     `let size = winit.window_size();`. The framebuffer size as
    //           `Size<i32, Physical>`.
    // Why:      The whole framebuffer is treated as damaged each frame (age 0).
    // What:     `let Some(winit) = state.backend.as_winit_mut() else { return; };`. Only the nested
    //           window has a framebuffer to present; a headless session never requests a redraw.
    // Why:      Headless pacing and captures do not go through this live present.
    let Some(winit) = state.backend.as_winit_mut() else {
        return Ok(());
    };
    let size = winit.window_size();

    // What:     `let damage = Rectangle::from_size(size);`. A rectangle covering the
    //           whole framebuffer, `Rectangle<i32, Physical>`.
    // Why:      Passed to `submit` as the region that changed.
    let damage = Rectangle::from_size(size);

    // What:     A nested block `{ ... }` scoping the render borrows so they end before
    //           `submit` is called.
    // Why:      `bind` mutably borrows the backend; releasing that borrow at the block's
    //           end lets `submit` (also `&mut backend`) run afterwards.
    {
        // What:     `let (renderer, mut framebuffer) = winit.bind().context(...)?;`.
        //           `bind()` returns `Result<(&mut GlesRenderer, Framebuffer), _>`;
        //           `.context(...)?` returns a bind failure to the caller. `renderer` is the GLES renderer;
        //           `framebuffer` is the target we draw into (`mut` because
        //           `render_output` borrows it mutably).
        // Why:      Get the drawing surface and renderer for this frame.
        let (renderer, mut framebuffer) = winit.bind().context(
            "binding the nested window's framebuffer failed; the parent display or its GPU context may have been \
             lost, so restart the session on a working display",
        )?;

        // What:     `render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
        //           &state.output, renderer, &mut framebuffer, 1.0, 0, [&state.space],
        //           &[], &mut state.damage_tracker, CLEAR_COLOR).unwrap();`. The turbofish
        //           pins the render-element type (surfaces from Wayland clients). The
        //           arguments are: output, renderer, framebuffer, scale `1.0`, buffer age
        //           `0` (force full redraw), the spaces to draw `[&state.space]`, extra
        //           custom elements `&[]` (none), the damage tracker, and the clear
        //           colour. `.context(...)?` returns a rendering error to the caller.
        // Why:      Composite the hosted window onto the framebuffer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // renderOutput(output, renderer, framebuffer, 1.0, 0, [space], [], damageTracker, CLEAR_COLOR);
        // ```
        render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
            &state.output,
            renderer,
            &mut framebuffer,
            1.0,
            0,
            [&state.space],
            &[],
            &mut state.damage_tracker,
            CLEAR_COLOR,
        )
        .map_err(|error| return anyhow::anyhow!("rendering the hosted window into the nested framebuffer failed: {error}"))?;
    }

    // What:     `winit.submit(Some(&[damage])).unwrap();`. Presents the frame,
    //           telling the parent which region changed. `Some(&[damage])` is a
    //           one-element slice of the whole-framebuffer rectangle. `.context(...)?`
    //           returns a swap failure to the caller.
    // Why:      Actually show the composited frame in the nested window.
    winit
        .submit(Some(&[damage]))
        .context("presenting the frame to the parent display failed; restart the session on a working display")?;

    // What:     `Instant::now()` reads the monotonic clock; the pacing record stores it.
    // Why:      Only a frame the parent accepted counts as presentation. While these keep
    //           arriving, the fallback timer stays idle and this path owns the cadence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.framePacing.parentPresented(performance.now());
    // ```
    state.frame_pacing.parent_presented(Instant::now());

    // What:     `send_frame_callbacks(state);`. Tell the client its last frame was shown so
    //           it draws the next one, and refresh space/popup bookkeeping.
    // Why:      Shared with the recorder, which needs the same "keep the app animating" step.
    send_frame_callbacks(state);

    // What:     `let _ = state.display_handle.flush_clients();`. Flush queued protocol
    //           events to all clients; `let _ =` discards the `Result` (a flush failure
    //           just means a client disconnected).
    // Why:      Deliver the frame callbacks and configures we just queued.
    let _ = state.display_handle.flush_clients();

    // What:     `state.backend.window().request_redraw();`. Ask winit to emit another
    //           `Redraw` event.
    // Why:      Keep the render loop going so the hosted app keeps animating.
    state.backend.window().request_redraw();

    // What:     `Ok(())`. The frame was presented.
    // Why:      Tell the caller the session can continue.
    return Ok(())
}

/// Send frame callbacks to every mapped window and refresh space/popup bookkeeping.
///
/// What:
///      `pub fn send_frame_callbacks(state: &mut Compositor)`.
///  Tells each window its
///           last frame was presented (so it draws the next one),
///  then refreshes the space
///           and cleans up dead popups.
/// Why:
///       Shared by the live redraw and the 60fps recorder:
///  both must keep an animating
///           client producing frames at the intended rate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sendFrameCallbacks(state) { ... }
/// ```
pub fn send_frame_callbacks(state: &mut Compositor) {
    // What:     `let elapsed = state.start_time.elapsed();`. Time since program start.
    // Why:      Frame callbacks carry this timestamp to the client.
    let elapsed = state.start_time.elapsed();

    // What:     `let output = state.output.clone();`. Clone the output handle (cheap,
    //           reference-counted) so the frame-callback closure can own it without
    //           borrowing `state` while `state.space` is also borrowed.
    // Why:      Avoids an overlapping-borrow error between `state.space.elements()` and a
    //           closure that reads `state.output`.
    let output = state.output.clone();

    // What:     `state.space.elements().for_each(|window| { window.send_frame(&output,
    //           elapsed, Some(Duration::ZERO), |_, _| Some(output.clone())); });`. Send a
    //           frame callback to each mapped window. `Some(Duration::ZERO)` is the throttle
    //           hint (draw as fast as possible); the inner closure tells Smithay which
    //           output each surface is on.
    // Why:      Tell the client "your last frame was shown; draw the next one".
    state.space.elements().for_each(|window| {
        window.send_frame(&output, elapsed, Some(Duration::ZERO), |_, _| {
            return Some(output.clone())
        });
    });

    // What:     `state.space.refresh();`. Recomputes window/output bookkeeping.
    // Why:      Keep the space's internal state consistent after a frame.
    state.space.refresh();

    // What:     `state.popups.cleanup();`. Drop popups whose surfaces are gone.
    // Why:      Prevent stale popups from lingering.
    state.popups.cleanup();
}
