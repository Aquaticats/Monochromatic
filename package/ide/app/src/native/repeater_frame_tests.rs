//! The rule of the toolkit behind the blank code column and the late minimap picture:
//!  what a frame shows of a repeater (a `for` in the markup) whose model the rendering notifier changed.
//!
//! Slint 1.18.1 makes the instances of a repeater before it begins a frame
//!  (`WindowInner::draw_contents` calls `ensure_tree_instantiated` first,
//!  `i-slint-core-1.18.1/window.rs` line 1846)
//!  and calls the rendering notifier inside the frame,
//!  after that and before the items are visited
//!  (`i-slint-renderer-femtovg-1.18.1/lib.rs` line 260).
//! The frame then draws the instances that exist,
//!  and the change shows only when it reaches an instance at once:
//!  a row set in a model that has it (`set_row_data`) updates its instance on the spot
//!  (`RepeaterTracker::row_changed`,
//!  `i-slint-core-1.18.1/model/repeater.rs` line 533);
//!  a model replaced in the property leaves the old instances to the frame
//!  (the model is read again only by `ensure_updated`,
//!  `model/repeater.rs` line 685);
//!  a model reset (`VecModel::set_vec`,
//!  and `show_slices` in `gpu_tile/types.rs` calls it when the number of rows changes)
//!  removes the instances and the frame draws none (`RepeaterTracker::reset`,
//!  `model/repeater.rs` line 612).
//! Nothing asks for the frame that does show the change:
//!  `evaluate_as_dependency_root` forgets the dependencies of the window's redraw tracker when the frame begins
//!  (`i-slint-core-1.18.1/properties.rs` line 1361),
//!  and the repeater registers its dirty flag again only when the items are visited,
//!  after the change
//!  (`Repeater::visit`,
//!  `model/repeater.rs` line 819),
//!  so the change marks no tracker and the window does not ask the adapter for a frame.
//! The test harness (`gpu_snapshot.rs`) draws with the same renderer and the same `draw_contents`,
//!  so it shows exactly this;
//!  an earlier note claimed that it draws every change of the notifier in the same snapshot,
//!  which these tests show is true of `set_row_data` alone.
//! The tests pass on Slint 1.18.1,
//!  and one of them will fail on a release that makes the frame show or ask for the change:
//!  the hand-over outside the frame (`GpuTile::submit_tile`,
//!  `GpuTile::submit_minimap`) can then be simplified.

/// The frames a window asked for,
///  and a frame drawn into pixels.
use super::gpu_snapshot::{install_gpu_platform, snapshot, take_frame_wanted};
/// The window handle,
///  the stages of a frame,
///  and the models a repeater shows.
use slint::{Color, ComponentHandle, Model, ModelRc, RenderingState, VecModel};
/// Shared ownership on the window thread.
use std::{cell::Cell, rc::Rc};

/// The markup of the probe:
///  one full-window rectangle for each color of the list.
mod probe_ui {
    slint::slint! {
        export component Probe inherits Window {
            in property <[color]> shades;
            preferred-width: 64px;
            preferred-height: 64px;
            background: black;
            for shade in root.shades : Rectangle {
                x: 0px;
                y: 0px;
                width: 64px;
                height: 64px;
                background: shade;
            }
        }
    }
}
/// The probe window.
use probe_ui::Probe;

/// What the repeater's model suffers.
#[derive(Clone, Copy, Debug)]
enum Change {
    /// A new model is set in the window's property.
    NewModel,
    /// The model the window shows is reset to a list of the same length.
    SetVecSameCount,
    /// The model the window shows is reset to a list of another length.
    SetVecOtherCount,
    /// One row of the model the window shows is replaced.
    SetRowData,
}

/// Where the change is made.
#[derive(Clone, Copy, Debug)]
enum Place {
    /// Inside the frame,
    ///  in the rendering notifier before the items are drawn.
    InFrame,
    /// Before the frame begins.
    BeforeFrame,
}

/// What the probe saw.
#[derive(Debug)]
struct Seen {
    /// The center pixel of the frame that began with the change,
    ///  or made it.
    that_frame: (u8, u8, u8),
    /// Whether the window asked for a frame after that frame.
    asked: bool,
    /// The center pixel of the frame that follows.
    next_frame: (u8, u8, u8),
}

/// The red of the first list and the blue of the changed one.
const RED: (u8, u8, u8) = (255, 0, 0);
/// The color of every pixel of a frame that draws no rectangle.
const BLACK: (u8, u8, u8) = (0, 0, 0);
/// The blue the change brings.
const BLUE: (u8, u8, u8) = (0, 0, 255);

/// The center pixel of a frame of `window`.
fn center(window: &Probe) -> (u8, u8, u8) {
    let shot = snapshot(window.window());
    let width = shot.width() as usize;
    let pixel = shot.as_slice()[32 * width + 32];
    return (pixel.r, pixel.g, pixel.b);
}

