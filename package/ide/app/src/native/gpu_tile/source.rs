//! The source tile:
//!  rendered scene by scene,
//!  each into a texture of its own,
//!  and the textures stack one below the other.
//!
//! A tile taller than one texture may be (the device's side,
//! or the rows where Vello counts 256 bins) is several
//!  textures,
//!  each with its own scene and its own image;
//!  the window shows them with no gap,
//!  so they read as the one tile.
//! No scene is copied:
//!  each is within the limits of a single render,
//!  and the renderer draws it straight into its texture.

/// The device,
/// the textures,
/// and the outcome the window applies.
use super::{Inner, Outcome, StackTexture, Target};
/// The markup's record of one texture of the tile.
use crate::native::ui::SourceSlice;
/// The scenes of the stack.
use ide_app::tile_scene::TileStack;
/// The image import.
use slint::Image;
/// Slint's own wgpu types.
use slint::wgpu_30::wgpu;

/// Draw the tile.
impl Inner {
    /// What:
    ///  Whether the tile already has textures of exactly the sizes of the scenes of `stack`.
    /// Why:
    ///  Rendering into the same textures needs no new image,
    ///  and the frame that follows samples the new content.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function tileTexturesFit(stack: TileStack): boolean;
    /// ```
    fn tile_textures_fit(&self, stack: &TileStack) -> bool {
        if self.tile.len() != stack.textures.len() {
            return false;
        }
        for (kept, scene) in self.tile.iter().zip(&stack.textures) {
            if kept.target.width != scene.width || kept.target.height != scene.height {
                return false;
            }
        }
        return true;
    }

    /// What:
    ///  Make one texture for each scene of `stack`,
    ///  top first,
    ///  and the image that shows it.
    ///  Nothing is kept when the device refuses to show a texture as an image.
    /// Why:
    ///  A tile of another shape starts from new textures and new images.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function makeTileTextures(device: GPUDevice, stack: TileStack, outcome: Outcome): boolean;
    /// ```
    fn make_tile_textures(
        &mut self,
        device: &wgpu::Device,
        stack: &TileStack,
        outcome: &mut Outcome,
    ) -> bool {
        self.tile.clear();
        for scene in &stack.textures {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ide source tile"),
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
            self.tile_textures_made += 1;
            match Image::try_from(texture.clone()) {
                Ok(image) => self.tile.push(StackTexture {
                    target: Target {
                        texture,
                        width: scene.width,
                        height: scene.height,
                    },
                    image,
                }),
                Err(error) => {
                    tracing::error!(%error, "the texture could not be shown as an image");
                    outcome.errors.push(format!("Cannot draw source: {error}"));
                    self.tile.clear();
                    return false;
                }
            }
        }
        return true;
    }

    /// Whether some scene of `stack` cannot be a texture of a device whose side is at most `limit`,
    ///  or the stack has none.
    fn oversize(stack: &TileStack, limit: u32) -> bool {
        let mut oversize = stack.textures.is_empty();
        for scene in &stack.textures {
            oversize |= scene.width == 0
                || scene.height == 0
                || scene.width > limit
                || scene.height > limit
                || scene.width != stack.width;
        }
        return oversize;
    }

    /// What:
    ///  Make the textures `stack` needs and put the slices that show them in `outcome`,
    ///  and answer whether it did.
    /// Why:
    ///  The window draws the tile through these slices (`source-slices` in the markup),
    ///  and a slice list the window gets inside a frame is not shown by that frame.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function replaceTileTextures(device: Device, stack: TileStack, outcome: Outcome): boolean;
    /// ```
    fn replace_tile_textures(
        &mut self,
        device: &wgpu::Device,
        stack: &TileStack,
        outcome: &mut Outcome,
    ) -> bool {
        if !self.make_tile_textures(device, stack, outcome) {
            return false;
        }
        let mut slices = Vec::with_capacity(self.tile.len());
        let mut top = 0_u32;
        for held in &self.tile {
            slices.push(SourceSlice {
                image: held.image.clone(),
                top: top as f32,
                height: held.target.height as f32,
            });
            top += held.target.height;
        }
        outcome.tile = Some(slices);
        return true;
    }

    /// What:
    ///  Make the textures a tile about to be drawn needs when its shape differs from the one the window shows,
    ///  so that the window holds their slices before the frame that draws the tile begins;
    ///  the outcome is empty when the textures fit,
    ///  when the device is not held yet (the first frame has not run),
    ///  or when the tile does not fit the device (the frame refuses it with a message).
    /// Why:
    ///  A slice list the window gets inside a frame is not shown by that frame,
    ///  and the tile would be missing from it.
    ///  Slices of textures made here are shown by the frame that draws the scenes into them,
    ///  because the notifier draws before the items are drawn.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function prepareTile(stack: TileStack): Outcome;
    /// ```
    pub(super) fn prepare_tile(&mut self, stack: &TileStack) -> Outcome {
        let mut outcome = Outcome::default();
        let Some((device, _)) = self.device.clone() else {
            return outcome;
        };
        let limit = device.limits().max_texture_dimension_2d;
        if Self::oversize(stack, limit) || self.tile_textures_fit(stack) {
            return outcome;
        }
        self.replace_tile_textures(&device, stack, &mut outcome);
        return outcome;
    }

    /// What:
    ///  Render the scenes of the tile,
    ///  top first,
    ///  each into its texture,
    ///  making new textures and handing the window new images when the shape of the stack changed.
    /// Why:
    ///  The textures abut,
    ///  each starting on the row where the one above ends,
    ///  so the window places them with these rows and the tile shows as one picture.
    ///  A scene that does not fit a texture of the device is refused with a message
    ///  and the textures the window already shows stay as they are.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function drawTile(stack: TileStack, outcome: Outcome): void;
    /// ```
    pub(super) fn draw_tile(&mut self, stack: &TileStack, outcome: &mut Outcome) {
        let Some((device, queue)) = self.device.clone() else {
            return;
        };
        let limit = device.limits().max_texture_dimension_2d;
        if Self::oversize(stack, limit) {
            tracing::error!(
                width = stack.width,
                height = stack.height,
                limit,
                textures = stack.textures.len(),
                "a tile does not fit the textures of this device"
            );
            outcome.errors.push(format!(
                "Cannot draw source: a {}x{} picture does not fit a texture of this device (at most {limit} on a side)",
                stack.width, stack.height
            ));
            return;
        }
        if !self.tile_textures_fit(stack) && !self.replace_tile_textures(&device, stack, outcome) {
            return;
        }
        for (index, scene) in stack.textures.iter().enumerate() {
            let Some(held) = self.tile.get(index) else {
                return;
            };
            let texture = held.target.texture.clone();
            if let Err(error) = self.render(&device, &queue, &texture, scene) {
                tracing::error!(%error, index, "the GPU could not render a scene of the tile");
                outcome.errors.push(format!("Cannot draw source: {error}"));
                return;
            }
            self.tile_textures_rendered += 1;
        }
        self.tiles_rendered += 1;
    }
}
