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
        let mut reusable = self.minimap.len() == heights.len();
        for (kept, height) in self.minimap.iter().zip(heights) {
            reusable &= kept.target.width == width && kept.target.height == *height;
        }
        if reusable {
            return true;
        }
        tracing::debug!(
            width,
            before = ?self.minimap.iter().map(|kept| return kept.target.height).collect::<Vec<_>>(),
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
        let Some(first) = chunks.first() else {
            return;
        };
        let width = first.width;
        let mut heights = Vec::with_capacity(chunks.len());
        let mut ragged = false;
        for chunk in chunks {
            heights.push(chunk.height);
            ragged |= chunk.width != width;
        }
        let height: u32 = heights.iter().sum();
        let limit = device.limits().max_texture_dimension_2d;
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
            tracing::error!(
                width,
                height,
                limit,
                ragged,
                "a minimap picture does not fit the textures of this device"
            );
            outcome.errors.push(format!(
                "Cannot draw source: a {width}x{height} minimap does not fit a texture of this device \
                 (at most {limit} on a side)"
            ));
            return;
        }
        let mut sizes = Vec::with_capacity(runs.len());
        for run in &runs {
            sizes.push(heights[run.clone()].iter().sum::<u32>());
        }
        if !self.stack_textures(&device, (width, &sizes), outcome) {
            return;
        }
        let mut shown = Vec::with_capacity(runs.len());
        let mut start = 0;
        for (run, texture_height) in runs.iter().zip(&sizes) {
            let Some(held) = self.minimap.get(shown.len()) else {
                return;
            };
            let texture = held.target.texture.clone();
            shown.push(MinimapSlice {
                image: held.image.clone(),
                top: (top + start) as f32,
                height: *texture_height as f32,
            });
            start += texture_height;
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
        outcome.minimap = Some(Placement {
            width,
            top,
            slices: shown,
        });
    }
}