/// What:
///  Show a list of one red rectangle,
///  make `change` at `place`,
///  and draw the frame that follows it,
///  the window's ask for another,
///  and one frame more.
/// Why:
///  The pixels of the frame that holds the change,
///  and of the next,
///  are what a user sees of a change the notifier made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function observe(change: Change, place: Place): Seen;
/// ```
fn observe(change: Change, place: Place) -> Seen {
    install_gpu_platform();
    let window = Probe::new().expect("probe window");
    let blue = Color::from_rgb_u8(0, 0, 255);
    let list = Rc::new(VecModel::from(vec![Color::from_rgb_u8(255, 0, 0)]));
    window.set_shades(ModelRc::from(Rc::clone(&list)));
    window.show().expect("show the probe window");
    assert_eq!(center(&window), RED, "the first list is not drawn");
    take_frame_wanted();
    let make = {
        let weak_window = window.as_weak();
        let shown_list = Rc::clone(&list);
        move || match change {
            Change::NewModel => {
                if let Some(held) = weak_window.upgrade() {
                    held.set_shades(ModelRc::from(Rc::new(VecModel::from(vec![blue]))));
                }
            }
            Change::SetVecSameCount => shown_list.set_vec(vec![blue]),
            Change::SetVecOtherCount => shown_list.set_vec(vec![blue, blue]),
            Change::SetRowData => shown_list.set_row_data(0, blue),
        }
    };
    match place {
        Place::InFrame => {
            let armed = Cell::new(true);
            window
                .window()
                .set_rendering_notifier(move |state, _| {
                    if matches!(state, RenderingState::BeforeRendering) && armed.replace(false) {
                        make();
                    }
                })
                .expect("a rendering notifier");
        }
        Place::BeforeFrame => make(),
    }
    let that_frame = center(&window);
    let asked = take_frame_wanted();
    let next_frame = center(&window);
    window.hide().expect("hide the probe window");
    return Seen {
        that_frame,
        asked,
        next_frame,
    };
}

/// A model set in the property inside the frame leaves the frame drawing the old list,
///  and nothing asks for the frame that draws the new one.
#[test]
fn a_model_set_inside_a_frame_is_drawn_by_a_later_frame_that_nothing_asks_for() {
    let seen = observe(Change::NewModel, Place::InFrame);
    assert_eq!(
        seen.that_frame, RED,
        "the frame that set a new model drew it: {seen:?}"
    );
    assert!(
        !seen.asked,
        "the window asked for a frame after a model was set inside a frame: {seen:?}"
    );
    assert_eq!(
        seen.next_frame, BLUE,
        "the frame after the change did not draw the new model: {seen:?}"
    );
}

/// A model reset inside the frame,
///  to a list of the same length or another,
///  leaves the frame drawing nothing of the list,
///  and nothing asks for the frame that draws the new rows.
#[test]
fn a_model_reset_inside_a_frame_draws_nothing_in_it_and_asks_for_no_frame() {
    for change in [Change::SetVecSameCount, Change::SetVecOtherCount] {
        let seen = observe(change, Place::InFrame);
        assert_eq!(
            seen.that_frame, BLACK,
            "{change:?}: the frame that reset the model drew rows of it: {seen:?}"
        );
        assert!(
            !seen.asked,
            "{change:?}: the window asked for a frame after a model was reset inside a frame: {seen:?}"
        );
        assert_eq!(
            seen.next_frame, BLUE,
            "{change:?}: the frame after the reset did not draw the new rows: {seen:?}"
        );
    }
}

/// A row replaced inside the frame in the model that has it is drawn by that frame:
///  the instance is updated when the row changes.
#[test]
fn a_row_replaced_inside_a_frame_is_drawn_by_that_frame() {
    let seen = observe(Change::SetRowData, Place::InFrame);
    assert_eq!(
        seen.that_frame, BLUE,
        "the frame that replaced a row did not draw it: {seen:?}"
    );
    assert_eq!(
        seen.next_frame, BLUE,
        "the frame after the change drew another row: {seen:?}"
    );
}

/// The same changes made before the frame begins are drawn by the first frame,
///  which is the hand-over of the tile and the picture outside the frame.
#[test]
fn every_change_made_before_a_frame_is_drawn_by_that_frame() {
    for change in [
        Change::NewModel,
        Change::SetVecSameCount,
        Change::SetVecOtherCount,
        Change::SetRowData,
    ] {
        let seen = observe(change, Place::BeforeFrame);
        assert_eq!(
            seen.that_frame, BLUE,
            "{change:?}: the frame after a change made before it did not draw the change: {seen:?}"
        );
        assert_eq!(
            seen.next_frame, BLUE,
            "{change:?}: the frame after that drew something else: {seen:?}"
        );
    }
}
