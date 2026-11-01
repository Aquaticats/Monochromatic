//! The native tests' view of the GPU tile:
//!  a frame is rendered so the scene the window thread handed over is drawn,
//!  and the tile's texture is read back from the GPU.
//!  The window's own snapshot (`gpu_snapshot::snapshot`) shows the tile composited with the selection fills,
//!  the caret,
//! and the minimap;
//!  this shows the tile alone,
//! with its alpha.

/// The window and its state.
use super::{AppWindow, State};
/// A frame of the window,
/// which draws the waiting scenes,
///  and a scene drawn without a window.
use super::{gpu_snapshot::snapshot, gpu_tile::render_scene};
/// What a tile draws,
/// and the palette it draws it in.
use ide_app::{code_colors::CodeColors, shaped_text::ShapedView, tile_scene::paint_tile};
/// Toolkit windows and pixel buffers.
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer};
/// Shared state.
use std::{cell::RefCell, rc::Rc};

/// What:
///  The most two renders of one scene may differ on the GPU,
/// in levels of one channel of a premultiplied texel.
/// Why:
///  Vello sums the coverage of a pixel's path segments in the order the GPU's atomics put them,
///  so two renders of the same scene differ by one level in a few texels
///  (measured in `gpu_texture_probe_tests`:
/// 40 renders of a 697 by 1392 tile,
/// worst difference 1).
///  Tests that compare two renders accept this and nothing more;
///  every one of them has a positive control whose difference is far larger.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const RENDER_NOISE = 1;
/// ```
pub(super) const RENDER_NOISE: u8 = 1;

/// Whether two straight-alpha texels are the same picture to within [`RENDER_NOISE`],
///  compared premultiplied,
/// so the color of an invisible texel does not count.
pub(super) fn same_texel(left: Rgba8Pixel, right: Rgba8Pixel) -> bool {
    let premultiply =
        |value: u8, alpha: u8| return (u32::from(value) * u32::from(alpha) / 255) as u8;
    return premultiply(left.r, left.a).abs_diff(premultiply(right.r, right.a)) <= RENDER_NOISE
        && premultiply(left.g, left.a).abs_diff(premultiply(right.g, right.a)) <= RENDER_NOISE
        && premultiply(left.b, left.a).abs_diff(premultiply(right.b, right.a)) <= RENDER_NOISE
        && left.a.abs_diff(right.a) <= RENDER_NOISE;
}

/// Whether two reads of the same rows agree texel for texel to within [`RENDER_NOISE`].
pub(super) fn same_rows(left: &[[u8; 4]], right: &[[u8; 4]]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (one, other) in left.iter().zip(right) {
        let first = Rgba8Pixel::new(one[0], one[1], one[2], one[3]);
        let second = Rgba8Pixel::new(other[0], other[1], other[2], other[3]);
        if !same_texel(first, second) {
            return false;
        }
    }
    return true;
}

/// Whether two tile reads have the same size and agree texel for texel to within [`RENDER_NOISE`].
pub(super) fn same_buffers(
    left: &SharedPixelBuffer<Rgba8Pixel>,
    right: &SharedPixelBuffer<Rgba8Pixel>,
) -> bool {
    if left.width() != right.width() || left.height() != right.height() {
        return false;
    }
    return same_pixels(left.as_slice(), right.as_slice());
}

/// Whether two runs of texels are the same length and agree texel for texel to within [`RENDER_NOISE`].
pub(super) fn same_pixels(left: &[Rgba8Pixel], right: &[Rgba8Pixel]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (one, other) in left.iter().zip(right) {
        if !same_texel(*one, *other) {
            return false;
        }
    }
    return true;
}

/// What:
///  The tile's pixels as straight RGBA,
///  read back after a frame that draws the scene the window thread last handed over,
///  or nothing when no tile was drawn yet.
/// Why:
///  A scene is drawn into its texture by the rendering notifier just before a frame,
///  so a frame has to run before the texture holds the tile that was built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tilePixels(window: AppWindow, source: State): Pixels | undefined;
/// ```
pub(super) fn tile_pixels(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let _frame = snapshot(window.window());
    let gpu = source.borrow().gpu.clone();
    return gpu.read_tile();
}

