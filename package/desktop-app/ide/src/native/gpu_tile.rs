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
/// A scene rendered without a window.
#[cfg(test)]
pub(super) use readback::{render_scene, render_stack};
/// How often the window was given a new list of textures.
#[cfg(test)]
pub(super) use types::SLICE_LISTS_SET;
/// The small types,
/// shared with the slice and the source tile.
use types::{Outcome, Placement, StackTexture, Target};

/// The window the textures are shown in,
///  and the chaining of the debug frame records.
use super::{AppWindow, frame_trace};
/// The tile scenes that are drawn into textures.
use ide_app::tile_scene::{TileScene, TileStack};
/// Slint's own wgpu types,
/// the ones the window's device is made of.
use slint::wgpu_30::wgpu;
/// The window handle,
/// the rendering stages,
/// the graphics API,
/// and the image import.
use slint::{ComponentHandle, GraphicsAPI, Image, RenderingState};
/// Shared ownership of the GPU state on the window thread.
use std::{cell::RefCell, rc::Rc};
/// Vello's renderer and its settings.
use vello::{
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, peniko::Color as VelloColor,
};

/// What:
///  Everything the notifier and the window thread share:
///  the device once Slint has one,
///  the renderer,
///  the textures,
///  and the scenes waiting to be drawn.
/// Why:
///  The window thread hands a scene over at any time and the notifier draws it at the next frame,
///  so both reach the same state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gpu = { device?: Device; renderer?: Renderer; targets: (Target | undefined)[]; pending: Scene[] };
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
}

/// Hand scenes over.
impl GpuTile {
    /// Make the scenes of the tile the ones to draw at the next frame;
    /// a tile not drawn yet is replaced.
    /// A tile whose textures differ in size from the last one's gets its new textures,
    /// and the window its new slices,
    ///  here and not in the frame,
    /// because the window does not show a slice list it was given inside the frame that draws the tile.
    pub(super) fn submit_tile(&self, window: &AppWindow, stack: TileStack) {
        let mut inner = self.inner.borrow_mut();
        let outcome = inner.prepare_tile(&stack);
        inner.pending_tile = Some(stack);
        drop(inner);
        outcome.apply(window);
    }

    /// Make the scene of the wrap indicator the one to draw at the next frame.
    pub(super) fn submit_sign(&self, scene: TileScene) {
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
    /// because the window does not show a slice list it was given inside the frame that draws the picture.
    pub(super) fn submit_minimap(&self, window: &AppWindow, chunks: Vec<TileScene>, top: u32) {
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

/// Set up,
/// draw,
/// and tear down on the renderer's calls.
impl Inner {
    /// What:
    ///  The uses of a picture's texture:
    ///  Vello writes it as a storage texture,
    ///  and Slint samples it.
    ///  A test build can also read it back.
    /// Why:
    ///  Reading a texture back is how the native tests look at the tile itself,
    ///  without the selection fills,
    /// caret,
    /// and minimap that the window draws around it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function usage(): GPUTextureUsage;
    /// ```
    fn usage() -> wgpu::TextureUsages {
        let drawn = wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT;
        if cfg!(test) {
            return drawn | wgpu::TextureUsages::COPY_SRC;
        }
        return drawn;
    }

    /// What:
    ///  Take the device of the first frame and make Vello's renderer for it.
    ///  The largest texture the device offers is recorded for the tile's height.
    /// Why:
    ///  Vello compiles its compute shaders here,
    /// once.
    ///  A device whose limit differs from the one the tiles were cut for is reported:
    ///  the launcher and the harness record the device's own limit before any window exists,
    ///  so a difference means another device was given to the window.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function setup(device: GPUDevice, queue: GPUQueue): Outcome;
    /// ```
    fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Outcome {
        let mut outcome = Outcome::default();
        let limit = device.limits().max_texture_dimension_2d;
        outcome.texture_limit = Some(limit);
        if limit != ide_app::shaped_text::texture_dimension() {
            tracing::warn!(
                limit,
                assumed = ide_app::shaped_text::texture_dimension(),
                "the window's device offers another texture size than the tiles were cut for"
            );
            ide_app::shaped_text::set_texture_dimension(limit);
        }
        match Renderer::new(
            device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        ) {
            Ok(renderer) => {
                tracing::debug!(limit, "the GPU text renderer is ready");
                self.renderer = Some(renderer);
            }
            Err(error) => {
                tracing::error!(%error, "the GPU text renderer could not be made");
                outcome.errors.push(format!(
                    "Cannot draw source: the GPU text renderer could not start: {error}"
                ));
            }
        }
        self.device = Some((device.clone(), queue.clone()));
        return outcome;
    }

    /// Release the renderer and the textures before the device goes away.
    fn teardown(&mut self) {
        self.renderer = None;
        self.sign = None;
        self.tile = Vec::new();
        self.minimap = Vec::new();
        self.scratch = Vec::new();
        self.device = None;
    }

    /// What:
    ///  Draw the scenes that wait,
    /// if the renderer is ready.
    /// Why:
    ///  This runs just before Slint draws the frame,
    ///  so the textures hold the new scenes when the images that show them are sampled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function drawPending(): Outcome;
    /// ```
    fn draw_pending(&mut self) -> Outcome {
        let mut outcome = Outcome::default();
        if self.renderer.is_none() || self.device.is_none() {
            return outcome;
        }
        if let Some(stack) = self.pending_tile.take() {
            self.draw_tile(&stack, &mut outcome);
        }
        if let Some(scene) = self.pending_sign.take() {
            self.draw_sign(&scene, &mut outcome);
        }
        if let Some((chunks, top)) = self.pending_minimap.take() {
            self.draw_stack(&chunks, top, &mut outcome);
        }
        return outcome;
    }

    /// What:
    ///  Render `scene`,
    ///  the gutter's wrap indicator,
    ///  into its texture,
    ///  making a new texture,
    /// and giving the window a new image,
    /// when the size changed.
    /// Why:
    ///  Rendering into the same texture needs no new image,
    ///  and the frame that follows samples the new content.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function drawSign(scene: TileScene, outcome: Outcome): void;
    /// ```
    fn draw_sign(&mut self, scene: &TileScene, outcome: &mut Outcome) {
        let Some((device, queue)) = self.device.clone() else {
            return;
        };
        let limit = device.limits().max_texture_dimension_2d;
        if scene.width == 0 || scene.height == 0 || scene.width > limit || scene.height > limit {
            tracing::error!(
                width = scene.width,
                height = scene.height,
                limit,
                "a scene does not fit a texture of this device"
            );
            outcome.errors.push(format!(
                "Cannot draw source: a {}x{} picture does not fit a texture of this device (at most {limit} on a side)",
                scene.width, scene.height
            ));
            return;
        }
        let reusable = match &self.sign {
            Some(target) => target.width == scene.width && target.height == scene.height,
            None => false,
        };
        if !reusable {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ide wrap indicator"),
                size: wgpu::Extent3d {
                    width: scene.width,
                    height: scene.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: Self::usage(),
                view_formats: &[],
            });
            match Image::try_from(texture.clone()) {
                Ok(image) => outcome.sign = Some(image),
                Err(error) => {
                    tracing::error!(%error, "the texture could not be shown as an image");
                    outcome.errors.push(format!("Cannot draw source: {error}"));
                    return;
                }
            }
            self.sign = Some(Target {
                texture,
                width: scene.width,
                height: scene.height,
            });
        }
        let Some(target) = self.sign.as_ref() else {
            return;
        };
        let texture = target.texture.clone();
        match self.render(&device, &queue, &texture, scene) {
            Ok(()) => self.signs_rendered += 1,
            Err(error) => {
                tracing::error!(%error, "the GPU could not render the wrap indicator");
                outcome.errors.push(format!("Cannot draw source: {error}"));
            }
        }
    }

    /// What:
    ///  Render `scene` into `texture` with the one Vello renderer.
    /// Why:
    ///  The tile,
    ///  the wrap indicator,
    ///  and every chunk of the minimap share one renderer,
    ///  so the renderer's scratch buffers exist once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function render(device: GPUDevice, queue: GPUQueue, texture: GPUTexture, scene: TileScene): void;
    /// ```
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        scene: &TileScene,
    ) -> Result<(), vello::Error> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(());
        };
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        return renderer.render_to_texture(
            device,
            queue,
            &scene.scene,
            &view,
            &RenderParams {
                base_color: VelloColor::from_rgba8(0, 0, 0, 0),
                width: scene.width,
                height: scene.height,
                antialiasing_method: AaConfig::Area,
            },
        );
    }
}

