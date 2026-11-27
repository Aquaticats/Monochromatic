//! The GPU's picture of the code view:
//!  the shaped rows of a tile as a list of paint items,
//!  and the Vello scene those items encode to.
//!
//! Nothing here touches a pixel.
//! A glyph run is drawn from the same Parley layout that measures it,
//!  a diagnostic underline is a stroked path,
//!  and the selected part of a row is a clipped second copy of its glyph runs in the selection ink.
//! The scene is built on whichever thread builds the tile and drawn into a texture by `native/gpu_tile.rs`.
//!
//! The parts:
//!  `encode` writes a scene from the items,
//!  `frame` keeps the underline runs of an annotation frame,
//!  `item` holds the data of the paint commands,
//!  `layout` turns shaped rows into items,
//!  `read` says what each draw command shows,
//!  `split` finds where a row's ink changes,
//!  and `underline` strokes each severity's line style.

/// Items to scene:
///  the one place a Vello scene is written.
mod encode;
/// The underline runs of an annotation frame,
/// in the texture they reach.
mod frame;
/// The data of the paint commands.
mod item;
/// Shaped rows as paint items,
///  culled to the tile and inked for their role.
mod layout;
/// A tile's items as what each draw command shows,
///  which the tests read instead of pixels.
pub mod read;
/// Where a row's ink changes.
mod split;
/// The line style of each severity as strokes.
mod underline;

/// The palette that turns a syntax role into an ink.
use crate::code_colors::CodeColors;
/// Shaped rows and the frame they belong to,
///  the textures a tile is stacked from,
///  and the largest tile.
use crate::shaped_text::{
    ShapedView,
    stack_fits,
    stack_rows,
    texture_dimension,
    texture_fits,
    texture_heights,
    tile_texture_rows,
};
/// Failures of the layout walk reach the caller.
use anyhow::{
    Result,
    bail,
};
/// Items to scene.
pub use encode::encode;
/// The underline runs of a texture.
use frame::underline_frame;
/// The paint commands of a tile.
pub use item::{
    GlyphItem,
    Item,
    StrokeItem,
    TileScene,
    TileStack,
};
/// The items of one shaped view,
///  the margin a row may lie outside the tile,
///  the ink of every role,
///  and the band of one visual row.
use layout::{
    CULL_MARGIN,
    Inking,
    layout_items,
    row_band,
    selected_intervals,
};
/// A shaped paragraph and its positioned runs.
use parley::Layout;
/// A selection edge on whole pixels,
/// and the change of ink at the edges of a selection.
pub use split::snapped;
/// The rows of a tile one texture holds.
use std::ops::Range;

/// What:
///  The paint items of one shaped view:
///  every row's glyphs with the selected part in the selection ink,
///  then the hint and message texts,
///  then the diagnostic underlines.
///  `horizontal` is the fractional offset of the text,
///  in logical pixels.
/// Why:
///  This is the whole picture of a tile,
///  in drawing order,
///  and the tests read it instead of a pixel buffer.
///
/// # Errors
/// A layout holds an inline box,
/// which source text never produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tileItems(view: ShapedView, colors: CodeColors, horizontal: number): Item[];
/// ```
pub fn tile_items(
    view: &ShapedView,
    colors: CodeColors,
    horizontal: f32,
) -> Result<Vec<Item>> {
    return tile_items_in(view, colors, horizontal, 0..view.height);
}

/// What:
///  The paint items of the `rows` of one shaped view,
///  placed from the first of those rows:
///  the items of [`tile_items`] that can reach one texture of the stack the tile is drawn across.
/// Why:
///  The texture is `rows.end - rows.start` pixels tall and its top is the tile's row `rows.start`.
///  The shift is a whole number of pixels,
///  so a glyph that two textures share is drawn in each at the same fractional position,
///  and the two halves meet as one drawing would.
///  Rows,
/// hints,
/// and underlines wholly outside the texture are left out,
///  so the renderer's buffers hold one texture's share and not the tile's.
///
/// # Errors
/// A layout holds an inline box,
/// which source text never produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tileItemsIn(view: ShapedView, colors: CodeColors, horizontal: number, rows: [number, number]): Item[];
/// ```
pub fn tile_items_in(
    view: &ShapedView,
    colors: CodeColors,
    horizontal: f32,
    rows: Range<u32>,
) -> Result<Vec<Item>> {
    let mut items: Vec<Item> = Vec::new();
    let factor = view.viewport.scale;
    let shift = rows.start as f32;
    let size = (view.width as f32, (rows.end - rows.start) as f32);
    for row in &view.rows {
        // Rows are placed by the frame's vertical mapping, relative to the top of the tile.
        let row_y = (row.top - view.origin) * factor - shift;
        // A row whose layout lies wholly outside the texture paints nothing in it.
        if row_y + row.baseline_shift + row.layout.height() < -CULL_MARGIN
            || row_y + row.baseline_shift > size.1 + CULL_MARGIN
        {
            continue;
        }
        let selected = selected_intervals(view, row, horizontal);
        let inking = Inking {
            colors,
            fixed: None,
            selected: &selected,
        };
        let origin = (-horizontal * factor, row_y + row.baseline_shift);
        layout_items(&mut items, &row.layout, origin, &inking, size)?;
    }
    if let Some(frame) = &view.annotations {
        for text in &frame.texts {
            // A virtual row lies its rise above the code row of the line it belongs to.
            let top = view.map.code_top(text.line) - text.rise;
            let origin = (
                (text.x - horizontal) * factor,
                (top - view.origin) * factor - shift,
            );
            let inking = Inking {
                colors,
                fixed: Some(text.ink),
                selected: &[],
            };
            layout_items(&mut items, &text.layout, origin, &inking, size)?;
        }
        underline_frame(&mut items, view, frame, colors, horizontal, &rows);
    }
    return Ok(items);
}

