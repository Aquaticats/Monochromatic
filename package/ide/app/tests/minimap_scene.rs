//! The picture the minimap draws,
//!  read through `tile_scene::read`:
//!  its slices and bands,
//!  the items drawn in the palette inks,
//!  the work a second build redoes,
//!  and the chunks and textures the plan cuts it into.
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this crate's modules live under `tests/minimap_scene/`.
//! Why:
//!  One test crate links the library once and shares the fixture text,
//!       palettes and source without unused-code warnings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import * as fixtures from './minimap_scene/fixtures';
//! ```

#[path = "minimap_scene/fixtures.rs"]
mod fixtures;

/// The picture's slices and bands, and the texture limit.
#[path = "minimap_scene/slices_and_bands.rs"]
mod slices_and_bands;

/// The glyphs, marks and strokes a slice draws.
#[path = "minimap_scene/drawn_items.rs"]
mod drawn_items;

/// The work a second build redoes, and what it leaves.
#[path = "minimap_scene/incremental_shaping.rs"]
mod incremental_shaping;

/// The chunks and textures the plan cuts, and their limits.
#[path = "minimap_scene/chunks_and_textures.rs"]
mod chunks_and_textures;
