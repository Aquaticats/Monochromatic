//! The limit of a tile that is drawn across stacked textures,
//! in the device's texture size,
//! Vello's bin count,
//! and the textures a stack may have:
//!  the rules that size a clip,
//!  and the guard that cuts a frame to the limit when the lines are taller than the map the caller chose its clip from.
//! No window is opened.

/// What:
///  `#[path = "..."]` names the file a module is read from;
///  this test crate's parts live under `tests/raster_limit/`.
/// Why:
///  One test crate links the library once and shares the scales,
///       widths,
///  and palette every part sweeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as clipAndCap from './raster_limit/clip_and_cap';
/// ```
#[path = "raster_limit/clip_and_cap.rs"]
mod clip_and_cap;

/// The shaper's bin side,
///  the texture limits,
///  the rules under test,
///  and the palette of a taller tile.
use ide_app::code_colors::CodeColors;
/// The shaped text's limits and the rules under test.
use ide_app::shaped_text::{
    BIN_SIDE,
    DEFAULT_TEXTURE_DIMENSION,
    MAX_BINS,
    MAX_STACK_TEXTURES,
    STACK_VIEW_ROOM,
    fit_clip,
    set_texture_dimension,
    stack_fits,
    stack_rows,
    texture_dimension,
    texture_fits,
    texture_heights,
    tile_cap,
    tile_texture_rows,
};

/// Display scales of ordinary and fractional displays.
const SCALES: [f32; 7] = [0.5, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0];

/// Raster widths in physical pixels:
/// a very narrow window,
/// ordinary windows,
/// the widths of the owner's report,
/// and widths near the one where the cap is a single code row.
const COLUMNS: [u64; 12] = [
    1, 24, 200, 782, 1354, 1920, 3840, 7680, 100_000, 600_000, 699_000, 1_000_000,
];

/// Heights in logical pixels that a window asks a tile for:
/// nothing,
/// three views of an ordinary window,
/// and three views of a window far taller than one texture.
const WANTED: [f32; 3] = [0.0, 1980.0, 36_000.0];

/// A palette for painting.
fn colors() -> CodeColors {
    return CodeColors {
        foreground: [240, 240, 240, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
}

/// A 2000 px tall window at display scale 2 on a device whose textures are at most 8192 px on a side:
/// the tile of three views (12,000 physical rows) is not cut,
/// it is two textures that together hold the whole view with overscan.
#[test]
fn a_tile_at_scale_two_in_a_2000_px_window_is_two_textures_on_a_device_limited_to_8192() {
    assert_eq!(
        texture_dimension(),
        8192,
        "the default is the smallest limit wgpu guarantees"
    );
    let scale = 2.0_f32;
    let view_height = 2000.0_f32;
    // The text area of a 1147 px wide window, in physical pixels.
    let columns = (1147.0_f32 * scale).ceil() as u64;
    let cap = tile_cap(columns, scale, view_height * 3.0);
    assert!(
        cap >= view_height * 3.0,
        "the cap {cap} px does not hold the three views that the window asks for"
    );
    // A whole-document tile of three views is not cut: it fits the stack.
    let first_top = 0.0;
    let bottom = view_height * 3.0;
    assert_eq!(
        fit_clip(first_top, bottom, None, columns, scale),
        None,
        "three views were cut"
    );
    let rows = (bottom * scale).ceil() as u64;
    assert!(!texture_fits(columns, rows), "one texture holds the tile");
    let heights = texture_heights(columns, rows);
    assert_eq!(heights.len(), 2, "heights {heights:?}");
    assert_eq!(u64::from(heights[0] + heights[1]), rows);
    assert_eq!(u64::from(heights[0]), tile_texture_rows(columns));
}

/// The device's texture size bounds a texture of the stack on its own when the bin count would allow more,
/// and the stack is as tall as its textures allow.
#[test]
fn the_texture_size_of_the_device_bounds_each_texture_of_the_stack() {
    let device_side: u32 = 4096;
    let device = u64::from(device_side);
    set_texture_dimension(device_side);
    // 200 columns are one bin wide: the bin count would allow more rows; the device allows `device`.
    assert_eq!(tile_texture_rows(200), device);
    assert!(texture_fits(200, device) && !texture_fits(200, device + 1));
    assert!(
        !texture_fits(device + 1, 1),
        "a tile wider than the texture size fits"
    );
    let rows = 10_000_u64;
    let remainder = u32::try_from(rows - 2 * device).expect("the remaining rows fit u32");
    assert_eq!(
        texture_heights(200, rows),
        vec![device_side, device_side, remainder]
    );
    assert!(
        stack_fits(200, MAX_STACK_TEXTURES * device)
            && !stack_fits(200, MAX_STACK_TEXTURES * device + 1)
    );
    set_texture_dimension(DEFAULT_TEXTURE_DIMENSION);
    assert!(tile_texture_rows(200) > device);
    assert_eq!(
        stack_rows(200),
        MAX_STACK_TEXTURES * u64::from(DEFAULT_TEXTURE_DIMENSION)
    );
}

/// The stack never holds fewer rows than the tallest window the device can show and the room for its re-center,
/// whatever the width:
/// at 16,384 columns the bin count allows 1024 rows to a texture and four textures are not enough.
#[test]
fn a_stack_is_never_under_the_tallest_window_and_its_room() {
    let device_side: u32 = 16_384;
    let device = u64::from(device_side);
    set_texture_dimension(device_side);
    // A bin is BIN_SIDE squared pixels: the bin count bounds a texture this wide to that many bins' rows.
    let per_texture = MAX_BINS * BIN_SIDE * BIN_SIDE / device;
    assert_eq!(tile_texture_rows(device), per_texture);
    assert_eq!(stack_rows(device), device + STACK_VIEW_ROOM);
    assert!(
        stack_fits(device, device + STACK_VIEW_ROOM)
            && !stack_fits(device, device + STACK_VIEW_ROOM + 1)
    );
    assert!(!stack_fits(device + 1, 1));
    set_texture_dimension(DEFAULT_TEXTURE_DIMENSION);
}
