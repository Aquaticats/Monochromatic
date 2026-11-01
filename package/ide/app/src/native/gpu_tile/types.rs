//! The small types the code view and the minimap share on the GPU:
//!  a texture with its size,
//!  a texture of a stack with the image that shows it,
//!  where the minimap's textures stand,
//!  and what a step of the notifier changed for the window.

/// The window the outcome is applied to,
/// and the markup's record of one texture of the minimap and of the tile.
use crate::native::{
    AppWindow,
    ui::{MinimapSlice, SourceSlice},
};
/// Slint's own wgpu types.
use slint::wgpu_30::wgpu;
/// The image import,
/// and the shared list the window shows the minimap's textures from.
use slint::{Image, Model, ModelRc, VecModel};
/// The list is shared with the window.
use std::rc::Rc;

/// What:
///  One texture and the size it was made with.
/// Why:
///  A texture is reused while the scene it receives has the same size,
///  and replaced,
/// with a new image on the window,
/// when it has not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { texture: GPUTexture; width: number; height: number };
/// ```
pub(super) struct Target {
    /// The texture Vello renders into and Slint samples.
    pub(super) texture: wgpu::Texture,
    /// Width in physical pixels.
    pub(super) width: u32,
    /// Height in physical pixels.
    pub(super) height: u32,
}

/// What:
///  One texture of a stack,
///  the minimap's picture or the source tile:
///  the texture,
///  and the image that shows it.
/// Why:
///  A picture taller than one texture may be is several of these.
///  The image is kept so a texture that is drawn into again is shown without a new image.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StackTexture = { target: Target; image: Image };
/// ```
pub(super) struct StackTexture {
    /// The texture and its size.
    pub(super) target: Target,
    /// The image that shows the texture.
    pub(super) image: Image,
}

/// What:
///  Where the minimap's picture stands in the whole picture:
///  its width,
///  its first row,
///  and the textures that stack to make it.
/// Why:
///  The window places the images with these numbers in the same step that shows the textures,
///  so a new picture never appears at the old picture's place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placement = { width: number; top: number; slices: { image: Image; top: number; height: number }[] };
/// ```
pub(super) struct Placement {
    /// Width of every texture in physical pixels.
    pub(super) width: u32,
    /// The row of the whole picture the first texture's first row is.
    pub(super) top: u32,
    /// The textures from the top,
    /// each starting on the row where the one before ends.
    pub(super) slices: Vec<MinimapSlice>,
}

/// How many times the window was given a new list of the minimap's textures,
/// counted for the tests:
/// a picture of the same shape must change the rows of the list it has and never replace it.
#[cfg(test)]
pub(in crate::native) static SLICE_LISTS_SET: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// What:
///  Show `slices` in the list `shown` that a window property holds,
///  changing only the rows of the list when it has the same number of them,
///  and only the rows whose texture,
///  first row,
///  or height differ;
///  a new list is returned for the window to take when it holds none of ours yet.
/// Why:
///  The markup draws one image for each row of the list,
///  and a new list makes it drop and make every image again,
///  which a picture of the same bands at another place does not need;
///  the window thread does this inside a frame of a scroll.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showSlices<Slice>(shown: Slice[], slices: Slice[]): Slice[] | undefined;
/// ```
fn show_slices<Slice: Clone + PartialEq + 'static>(
    shown: ModelRc<Slice>,
    slices: Vec<Slice>,
) -> Option<ModelRc<Slice>> {
    let list_name = std::any::type_name::<Slice>();
    let Some(list) = shown.as_any().downcast_ref::<VecModel<Slice>>() else {
        tracing::debug!(
            list_name,
            rows = slices.len(),
            "gave the window a new list of slices"
        );
        #[cfg(test)]
        SLICE_LISTS_SET.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        return Some(ModelRc::from(Rc::new(VecModel::from(slices))));
    };
    if list.row_count() != slices.len() {
        tracing::debug!(
            list_name,
            before = list.row_count(),
            after = slices.len(),
            "reset the window's list of slices to another number of rows"
        );
        list.set_vec(slices);
        return None;
    }
    let mut replaced = 0_usize;
    for (row, slice) in slices.into_iter().enumerate() {
        if list.row_data(row).as_ref() != Some(&slice) {
            list.set_row_data(row, slice);
            replaced += 1;
        }
    }
    tracing::debug!(
        list_name,
        replaced,
        "replaced rows of the window's list of slices"
    );
    return None;
}

/// What:
///  What a step of the notifier changed for the window:
///  new textures to show and messages to display.
/// Why:
///  The window's setters run after the shared state is released,
///  so a setter that reaches back into the application cannot find it borrowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { sign?: Image; tile?: SourceSlice[]; minimap?: Placement; errors: string[] };
/// ```
#[derive(Default)]
pub(super) struct Outcome {
    /// The device's largest texture side,
    /// which the window is never larger than,
    /// when the device arrived.
    pub(super) texture_limit: Option<u32>,
    /// The wrap indicator's image,
    /// when it has a new texture to show.
    pub(super) sign: Option<Image>,
    /// The textures of the source tile from the top,
    /// when the tile has new ones to show.
    pub(super) tile: Option<Vec<SourceSlice>>,
    /// Where the minimap's textures stand,
    /// when a picture was drawn.
    pub(super) minimap: Option<Placement>,
    /// Failures to show as the window's error.
    pub(super) errors: Vec<String>,
}

/// Show what a step changed.
impl Outcome {
    /// Set the images and the last error on `window`.
    pub(super) fn apply(self, window: &AppWindow) {
        if let Some(limit) = self.texture_limit {
            window.set_texture_limit(i32::try_from(limit).unwrap_or(i32::MAX));
        }
        if let Some(image) = self.sign {
            window.set_wrap_sign(image);
        }
        if let Some(slices) = self.tile
            && let Some(list) = show_slices(window.get_source_slices(), slices)
        {
            window.set_source_slices(list);
        }
        if let Some(place) = self.minimap {
            let mut height = 0.0;
            for slice in &place.slices {
                height += slice.height;
            }
            if let Some(list) = show_slices(window.get_minimap_slices(), place.slices) {
                window.set_minimap_slices(list);
            }
            window.set_minimap_picture_width(place.width as f32);
            window.set_minimap_picture_height(height);
            window.set_minimap_picture_top(place.top as f32);
        }
        if let Some(message) = self.errors.last() {
            window.set_error_message(slint::SharedString::from(message.as_str()));
        }
    }
}
