//! A minimap picture of another shape reaches the window before the frame that draws it.
//!  The GPU draws the picture's chunks into textures during a frame,
//!  and a picture whose textures differ in size from the last one's needs new textures and a new list of slices
//!  (`minimap-slices`,
//!  the list the markup draws with a `for`).
//!  A list the window is given inside the frame is not shown by that frame,
//!  and an idle window draws no other frame by itself,
//!  so the first picture of a file,
//!  whose list is a new one,
//!  stayed blank until the next picture came;
//!  a picture handed over for the next frame must bring its slices with it.
//! The test harness draws every property the notifier set in the same snapshot,
//!  so it cannot show the blank minimap;
//!  these tests check the contract the real window needs:
//!  when a picture is handed over and before any frame is drawn,
//!  the window holds slices of the new textures.

/// The frame that gives the window its device,
///  and the frame that draws a picture.
use super::gpu_snapshot::snapshot;
/// The painted textures,
///  the window that shows them,
///  and the count of the pixels they are drawn in.
use super::tile_slice_frame_tests::{filled, ink_pixels, painted_window};
/// The window of the application's markup before any frame,
///  and the width of the textures the tests make.
use super::tile_slice_frame_tests::{STACK_WIDTH, unframed};
/// The scenes of a picture's chunks.
use ide_app::tile_scene::TileScene;
/// The window handle and the model trait.
use slint::{ComponentHandle, Model, ModelRc};
/// An empty scene:
///  the textures are made and drawn,
///  with nothing in them.
use vello::Scene;

/// What:
///  The chunks of a picture,
///  `STACK_WIDTH` wide and `heights` tall,
///  top first,
///  whose scenes are empty.
/// Why:
///  The notifier makes textures and slices from the sizes alone,
///  so the tests need no text to shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chunks(heights: number[]): TileScene[];
/// ```
fn chunks(heights: &[u32]) -> Vec<TileScene> {
    let mut made = Vec::with_capacity(heights.len());
    for height in heights {
        made.push(TileScene {
            scene: Scene::new(),
            width: STACK_WIDTH,
            height: *height,
        });
    }
    return made;
}

/// The heights and first rows of the slices the window draws the minimap with,
///  top first.
fn slice_rows(window: &super::AppWindow) -> Vec<(u32, u32)> {
    let slices = window.get_minimap_slices();
    let mut rows = Vec::with_capacity(slices.row_count());
    for row in 0..slices.row_count() {
        let slice = slices.row_data(row).expect("a slice of the list");
        rows.push((slice.height as u32, slice.top as u32));
    }
    return rows;
}

/// The heights of the textures the GPU holds for the picture,
///  top first.
fn texture_heights(gpu: &super::gpu_tile::GpuTile) -> Vec<u32> {
    return gpu
        .minimap_texture_sizes()
        .iter()
        .map(|size| return size.1)
        .collect();
}

/// The first picture of a file,
///  and the first one after another file was shown,
///  hand their slices to the window before any frame is drawn:
///  the window's list for another file is a new one,
///  which a frame does not show when the frame set it.
#[test]
fn the_first_picture_of_a_file_gives_the_window_its_slices_before_any_frame() {
    let (window, gpu) = unframed();
    // The first frame gives the window's device to the notifier.
    let _setup = snapshot(window.window());
    gpu.submit_minimap(&window, chunks(&[40, 24]), 0);
    assert_eq!(
        slice_rows(&window),
        vec![(64, 0)],
        "the window holds no slices of the first picture after its hand-over"
    );
    assert_eq!(
        texture_heights(&gpu),
        vec![64],
        "the textures the slices show are not the picture's"
    );
    let _drawn = snapshot(window.window());
    // What the window does for another file: the textures and the list are dropped.
    gpu.forget_minimap();
    window.set_minimap_slices(ModelRc::default());
    gpu.submit_minimap(&window, chunks(&[30, 30, 8]), 0);
    assert_eq!(
        slice_rows(&window),
        vec![(68, 0)],
        "the window holds no slices of the first picture of the next file after its hand-over"
    );
    assert_eq!(
        window.get_minimap_picture_height(),
        68.0,
        "the window does not hold the height of the picture after its hand-over"
    );
    window.hide().expect("close tile window");
}

