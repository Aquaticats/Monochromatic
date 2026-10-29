//! The window's side of a request:
//!  what a picture made now would be valid for,
//!  and posting a request to the worker when that changes,
//!  at the pace `pace.rs` decides.

/// What a change of the key calls for,
///  and when.
use crate::native::minimap::pace::{Plan, plan};
/// A request that is awaited is answered by polling the worker.
use crate::native::minimap::picture::start_poll;
/// The worker,
///  its request type,
///  and the key a picture is valid for.
use crate::native::minimap_worker::{Job, Key, MinimapWorker};
/// The window,
///  its state,
///  the displayed text's stamp,
///  the palette,
///  and what the window draws over the text.
use crate::native::{
    AppWindow, State,
    annotate::{colors as annotation_colors, displayed},
    find::present::present,
    render::code_colors,
};
/// The matches the window paints.
use ide_app::find::FindRange;
/// The match list the find paints.
use ide_app::find_navigation::paint_ranges;
/// The inks of the selection and match fills,
/// the geometry that names the bands the track shows,
/// and the size of a band.
use ide_app::minimap::{BAND, Fills, Geometry, Heading};
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;
/// The list of the picture's textures,
///  weak handles,
///  and the timers.
use slint::{ComponentHandle, ModelRc, TimerMode};
/// Shared state and its cell,
///  the syntax snapshot a job shares,
///  and the clock of the newest request.
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Instant};

/// What the window draws over the text that the picture shows with it:
/// the inks of the selection and match fills.
fn fills_of(window: &AppWindow, active: bool) -> Fills {
    let selection = window.get_selection_fill().color();
    let foreground = code_colors(window).foreground;
    return Fills {
        selection: [
            selection.red(),
            selection.green(),
            selection.blue(),
            selection.alpha(),
        ],
        foreground,
        boundary: active,
    };
}

/// What:
///  What a picture made now would be valid for,
/// or nothing while the minimap has no size.
/// Why:
///  A request is posted only when this differs from the last one;
/// it holds the bands the track shows with the view scrolled where it is,
/// not the scroll offset,
/// so a scroll step that keeps the bands never changes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyOf(window: AppWindow, current: State): Key | undefined;
/// ```
pub(super) fn key_of(window: &AppWindow, current: &State) -> Option<Key> {
    let (width, height) = current.minimap.size;
    if width < 1.0 || height < 1.0 {
        return None;
    }
    let position = current.document.position();
    let selection = if position.anchor == position.head {
        None
    } else {
        Some((
            position.anchor.min(position.head),
            position.anchor.max(position.head),
        ))
    };
    let ranges = paint_ranges(
        &current.find,
        current.file_generation,
        current.document.revision(),
    );
    let find = if ranges.is_empty() {
        (0, 0)
    } else {
        (Arc::as_ptr(&ranges).cast::<()>() as usize, ranges.len())
    };
    let active = present(current).active;
    let geometry = Geometry::new(current.row_map.height(), height);
    // A track taller than one texture is drawn as several that stack, so every band the track shows is kept.
    // The side the view is heading toward holds extra bands, so a fling does not outrun the picture.
    let bands = geometry.bands_toward(
        -window.get_scroll_y(),
        BAND,
        usize::MAX,
        current.minimap.heading,
    );
    return Some(Key {
        stamp: displayed(current),
        styles: Arc::as_ptr(&current.styles).cast::<()>() as usize,
        rows: current.row_key,
        wrap: if has_wrap_width(current.width) {
            current.width.to_bits()
        } else {
            0
        },
        colors: code_colors(window),
        scale: window.window().scale_factor().to_bits(),
        size: (width.to_bits(), height.to_bits()),
        window: (bands.start, bands.end),
        texture_limit: ide_app::shaped_text::texture_dimension(),
        selection,
        find,
        active: active.map(range_pair),
        annotations: current.annotations.version(),
        inks: (
            fills_of(window, active.is_some()),
            annotation_colors(window),
        ),
    });
}

/// A match as the two characters it lies between.
fn range_pair(range: FindRange) -> (usize, usize) {
    return (range.start, range.end);
}

/// What:
///  Update which way the view is heading from where the top of the track is now.
/// Why:
///  The picture holds extra bands on the side the view is heading toward;
/// the heading must be settled before the key is made,
/// and it changes only after a whole band of movement,
/// so a step of a scroll never changes the key by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function follow(window: AppWindow, current: State): void;
/// ```
fn follow(window: &AppWindow, current: &mut State) {
    let Some(top) = track_top(window, current) else {
        return;
    };
    current.minimap.heading = current.minimap.heading.follow(top, BAND);
}

/// The top of the part of the picture the track shows,
/// in logical pixels of the text,
/// or nothing while the minimap has no size.
fn track_top(window: &AppWindow, current: &State) -> Option<f32> {
    let (width, height) = current.minimap.size;
    if width < 1.0 || height < 1.0 {
        return None;
    }
    let geometry = Geometry::new(current.row_map.height(), height);
    return Some(geometry.track_top(-window.get_scroll_y()));
}

