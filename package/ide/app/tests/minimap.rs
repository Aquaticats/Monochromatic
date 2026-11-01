//! The minimap's geometry without a window:
//!  the picture's scale,
//! how far it slides past the track,
//! the viewport box,
//! and the maps between a minimap position,
//! a scroll offset,
//!  and a visual row.
//!  The picture has its own test file (`minimap_scene.rs`).

/// The visual row counts and the map built from them.
use ide_app::line_rows::LineRows;
/// The geometry and its constants.
use ide_app::minimap::{
    BAND, Geometry, Heading, LEAD, MIN_BOX, NARROWEST, Target, WIDTH, band_count, text_span,
    text_x, width_for,
};
/// The vertical mapping the minimap shows.
use ide_app::row_map::{CODE_ROW, RowMap};
/// The width of an unwrapped raster.
use ide_app::shaped_text::UNWRAPPED_RASTER_WIDTH;

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

/// A position on the minimap shows the visual row of the vertical mapping it lies on,
/// wrapped rows and blocks
///  included,
/// and the row's own position shows that row again.
#[test]
fn a_position_names_the_visual_row_it_shows_and_the_row_names_its_position_back() {
    let mut rows = LineRows::plain(100);
    rows.apply([(10, 3), (40, 2)]);
    // A block of 48 px stands above line 5.
    let map = RowMap::new(100, &[(5, 48.0)]).with_rows(&rows);
    let geometry = Geometry::new(map.height(), 1000.0);
    for target in [
        Target { line: 0, row: 0 },
        Target { line: 4, row: 0 },
        Target { line: 5, row: 0 },
        Target { line: 10, row: 0 },
        Target { line: 10, row: 1 },
        Target { line: 10, row: 2 },
        Target { line: 40, row: 1 },
        Target { line: 99, row: 0 },
    ] {
        // The middle of the row's pixels, which are CODE_ROW over six tall.
        let y = geometry.y_of_row(&map, target, 0.0) + 0.5;
        assert_eq!(geometry.target(&map, y, 0.0), target, "the row at {y}");
    }
    // The block above line 5 shows line 5's first row.
    let in_block = geometry.y_of(map.block_top(5) + 10.0, 0.0);
    assert_eq!(
        geometry.target(&map, in_block, 0.0),
        Target { line: 5, row: 0 }
    );
    // Both ends.
    assert_eq!(
        geometry.target(&map, -10.0, 0.0),
        Target { line: 0, row: 0 }
    );
    assert_eq!(geometry.target(&map, 1e6, 0.0), Target { line: 99, row: 0 });
}

/// In a long text a position names a row of the part of the picture that is in view:
/// the row lies on that position,
/// and the part in view follows the scroll.
#[test]
fn in_a_long_text_a_position_names_a_row_that_lies_on_it_at_every_scroll() {
    let map = RowMap::plain(50_000);
    let geometry = Geometry::new(map.height(), 600.0);
    for share in [0.0_f32, 0.25, 0.5, 1.0] {
        let offset = geometry.max_offset() * share;
        for y in [0.0_f32, 0.5, 100.0, 299.5, 599.9] {
            let target = geometry.target(&map, y, offset);
            let top = geometry.y_of_row(&map, target, offset);
            // A code row is four pixels of the picture.
            assert!(
                top <= y + 1e-2 && y - top < 4.0 + 1e-2,
                "at {share} of the range position {y} named line {} at {top}",
                target.line
            );
        }
    }
    // The middle of the track in the middle of the range shows the middle of the text.
    let middle = geometry.max_offset() / 2.0;
    assert!((24_999..=25_001).contains(&geometry.target(&map, 300.0, middle).line));
    // The top of the track at the top shows the first line and the bottom at the end shows the last.
    assert_eq!(geometry.target(&map, 0.0, 0.0).line, 0);
    assert_eq!(
        geometry.target(&map, 600.0, geometry.max_offset()).line,
        49_999
    );
}

/// The bands to keep are the ones the track shows and one on each side,
/// held to the text,
/// and the same bands for a scroll that stays within a band.
#[test]
fn the_bands_to_keep_surround_the_part_of_the_picture_in_view() {
    let step = 24.0 * 33.0;
    let content = 50_000.0 * CODE_ROW;
    let geometry = Geometry::new(content, 600.0);
    let total = band_count(content, step);
    assert_eq!(total, (content / step).ceil() as usize);
    for share in [0.0_f32, 0.001, 0.4, 0.999, 1.0] {
        let offset = geometry.max_offset() * share;
        let bands = geometry.bands_for(offset, step, usize::MAX);
        let shown_from = geometry.slide(offset) / geometry.scale();
        let shown_to = (geometry.slide(offset) + geometry.track()) / geometry.scale();
        assert!(bands.end <= total && bands.start < bands.end);
        // A whole band of margin lies on each side of the part in view, but at the ends of the text.
        assert!(
            bands.start == 0 || (bands.start as f32 + 1.0) * step <= shown_from,
            "at {share} the bands start at {} after the part in view at {shown_from} less a band",
            bands.start
        );
        assert!(
            bands.end == total || bands.end as f32 * step >= shown_to + step,
            "at {share} the bands end at {} before the part in view ends at {shown_to} plus a band",
            bands.end
        );
        // The picture's 600 px are 3,600 px of text, five bands of this height, with one more on each side.
        assert!(bands.len() <= 8, "{} bands for one view", bands.len());
    }
    // A scroll within a band's height of the picture's slide names the same bands.
    let at = geometry.max_offset() * 0.5;
    let same = geometry.bands_for(at + 1.0, step, usize::MAX);
    assert_eq!(geometry.bands_for(at, step, usize::MAX), same);
    // A short text is one window of all its bands.
    let short = Geometry::new(10.0 * CODE_ROW, 600.0);
    assert_eq!(short.bands_for(0.0, step, usize::MAX), 0..1);
    assert_eq!(
        Geometry::new(0.0, 600.0).bands_for(0.0, step, usize::MAX),
        0..0
    );
}

