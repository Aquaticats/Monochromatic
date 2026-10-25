//! A wheel turn that the toolkit eases over a fixed time,
//!  and the one tile the IDE asks for at the place the ease ends.
//!  The toolkit's scroll moves any wheel distance in 180 ms (Slint 1.18.1 `items/flickable.rs`),
//!  so a far turn sweeps the view past the tile faster than the tile worker can build a tile for any step on the way.
//!  The window learns the distance from the wheel event itself,
//!  asks for the tile of the end at once,
//!  and asks for no tile for the steps that the tile of the end supersedes.

/// The window and its state,
///  the vertical mapping,
///  the tile's request and hold rule,
///  and the render a request ends in.
use super::{
    AppWindow, State, render, rows, tile,
    viewport::{holds, place_scrolled, request_window, wrap_width},
};
/// The winit events the window's event hook sees before the toolkit does.
use i_slint_backend_winit::winit::event::{MouseScrollDelta, WindowEvent};
/// The weak handles of a timer's callback and the single-shot timer.
use slint::{ComponentHandle, Timer, TimerMode};
/// Shared state and the clocks.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// What:
///  How long the toolkit's scroll takes to move a wheel distance,
///  whatever the distance.
/// Why:
///  `WHEEL_SCROLL_DURATION` in Slint 1.18.1 `items/flickable.rs`;
///  a turn that arrives while an ease runs adds to the end of that ease,
///  and after this time the end is the offset itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EASE_MS = 180;
/// ```
const EASE: Duration = Duration::from_millis(180);

/// What:
///  The time after an ease should have ended during which the window still follows it.
/// Why:
///  The last step of the ease reaches the end and releases the window,
///  but a scroll that something else cut short never reaches it;
///  the single wake-up at this time then asks for the tile of the offset the view stopped at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EASE_SLACK_MS = 220;
/// ```
const EASE_SLACK: Duration = Duration::from_millis(220);

/// What:
///  Logical pixels the toolkit's winit backend turns one wheel notch into.
/// Why:
///  `LineDelta(lx, ly) => (lx * 60., ly * 60.)` in `i-slint-backend-winit-1.18.1/winitwindowadapter.rs`;
///  the window reads the same events and must compute the same distance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOTCH = 60;
/// ```
const NOTCH: f32 = 60.0;

/// What:
///  The longest time between the event hook noting a turn and the scroll reporting that it took the turn.
/// Why:
///  Both happen in the dispatch of one event,
///  so a note older than this belongs to an event the scroll did not take
///  (a turn over the sidebar,
///  or Shift held).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TURN_WINDOW_MS = 50;
/// ```
const TURN_WINDOW: Duration = Duration::from_millis(50);

/// What:
///  A wheel ease the window knows the end of:
///  where it started,
///  where it ends,
///  when it started,
///  and whether the tile of the end has been requested so that the steps on the way ask for none.
/// Why:
///  The toolkit's scroll reports only the steps,
///  never its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ease = { from: number; dest: number; started: number; aimed: boolean };
/// ```
#[derive(Clone, Copy, Debug)]
pub(super) struct Ease {
    /// The offset the view stood at when the turn arrived.
    from: f32,
    /// The offset the ease ends at.
    dest: f32,
    /// When the turn arrived.
    started: Instant,
    /// Whether the tile of `dest` is requested in place of the tiles of the steps.
    aimed: bool,
}

/// What:
///  The wheel turns the window has noted:
///  the one the event hook saw that the scroll has not yet taken,
///  the ease the last one started,
///  and the wake-up that releases a followed ease nothing ended.
/// Why:
///  The window thread owns all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Wheel = { pending?: [distance: number, at: number]; ease?: Ease; end: Timer };
/// ```
#[derive(Default)]
pub(super) struct Wheel {
    /// The vertical distance of the last wheel event the hook saw,
    ///  in logical pixels,
    ///  and when.
    pending: Option<(f32, Instant)>,
    /// The ease the last turn started.
    ease: Option<Ease>,
    /// Releases the followed ease when it should have ended and no step ended it.
    end: Timer,
}