/// What:
///  The scene of one shaped view as one texture,
///  with the size of the texture it is drawn into.
/// Why:
///  A view that one texture holds is the common case,
///  and the tests read its scene whole.
///
/// # Errors
/// The view is larger than one texture may be (the device's texture size and Vello's bin count),
/// or a layout holds an inline box,
/// which source text never produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paintTile(view: ShapedView, colors: CodeColors, horizontal: number): TileScene;
/// ```
pub fn paint_tile(
    view: &ShapedView,
    colors: CodeColors,
    horizontal: f32,
) -> Result<TileScene> {
    if !texture_fits(u64::from(view.width), u64::from(view.height)) {
        bail!(
            "Source viewport {}x{} does not fit one texture ({} rows of this width, {} px on a side); paint it as a stack",
            view.width,
            view.height,
            tile_texture_rows(u64::from(view.width)),
            texture_dimension()
        );
    }
    let items = tile_items(view, colors, horizontal)?;
    return Ok(TileScene {
        scene: encode(&items, view.height as f32),
        width: view.width,
        height: view.height,
    });
}

/// What:
///  The scenes of one shaped view across the textures that stack to show it:
///  one scene for each of the heights [`texture_heights`] gives for the view's width and height,
///  top first.
/// Why:
///  A window larger than one texture allows is drawn across several,
///  each within the device's texture size and Vello's bin count,
///  and the textures abut without a gap or an overlap,
///  so the stack shows the rows one texture would.
///  The tile worker and the window thread build the same stack from the same rows,
///  so a tile built on either looks the same.
///
/// # Errors
/// The view is taller than the stack may be ([`stack_rows`]) or wider than a texture may be,
/// or a layout holds an inline box,
/// which source text never produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paintStack(view: ShapedView, colors: CodeColors, horizontal: number): TileStack;
/// ```
pub fn paint_stack(
    view: &ShapedView,
    colors: CodeColors,
    horizontal: f32,
) -> Result<TileStack> {
    // The window never asks for a view past the limit, because the shaper cuts a tile of a very tall line to a clip;
    // this guards a caller that did not, since the GPU would reject a texture past its size with a validation error.
    // The bin rule is what keeps every render inside Vello's coarse stage, and the 64 MiB byte rule it implies is what
    // keeps Vello's command list large enough: its default size serves 131,072 tiles of 16 by 16 pixels, 128 MiB of
    // texture, and a render past that is wrong with no failure reported.
    // `tests/tile_vello_limit.rs` holds the rule to half of that floor.
    if !stack_fits(u64::from(view.width), u64::from(view.height)) {
        bail!(
            "Source viewport {}x{} exceeds the tile stack limit ({} rows of this width, {} px on a side); reduce the window size or scale",
            view.width,
            view.height,
            stack_rows(u64::from(view.width)),
            texture_dimension()
        );
    }
    let mut textures = Vec::new();
    let mut top = 0_u32;
    for height in texture_heights(u64::from(view.width), u64::from(view.height)) {
        let rows = top..top + height;
        let items = tile_items_in(view, colors, horizontal, rows)?;
        textures.push(TileScene {
            scene: encode(&items, height as f32),
            width: view.width,
            height,
        });
        top += height;
    }
    return Ok(TileStack {
        textures,
        width: view.width,
        height: view.height,
    });
}

/// What:
///  The scene of one layout alone,
///  at the physical `origin`,
///  in one `ink`,
///  for a texture of `size` physical pixels.
/// Why:
///  Small pictures,
///  such as the gutter's wrap indicator,
///  are drawn by the same glyph path as source text.
///
/// # Errors
/// The layout holds an inline box,
/// which source text never produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function layoutScene(layout: Layout, origin: [number, number], ink: Rgba, size: [number, number]): TileScene;
/// ```
pub fn layout_scene(
    layout: &Layout<u32>,
    origin: (f32, f32),
    ink: [u8; 4],
    size: (u32, u32),
) -> Result<TileScene> {
    let mut items: Vec<Item> = Vec::new();
    let inking = Inking {
        colors: CodeColors {
            foreground: ink,
            selected: ink,
            dark: false,
        },
        fixed: Some(ink),
        selected: &[],
    };
    layout_items(
        &mut items,
        layout,
        origin,
        &inking,
        (size.0 as f32, size.1 as f32),
    )?;
    return Ok(TileScene {
        scene: encode(&items, size.1 as f32),
        width: size.0,
        height: size.1,
    });
}
