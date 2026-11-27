//! The textures of a picture:
//!  the scratch texture one chunk is rendered into,
//!  and the textures of the picture,
//!  reused while its shape stays the same.

/// The device,
/// the picture's textures,
/// and the outcome the window applies.
use super::super::{
    Inner,
    Outcome,
    StackTexture,
    Target,
};
/// The scratch textures kept at most.
use super::SCRATCH_KEPT;
/// The picture's texture as an image the window shows.
use slint::Image;
/// The GPU's own types.
use slint::wgpu_30::wgpu;

/// The textures of one picture,
///  made when its shape changes and reused while it stays the same.
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
    pub(super) fn scratch(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> wgpu::Texture {
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
    pub(super) fn stack_fits(
        &self,
        width: u32,
        heights: &[u32],
    ) -> bool {
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
    pub(super) fn stack_textures(
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
}