/// The size of the tile's texture in physical pixels,
///  after a frame that draws the scene the window thread last handed over.
pub(super) fn tile_size(window: &AppWindow, source: &Rc<RefCell<State>>) -> (u32, u32) {
    let _frame = snapshot(window.window());
    let gpu = source.borrow().gpu.clone();
    return gpu.tile_size().expect("a tile texture");
}

/// The width and height in physical pixels of each texture of the tile,
/// top first,
///  after a frame that draws the scene the window thread last handed over.
pub(super) fn tile_textures(window: &AppWindow, source: &Rc<RefCell<State>>) -> Vec<(u32, u32)> {
    let _frame = snapshot(window.window());
    let gpu = source.borrow().gpu.clone();
    return gpu.tile_texture_sizes();
}

/// The wrap indicator's pixels as straight RGBA,
///  read back after a frame,
///  or nothing when none was drawn yet.
pub(super) fn sign_pixels(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let _frame = snapshot(window.window());
    let gpu = source.borrow().gpu.clone();
    return gpu.read_sign();
}

/// What:
///  The minimap's textures as straight RGBA,
///  stacked in one buffer,
///  read back after a frame that draws the picture the window thread last handed over,
///  or nothing when none was drawn yet.
///  The buffer holds the rows `window.get_minimap_picture_top()` and below of the whole picture,
///  as many as the textures together have.
/// Why:
///  The minimap is a texture the toolkit shows as an image,
///  which the toolkit cannot read back,
///  so the tests read the GPU's own copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function minimapPixels(window: AppWindow, source: State): Pixels | undefined;
/// ```
pub(super) fn minimap_pixels(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let _frame = snapshot(window.window());
    let gpu = source.borrow().gpu.clone();
    return gpu.read_minimap();
}

/// Run one frame so the slice the window thread last handed over is drawn,
/// and the image and its place are set on the window.
pub(super) fn draw_minimap(window: &AppWindow) {
    let _frame = snapshot(window.window());
}

/// What:
///  The rows of the window's visible viewport as RGBA pixels,
///  read from the tile's texture.
/// Why:
///  The visible band is the part of the tile the reader sees;
///  rows above and below it are overscan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shown(window: AppWindow, source: State): number[][];
/// ```
pub(super) fn shown(window: &AppWindow, source: &Rc<RefCell<State>>) -> Vec<[u8; 4]> {
    let scale = window.window().scale_factor();
    let image = tile_pixels(window, source).expect("tile texture pixels");
    let width = image.width() as usize;
    let view_top = -window.get_scroll_y();
    let first = ((view_top - window.get_image_y()) * scale).floor().max(0.0) as usize;
    let last = (((view_top + window.get_viewport_height() - window.get_image_y()) * scale).ceil()
        as usize)
        .min(image.height() as usize);
    let pixels = image.as_slice();
    let mut rows = Vec::new();
    for row in first..last {
        for pixel in &pixels[row * width..(row + 1) * width] {
            rows.push([pixel.r, pixel.g, pixel.b, pixel.a]);
        }
    }
    return rows;
}

/// What:
///  The pixels the GPU draws for `view` at the fractional offset `horizontal`,
///  straight RGBA,
/// row by row from the top,
///  from a scene rendered on the test thread's device with no window around it.
/// Why:
///  A tile at a display scale a window cannot be set to,
///  and the tile of a shaped view that no window holds,
///  are looked at the way the window draws them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderedTile(view: ShapedView, colors: CodeColors, horizontal: number): Pixels;
/// ```
pub(super) fn rendered_tile(
    view: &ShapedView,
    colors: CodeColors,
    horizontal: f32,
) -> SharedPixelBuffer<Rgba8Pixel> {
    let scene = paint_tile(view, colors, horizontal).expect("the shaped view builds a scene");
    return render_scene(&scene);
}
