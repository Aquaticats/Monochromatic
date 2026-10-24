//! A tile of another size reaches the window before the frame that draws it.
//!  The GPU draws the scene into textures during a frame,
//!  and a tile whose textures differ in size from the last one's needs new textures and a new list of slices
//!  (`source-slices`,
//!  the list the markup draws with a `for`).
//!  A list the window is given inside the frame is not shown by that frame (the real window draws nothing in the
//!  code column in it),
//!  and an idle window draws no frame by itself,
//!  so a tile handed over for the next frame must bring its slices with it:
//!  otherwise the code column stays blank beside the new rows until some other event draws a frame.
//! The test harness draws every property the notifier set in the same snapshot,
//!  so it cannot show the blank column;
//!  these tests check the contract the real window needs:
//!  when a tile is handed over and before any frame is drawn,
//!  the window holds slices of the new textures.

/// A press on the minimap,
///  the viewport box,
///  and the settling of the tile.
use super::minimap_tests::{calm, geometry, lift, over, press};
/// One frame,
///  which draws the scenes waiting for it.
use super::tile_reads::draw_minimap;
/// Opening a long plain text and waiting for its first layout.
use super::wrap_tests::{open, settled};
/// The reader fixture,
///  the window,
///  and the source state.
use super::{AppWindow, State, find_tests::Reader};
/// The window handle and the model trait.
use slint::{ComponentHandle, Model};
/// The shared source state.
use std::{cell::RefCell, rc::Rc};

/// Lines of the fixture text.
const LINES: usize = 1500;

/// The text:
/// `LINES` numbered lines of one length each.
fn numbered() -> String {
    let mut text = String::new();
    for index in 0..LINES {
        text.push_str(&format!("line {index:04}: value\n"));
    }
    return text;
}

/// What:
///  The heights in physical pixels of the slices the window draws the tile with,
///  top first.
/// Why:
///  The slices are what the markup shows,
///  and each is as tall as the texture it shows,
///  so they say which textures the next frame draws.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sliceHeights(window: AppWindow): number[];
/// ```
pub(super) fn slice_heights(window: &AppWindow) -> Vec<u32> {
    let slices = window.get_source_slices();
    let mut heights = Vec::with_capacity(slices.row_count());
    for row in 0..slices.row_count() {
        let slice = slices.row_data(row).expect("a slice of the list");
        heights.push(slice.height as u32);
    }
    return heights;
}

/// The heights of the textures the GPU holds for the tile,
///  top first.
pub(super) fn texture_heights(source: &Rc<RefCell<State>>) -> Vec<u32> {
    let sizes = source.borrow().gpu.tile_texture_sizes();
    return sizes.iter().map(|size| return size.1).collect();
}

/// What:
///  A reader on the numbered text at its start,
///  with the frame that draws its first tile drawn.
///  The heights of the slices it shows are returned with it.
/// Why:
///  The press that follows must hand over a tile of another size,
///  which the test checks against these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scene(): [TempDir, Reader, number[]];
/// ```
fn scene() -> (tempfile::TempDir, Reader, Vec<u32>) {
    let (project, opened) = open(&numbered());
    settled(&opened);
    calm(&opened);
    draw_minimap(&opened.window);
    let heights = slice_heights(&opened.window);
    assert!(!heights.is_empty(), "the start drew no tile");
    return (project, opened, heights);
}

/// What:
///  Press the minimap at `share` of its track,
///  which is outside the viewport box at the start of the text,
///  and let the tile of the new view reach the window.
/// Why:
///  A press far from the start makes the tile a different height from the first one:
///  the first tile is cut short by the start of the text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pressAt(reader: Reader, share: number): void;
/// ```
fn press_at(reader: &Reader, share: f32) {
    let y = geometry(reader).track() * share;
    press(reader, over(&reader.window, 10.0, y));
    lift(reader, over(&reader.window, 10.0, y));
    calm(reader);
}

/// A press on the minimap hands a tile of another height to the window:
///  before any frame is drawn the window already holds slices of the new textures,
///  so the frame that draws the scenes into them shows them.
#[test]
fn a_tile_of_another_size_gives_the_window_its_slices_before_any_frame() {
    let (_project, opened, before) = scene();
    press_at(&opened, 0.5);
    let shown = slice_heights(&opened.window);
    assert_ne!(
        shown, before,
        "the window holds the slices of the earlier tile after the press: a frame would draw the scenes into textures it does not show"
    );
    assert_eq!(
        shown,
        texture_heights(&opened.source),
        "the slices the window holds are not those of the textures the GPU draws the tile into"
    );
    opened.window.hide().expect("close tile window");
}