/// A picture whose textures differ in size from the last one's hands the window its slices before any frame,
///  and a picture of the same shape at another place leaves the slices for the frame that draws it.
#[test]
fn a_picture_of_another_shape_gives_the_window_its_slices_before_any_frame() {
    let (window, gpu) = unframed();
    let _setup = snapshot(window.window());
    gpu.submit_minimap(&window, chunks(&[40, 24]), 0);
    let _first = snapshot(window.window());
    assert_eq!(
        slice_rows(&window),
        vec![(64, 0)],
        "the first picture is not shown"
    );
    // Another height: new textures and a new row of the list.
    gpu.submit_minimap(&window, chunks(&[100]), 0);
    assert_eq!(
        slice_rows(&window),
        vec![(100, 0)],
        "the window holds the slices of the earlier picture after the hand-over"
    );
    assert_eq!(texture_heights(&gpu), vec![100]);
    let _second = snapshot(window.window());
    // The same shape at another place: the textures are reused and the place is the frame's.
    gpu.submit_minimap(&window, chunks(&[100]), 700);
    assert_eq!(
        slice_rows(&window),
        vec![(100, 0)],
        "a picture of the same shape replaced the slices of the window outside a frame"
    );
    let _third = snapshot(window.window());
    assert_eq!(
        slice_rows(&window),
        vec![(100, 700)],
        "the frame that drew the picture of the same shape did not place it"
    );
    window.hide().expect("close tile window");
}

/// What:
///  The chunks of a picture,
///  `STACK_WIDTH` wide and `heights` tall,
///  top first,
///  each filled with the color the painted tiles use.
/// Why:
///  A frame that draws the picture shows pixels of that color,
///  so the test counts what a frame drew.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paintedChunks(heights: number[]): TileScene[];
/// ```
fn painted_chunks(heights: &[u32]) -> Vec<TileScene> {
    let mut made = Vec::with_capacity(heights.len());
    for height in heights {
        made.push(TileScene {
            scene: filled(STACK_WIDTH, *height),
            width: STACK_WIDTH,
            height: *height,
        });
    }
    return made;
}

/// What:
///  Hand `heights` to the window as a painted picture,
///  draw the one frame that follows with nothing drawn in between,
///  draw a second frame,
///  and say how many pixels of the picture each drew.
/// Why:
///  A frame that does not show the picture draws fewer pixels of it than the frame after,
///  which is the late or the missing picture of the minimap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handOverAndDraw(window: AppWindow, gpu: GpuTile, heights: number[]): [number, number];
/// ```
fn hand_over_and_draw(
    window: &super::AppWindow,
    gpu: &super::gpu_tile::GpuTile,
    heights: &[u32],
) -> (u32, u32) {
    gpu.submit_minimap(window, painted_chunks(heights), 0);
    let first = ink_pixels(window);
    let settled = ink_pixels(window);
    return (first, settled);
}

/// The frame that follows the hand-over of a picture shows the picture,
///  whether it is the first of a file (the window's list was emptied for another file),
///  or has more textures than the one before it,
///  or fewer:
///  the window holds the slices when the frame begins.
/// A list the window is given inside the frame is not drawn by it (`repeater_frame_tests`),
///  and an idle window draws no other frame by itself.
#[test]
fn the_frame_that_follows_a_picture_shows_it_whatever_the_number_of_textures() {
    let (window, gpu) = painted_window();
    let tall = vec![512; 17];
    // What the file switch does to the window.
    let switch = || {
        gpu.forget_minimap();
        window.set_minimap_slices(ModelRc::default());
    };
    let mut steps: Vec<(&str, Vec<u32>, bool)> = Vec::new();
    steps.push(("the first picture", vec![40, 24], false));
    steps.push(("the first picture of the next file", vec![30, 30, 8], true));
    steps.push(("a picture of more textures", tall, false));
    steps.push(("a picture of fewer textures", vec![100], false));
    steps.push(("a picture of other heights in the same number of textures", vec![60], false));
    for (name, heights, new_file) in steps {
        if new_file {
            switch();
        }
        let (first, settled) = hand_over_and_draw(&window, &gpu, &heights);
        assert!(
            settled > 0,
            "{name}: the settled frame drew none of the picture, so the test counts nothing"
        );
        assert_eq!(
            first, settled,
            "{name}: the frame that followed the hand-over drew {first} pixels of the picture and the next frame {settled}"
        );
    }
    window.hide().expect("close tile window");
}
