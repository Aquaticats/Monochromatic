//! What the window keeps for the minimap besides the picture,
//!  and the drag of the viewport box that is part of it.

/// The press the next change of the layout maps again.
use crate::native::minimap::aim::Aim;
/// The key the newest request and the picture on screen are valid for,
///  and the worker the window starts on first use.
use crate::native::minimap_worker::{Key, MinimapWorker};
/// Which way the view is heading.
use ide_app::minimap::Heading;
/// The three timers the state owns:
///  one polls the worker's replies,
///  one holds the quiet time,
///  and one waits for a frame to draw a picture.
use slint::Timer;
/// When the newest request was posted.
use std::time::Instant;

/// What:
///  Where a drag of the viewport box started:
/// the pointer's distance from the top of the minimap and the scroll
///  offset at that moment.
/// Why:
///  The box follows the pointer one to one from where it was grabbed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drag = { startY: number; startOffset: number };
/// ```
#[derive(Clone, Copy)]
pub(super) struct Drag {
    /// Pointer distance from the top of the minimap at the press.
    pub(super) start_y: f32,
    /// Scroll offset at the press,
    /// or after the jump the press made.
    pub(super) start_offset: f32,
}

/// What:
///  Everything the window keeps for the minimap besides the picture,
/// which the toolkit holds.
/// Why:
///  One value in the state,
/// so the minimap adds a single field there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MinimapState = { size: [number, number]; worker?: Worker; posted?: Key; shown?: Key; drag?: Drag };
/// ```
pub(in crate::native) struct MinimapState {
    /// Logical width and height of the minimap column,
    /// as the markup last reported them;
    /// zero before the layout.
    pub(in crate::native) size: (f32, f32),
    /// The worker thread,
    /// started with the first request.
    pub(super) worker: Option<MinimapWorker>,
    /// Polls the worker's replies while one is awaited.
    pub(super) poll: Timer,
    /// Posts a request after the quiet time.
    pub(super) debounce: Timer,
    /// Asks for a frame when a picture handed to the GPU is still waiting after the time a frame needs to come.
    pub(super) redraw: Timer,
    /// What the newest request was for.
    pub(in crate::native) posted: Option<Key>,
    /// When the newest request was posted.
    pub(super) last_post: Option<Instant>,
    /// What the previous look at the window,
    /// in `note`,
    /// was for.
    pub(super) noted: Option<Key>,
    /// What the picture on screen is valid for.
    pub(in crate::native) shown: Option<Key>,
    /// Which way the view last moved by a whole band,
    /// which names the side of the picture that holds the extra bands.
    pub(in crate::native) heading: Heading,
    /// Number of the last request posted.
    pub(super) next_id: u64,
    /// Number of the request whose picture is on screen.
    pub(in crate::native) shown_id: u64,
    /// Number of the request whose picture is awaited,
    /// or zero.
    pub(super) awaited: u64,
    /// The drag in progress.
    pub(super) drag: Option<Drag>,
    /// The last press that nothing has followed,
    /// which the next change of the layout maps again.
    pub(super) aim: Option<Aim>,
    /// Requests posted,
    /// counted in debug builds for the structural tests.
    #[cfg(debug_assertions)]
    pub(in crate::native) posts: u64,
    /// Pictures set on the window,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    pub(in crate::native) images_set: u64,
    /// Finished pictures dropped as stale,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    pub(in crate::native) dropped: u64,
    /// Lines shaped for the pictures handed to the GPU,
    /// summed in debug builds.
    #[cfg(debug_assertions)]
    pub(in crate::native) shaped: u64,
    /// The cost of the picture on screen,
    /// kept in debug builds.
    #[cfg(debug_assertions)]
    pub(in crate::native) last_stats: Option<ide_app::minimap::Stats>,
}

/// A minimap that has built nothing yet.
impl MinimapState {
    /// The state of a window before its first layout.
    pub(in crate::native) fn new() -> Self {
        return Self {
            size: (0.0, 0.0),
            worker: None,
            poll: Timer::default(),
            debounce: Timer::default(),
            redraw: Timer::default(),
            posted: None,
            last_post: None,
            noted: None,
            shown: None,
            heading: Heading::Still { at: 0.0 },
            next_id: 0,
            shown_id: 0,
            awaited: 0,
            drag: None,
            aim: None,
            #[cfg(debug_assertions)]
            posts: 0,
            #[cfg(debug_assertions)]
            images_set: 0,
            #[cfg(debug_assertions)]
            dropped: 0,
            #[cfg(debug_assertions)]
            shaped: 0,
            #[cfg(debug_assertions)]
            last_stats: None,
        };
    }
}
