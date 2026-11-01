//! Keep the hosted client drawing when the parent compositor stops presenting this window.
//!
//! The live redraw path sends frame callbacks only after presenting to the parent,
//!  and winit
//! delivers the next redraw only after the parent's own frame callback.
//!  A locked or hidden
//! parent window therefore starved the hosted client:
//!  it committed no new buffer,
//!  and
//! `screenshot` kept compositing its last one.
//!  This module paces the client from a timer
//! whenever the parent has been silent,
//!  so capture never depends on host visibility.

/// What:
///      `Duration` is a span of time;
///  `Instant` is an opaque monotonic timestamp,
///  not a
///           wall clock (sibling:
///  `SystemTime`,
///  which can jump when the clock is adjusted).
/// Why:
///       Stall detection compares "now" against the last presentation on a clock that
///           never runs backwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Duration ~ a millisecond count; Instant ~ performance.now().
/// ```
use std::time::{Duration, Instant};

/// What:
///      A grouped `use` of calloop's timer source,
///  its reschedule instruction,
///  and the
///           handle used to register event sources on the compositor's event loop.
/// Why:
///       The fallback runs on the same single thread as every protocol handler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // setInterval(pace, FRAME_INTERVAL_MS) on the one event loop.
/// ```
use smithay::reexports::calloop::{
    timer::{TimeoutAction, Timer},
    LoopHandle,
};

/// What:
///      `use tracing::info;` imports the structured informational log macro.
/// Why:
///       Entering and leaving fallback pacing explains otherwise invisible host states.
use tracing::info;

/// What:
///      A grouped `use` of the output refresh constant,
///  the shared frame-callback
///           sender,
///  and the compositor state type from this package (`crate`).
/// Why:
///       The fallback sends the same callbacks the live redraw and the recorder send.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OUTPUT_REFRESH_MHZ } from "./backend";
/// import { sendFrameCallbacks } from "./render";
/// import { type Compositor } from "./state";
/// ```
use crate::{backend::OUTPUT_REFRESH_MHZ, backend_select::BackendKind, render, state::Compositor};

/// One frame at the nested output's advertised refresh rate.
///
/// What:
///      `pub const FRAME_INTERVAL: Duration`.
///  The rate is in millihertz,
///  so the period in
///           nanoseconds is 10^12 divided by it.
///  `as u64` converts the signed constant to the
///           unsigned 64-bit type `Duration::from_nanos` takes (siblings:
///  `u32`,
///  `i64`).
/// Why:
///       A starved client is paced at the rate the nested output promises,
///  not faster.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FRAME_INTERVAL_MS = 1000 / 60;
/// ```
pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000_000 / OUTPUT_REFRESH_MHZ as u64);

/// Parent silence at or beyond this span means the parent is not presenting this window.
///
/// What:
///      `pub const STALL_THRESHOLD: Duration`,
///  three frames at 60 Hz.
/// Why:
///       A parent presenting at 30 Hz or faster never trips the fallback,
///  so the visible
///           path keeps owning the cadence whenever it works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STALL_THRESHOLD_MS = 50;
/// ```
pub const STALL_THRESHOLD: Duration = Duration::from_millis(50);

/// Decide whether the fallback timer must send frame callbacks.
///
/// What:
///      `pub fn parent_stalled(recording: bool, since_presented: Duration) -> bool`.
///           A pure function of two plain values,
///  so tests need no display or clock.
/// Why:
///       While recording,
///  the recorder's own timer already sends frame callbacks at the
///           requested rate,
///  and a second sender would raise the client above that rate.
///           Otherwise the fallback takes over once the parent has been silent long enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parentStalled(recording: boolean, sincePresentedMs: number): boolean {
///   return !recording && sincePresentedMs >= STALL_THRESHOLD_MS;
/// }
/// ```
///
/// @example
/// ```ts
/// parentStalled(false, 50); // => true
/// parentStalled(false, 49); // => false
/// parentStalled(true, 3_600_000); // => false
/// ```
pub fn parent_stalled(recording: bool, since_presented: Duration) -> bool {
    if recording {
        return false;
    }
    return since_presented >= STALL_THRESHOLD;
}

