//! The tiles painted from wrapped rows:
//!  a tile past the stack limit is refused,
//!  a clipped tile holds the pixels of its band,
//!  and a line past the real texture limit paints inside a clip.

use crate::fixtures::{
    PALETTE,
    viewport,
    words,
};
use ide_app::{
    document::Document,
    row_map::{
        CODE_ROW,
        RowMap,
    },
    shaped_text::{
        Clip,
        TextShaper,
        Viewport,
        stack_fits,
        stack_rows,
        texture_dimension,
    },
    tile_scene::{
        paint_stack,
        paint_tile,
        read::drawn_runs,
        tile_items,
    },
};

/// The painter itself refuses a tile past its limit with a diagnostic,
/// never a panic;
///  the window never asks for one,
/// because it clips a tile of a very tall line (`Viewport::clip`).
///  The limit is four textures of the device's texture size and Vello's bin count (`stack_rows`):
/// at a text width of 782 px on a device of 8192 that is 32,768 pixel rows,
/// about 1,365 rows at scale 1 (about 117,000 characters in one line) and about 683 rows at scale 2.
#[test]
fn a_tile_past_the_stack_limit_is_refused_not_painted() {
    // 15,000 characters at 100 px, scale 3: 1,350 rows of 72 physical pixels, 300 px wide, 117 MB.
    let document = Document::new(&"x".repeat(15_000));
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    let mut view = shaper.prepare_rows(
        &document,
        Viewport {
            first: 0,
            count: 1,
            width: 100.0,
            scale: 3.0,
            wrap: Some(100.0),
            clip: None,
        },
        &[],
        &plain,
    );
    assert!(view.rows[0].visual_rows() > 1000);
    // The shaper cuts the frame it prepares to the limit, so the painter is handed an oversized one directly.
    assert!(
        view.viewport.clip.is_some(),
        "the shaper did not cut the oversized frame"
    );
    assert!(
        paint_stack(&view, PALETTE, 0.0).is_ok(),
        "the cut frame is within the limit and builds a stack"
    );
    // A frame that one texture cannot hold is not a tile, and is painted as a stack.
    assert!(
        paint_tile(&view, PALETTE, 0.0).is_err(),
        "a frame of several textures was painted as one"
    );
    view.height = stack_rows(u64::from(view.width)) as u32 + 1;
    let refused = paint_stack(&view, PALETTE, 0.0);
    let mut message = None;
    if let Some(error) = refused.err() {
        message = Some(format!("{error:#}"));
    }
    let refused_with_limit = match message.as_deref() {
        Some(text) => text.contains("tile stack limit"),
        None => false,
    };
    assert!(
        refused_with_limit,
        "the oversized tile was not refused with the stack limit: {message:?}"
    );
    // The device's texture size bounds the width as well, whatever the rows.
    view.height = 4;
    view.width = texture_dimension() + 1;
    assert!(
        paint_stack(&view, PALETTE, 0.0).is_err(),
        "a tile wider than the device's texture size was accepted"
    );
}