/// What:
///  Note a wheel event of the windowing system before the toolkit handles it.
/// Why:
///  The toolkit's scroll takes only the first turn of a burst into its filter before its children,
///  and gives the end of an ease to nobody;
///  the event hook sees every turn,
///  and the scroll's `flicked` report says which turns it took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteEvent(state: Shared<State>, event: WindowEvent): void;
/// ```
pub(super) fn note_event(state: &Rc<RefCell<State>>, event: &WindowEvent) {
    // Only a notched wheel is eased by the toolkit; a pan sets the offset step by step and asks for nothing.
    if let WindowEvent::MouseWheel {
        delta: MouseScrollDelta::LineDelta(_, lines),
        ..
    } = event
    {
        tracing::debug!(lines = *lines, "the windowing system turned the wheel");
        note_turn(state, lines * NOTCH);
    }
}

/// Remember a wheel turn of `distance` logical pixels (positive moves the text down,
///  as the toolkit's scroll does)
/// until the scroll reports that it took the turn.
pub(super) fn note_turn(state: &Rc<RefCell<State>>, distance: f32) {
    if let Ok(mut current) = state.try_borrow_mut() {
        current.wheel.pending = Some((distance, Instant::now()));
    }
}

/// What:
///  The source scroll took a turn or a pan:
///  work out where the ease ends and ask for the tile of that place.
/// Why:
///  A pan or a turn the hook did not see has no note and changes nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flicked(window: AppWindow, state: Shared<State>): void;
/// ```
pub(super) fn flicked(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let Ok(mut current) = state.try_borrow_mut() else {
        return;
    };
    // The toolkit reports `flicked` twice for one notched turn (inside its wheel branch and again at the end of the
    // event), so the note is taken, not read: the second report finds nothing and changes nothing.
    let pending = current.wheel.pending.take();
    drop(current);
    if let Some((distance, at)) = pending
        && at.elapsed() < TURN_WINDOW
    {
        turn(window, state, distance);
    }
}

/// What:
///  A wheel turn of `distance` logical pixels began an ease:
///  record where it ends,
///  and when the place of the end is not held by the displayed tile,
///  or by the tile already requested for an earlier turn,
///  request the one tile for it.
/// Why:
///  The ease ends at the offset it started at plus the distance,
///  plus what an unfinished ease before it had left
///  (`remaining_distance` in Slint's `flickable.rs`,
///  which is the end of that ease),
///  limited to the text.
///  A tile for the end lands while the ease still runs and holds the view when the ease arrives,
///  where a tile for each step would arrive after the view had left it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function turn(window: AppWindow, state: Shared<State>, distance: number): void;
/// ```
pub(super) fn turn(window: &AppWindow, state: &Rc<RefCell<State>>, distance: f32) {
    let height = window.get_viewport_height();
    let now = Instant::now();
    let mut current = state.borrow_mut();
    // The last offset a step or a placement reported: the toolkit's property already holds the first step of the ease
    // while its `flicked` report is being made, and may even hold the end of it.
    let real = current.offset;
    let mut base = real;
    let mut followed = false;
    if let Some(ease) = current.wheel.ease {
        let age = now.duration_since(ease.started);
        if age < EASE {
            base = ease.dest;
        }
        followed = ease.aimed && age < EASE + EASE_SLACK;
    }
    let dest = (base - distance).clamp(0.0, rows::limit(&current.row_map, height));
    let shown = current.shown;
    let mut covered = holds(&current, shown, dest, height);
    if !covered && followed {
        let wanted = tile::requested(&current);
        covered = holds(&current, wanted, dest, height);
    }
    // A width the tile was never wrapped to, or one the next render is to change, takes the usual path.
    let steady = shown.width > 0.0 && shown.width == current.width && !current.rewrap;
    let aimed = if covered { followed } else { steady };
    if !covered && steady {
        let wrap = current.width;
        request_window(&mut current, dest, wrap, height);
        current.scroll_wanted = Some(tile::requested(&current));
    }
    tracing::debug!(
        real,
        dest,
        distance,
        covered,
        aimed,
        "a wheel turn began an ease"
    );
    current.wheel.ease = Some(Ease {
        from: real,
        dest,
        started: now,
        aimed,
    });
    if aimed {
        arm_release(window, state, &current);
    }
}