/// What:
///  Install the one rendering notifier of `window`:
///  it sets up Vello on the first frame,
///  draws the waiting scenes before each frame,
///  releases them at teardown,
///  and writes the debug frame records.
///  `gpu` is the handle the window state holds.
/// Why:
///  A window takes one notifier,
///  so the frame records and the drawing share it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bind(window: AppWindow, gpu: GpuTile): void;
/// ```
pub(super) fn bind(window: &AppWindow, gpu: &GpuTile) {
    let inner = Rc::clone(&gpu.inner);
    let weak = window.as_weak();
    let installed = window.window().set_rendering_notifier(move |state, api| {
        frame_trace::log_rendering_stage(&state);
        let GraphicsAPI::WGPU30 { device, queue, .. } = api else {
            tracing::error!("the window's renderer offers no wgpu 30 device");
            return;
        };
        let outcome = match state {
            RenderingState::RenderingSetup => inner.borrow_mut().setup(device, queue),
            RenderingState::BeforeRendering => inner.borrow_mut().draw_pending(),
            RenderingState::RenderingTeardown => {
                inner.borrow_mut().teardown();
                return;
            }
            _ => return,
        };
        if let Some(active) = weak.upgrade() {
            // What: New textures for the tile reach the window's list of slices inside the frame that made them,
            //       and that frame does not show them; the window asks for the frame that does.
            // Why: Nothing else asks for it. The window draws no frame by itself, and before the hint skip rule the
            //      answer to a hint request repainted the source a moment later and so drew one by chance.
            //      A press whose request is skipped has no answer, and its code column stayed blank.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const replaced = outcome.tile !== undefined; apply(outcome, window); if (replaced) window.requestRedraw();
            // ```
            let replaced = outcome.tile.is_some();
            outcome.apply(&active);
            if replaced {
                tracing::debug!(
                    "replaced the slices of the source tile inside a frame, so asking for the next"
                );
                active.window().request_redraw();
            }
        }
    });
    if let Err(error) = installed {
        tracing::error!(%error, "the source view cannot be drawn: the renderer takes no notifier");
        window.set_error_message(slint::SharedString::from(format!(
            "Cannot draw source: the window's renderer takes no rendering notifier ({error})"
        )));
    }
}
