//! Taking in the finished pictures:
//!  polling the worker's replies,
//!  and handing to the GPU each one that is still valid for the displayed text.

/// The reply type of the worker.
use crate::native::minimap_worker::Reply;
/// The window,
///  its state,
///  and the displayed text's stamp.
use crate::native::{AppWindow, State, annotate::displayed};
/// The list of the picture's textures,
///  weak handles,
///  and the timers.
use slint::{ComponentHandle, ModelRc, TimerMode};
/// Shared state and its cell,
///  and the poll period.
use std::{cell::RefCell, rc::Rc, time::Duration};

/// Poll period of the worker's replies,
/// the same as the other background pollers.
const POLL: Duration = Duration::from_millis(20);

/// What:
///  How long a picture handed to the GPU waits for a frame that comes anyway before the window asks for one.
/// Why:
///  A scroll draws a frame at every step of the input,
/// so a picture that arrives between two steps is drawn by the next one;
/// a frame asked for at the hand-over is a second frame in that step's time.
///  In the nested compositor's recording the text then did not move for one tick,
/// in every scroll measured (`doc/handover/ide-gpu-stages-5-6.md`).
///  A frame period of a display at 50 Hz or more is shorter than this wait,
/// so the picture is drawn by a frame of the scroll before the wait ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FRAME_WAIT_MS = 20;
/// ```
const FRAME_WAIT: Duration = Duration::from_millis(20);

/// How many frames were asked for to draw a picture,
/// counted for the tests:
/// a window that draws frames anyway must not ask for one.
#[cfg(test)]
pub(in crate::native) static PICTURE_FRAMES_ASKED: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// What:
///  Ask for a frame to draw the picture just handed to the GPU,
/// unless a frame draws it first.
///  The request is made once `FRAME_WAIT` has passed and the picture is still waiting.
/// Why:
///  The GPU draws the picture just before a frame,
/// and a picture changes no property the window watches,
/// so an idle window would keep the old picture until something else drew a frame.
///  A window that is already drawing frames,
/// as a scroll does,
/// draws the picture in the next one without a frame of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askForAFrameUnlessOneComes(window: AppWindow, state: Shared<State>): void;
/// ```
fn ask_for_a_frame_unless_one_comes(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let weak_window = window.as_weak();
    let weak_state = Rc::downgrade(state);
    state
        .borrow()
        .minimap
        .redraw
        .start(TimerMode::SingleShot, FRAME_WAIT, move || {
            let (Some(active), Some(shared)) = (weak_window.upgrade(), weak_state.upgrade()) else {
                return;
            };
            let gpu = shared.borrow().gpu.clone();
            if !gpu.minimap_waiting() {
                return;
            }
            tracing::debug!("asking for a frame to draw a minimap picture no frame had drawn");
            ask_for_a_frame(&active);
        });
}

/// Ask the toolkit for a frame,
/// which draws the picture waiting for one.
fn ask_for_a_frame(window: &AppWindow) {
    #[cfg(test)]
    PICTURE_FRAMES_ASKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    window.window().request_redraw();
}

/// What:
///  Hand a finished picture to the GPU when it is for the displayed text and not older than the one on screen;
///  otherwise count it as dropped.
///  This is the window thread's whole cost of a picture:
/// one comparison and the hand-over of a scene.
///  The GPU draws the scene into the minimap's texture before the next frame
/// and sets the image and its place on the window in the same step.
/// Why:
///  A picture of an older text,
/// or of an older request,
/// must never replace a newer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function apply(window: AppWindow, state: Shared<State>, reply: Reply): void;
/// ```
pub(in crate::native) fn apply(window: &AppWindow, state: &Rc<RefCell<State>>, reply: Reply) {
    let mut current = state.borrow_mut();
    if reply.id == current.minimap.awaited {
        current.minimap.awaited = 0;
    }
    let fresh = reply.id >= current.minimap.shown_id && reply.key.stamp == displayed(&current);
    if !fresh {
        #[cfg(debug_assertions)]
        {
            current.minimap.dropped += 1;
        }
        tracing::debug!(id = reply.id, "dropped a stale minimap picture");
        return;
    }
    current.minimap.shown_id = reply.id;
    current.minimap.shown = Some(reply.key);
    #[cfg(debug_assertions)]
    {
        current.minimap.images_set += 1;
        current.minimap.shaped += reply.stats.shaped as u64;
        current.minimap.last_stats = Some(reply.stats);
    }
    let gpu = current.gpu.clone();
    drop(current);
    if reply.chunks.is_empty() {
        gpu.forget_minimap();
        window.set_minimap_slices(ModelRc::default());
        window.set_minimap_picture_width(0.0);
        window.set_minimap_picture_height(0.0);
        window.set_minimap_picture_top(reply.top as f32);
    } else {
        // The markup draws the picture at exactly this many physical pixels, so no pixel of it is resampled, and
        // starts it at the row of the whole picture the worker built it from; the GPU sets these with the images
        // of its textures, which are as many as the device's texture limit asks for.
        gpu.submit_minimap(window, reply.chunks, reply.top);
        ask_for_a_frame_unless_one_comes(window, state);
    }
    tracing::debug!(
        id = reply.id,
        top = reply.top,
        elapsed_us = reply.elapsed.as_micros() as u64,
        "set a minimap picture"
    );
}

/// Take in the finished pictures:
/// each one that is still valid for the displayed text is set on the window.
fn poll(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let replies = {
        let mut current = state.borrow_mut();
        let Some(worker) = current.minimap.worker.as_mut() else {
            current.minimap.poll.stop();
            return;
        };
        match worker.poll() {
            Ok(replies) => replies,
            Err(error) => {
                tracing::error!(%error, "minimap worker stopped");
                current.minimap.worker = None;
                current.minimap.posted = None;
                current.minimap.awaited = 0;
                current.minimap.poll.stop();
                return;
            }
        }
    };
    for reply in replies {
        apply(window, state, reply);
    }
    let current = state.borrow_mut();
    if current.minimap.awaited == 0 {
        current.minimap.poll.stop();
    }
}

/// Start the repeated poll of the worker's replies unless it is already running.
pub(super) fn start_poll(window: &AppWindow, state: &Rc<RefCell<State>>, current: &mut State) {
    if current.minimap.poll.running() {
        return;
    }
    let weak_window = window.as_weak();
    let weak_state = Rc::downgrade(state);
    // The poll holds weak handles only, so the timer never keeps the window or its state alive.
    current
        .minimap
        .poll
        .start(TimerMode::Repeated, POLL, move || {
            let (Some(active), Some(shared)) = (weak_window.upgrade(), weak_state.upgrade()) else {
                return;
            };
            poll(&active, &shared);
        });
}