/// Wake the window at the time a followed ease should have ended,
///  once,
/// replacing any earlier wake-up.
fn arm_release(window: &AppWindow, state: &Rc<RefCell<State>>, current: &State) {
    let weak_window = window.as_weak();
    let weak_state = Rc::downgrade(state);
    current
        .wheel
        .end
        .start(TimerMode::SingleShot, EASE + EASE_SLACK, move || {
            let (Some(active), Some(shared)) = (weak_window.upgrade(), weak_state.upgrade()) else {
                return;
            };
            finish(&active, &shared);
        });
}

/// What:
///  The followed ease should have ended and the steps did not end it:
///  ask for the tile of the offset the view stands at.
/// Why:
///  Something else moved the view (a map change that kept the text still,
///  for instance),
///  and without a last step nobody would ask for the tile the view needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(window: AppWindow, state: Shared<State>): void;
/// ```
fn finish(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let offset = -window.get_scroll_y();
    let width = window.get_viewport_width();
    let height = window.get_viewport_height();
    let mut current = state.borrow_mut();
    if aimed_ease(&current).is_none() {
        return;
    }
    tracing::debug!(
        offset,
        "a followed wheel ease ended without a step at its end"
    );
    release(&mut current);
    let changed = place_scrolled(&mut current, offset, width, height);
    drop(current);
    if changed {
        render(window, state);
    }
}

/// What:
///  The map under the end of the followed ease changed (rows arrived),
///  so ask for the tile of the same end again unless the displayed tile holds it.
/// Why:
///  The window asked for the tile of the end under the old map;
///  the lines that tile holds keep their numbers,
///  but their places moved with the new rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refit(current: State, width: number, height: number): void;
/// ```
pub(super) fn refit(current: &mut State, width: f32, height: f32) {
    let Some(ease) = aimed_ease(current) else {
        return;
    };
    let shown = current.shown;
    if holds(current, shown, ease.dest, height) {
        return;
    }
    let wrap = match wrap_width(width) {
        Some(known) => known,
        None => current.width,
    };
    request_window(current, ease.dest, wrap, height);
    current.scroll_wanted = Some(tile::requested(current));
}

/// The ease the window follows,
///  when it follows one.
fn aimed_ease(current: &State) -> Option<Ease> {
    return match current.wheel.ease {
        Some(ease) if ease.aimed => Some(ease),
        _ => None,
    };
}

/// Stop following the ease:
///  the steps ask for tiles again.
pub(super) fn release(current: &mut State) {
    if let Some(ease) = current.wheel.ease.as_mut() {
        ease.aimed = false;
    }
    current.wheel.end.stop();
}

/// What:
///  Whether a scroll step at `offset` is a step on the way to the end of an ease the window follows.
/// Why:
///  Such a step asks for no tile:
///  the tile of the end is requested or shown already.
///  The step at the end itself (within half a pixel) and every step after the time of the ease are ordinary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function following(current: State, offset: number): boolean;
/// ```
pub(super) fn following(current: &State, offset: f32) -> bool {
    let Some(ease) = aimed_ease(current) else {
        return false;
    };
    let on_the_way =
        offset >= ease.from.min(ease.dest) - 0.5 && offset <= ease.from.max(ease.dest) + 0.5;
    return ease.started.elapsed() < EASE + EASE_SLACK
        && on_the_way
        && (offset - ease.dest).abs() > 0.5;
}

/// Whether the tile requested for the window is not yet queued for the worker,
/// so a step that asks for nothing must still render once.
pub(super) fn unsent(current: &State) -> bool {
    return match current.scroll_wanted {
        Some(wanted) => current.tile_job != Some(wanted),
        None => false,
    };
}

/// What:
///  Whether a finished tile at `bounds` holds the end of the ease the window follows.
/// Why:
///  The tile of the end lands while the ease still runs and the view is far from it;
///  it is the tile that was asked for,
///  not a stale one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsEnd(current: State, bounds: TileBounds, height: number): boolean;
/// ```
pub(super) fn holds_end(current: &State, bounds: tile::TileBounds, height: f32) -> bool {
    let Some(ease) = aimed_ease(current) else {
        return false;
    };
    return holds(current, bounds, ease.dest, height);
}

/// Whether the window follows an ease to its end,
///  for the tests.
#[cfg(test)]
pub(super) fn is_aimed(current: &State) -> bool {
    return aimed_ease(current).is_some();
}
