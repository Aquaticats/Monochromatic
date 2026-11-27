//! The minimap's geometry without a window:
//!  the picture's scale,
//! how far it slides past the track,
//! the viewport box,
//! and the maps between a minimap position,
//! a scroll offset,
//!  and a visual row.
//!  The picture has its own test file (`minimap_scene.rs`).

/// The picture's width,
/// scale and slide,
///  the box over the view,
///  and the ends a position is held to.
#[path = "minimap/picture_scale_and_the_box.rs"]
mod picture_scale_and_the_box;

/// The maps between a position,
/// a text position and a drag,
///  and the share of the text's width across the picture.
#[path = "minimap/positions_on_the_picture.rs"]
mod positions_on_the_picture;

/// The visual row a position names,
///  and the position a row names back.
#[path = "minimap/rows_a_position_names.rs"]
mod rows_a_position_names;

/// The bands kept around the part of the picture in view,
///  the cap on them,
///  and the heading a fling follows.
#[path = "minimap/bands_the_picture_keeps.rs"]
mod bands_the_picture_keeps;