/// Post a request for the picture `key` describes,
/// starting the worker on first use.
pub(super) fn post(window: &AppWindow, state: &Rc<RefCell<State>>, current: &mut State, key: Key) {
    if current.minimap.worker.is_none() {
        match MinimapWorker::new(current.shaper.fork()) {
            Ok(worker) => current.minimap.worker = Some(worker),
            Err(error) => {
                tracing::error!(%error, "minimap worker could not start");
                return;
            }
        }
    }
    current.minimap.next_id += 1;
    let id = current.minimap.next_id;
    let text = current.document.text();
    let job = Job {
        id,
        key,
        document: current.document.clone(),
        styles: Arc::clone(&current.styles),
        typography: current.shaper.typography().clone(),
        map: current.row_map.clone(),
        colors: key.colors,
        inks: key.inks.1,
        fills: key.inks.0,
        blocks: current.blocks.clone(),
        marks: current
            .annotations
            .marks(displayed(current), text, 0, text.len_lines()),
        ranges: paint_ranges(
            &current.find,
            current.file_generation,
            current.document.revision(),
        ),
        active: present(current).active,
        wrap: has_wrap_width(current.width).then_some(current.width),
        scale: window.window().scale_factor(),
        size: current.minimap.size,
    };
    let Some(worker) = current.minimap.worker.as_mut() else {
        return;
    };
    if let Err(error) = worker.post(job) {
        tracing::error!(%error, "minimap request failed");
        current.minimap.worker = None;
        current.minimap.posted = None;
        return;
    }
    current.minimap.posted = Some(key);
    current.minimap.last_post = Some(Instant::now());
    current.minimap.awaited = id;
    #[cfg(debug_assertions)]
    {
        current.minimap.posts += 1;
    }
    tracing::debug!(
        id,
        lines = current.row_map.lines(),
        height = current.row_map.height(),
        "posted a minimap request"
    );
    start_poll(window, state, current);
}

/// Post the request that is due now,
/// if what the picture is valid for changed since the last one.
fn flush(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let mut current = state.borrow_mut();
    let Some(key) = key_of(window, &current) else {
        return;
    };
    if current.minimap.posted == Some(key) {
        return;
    }
    post(window, state, &mut current, key);
}

/// What:
///  Called after every render and when the minimap is resized:
///  post a request at once for the first picture of a file and for a new palette or scale,
///  at a steady pace while only the vertical mapping changes,
///  and after the quiet time for every other change.
///  A step that changes nothing the picture depends on costs a comparison of a few words.
/// Why:
///  The picture follows the text,
/// the syntax classes,
/// the wrap width,
/// the vertical mapping,
/// the size,
/// and the bands the track shows;
///  the scroll offset is none of them,
/// because the viewport box and the picture's slide move in the markup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function note(window: AppWindow, state: Shared<State>): void;
/// ```
pub(in crate::native) fn note(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let mut current = state.borrow_mut();
    follow(window, &mut current);
    let Some(key) = key_of(window, &current) else {
        return;
    };
    // What this look is for, kept for the next one to tell what changed in between.
    let before = current.minimap.noted.replace(key);
    if current.minimap.posted == Some(key) {
        return;
    }
    let other_file = match current.minimap.shown {
        Some(shown) => shown.stamp.file != key.stamp.file,
        None => false,
    };
    if other_file {
        // Another file: the picture on screen is wrong, not just old, and the heading was the other text's.
        if let Some(top) = track_top(window, &current) {
            current.minimap.heading = Heading::Still { at: top };
        }
        let Some(at_rest) = key_of(window, &current) else {
            return;
        };
        // The look just kept was made with the other text's heading.
        current.minimap.noted = Some(at_rest);
        current.minimap.shown = None;
        let gpu = current.gpu.clone();
        drop(current);
        gpu.forget_minimap();
        window.set_minimap_slices(ModelRc::default());
        let mut again = state.borrow_mut();
        again.minimap.debounce.stop();
        post(window, state, &mut again, at_rest);
        return;
    }
    let mut since = None;
    if let Some(at) = current.minimap.last_post {
        since = Some(at.elapsed());
    }
    let wait = match plan(
        current.minimap.posted,
        before,
        key,
        since,
        current.minimap.debounce.running(),
    ) {
        Plan::Post => {
            current.minimap.debounce.stop();
            post(window, state, &mut current, key);
            return;
        }
        Plan::Keep => return,
        Plan::Wait(wait) => wait,
    };
    let weak_window = window.as_weak();
    let weak_state = Rc::downgrade(state);
    current
        .minimap
        .debounce
        .start(TimerMode::SingleShot, wait, move || {
            let (Some(active), Some(shared)) = (weak_window.upgrade(), weak_state.upgrade()) else {
                return;
            };
            flush(&active, &shared);
        });
}