/// Decide whether a new screen size or scale must send frame callbacks to the hosted clients at once.
///
/// What:
///      `pub fn resize_needs_early_callbacks(recording: bool, geometry_changed: bool) -> bool`.
///           A pure function of two plain values,
///  so tests need no display or clock.
/// Why:
///       While recording,
///  the recorder tick is the only sender of frame callbacks,
///  and winit holds a
///           client's redraw back until the callback of its previous frame arrives.
///           A client whose size event lands between two ticks would draw only after the next
///           capture and be captured one tick later,
///  with the output mode already at the new size.
///           Sending the callbacks right after the new configure lets it draw and commit within the
///           same tick.
///  Outside a recording the live redraw path sends them after every presentation,
///           and a command that changes nothing has no new size to draw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resizeNeedsEarlyCallbacks(recording: boolean, geometryChanged: boolean): boolean {
///   return recording && geometryChanged;
/// }
/// ```
///
/// @example
/// ```ts
/// resizeNeedsEarlyCallbacks(true, true); // => true
/// resizeNeedsEarlyCallbacks(true, false); // => false
/// resizeNeedsEarlyCallbacks(false, true); // => false
/// ```
pub fn resize_needs_early_callbacks(recording: bool, geometry_changed: bool) -> bool {
    return recording && geometry_changed;
}

/// Decide whether the pacing timer must send frame callbacks on this tick.
///
/// What:
///      `pub fn callbacks_due(backend: BackendKind, recording: bool, since_presented: Duration) -> bool`.
///  A pure function of plain values.
///  The nested window defers to `parent_stalled`.
///  Headless has no parent that presents,
///  so the timer is the only source and a callback is due on every tick while no recording runs.
/// Why:
///       A headless client must get callbacks from the first tick at the output refresh rate,
///  with no startup gap and no stall threshold;
///  while recording,
///  the recorder's own timer is the source in both modes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function callbacksDue(backend, recording, sincePresentedMs): boolean {
///   return backend === 'headless' ? !recording : parentStalled(recording, sincePresentedMs);
/// }
/// ```
///
/// @example
/// ```ts
/// callbacksDue('headless', false, 0); // => true
/// callbacksDue('nested-winit', false, 0); // => false
/// ```
pub fn callbacks_due(backend: BackendKind, recording: bool, since_presented: Duration) -> bool {
    return match backend {
        BackendKind::Headless => !recording,
        BackendKind::NestedWinit => parent_stalled(recording, since_presented),
    }
}

/// When the parent last presented,
///  and whether the fallback is currently pacing the client.
///
/// What:
///      `pub struct FramePacing { ... }` is a record with two private fields.
/// Why:
///       One small owner keeps the stall decision and its transition logging together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FramePacing { #lastPresented: number; #fallbackActive = false; }
/// ```
pub struct FramePacing {
    /// Monotonic time of the latest frame submitted to the parent compositor.
    last_presented: Instant,
    /// True between the first fallback tick and the parent's next presentation.
    fallback_active: bool,
    /// Which backend the session runs on;
    /// decides when the timer sends callbacks.
    backend: BackendKind,
}

/// Stall bookkeeping driven by the live redraw path and the fallback timer.
impl FramePacing {
    /// Start with the parent considered current as of `now`.
    ///
    /// What:
    ///      `pub fn new(now: Instant) -> Self`.
    ///  `Self` names the type being implemented.
    /// Why:
    ///       Taking `now` as a parameter lets tests pass exact times instead of sleeping.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(now: number) { this.#lastPresented = now; }
    /// ```
    pub fn new(now: Instant) -> Self {
        return Self::for_backend(BackendKind::NestedWinit, now);
    }

