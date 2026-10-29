//! What the native tests ask of the GPU state:
//!  how much was rendered,
//!  how big the textures are,
//!  and the textures read back as pixels.

/// The readback of one texture.
use super::readback::read_texture;
/// The handle the window state holds,
/// and the state behind it.
use super::{GpuTile, Inner, StackTexture};
/// The pixels of a read.
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// Counts,
/// sizes,
/// and reads for the structural tests.
impl GpuTile {
    /// How many tile scenes the GPU has rendered into a texture,
    /// for the structural tests:
    /// scrolling inside the tile renders none.
    pub(in crate::native) fn tiles_rendered(&self) -> u64 {
        return self.inner.borrow().tiles_rendered;
    }

    /// How many wrap indicator scenes the GPU has rendered.
    pub(in crate::native) fn signs_rendered(&self) -> u64 {
        return self.inner.borrow().signs_rendered;
    }

    /// How many scenes of tiles the GPU has rendered,
    /// for the structural tests:
    /// a tile of several textures is several scenes.
    pub(in crate::native) fn tile_textures_rendered(&self) -> u64 {
        return self.inner.borrow().tile_textures_rendered;
    }

    /// How many textures the GPU has made for tiles,
    /// for the structural tests:
    /// a tile of the shape the last one had is drawn into the textures it has.
    pub(in crate::native) fn tile_textures_made(&self) -> u64 {
        return self.inner.borrow().tile_textures_made;
    }

    /// The tile's textures read back from the GPU as straight RGBA and stacked into one buffer,
    /// row by row from the top,
    /// or nothing before a tile is drawn.
    /// A frame must have been rendered since the last scene was handed over.
    pub(in crate::native) fn read_tile(&self) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        let inner = self.inner.borrow();
        return inner.read_textures(&inner.tile);
    }

    /// The wrap indicator's texture read back from the GPU as straight RGBA,
    /// or nothing before it is drawn.
    pub(in crate::native) fn read_sign(&self) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        let inner = self.inner.borrow();
        let (device, queue) = inner.device.as_ref()?;
        let target = inner.sign.as_ref()?;
        return Some(read_texture(
            device,
            queue,
            &target.texture,
            (target.width, target.height),
        ));
    }

    /// The minimap's textures read back from the GPU as straight RGBA and stacked into one buffer,
    /// row by row from the top of the picture,
    /// or nothing before a picture is drawn.
    pub(in crate::native) fn read_minimap(&self) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        let inner = self.inner.borrow();
        return inner.read_textures(&inner.minimap);
    }

    /// How many minimap pictures the GPU has rendered into textures,
    /// for the structural tests:
    /// a scroll inside the bands of the picture renders none.
    pub(in crate::native) fn minimaps_rendered(&self) -> u64 {
        return self.inner.borrow().minimaps_rendered;
    }

    /// How many minimap chunks the GPU has rendered,
    /// for the structural tests:
    /// a tall picture is several renders.
    pub(in crate::native) fn minimap_chunks_rendered(&self) -> u64 {
        return self.inner.borrow().minimap_chunks_rendered;
    }

    /// The width and height in physical pixels of each texture of the minimap's picture,
    /// top first,
    /// or nothing before the first picture is drawn.
    pub(in crate::native) fn minimap_texture_sizes(&self) -> Vec<(u32, u32)> {
        let inner = self.inner.borrow();
        let mut sizes = Vec::with_capacity(inner.minimap.len());
        for texture in &inner.minimap {
            sizes.push((texture.target.width, texture.target.height));
        }
        return sizes;
    }

    /// The width and height in physical pixels of each texture of the tile,
    /// top first,
    /// or nothing before the first tile is drawn.
    pub(in crate::native) fn tile_texture_sizes(&self) -> Vec<(u32, u32)> {
        let inner = self.inner.borrow();
        let mut sizes = Vec::with_capacity(inner.tile.len());
        for texture in &inner.tile {
            sizes.push((texture.target.width, texture.target.height));
        }
        return sizes;
    }

    /// The size of the tile in physical pixels:
    /// the width of its textures and their heights together,
    /// or nothing before the first tile is drawn.
    pub(in crate::native) fn tile_size(&self) -> Option<(u32, u32)> {
        let inner = self.inner.borrow();
        let first = inner.tile.first()?;
        let mut height = 0;
        for texture in &inner.tile {
            height += texture.target.height;
        }
        return Some((first.target.width, height));
    }

    /// Whether a scene waits for the next frame.
    pub(in crate::native) fn has_pending(&self) -> bool {
        let inner = self.inner.borrow();
        return inner.pending_tile.is_some()
            || inner.pending_sign.is_some()
            || inner.pending_minimap.is_some();
    }
}

/// Reads of the textures.
impl Inner {
    /// The textures of a stack read back from the GPU and stacked into one buffer of their common width,
    /// or nothing when there is no texture or the textures differ in width.
    fn read_textures(&self, textures: &[StackTexture]) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        let (device, queue) = self.device.as_ref()?;
        let first = textures.first()?;
        let width = first.target.width;
        let mut height = 0;
        for texture in textures {
            height += texture.target.height;
        }
        let mut stacked = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        let mut row = 0;
        for texture in textures {
            if texture.target.width != width {
                return None;
            }
            let part = read_texture(
                device,
                queue,
                &texture.target.texture,
                (width, texture.target.height),
            );
            let start = row as usize * width as usize;
            let slots = stacked.make_mut_slice();
            slots[start..start + part.as_slice().len()].copy_from_slice(part.as_slice());
            row += texture.target.height;
        }
        return Some(stacked);
    }
}