/// A texture holds only so many bands:
/// a window capped to that many starts at the band the track starts in,
/// keeps the bands below it,
/// and never names more than the cap.
#[test]
fn a_cap_on_the_bands_keeps_those_from_the_top_of_the_track() {
    let step = 768.0;
    let content = 50_000.0 * CODE_ROW;
    // A view 12,000 px tall: its track is 12,000 px, 15.6 bands of the text's 768 px at a sixth.
    let geometry = Geometry::new(content, 12_000.0);
    let offset = geometry.max_offset() * 0.4;
    let free = geometry.bands_for(offset, step, usize::MAX);
    assert!(free.len() > 20, "the free window is {} bands", free.len());
    for cap in [1_usize, 8, 16, 64] {
        let capped = geometry.bands_for(offset, step, cap);
        assert!(
            capped.len() <= cap,
            "{} bands past the cap {cap}",
            capped.len()
        );
        assert!(capped.start >= free.start && capped.end <= free.end);
        if cap < free.len() {
            let shown_from = geometry.slide(offset) / geometry.scale();
            assert!(
                capped.start as f32 * step <= shown_from
                    && shown_from < (capped.start as f32 + 1.0) * step,
                "the capped window {capped:?} does not start in the band the track starts in ({shown_from})"
            );
        }
    }
}

/// A view heading down holds `LEAD` bands past the track and one before it,
/// a view heading up the other way round,
/// and a view at rest one on each side:
/// the window of a fling reaches the bands the next picture needs.
#[test]
fn the_window_of_a_fling_reaches_ahead_on_the_side_it_heads_toward() {
    let step = BAND;
    let content = 50_000.0 * CODE_ROW;
    let geometry = Geometry::new(content, 600.0);
    let offset = geometry.max_offset() * 0.4;
    let still = geometry.bands_for(offset, step, usize::MAX);
    assert_eq!(
        geometry.bands_toward(offset, step, usize::MAX, Heading::Still { at: 7.0 }),
        still,
        "a view at rest keeps one band on each side"
    );
    let down = geometry.bands_toward(offset, step, usize::MAX, Heading::Down { at: 0.0 });
    let up = geometry.bands_toward(offset, step, usize::MAX, Heading::Up { at: 0.0 });
    assert_eq!(
        down.start, still.start,
        "heading down keeps one band behind"
    );
    assert_eq!(
        down.end,
        still.end - 1 + LEAD,
        "heading down holds the lead after the track"
    );
    assert_eq!(up.end, still.end, "heading up keeps one band behind");
    assert_eq!(
        up.start,
        still.start + 1 - LEAD,
        "heading up holds the lead before the track"
    );
    // The window never leaves the text: at its ends the lead is cut short.
    let total = band_count(content, step);
    let last = geometry.bands_toward(
        geometry.max_offset(),
        step,
        usize::MAX,
        Heading::Down { at: 0.0 },
    );
    assert_eq!(last.end, total);
    let first = geometry.bands_toward(0.0, step, usize::MAX, Heading::Up { at: 0.0 });
    assert_eq!(first.start, 0);
    // A cap on the bands still wins over the lead.
    let capped = geometry.bands_toward(offset, step, 3, Heading::Down { at: 0.0 });
    assert!(capped.len() <= 3, "{} bands past the cap", capped.len());
}

/// A heading is confirmed by a move of a whole band from where it was decided,
/// kept through a creep of less than a band either way,
/// and turned by a whole band against it.
#[test]
fn a_heading_follows_the_track_by_whole_bands() {
    let step = BAND;
    let still = Heading::Still { at: 1_000.0 };
    assert_eq!(
        still.follow(1_000.0 + step - 1.0, step),
        still,
        "less than a band changes nothing"
    );
    assert_eq!(still.follow(1_000.0 - step + 1.0, step), still);
    let down = still.follow(1_000.0 + step, step);
    assert_eq!(down, Heading::Down { at: 1_000.0 + step });
    // A creep back within a band keeps the heading and the place where it was decided.
    assert_eq!(down.follow(1_000.0 + step - 100.0, step), down);
    assert_eq!(down.follow(1_000.0 + 2.0 * step - 1.0, step), down);
    // The next band confirms it again, one step further.
    assert_eq!(
        down.follow(1_000.0 + 2.0 * step, step),
        Heading::Down {
            at: 1_000.0 + 2.0 * step
        }
    );
    // A whole band back turns it.
    assert_eq!(down.follow(1_000.0, step), Heading::Up { at: 1_000.0 });
    assert_eq!(Heading::Still { at: 0.0 }.margins(), (1, 1));
    assert_eq!(down.margins(), (1, LEAD));
    assert_eq!(Heading::Up { at: 0.0 }.margins(), (LEAD, 1));
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