    /// Start pacing for `backend`,
    /// with the parent considered current as of `now`.
    ///
    /// What:
    ///      `pub fn for_backend(backend: BackendKind, now: Instant) -> Self`.
    /// Why:
    ///       The headless backend never has a parent presentation,
    /// so its timer owns the cadence.
    pub fn for_backend(backend: BackendKind, now: Instant) -> Self {
        return Self { last_presented: now, fallback_active: false, backend };
    }

    /// Record that a frame was just submitted to the parent compositor.
    ///
    /// What:
    ///      `pub fn parent_presented(&mut self, now: Instant)`.
    ///  `&mut self` lends this
    ///           record for modification.
    /// Why:
    ///       The visible path owns the cadence again,
    ///  so the fallback stands down.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parentPresented(now: number): void { this.#lastPresented = now; this.#fallbackActive = false; }
    /// ```
    pub fn parent_presented(&mut self, now: Instant) {
        self.last_presented = now;
        if self.fallback_active {
            self.fallback_active = false;
            info!("parent compositor presented again; fallback frame pacing stopped");
        }
    }

    /// Report whether the fallback must send frame callbacks now,
    ///  logging each transition once.
    ///
    /// What:
    ///      `pub fn fallback_due(&mut self, recording: bool, now: Instant) -> bool`.
    ///           `saturating_duration_since` returns zero instead of panicking when `now` is
    ///           earlier than the stored time.
    /// Why:
    ///       The timer asks every frame;
    ///  the log must not repeat every frame.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fallbackDue(recording: boolean, now: number): boolean {
    ///   return parentStalled(recording, Math.max(0, now - this.#lastPresented));
    /// }
    /// ```
    pub fn fallback_due(&mut self, recording: bool, now: Instant) -> bool {
        let since_presented = now.saturating_duration_since(self.last_presented);
        let due = callbacks_due(self.backend, recording, since_presented);
        if due && !self.fallback_active {
            self.fallback_active = true;
            match self.backend {
                BackendKind::NestedWinit => info!(
                    silent_ms = since_presented.as_millis(),
                    "parent compositor is not presenting this window; pacing the hosted client from a timer",
                ),
                BackendKind::Headless => info!(
                    interval_us = FRAME_INTERVAL.as_micros(),
                    "headless: no parent compositor; pacing the hosted client from a timer at the output refresh rate",
                ),
            }
        }
        return due;
    }
}

/// Register the fallback pacing timer and mark the parent current as of now.
///
/// What:
///      `pub fn register(loop_handle: &LoopHandle<Compositor>, state: &mut Compositor)`.
///           `&` lends the loop handle read-only;
///  `&mut` lends the state for one assignment.
///           The closure `|_, _, timer_state: &mut Compositor| { ... }` ignores the fire time
///           and metadata and receives the state on every tick.
///  `.expect(...)` stops the
///           program with a message if registration fails.
/// Why:
///       Called right before the loop starts,
///  so setup time is not mistaken for a stall.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function register(state: Compositor): void {
///   state.framePacing = new FramePacing(performance.now());
///   setInterval(() => {
///     if (state.framePacing.fallbackDue(state.recorder !== undefined, performance.now())) {
///       sendFrameCallbacks(state);
///     }
///   }, FRAME_INTERVAL_MS);
/// }
/// ```
pub fn register(loop_handle: &LoopHandle<Compositor>, state: &mut Compositor) {
    state.frame_pacing = FramePacing::for_backend(state.backend.kind(), Instant::now());
    loop_handle
        .insert_source(Timer::from_duration(FRAME_INTERVAL), |_, _, timer_state: &mut Compositor| {
            // The recorder owns the cadence while it runs; the end-of-cycle flush delivers these.
            if timer_state.frame_pacing.fallback_due(timer_state.recorder.is_some(), Instant::now()) {
                render::send_frame_callbacks(timer_state);
            }
            return TimeoutAction::ToDuration(FRAME_INTERVAL);
        })
        .expect("failed to register the fallback frame pacing timer");
}

/// Verifies the stall decision and its transitions with injected times.
#[cfg(test)]
#[path = "frame_pacing_tests.rs"]
mod tests;
