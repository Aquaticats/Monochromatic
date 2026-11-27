//! The picture's scale and the box that covers the view:
//!  the width a narrow view leaves,
//!  the scale a long text keeps while its picture slides past the track,
//!  and the ends both offsets and positions are held to.

/// The geometry and its constants.
use ide_app::minimap::{
    Geometry,
    MIN_BOX,
    NARROWEST,
    WIDTH,
    width_for,
};
/// The vertical mapping the minimap shows.
use ide_app::row_map::CODE_ROW;

/// The minimap is as wide as CodeGlance's 110 pixels,
/// and shrinks to a sixth of a narrow view down to 15.
#[test]
fn the_width_follows_the_view_down_to_fifteen_pixels() {
    assert_eq!(width_for(1000.0), WIDTH);
    assert_eq!(width_for(660.0), WIDTH);
    assert_eq!(width_for(480.0), 80.0);
    assert_eq!(width_for(240.0), 40.0);
    assert_eq!(width_for(60.0), NARROWEST);
    assert_eq!(width_for(0.0), NARROWEST);
}

/// A text that fits the view at four pixels a row is drawn at that scale,
/// and the box covers the view's share.
#[test]
fn a_text_that_fits_is_drawn_at_four_pixels_a_row() {
    // 100 code rows, 2,400 px, in a view of 600: 400 px of picture, a box of 100 px.
    let geometry = Geometry::new(100.0 * CODE_ROW, 600.0);
    assert_eq!(geometry.scale(), 4.0 / CODE_ROW);
    assert_eq!(geometry.track(), 400.0);
    assert_eq!(
        geometry.slide(900.0),
        0.0,
        "a picture that fits does not slide"
    );
    assert_eq!(geometry.box_height(), 100.0);
    assert_eq!(geometry.room(), 300.0);
    assert_eq!(geometry.max_offset(), 1800.0);
    assert_eq!(geometry.box_top(0.0), 0.0);
    assert_eq!(geometry.box_top(900.0), 150.0);
    assert_eq!(geometry.box_top(1800.0), 300.0);
}

/// A text that fits the view itself is drawn in full,
/// and the box covers all of it at the top.
#[test]
fn a_text_shorter_than_the_view_is_covered_by_the_box() {
    let geometry = Geometry::new(10.0 * CODE_ROW, 600.0);
    assert_eq!(geometry.track(), 40.0);
    assert_eq!(geometry.box_height(), 40.0, "the whole text is in view");
    assert_eq!(geometry.room(), 0.0);
    assert_eq!(geometry.max_offset(), 0.0);
    for offset in [-50.0, 0.0, 70.0, 1e6] {
        assert_eq!(geometry.box_top(offset), 0.0);
    }
    // A text of three rows is smaller than the least box.
    let tiny = Geometry::new(3.0 * CODE_ROW, 600.0);
    assert_eq!(tiny.track(), 12.0);
    assert_eq!(tiny.box_height(), 12.0);
}

/// A text with more rows than the track has pixels keeps the scale:
/// the picture slides past the track with the scroll,
/// and the box keeps the view's share of the scale.
#[test]
fn a_long_text_keeps_the_scale_and_its_picture_slides_past_the_track() {
    let content = 50_000.0 * CODE_ROW;
    let geometry = Geometry::new(content, 600.0);
    assert_eq!(
        geometry.scale(),
        4.0 / CODE_ROW,
        "the scale does not follow the length"
    );
    assert_eq!(geometry.track(), 600.0);
    // The view is 600 of 1,200,000 px, a sixth of that in the picture.
    assert_eq!(geometry.box_height(), 100.0);
    // A view under 120 px would have a box under the least height.
    assert_eq!(Geometry::new(content, 90.0).box_height(), MIN_BOX);
    assert_eq!(geometry.room(), 500.0);
    assert_eq!(geometry.box_top(0.0), 0.0);
    assert_eq!(geometry.box_top(geometry.max_offset()), 500.0);
    // The picture is 200,000 px tall: it has slid nothing at the top and all of its overhang at the end.
    assert_eq!(geometry.slide(0.0), 0.0);
    let overhang = content / 6.0 - 600.0;
    assert!((geometry.slide(geometry.max_offset()) - overhang).abs() < 1e-1);
    // Whatever the offset, the box lies where the view's rows are in the picture as it has slid.
    for share in [0.0_f32, 0.1, 0.5, 0.77, 1.0] {
        let offset = geometry.max_offset() * share;
        let in_picture = offset / 6.0 - geometry.slide(offset);
        assert!(
            (in_picture - geometry.box_top(offset)).abs() < 1e-1,
            "at {share} of the range the view's rows lie at {in_picture} and the box at {}",
            geometry.box_top(offset)
        );
    }
}

/// An offset or a position outside the range is held to its ends.
#[test]
fn positions_outside_the_range_are_clamped_at_both_ends() {
    let geometry = Geometry::new(100.0 * CODE_ROW, 600.0);
    assert_eq!(geometry.box_top(-100.0), 0.0);
    assert_eq!(geometry.box_top(1e9), geometry.room());
    assert_eq!(geometry.content_y(-20.0, 0.0), 0.0);
    assert!((geometry.content_y(1e6, 0.0) - 100.0 * CODE_ROW).abs() < 1e-2);
    assert_eq!(geometry.y_of(-5.0, 0.0), 0.0);
    assert_eq!(geometry.y_of(1e9, 0.0), geometry.track());
    assert_eq!(geometry.drag_offset(500.0, 10.0, -1e6), 0.0);
    assert_eq!(
        geometry.drag_offset(500.0, 10.0, 1e6),
        geometry.max_offset()
    );
    assert_eq!(
        geometry.centered(0.0),
        0.0,
        "the first row cannot be centered"
    );
    assert_eq!(geometry.centered(1e9), geometry.max_offset());
    assert_eq!(geometry.centered(1000.0), 700.0);
}
