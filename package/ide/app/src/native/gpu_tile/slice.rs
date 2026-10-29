//! The minimap's picture:
//!  rendered in chunks of at most 512 rows,
//!  each into a small scratch texture and then copied into a texture of the picture.
//!  A picture taller than the device's texture limit is several textures,
//!  each of whole chunks,
//!  stacked one below the other.
//!
//! Vello 0.11 sizes its scratch buffers with fixed constants,
//!  and a scene that needs more than they hold renders blank without an error
//!  (`doc/planning/ide-vello-buffer-sizes.md`),
//!  so a tall picture is never one render.
//! Nor is it one texture past the limit:
//!  a texture of more than about 128 MiB renders wrong without an error
//!  (`doc/troubleshooting/vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.md`),
//!  so the textures stay within the device's side limit and the byte rule of the source tile.
//! The copy is a GPU command;
//! no pixel is touched by the CPU.

/// The device,
/// the textures,
/// and the outcome the window applies.
use super::{Inner, Outcome, Placement, StackTexture, Target};
/// The markup's record of one texture of the picture.
use crate::native::ui::MinimapSlice;
/// How the chunks are grouped into textures,
/// and the device's limit as the shaped text records it.
use ide_app::minimap::{texture_rows, texture_runs};
/// The scene of a chunk and the size of its region.
use ide_app::tile_scene::TileScene;
/// The image import.
use slint::Image;
/// Slint's own wgpu types.
use slint::wgpu_30::wgpu;
/// The runs of chunks that share a texture.
use std::ops::Range;

/// The scratch textures kept at most;
/// the chunks of a picture are all one size but the last.
const SCRATCH_KEPT: usize = 4;

/// Draw the picture.
impl Inner {
    /// What:
    ///  The uses of a texture of the picture:
    ///  the picture's usual uses,
    ///  and the chunks are copied into it.
    /// Why:
    ///  Slint's image import asks for the uses of every texture it shows,
    ///  and Vello never renders into this one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function sliceUsage(): GPUTextureUsage;
    /// ```
    fn slice_usage() -> wgpu::TextureUsages {
        return Self::usage() | wgpu::TextureUsages::COPY_DST;
    }

    /// What:
    ///  The scratch texture a chunk of `width` by `height` is rendered into:
    ///  the one kept from an earlier chunk of that size,
    ///  or a new one.
    /// Why:
    ///  The chunks of a picture are one size but the last,
    ///  and the next picture's are the same,
    ///  so a few small textures serve every picture.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function scratch(device: GPUDevice, width: number, height: number): GPUTexture;
    /// ```
    fn scratch(&mut self, device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        for kept in &self.scratch {
            if kept.width == width && kept.height == height {
                return kept.texture.clone();
            }
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ide minimap chunk"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        if self.scratch.len() >= SCRATCH_KEPT {
            self.scratch.remove(0);
        }
        self.scratch.push(Target {
            texture: texture.clone(),
            width,
            height,
        });
        return texture;
    }

    /// What:
    ///  Whether the picture already has textures of `width` pixels across and exactly the heights `heights`,
    ///  top first.
    /// Why:
    ///  Rendering into the same textures needs no new image,
    ///  and the picture of another shape needs new textures and a new list of slices.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function stackFits(width: number, heights: number[]): boolean;
    /// ```
    fn stack_fits(&self, width: u32, heights: &[u32]) -> bool {
        let mut fits = self.minimap.len() == heights.len();
        for (kept, height) in self.minimap.iter().zip(heights) {
            fits &= kept.target.width == width && kept.target.height == *height;
        }
        return fits;
    }