/// A clipped tile's raster holds exactly the pixels its band has in the unclipped raster of the same lines,
///  its origin and height follow the clip,
/// and a clip changes nothing about the rows' places.
#[test]
fn a_clipped_tile_holds_the_pixels_of_its_band() {
    let document = Document::new(&format!("short\n{}", words(9000)));
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(2);
    let whole = shaper.prepare_rows(&document, viewport(1, 1, 300.0), &[], &plain);
    let rows = whole.rows[0].visual_rows();
    assert!(rows > 40, "the fixture wraps to many rows: {rows}");
    let band = Clip {
        top: 10.0 * CODE_ROW,
        bottom: 30.0 * CODE_ROW,
    };
    let clipped = shaper.prepare_rows(
        &document,
        Viewport {
            clip: Some(band),
            ..viewport(1, 1, 300.0)
        },
        &[],
        &plain,
    );
    assert_eq!(clipped.origin, CODE_ROW + band.top);
    assert_eq!(clipped.height as f32, band.bottom - band.top);
    assert_eq!(clipped.width, whole.width);
    assert_eq!(
        clipped.rows[0].top, whole.rows[0].top,
        "a clip moves no row"
    );
    // The clip draws the runs of its band of the whole tile, at the same place relative to the band.
    let whole_items = tile_items(&whole, PALETTE, 0.0).expect("whole items");
    let part_items = tile_items(&clipped, PALETTE, 0.0).expect("clipped items");
    let part_runs = drawn_runs(&part_items);
    assert!(!part_runs.is_empty(), "the band has ink");
    let whole_runs = drawn_runs(&whole_items);
    for run in &part_runs {
        let in_whole = whole_runs.iter().any(|candidate| {
            return candidate.ids == run.ids
                && candidate.ink == run.ink
                && (candidate.baseline - band.top - run.baseline).abs() < 0.01
                && (candidate.left - run.left).abs() < 0.01;
        });
        assert!(
            in_whole,
            "a run of the clipped tile is not in its band of the whole tile"
        );
    }
    let scene = paint_tile(&clipped, PALETTE, 0.0).expect("clipped scene");
    assert_eq!((scene.width, scene.height), (clipped.width, clipped.height));
    // A rebase keeps the clip's distance below the first line's block.
    let mut moved = shaper.prepare_rows(
        &document,
        Viewport {
            clip: Some(band),
            ..viewport(1, 1, 300.0)
        },
        &[],
        &plain,
    );
    let lowered = RowMap::new(2, &[(0, 26.0)]);
    assert!(moved.rebase(&lowered));
    assert_eq!(moved.origin, 26.0 + CODE_ROW + band.top);
}

/// At the real limit a line that no raster can hold is cut by the shaper and painted in a clip,
/// at both display scales:
///  the band's raster is within the limit,
/// and every code row of it has ink.
#[test]
fn a_line_past_the_real_limit_paints_inside_a_clip() {
    let document = Document::new(&words(300_000));
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    for scale in [1.0_f32, 2.0] {
        let base = Viewport {
            scale,
            ..viewport(0, 1, 782.0)
        };
        let whole = shaper.prepare_rows(&document, base, &[], &plain);
        let rows = whole.rows[0].visual_rows();
        let columns = u64::from(whole.width);
        let limit_rows = stack_rows(columns);
        assert!(
            (rows as f32 * CODE_ROW * scale) as u64 > limit_rows,
            "scale {scale}: {rows} rows of the line fit one raster of {limit_rows} rows"
        );
        // The shaper cuts a line no raster can hold, so the frame it prepares for the whole line is within the limit.
        assert!(
            whole.viewport.clip.is_some(),
            "scale {scale}: the whole line was not cut"
        );
        assert!(
            stack_fits(columns, u64::from(whole.height)),
            "scale {scale}: the frame of the whole line passes the limit"
        );
        // The painter still refuses a frame past the limit that did not come through the shaper.
        let mut oversized = shaper.prepare_rows(&document, base, &[], &plain);
        oversized.height = limit_rows as u32 + 1;
        assert!(
            paint_stack(&oversized, PALETTE, 0.0).is_err(),
            "scale {scale}: a frame past the limit was painted"
        );
        // A band of 150 code rows from the middle of the line: well inside the limit at both scales.
        let top = (rows as f32 / 2.0).floor() * CODE_ROW;
        let band = Clip {
            top,
            bottom: top + 150.0 * CODE_ROW,
        };
        let clipped = shaper.prepare_rows(
            &document,
            Viewport {
                clip: Some(band),
                ..base
            },
            &[],
            &plain,
        );
        let scene = paint_tile(&clipped, PALETTE, 0.0).expect("the clipped band is painted");
        assert!(
            stack_fits(u64::from(scene.width), u64::from(scene.height))
                && scene.height <= texture_dimension(),
            "scale {scene_scale}: the clipped texture passes the limit",
            scene_scale = scale
        );
        assert_eq!(scene.height as f32, 150.0 * CODE_ROW * scale);
        let items = tile_items(&clipped, PALETTE, 0.0).expect("band items");
        let runs = drawn_runs(&items);
        for code_row in 0..150 {
            let (from, to) = (
                (code_row as f32) * CODE_ROW * scale,
                (code_row as f32 + 1.0) * CODE_ROW * scale,
            );
            assert!(
                runs.iter()
                    .any(|run| return run.baseline > from && run.baseline < to),
                "scale {scale}: code row {code_row} of the band draws no glyph"
            );
        }
    }
}
