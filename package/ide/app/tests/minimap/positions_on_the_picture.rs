//! Positions on the picture and what they name:
//!  a position and the text position it shows are each other's inverse,
//!  a drag moves the view by the distance it moves the box,
//!  and a position across the picture is the same share of the text's width.

/// The geometry and its constants.
use ide_app::minimap::{
    Geometry,
    text_span,
    text_x,
};
/// The vertical mapping the minimap shows.
use ide_app::row_map::CODE_ROW;
/// The width of an unwrapped raster.
use ide_app::shaped_text::UNWRAPPED_RASTER_WIDTH;

/// A position on the minimap and the text position it shows are each other's inverse,
/// at every scroll offset.
#[test]
fn a_minimap_position_and_its_text_position_are_each_others_inverse() {
    for (rows, view) in [
        (100.0, 600.0),
        (5_000.0, 600.0),
        (50_000.0, 800.0),
        (30.0, 900.0),
    ] {
        let geometry = Geometry::new(rows * CODE_ROW, view);
        let track = geometry.track();
        for share in [0.0_f32, 0.3, 1.0] {
            let offset = geometry.max_offset() * share;
            for step in 0..=20 {
                let y = track * step as f32 / 20.0;
                let back = geometry.y_of(geometry.content_y(y, offset), offset);
                assert!(
                    (back - y).abs() < 1e-1,
                    "{rows} rows in {view} at {share}: position {y} came back as {back}"
                );
            }
        }
    }
}

/// Dragging the box by a distance moves the view by that distance over the box's room,
/// times the scroll range,
///  and the box follows the pointer exactly except where the range stops it.
#[test]
fn dragging_the_box_moves_it_with_the_pointer() {
    for (rows, view) in [(100.0, 600.0), (5_000.0, 600.0), (50_000.0, 800.0)] {
        let geometry = Geometry::new(rows * CODE_ROW, view);
        let start = geometry.max_offset() / 3.0;
        let grabbed = geometry.box_top(start) + 7.0;
        for distance in [-30.0_f32, -1.0, 0.0, 1.0, 12.0, 40.0] {
            let offset = geometry.drag_offset(start, grabbed, grabbed + distance);
            let wanted = (geometry.box_top(start) + distance).clamp(0.0, geometry.room());
            assert!(
                (geometry.box_top(offset) - wanted).abs() < 1e-2,
                "{rows} rows: a drag of {distance} put the box at {} instead of {wanted}",
                geometry.box_top(offset)
            );
        }
    }
}

/// The picture is the text's whole width shrunk to the minimap's,
/// so a position across it is the same share of the text's width,
/// held to both edges,
/// and a wrapped text spans its wrap width.
#[test]
fn a_position_across_the_minimap_is_the_same_share_of_the_texts_width() {
    assert_eq!(text_x(0.0, 110.0, 880.0), 0.0);
    assert_eq!(text_x(55.0, 110.0, 880.0), 440.0);
    assert_eq!(text_x(110.0, 110.0, 880.0), 880.0);
    assert_eq!(text_x(27.5, 110.0, 880.0), 220.0);
    assert_eq!(text_x(-30.0, 110.0, 880.0), 0.0);
    assert_eq!(text_x(500.0, 110.0, 880.0), 880.0);
    // A minimap or a text without width maps everything to the left edge.
    assert_eq!(text_x(10.0, 0.0, 880.0), 0.0);
    assert_eq!(text_x(10.0, 110.0, 0.0), 0.0);
    assert_eq!(text_span(Some(673.0)), 673.0);
    assert_eq!(text_span(None), UNWRAPPED_RASTER_WIDTH);
}
