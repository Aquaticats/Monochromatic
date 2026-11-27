//! The code view and the minimap on the GPU:
//!  the stacked textures the source tile is drawn into,
//!  the gutter's wrap indicator,
//!  and the minimap's stacked textures,
//!  and the rendering notifier that draws them with one Vello renderer.
//!
//! Slint hands its wgpu device to the notifier only when the first frame is set up,
//!  so the Vello renderer and the textures are made there.
//! The window thread (or the tile worker) builds a Vello scene from the shaped rows and hands it over;
//!  the notifier renders it into a texture on Slint's own device just before Slint draws the frame,
//!  and the texture is shown as an ordinary image.
//! No pixel is filled by the CPU:
//!  the scene holds glyph outlines and paths,
//!  and the GPU turns them into pixels.

/// The drawing of one frame.
mod frames;
/// The window's one rendering notifier.
mod notifier;
/// Counts and reads of the GPU state for the native tests.
#[cfg(test)]
mod probe;
/// Reads of the textures for the native tests.
#[cfg(test)]
mod readback;
/// The minimap's picture,
/// rendered in chunks into textures that stack.
mod slice;
/// The source tile,
/// rendered into textures that stack.
mod source;
/// The textures,
/// the pictures,
/// and the outcome of a step.
mod types;
/// The window the textures are shown in.
use super::AppWindow;
/// The tile scenes that are drawn into textures.
use ide_app::tile_scene::{
    TileScene,
    TileStack,
};
/// The one rendering notifier of a window.
pub(super) use notifier::bind;
/// A scene rendered without a window.
#[cfg(test)]
pub(super) use readback::{
    render_scene,
    render_stack,
};
/// Slint's own wgpu types,
///  the ones the window's device is made of.
use slint::wgpu_30::wgpu;
/// Shared ownership of the GPU state on the window thread.
use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::Rc,
};
/// How often the window was given a new list of textures.
#[cfg(test)]
pub(super) use types::SLICE_LISTS_SET;
/// What a frame is drawn into.
use types::{
    StackTexture,
    Target,
};
/// Vello's renderer and its settings.
use vello::Renderer;

/// ```
#[derive(Default)]
struct Inner {
    /// The device and queue of the first frame's setup.
    device: Option<(wgpu::Device, wgpu::Queue)>,
    /// Vello's renderer,
    /// made at setup.
    renderer: Option<Renderer>,
    /// The texture of the wrap indicator.
    sign: Option<Target>,
    /// The textures of the source tile,
    /// top first.
    tile: Vec<StackTexture>,
    /// The textures of the minimap's picture,
    /// top first.
    minimap: Vec<StackTexture>,
    /// The scenes of a tile not drawn yet.
    pending_tile: Option<TileStack>,
    /// A wrap indicator scene not drawn yet.
    pending_sign: Option<TileScene>,
    /// The chunks of a minimap picture not drawn yet,
    /// with the row of the whole picture the picture's first row is.
    pending_minimap: Option<(Vec<TileScene>, u32)>,
    /// Small textures the chunks of a picture are rendered into before they are copied into its textures.
    scratch: Vec<Target>,
    /// Tiles rendered into their textures so far.
    tiles_rendered: u64,
    /// Scenes of tiles rendered so far,
    /// one for each texture of a tile.
    tile_textures_rendered: u64,
    /// Textures made for tiles so far:
    /// a tile of the shape the last one had makes none.
    tile_textures_made: u64,
    /// Wrap indicator scenes rendered so far.
    signs_rendered: u64,
    /// Minimap pictures rendered so far.
    minimaps_rendered: u64,
    /// Minimap chunks rendered so far.
    minimap_chunks_rendered: u64,
}

/// What:
///  The handle the window state holds:
///  a scene is handed to it,
/// and the notifier draws it.
/// Why:
///  The state lives next to the window on one thread,
///  so a shared,
/// checked borrow is all the sharing needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GpuTile { submitTile(scene: TileScene): void; submitSign(scene: TileScene): void }
/// ```
#[derive(Clone, Default)]
pub(super) struct GpuTile {
    /// The state the notifier also holds.
    inner: Rc<RefCell<Inner>>,
    /// The offset the text was drawn at when the latest frame began,
    ///  or nothing before the first frame.
    /// The notifier writes it,
    ///  and the pointer handlers read it:
    ///  a press is handled after the frame it was made over,
    ///  so the row under the pointer is the row of that frame.
    shown: Rc<Cell<Option<f32>>>,
}

/// Hand scenes over.
impl GpuTile {
    /// What:
    ///  The offset the text was drawn at in the latest frame that began,
    ///  in logical pixels from the top of the text,
    ///  or nothing while no frame has begun.
    /// Why:
    ///  The pointer maps to the rows of the picture the person sees,
    ///  and the window keeps animating between that frame and the moment the press is handled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shownOffset(): number | undefined;
    /// ```
    pub(super) fn shown_offset(&self) -> Option<f32> {
        return self.shown.get();
    }

    /// Make the scenes of the tile the ones to draw at the next frame;
    /// a tile not drawn yet is replaced.
    /// A tile whose textures differ in size from the last one's gets its new textures,
    /// and the window its new slices,
    ///  here and not in the frame,
    /// because a `for` draws a list that was reset or replaced inside a frame only from the next frame
    /// (`doc/troubleshooting/slint-repeater-model-change-inside-rendering-notifier.md`).
    pub(super) fn submit_tile(
        &self,
        window: &AppWindow,
        stack: TileStack,
    ) {
        let mut inner = self.inner.borrow_mut();
        let outcome = inner.prepare_tile(&stack);
        inner.pending_tile = Some(stack);
        drop(inner);
        outcome.apply(window);
    }

    /// Make the scene of the wrap indicator the one to draw at the next frame.
    pub(super) fn submit_sign(
        &self,
        scene: TileScene,
    ) {
        self.inner.borrow_mut().pending_sign = Some(scene);
    }

    /// Make the minimap's picture,
    /// given as the scenes of its chunks from the top,
    /// whose first row is the row `top` of the whole picture,
    /// the one to draw at the next frame;
    /// a picture not drawn yet is replaced.
    /// A picture whose textures differ in size from the last one's gets its new textures,
    /// and the window its new slices,
    /// here and not in the frame,
    /// because a `for` draws a list that was reset or replaced inside a frame only from the next frame
    /// (`doc/troubleshooting/slint-repeater-model-change-inside-rendering-notifier.md`).
    pub(super) fn submit_minimap(
        &self,
        window: &AppWindow,
        chunks: Vec<TileScene>,
        top: u32,
    ) {
        let mut inner = self.inner.borrow_mut();
        let outcome = inner.prepare_minimap(&chunks, top);
        inner.pending_minimap = Some((chunks, top));
        drop(inner);
        outcome.apply(window);
    }

    /// Whether a picture of the minimap waits for a frame to draw it.
    pub(super) fn minimap_waiting(&self) -> bool {
        return self.inner.borrow().pending_minimap.is_some();
    }

    /// Forget the minimap's textures and any picture waiting,
    /// so the next picture makes new textures and new images.
    /// The window calls this when it clears the image,
    /// for another file.
    pub(super) fn forget_minimap(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.pending_minimap = None;
        inner.minimap.clear();
    }
}
