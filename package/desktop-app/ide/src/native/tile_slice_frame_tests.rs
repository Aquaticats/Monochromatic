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
//! The first tile of a window is the one tile that cannot do this:
//!  the window's device arrives with the first frame,
//!  so a tile handed over before it has no textures to make,
//!  and the notifier makes them inside that frame.
//!  The window does not show them in it,
//!  so the notifier asks for the next frame;
//!  the tests of that step read the ask,
//!  because the harness snapshot draws the slices in the frame that set them.

/// The frames a window asked for.
use super::gpu_snapshot::{install_gpu_platform, snapshot, take_frame_wanted};
/// The notifier of a window and the handle the state holds.
use super::gpu_tile::{GpuTile, bind};
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
/// The scenes of a tile.
use ide_app::tile_scene::{TileScene, TileStack};
/// The window handle and the model trait.
use slint::{ComponentHandle, Model};
/// The shared source state.
use std::{cell::RefCell, rc::Rc};
/// An empty scene:
///  the textures are made and drawn,
///  with nothing in them,
///  and the scene that fills a texture with one color.
use vello::{
    Scene,
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};

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

/// Width of the textures of the stacks the first-tile tests hand over.
pub(super) const STACK_WIDTH: u32 = 64;

/// What:
///  A tile of `heights.len()` textures `STACK_WIDTH` wide and `heights` tall,
///  top first,
///  whose scenes are empty.
/// Why:
///  The notifier makes textures and slices from the sizes alone,
///  so the tests need no text to shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stack(heights: number[]): TileStack;
/// ```
fn stack(heights: &[u32]) -> TileStack {
    let mut textures = Vec::with_capacity(heights.len());
    for height in heights {
        textures.push(TileScene {
            scene: Scene::new(),
            width: STACK_WIDTH,
            height: *height,
        });
    }
    return TileStack {
        textures,
        width: STACK_WIDTH,
        height: heights.iter().sum(),
    };
}

/// What:
///  A shown window of the application's markup with the notifier bound to a new `GpuTile`,
///  before any frame is drawn.
///  The ask for a frame that showing it made is forgotten.
/// Why:
///  The device reaches the notifier in the first frame,
///  so until a snapshot is drawn the tile is the first one of a window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unframedWindow(): [AppWindow, GpuTile];
/// ```
pub(super) fn unframed() -> (AppWindow, GpuTile) {
    install_gpu_platform();
    let window = AppWindow::new().expect("window of the application's markup");
    let gpu = GpuTile::default();
    bind(&window, &gpu);
    window.show().expect("show the window");
    take_frame_wanted();
    return (window, gpu);
}

/// The first tile of a window is handed over before the device is held:
///  the window holds no slices until the first frame,
///  that frame makes the textures and gives the window their slices,
///  and it asks for the next frame,
///  which is the one that shows them.
#[test]
fn the_first_tile_of_a_window_gets_its_slices_in_the_first_frame_and_asks_for_the_next() {
    let (window, gpu) = unframed();
    gpu.submit_tile(&window, stack(&[40, 24]));
    assert!(
        slice_heights(&window).is_empty(),
        "the window held slices before any device was held, so the test does not reach the in-frame step"
    );
    assert!(
        gpu.tile_texture_sizes().is_empty(),
        "textures were made before the first frame, so the test does not reach the in-frame step"
    );
    // The first frame: setup, then the scenes drawn.
    let _frame = snapshot(window.window());
    let asked = take_frame_wanted();
    assert_eq!(
        gpu.tile_texture_sizes(),
        vec![(STACK_WIDTH, 40), (STACK_WIDTH, 24)],
        "the first frame did not make the textures of the tile"
    );
    assert_eq!(
        slice_heights(&window),
        vec![40, 24],
        "the first frame did not give the window the slices of the tile it drew"
    );
    assert!(
        asked,
        "the frame that gave the window the slices of the first tile asked for no frame to show them"
    );
    window.hide().expect("close tile window");
}

/// A frame that gives the window no new slices asks for nothing:
///  a tile of the shape the first one had is drawn without a request,
///  and a tile of another shape,
///  handed over once the device is held,
///  brings its slices before the frame
///  and asks for nothing in it.
/// The request of the first frame is therefore the notifier's own and not something every frame makes.
#[test]
fn a_frame_that_makes_no_slices_in_it_asks_for_no_frame() {
    let (window, gpu) = unframed();
    gpu.submit_tile(&window, stack(&[40, 24]));
    let _first = snapshot(window.window());
    assert!(
        take_frame_wanted(),
        "the first frame asked for no frame, so the test cannot tell a frame that asks from one that does not"
    );
    // The frame that shows the slices: the toolkit makes the images of the list there and may ask for a frame itself.
    let _shown = snapshot(window.window());
    // The same shape: the textures are drawn again, no slices are made.
    gpu.submit_tile(&window, stack(&[40, 24]));
    take_frame_wanted();
    let _same = snapshot(window.window());
    assert!(
        !take_frame_wanted(),
        "a frame that made no slices asked for the next"
    );
    // Another shape, with the device held: the window has the slices before the frame.
    gpu.submit_tile(&window, stack(&[30, 30, 8]));
    assert_eq!(
        slice_heights(&window),
        vec![30, 30, 8],
        "a tile handed over once the device is held did not bring its slices"
    );
    take_frame_wanted();
    let _other = snapshot(window.window());
    assert!(
        !take_frame_wanted(),
        "the frame that drew a tile whose slices were already shown asked for the next"
    );
    window.hide().expect("close tile window");
}