    /// What:
    ///  Make the textures of a picture of `width` pixels across whose textures are `sizes` tall,
    ///  top first,
    ///  unless the picture already has textures of exactly these sizes.
    ///  Nothing when the device refuses to show a texture as an image.
    /// Why:
    ///  Rendering into the same texture needs no new image,
    ///  and the frame that follows samples the new content;
    ///  a picture of another shape starts from new textures and new images.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function textures(device: GPUDevice, width: number, heights: number[], outcome: Outcome): boolean;
    /// ```
    fn stack_textures(
        &mut self,
        device: &wgpu::Device,
        shape: (u32, &[u32]),
        outcome: &mut Outcome,
    ) -> bool {
        let (width, heights) = shape;
        if self.stack_fits(width, heights) {
            return true;
        }
        let mut before = Vec::with_capacity(self.minimap.len());
        for kept in &self.minimap {
            before.push(kept.target.height);
        }
        tracing::debug!(
            width,
            ?before,
            after = ?heights,
            "replaced the textures of the minimap picture"
        );
        self.minimap.clear();
        for height in heights {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ide minimap slice"),
                size: wgpu::Extent3d {
                    width,
                    height: *height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: Self::slice_usage(),
                view_formats: &[],
            });
            match Image::try_from(texture.clone()) {
                Ok(image) => self.minimap.push(StackTexture {
                    target: Target {
                        texture,
                        width,
                        height: *height,
                    },
                    image,
                }),
                Err(error) => {
                    tracing::error!(%error, "the minimap texture could not be shown as an image");
                    outcome.errors.push(format!("Cannot draw source: {error}"));
                    self.minimap.clear();
                    return false;
                }
            }
        }
        return true;
    }

    /// What:
    ///  How the chunks of a picture are grouped into textures on a device whose side is at most `limit`,
    ///  or why they do not fit one.
    /// Why:
    ///  The window is given the picture's slices when the picture is handed over and the frame draws into the textures
    ///  they show,
    ///  so both steps need the same grouping.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function shape(chunks: TileScene[], limit: number): Shape | Misfit;
    /// ```
    fn shape(chunks: &[TileScene], limit: u32) -> Result<Shape, Misfit> {
        let width = match chunks.first() {
            Some(first) => first.width,
            None => 0,
        };
        let mut heights = Vec::with_capacity(chunks.len());
        let mut ragged = false;
        for chunk in chunks {
            heights.push(chunk.height);
            ragged |= chunk.width != width;
        }
        let height: u32 = heights.iter().sum();
        // The texture limit the shaped text records is the device's, unless a test lowered it.
        let runs = texture_runs(
            &heights,
            width,
            limit.min(ide_app::shaped_text::texture_dimension()),
        );
        let allowed = u64::from(texture_rows(width, limit));
        let mut oversize = false;
        for run in &runs {
            let mut rows: u64 = 0;
            for chunk_height in &heights[run.clone()] {
                rows += u64::from(*chunk_height);
            }
            oversize |= rows > allowed;
        }
        if ragged || width == 0 || height == 0 || width > limit || oversize {
            return Err(Misfit {
                width,
                height,
                ragged,
            });
        }
        let mut sizes = Vec::with_capacity(runs.len());
        for run in &runs {
            sizes.push(heights[run.clone()].iter().sum::<u32>());
        }
        return Ok(Shape { width, runs, sizes });
    }

    /// What:
    ///  Where the picture's textures stand in the whole picture,
    ///  given that its first row is the row `top`.
    /// Why:
    ///  The window places the images with these numbers in the same step that shows the textures.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function placement(shape: Shape, top: number): Placement;
    /// ```
    fn placement(&self, shape: &Shape, top: u32) -> Placement {
        let mut slices = Vec::with_capacity(self.minimap.len());
        let mut start = 0;
        for held in &self.minimap {
            slices.push(MinimapSlice {
                image: held.image.clone(),
                top: (top + start) as f32,
                height: held.target.height as f32,
            });
            start += held.target.height;
        }
        return Placement {
            width: shape.width,
            top,
            slices,
        };
    }

    /// What:
    ///  Make the textures a picture about to be drawn needs when its shape differs from the one the window shows,
    ///  and put where they stand in the outcome,
    ///  so that the window holds their slices before the frame that draws the picture begins;
    ///  the outcome is empty when the textures fit,
    ///  when the device is not held yet (the first frame has not run),
    ///  or when the picture does not fit the device (the frame refuses it with a message).
    /// Why:
    ///  A slice list the window gets inside a frame is not shown by that frame,
    ///  and an idle window draws no other:
    ///  the first picture of a file,
    ///  whose list is a new one,
    ///  stayed blank until the next picture came.
    ///  Slices of textures made here are shown by the frame that draws the chunks into them,
    ///  because the notifier draws before the items are drawn.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function prepareMinimap(chunks: TileScene[], top: number): Outcome;
    /// ```
    pub(super) fn prepare_minimap(&mut self, chunks: &[TileScene], top: u32) -> Outcome {
        let mut outcome = Outcome::default();
        let Some((device, _)) = self.device.clone() else {
            return outcome;
        };
        let limit = device.limits().max_texture_dimension_2d;
        let Ok(shape) = Self::shape(chunks, limit) else {
            return outcome;
        };
        if self.stack_fits(shape.width, &shape.sizes) {
            return outcome;
        }
        if self.stack_textures(&device, (shape.width, &shape.sizes), &mut outcome) {
            outcome.minimap = Some(self.placement(&shape, top));
        }
        return outcome;
    }

    /// What:
    ///  Render the chunks of the minimap's picture,
    ///  top first,
    ///  into its textures,
    ///  making new textures and new images when the picture's shape changed,
    ///  and say where the picture stands in the whole picture.
    /// Why:
    ///  The chunks stack without a gap,
    ///  so the picture is as wide as they are and as tall as they are together;
    ///  the textures hold whole chunks each,
    ///  none taller than the device's limit,
    ///  and the textures stack without a gap as the chunks do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function drawStack(chunks: TileScene[], top: number, outcome: Outcome): void;
    /// ```
    pub(super) fn draw_stack(&mut self, chunks: &[TileScene], top: u32, outcome: &mut Outcome) {
        let Some((device, queue)) = self.device.clone() else {
            return;
        };
        if chunks.is_empty() {
            return;
        }
        let limit = device.limits().max_texture_dimension_2d;
        let shape = match Self::shape(chunks, limit) {
            Ok(shape) => shape,
            Err(misfit) => {
                tracing::error!(
                    width = misfit.width,
                    height = misfit.height,
                    limit,
                    ragged = misfit.ragged,
                    "a minimap picture does not fit the textures of this device"
                );
                outcome.errors.push(format!(
                    "Cannot draw source: a {}x{} minimap does not fit a texture of this device \
                     (at most {limit} on a side)",
                    misfit.width, misfit.height
                ));
                return;
            }
        };
        let width = shape.width;
        if !self.stack_textures(&device, (width, &shape.sizes), outcome) {
            return;
        }
        for (index, run) in shape.runs.iter().enumerate() {
            let Some(held) = self.minimap.get(index) else {
                return;
            };
            let texture = held.target.texture.clone();
            let mut row = 0;
            for chunk in &chunks[run.clone()] {
                let scratch = self.scratch(&device, width, chunk.height);
                if let Err(error) = self.render(&device, &queue, &scratch, chunk) {
                    tracing::error!(%error, row, "the GPU could not render a chunk of the minimap");
                    outcome.errors.push(format!("Cannot draw source: {error}"));
                    return;
                }
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ide minimap chunk copy"),
                });
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &scratch,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: row, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width,
                        height: chunk.height,
                        depth_or_array_layers: 1,
                    },
                );
                queue.submit(std::iter::once(encoder.finish()));
                row += chunk.height;
                self.minimap_chunks_rendered += 1;
            }
        }
        self.minimaps_rendered += 1;
        outcome.minimap = Some(self.placement(&shape, top));
    }
}

/// What:
///  How the chunks of a picture are grouped into textures:
///  the width,
///  the chunks each texture holds,
///  and the textures' heights.
/// Why:
///  The hand-over and the frame that draws the picture agree on the textures by computing them the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shape = { width: number; runs: [number, number][]; sizes: number[] };
/// ```
struct Shape {
    /// Width of every texture in physical pixels.
    width: u32,
    /// The chunks of each texture,
    /// top first.
    runs: Vec<Range<usize>>,
    /// The height of each texture in physical pixels,
    /// top first.
    sizes: Vec<u32>,
}

/// What:
///  Why a picture does not fit the textures of a device:
///  its size and whether its chunks differ in width.
/// Why:
///  The frame that refuses the picture names these in its message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Misfit = { width: number; height: number; ragged: boolean };
/// ```
struct Misfit {
    /// Width of the first chunk in physical pixels.
    width: u32,
    /// Height of the chunks together in physical pixels.
    height: u32,
    /// Whether some chunk differs in width from the first.
    ragged: bool,
}
