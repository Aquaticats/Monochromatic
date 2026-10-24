//! A tile of another size is shown by a frame that follows its hand-over.
//!  The GPU draws the scene into textures during a frame,
//!  and a tile whose textures differ in size from the last one's gets new textures,
//!  which the window learns of only inside that frame (`Outcome::apply` in the rendering notifier):
//!  the real window draws nothing in the code column in that frame,
//!  and an idle window draws no frame by itself,
//!  so the frame that replaced the textures must ask for the next,
//!  or the code column stays blank beside the new rows until some other event draws a frame.
//! The test harness draws every property the notifier set in the same snapshot,
//!  so it cannot show the blank column;
//!  these tests draw frames the way an event loop does,
//!  one for each time a frame was asked for,
//!  and check that a second one was asked for.

/// The reader fixture, the window, and the source state.
use super::find_tests::Reader;
/// The frames an event loop would draw.
use super::gpu_snapshot::draw_wanted_frames;
/// A press on the minimap, the viewport box, and the settling of the tile.
use super::minimap_tests::{calm, geometry, lift, over, press};
/// Opening a long plain text and waiting for its first layout.
use super::wrap_tests::{open, settled};
/// The window handle.
use slint::ComponentHandle;

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
///  A reader on the numbered text at its start,
///  with the frames it asked for drawn.
///  The textures of its tile are returned with it.
/// Why:
///  The press that follows must hand over a tile of another size,
///  which the test checks against these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scene(): [TempDir, Reader, Array<[width: number, height: number]>];
/// ```
fn scene() -> (tempfile::TempDir, Reader, Vec<(u32, u32)>) {
    let (project, opened) = open(&numbered());
    settled(&opened);
    calm(&opened);
    draw_wanted_frames(opened.window.window());
    let sizes = opened.source.borrow().gpu.tile_texture_sizes();
    assert!(!sizes.is_empty(), "the start drew no tile");
    return (project, opened, sizes);
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
///  the frame that gave the window the new textures asks for the next,
///  which shows them.
#[test]
fn a_frame_that_replaces_the_textures_of_the_tile_asks_for_the_next() {
    let (_project, opened, before) = scene();
    press_at(&opened, 0.5);
    let drawn = draw_wanted_frames(opened.window.window());
    let after = opened.source.borrow().gpu.tile_texture_sizes();
    assert_ne!(
        before, after,
        "the press kept the textures of the tile, so the test cannot tell whether new ones are shown"
    );
    assert!(
        drawn.frames >= 2,
        "the frame that gave the window new textures asked for no frame to show them: {} drawn",
        drawn.frames
    );
    opened.window.hide().expect("close tile window");
}
