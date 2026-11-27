//! The drawing of one frame:
//!  the device taken at setup,
//!  the scenes waiting for the frame drawn into their textures,
//!  and everything released at teardown.

/// The GPU state this draws with.
use super::Inner;
/// The textures,
///  what a step changed,
///  and the outcome the window applies.
use super::types::{
    Outcome,
    Target,
};
/// The scene a texture is drawn from.
use ide_app::tile_scene::TileScene;
/// The image a texture is shown as.
use slint::Image;
/// Slint's own wgpu types,
///  the ones the window's device is made of.
use slint::wgpu_30::wgpu;
/// Vello's renderer and its settings.
use vello::{
    AaConfig,
    AaSupport,
    RenderParams,
    Renderer,
    RendererOptions,
    peniko::Color as VelloColor,
};

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
    pub(super) fn usage() -> wgpu::TextureUsages {
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
    pub(super) fn setup(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Outcome {
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
    pub(super) fn teardown(&mut self) {
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
    pub(super) fn draw_pending(&mut self) -> Outcome {
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
    fn draw_sign(
        &mut self,
        scene: &TileScene,
        outcome: &mut Outcome,
    ) {
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
    pub(super) fn render(
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