/// The color the painted stacks are filled with:
///  nothing else in the window is this color.
const INK: (u8, u8, u8) = (255, 0, 255);

/// What:
///  A scene that fills a texture of `width` by `height` with [`INK`].
/// Why:
///  The window's pixels then say how much of the tile a frame drew.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filled(width: number, height: number): Scene;
/// ```
pub(super) fn filled(width: u32, height: u32) -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(INK.0, INK.1, INK.2, 255),
        None,
        &Rect::new(0.0, 0.0, f64::from(width), f64::from(height)),
    );
    return scene;
}

/// What:
///  A tile of `heights.len()` textures `STACK_WIDTH` wide and `heights` tall,
///  top first,
///  each filled with [`INK`].
/// Why:
///  A frame that draws the tile shows `STACK_WIDTH` times the heights together in pixels of that color.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paintedStack(heights: number[]): TileStack;
/// ```
fn painted_stack(heights: &[u32]) -> TileStack {
    let mut textures = Vec::with_capacity(heights.len());
    for height in heights {
        textures.push(TileScene {
            scene: filled(STACK_WIDTH, *height),
            width: STACK_WIDTH,
            height: *height,
        });
    }
    return TileStack {
        textures,
        width: STACK_WIDTH,
        height: heights.iter().sum(),
    };
}

/// How many pixels of the frame `window` draws now are [`INK`].
pub(super) fn ink_pixels(window: &AppWindow) -> u32 {
    let shot = snapshot(window.window());
    let mut count = 0;
    for pixel in shot.as_slice() {
        if (pixel.r, pixel.g, pixel.b) == INK {
            count += 1;
        }
    }
    return count;
}

/// What:
///  A shown window of the application's markup with the notifier bound to a new `GpuTile`,
///  whose first frame has run and so holds the device,
///  and whose code view is as wide as the painted textures.
/// Why:
///  Every tile handed to it afterwards takes the path of a tile that replaces the one before,
///  and the picture a frame draws of it is counted in [`INK`] pixels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paintedWindow(): [AppWindow, GpuTile];
/// ```
pub(super) fn painted_window() -> (AppWindow, GpuTile) {
    let (window, gpu) = unframed();
    window.set_image_width(STACK_WIDTH as f32);
    let _setup = snapshot(window.window());
    return (window, gpu);
}

/// Hand `heights` to the window as a painted tile and draw the one frame that follows,
///  with nothing drawn in between.
fn hand_over_and_draw(window: &AppWindow, gpu: &GpuTile, heights: &[u32]) -> u32 {
    gpu.submit_tile(window, painted_stack(heights));
    return ink_pixels(window);
}

/// The frame that follows the hand-over of a tile shows the tile,
///  whatever the number of textures it has and the one before it had:
///  the window holds the slices when the frame begins,
///  so the frame that draws the scenes into the textures draws the textures.
/// A list the window is given inside the frame is not drawn by it
///  (`repeater_frame_tests`),
///  and an idle window draws no other frame by itself,
///  which is the blank code column.
/// The sequence has a first tile on a window that holds the device,
///  a tile of more textures,
///  of fewer,
///  and of the same number with other heights
///  (the one change a frame does show),
///  and each is counted in the pixels of the frame that follows its hand-over alone.
#[test]
fn the_frame_that_follows_a_hand_over_shows_the_tile_whatever_the_number_of_textures() {
    let (window, gpu) = painted_window();
    let area = |heights: &[u32]| return STACK_WIDTH * heights.iter().sum::<u32>();
    for heights in [
        vec![40, 24],
        vec![30, 30, 8],
        vec![100],
        vec![60, 40],
        vec![50, 10],
    ] {
        let drawn = hand_over_and_draw(&window, &gpu, &heights);
        assert_eq!(
            drawn,
            area(&heights),
            "the frame after the hand-over of a tile of the textures {heights:?} drew {drawn} pixels of it instead of {}",
            area(&heights)
        );
    }
    window.hide().expect("close tile window");
}
