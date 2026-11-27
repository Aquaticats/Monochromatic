//! The clip and the cap a frame must respect:
//!  a clip made for the cap fits the stack at every width and scale,
//!  the cap holds the view of the tallest window,
//!  the guard cuts a larger frame,
//!  and a clip asked for past the limit is cut.

/// The display scales,
///  raster widths,
///  and asked heights the sweep covers.
use crate::{
    COLUMNS,
    SCALES,
    WANTED,
    colors,
};
/// Canonical source of the lines under test.
use ide_app::document::Document;
/// The vertical mapping the frames are placed by,
/// and the height of one code row.
use ide_app::row_map::{
    CODE_ROW,
    RowMap,
};
/// The shaper,
/// the viewport,
/// and the rules under test.
use ide_app::shaped_text::{
    TextShaper,
    Viewport,
    band_clip,
    fit_clip,
    set_texture_dimension,
    stack_cap,
    stack_fits,
    texture_fits,
    tile_cap,
};
/// The tile's paint commands.
use ide_app::tile_scene::{
    Item,
    paint_stack,
    tile_items,
};

/// The cap is never under one code row,
/// never over the stack's bound,
/// and a clip made for it fits the stack,
/// for every width,
/// scale,
/// and asked height,
/// including the widths where one code row is all the cap allows.
#[test]
fn a_clip_made_for_the_cap_fits_the_stack_at_every_width_and_scale() {
    for scale in SCALES {
        for columns in COLUMNS {
            for wanted in WANTED {
                let cap = tile_cap(columns, scale, wanted);
                assert!(
                    cap >= CODE_ROW,
                    "columns {columns} scale {scale} wanted {wanted}: cap {cap}"
                );
                assert!(
                    cap <= stack_cap(columns, scale),
                    "columns {columns} scale {scale} wanted {wanted}: cap {cap} is over the stack's"
                );
                // Sums stay exact in single precision at ten million pixels, which is above every cap but the narrowest.
                let Some(clip) = band_clip(0.0, 10_000_000.0, (5_000_000.0, 5_010_000.0), cap)
                else {
                    assert!(
                        stack_fits(columns, (10_000_000.0_f32 * scale).ceil() as u64),
                        "columns {columns} scale {scale}: ten million pixels were let through and do not fit"
                    );
                    continue;
                };
                let rows = ((clip.bottom - clip.top).max(CODE_ROW) * scale).ceil() as u64;
                // A width where even one code row is past the limit has nothing to cut to.
                let one_row = (CODE_ROW * scale).ceil() as u64;
                if stack_fits(columns, one_row) {
                    assert!(
                        stack_fits(columns, rows),
                        "columns {columns} scale {scale}: a clip of {rows} rows passes the limit"
                    );
                }
            }
        }
    }
}

/// A window the device can show always has its view held by the cap,
/// at every scale and width the device allows:
/// the tile never goes under the view and the room that the re-center keeps clear.
#[test]
fn the_cap_holds_the_view_of_the_tallest_window_the_device_can_show() {
    for side in [8192_u32, 16_384] {
        set_texture_dimension(side);
        for scale in [1.0_f32, 1.5, 2.0, 3.0] {
            for columns in [200_u64, 1358, 3840, 7259, 8192, 16_384] {
                if columns > u64::from(side) {
                    continue;
                }
                let view = side as f32 / scale;
                let cap = tile_cap(columns, scale, view * 3.0);
                assert!(
                    cap >= view + 4.0 * CODE_ROW,
                    "side {side} scale {scale} columns {columns}: cap {cap} does not hold a view of {view}"
                );
            }
        }
    }
    set_texture_dimension(8192);
}

/// The guard leaves a frame that fits as it is,
/// asked clip included,
/// and cuts one that does not around its middle,
/// to a clip that fits,
/// inside the lines,
/// on whole code rows.
#[test]
fn the_guard_keeps_a_fitting_frame_and_cuts_a_larger_one_around_its_middle() {
    for scale in SCALES {
        for columns in COLUMNS {
            if !stack_fits(columns, (CODE_ROW * scale).ceil() as u64 * 2) {
                continue;
            }
            let cap = stack_cap(columns, scale);
            let first_top = 480.0;
            for lines in [cap * 0.3, cap * 0.99, cap * 1.5, cap * 7.3] {
                let bottom = first_top + (lines / CODE_ROW).ceil() * CODE_ROW;
                let fitted = fit_clip(first_top, bottom, None, columns, scale);
                let whole = ((bottom - first_top) * scale).ceil() as u64;
                match fitted {
                    None => assert!(
                        stack_fits(columns, whole),
                        "columns {columns} scale {scale}: {lines} px passed as they are"
                    ),
                    Some(clip) => {
                        assert!(!stack_fits(columns, whole), "a fitting frame was cut");
                        let rows = ((clip.bottom - clip.top) * scale).ceil() as u64;
                        assert!(stack_fits(columns, rows), "the cut frame passes the limit");
                        assert!(clip.top >= 0.0 && clip.bottom <= bottom - first_top);
                        assert_eq!(clip.top % CODE_ROW, 0.0);
                        assert_eq!(clip.bottom % CODE_ROW, 0.0);
                        let middle = (bottom - first_top) / 2.0;
                        assert!(
                            clip.top <= middle && middle <= clip.bottom,
                            "columns {columns} scale {scale}: the cut [{}, {}] misses the middle {middle}",
                            clip.top,
                            clip.bottom
                        );
                    }
                }
            }
            // A clip that fits is kept, and one that does not is cut around its own middle.
            let asked = band_clip(
                first_top,
                first_top + cap * 6.0,
                (first_top, first_top + cap * 6.0),
                cap,
            );
            let kept = fit_clip(first_top, first_top + cap * 6.0, asked, columns, scale);
            assert_eq!(kept, asked, "a clip that fits was changed");
        }
    }
}

/// Lines that wrap to many rows,
/// shaped against a map that counts each of them as one row,
/// as the window's map does before its background pass reached them:
/// the frame is cut to the limit and paints,
/// where the clip the caller asked for (none) would have been refused.
#[test]
fn a_frame_taller_than_its_map_held_is_cut_to_the_limit_and_painted() {
    let line = "word ".repeat(60);
    let text = format!("{}\n", line.trim_end()).repeat(300);
    let document = Document::new(&text);
    let lines = document.text().len_lines();
    let map = RowMap::plain(lines);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: lines,
        width: 224.0,
        scale: 1.0,
        wrap: Some(200.0),
        clip: None,
    };
    let view = shaper.prepare_rows(&document, viewport, &[], &map);
    let whole = view.map.block_top(lines.min(300));
    assert!(
        !stack_fits(u64::from(view.width), whole as u64),
        "the fixture's lines ({whole} px at {} columns) are within the stack",
        view.width
    );
    assert!(
        stack_fits(u64::from(view.width), u64::from(view.height)),
        "the frame is {}x{}, past the limit",
        view.width,
        view.height
    );
    let clip = view.viewport.clip.expect("the frame was cut");
    assert_eq!(view.origin, clip.top);
    assert_eq!(view.height as f32, clip.bottom - clip.top);
    let items = tile_items(&view, colors(), 0.0).expect("a frame within the limit draws");
    let mut glyph_runs = 0;
    for item in &items {
        if let Item::Glyphs(run) = item
            && !run.glyphs.is_empty()
        {
            glyph_runs += 1;
        }
    }
    assert!(glyph_runs > 0, "the cut frame draws no glyph");
    let stack = paint_stack(&view, colors(), 0.0).expect("a frame within the limit builds a stack");
    assert_eq!((stack.width, stack.height), (view.width, view.height));
    let mut together = 0;
    for texture in &stack.textures {
        assert!(texture_fits(
            u64::from(texture.width),
            u64::from(texture.height)
        ));
        together += texture.height;
    }
    assert_eq!(together, view.height);
}

/// A caller's clip that is itself too tall is cut too,
/// and a clip that fits is left alone.
#[test]
fn a_clip_asked_for_that_passes_the_limit_is_cut_and_one_that_fits_is_kept() {
    let line = "word ".repeat(60);
    let text = format!("{}\n", line.trim_end()).repeat(300);
    let document = Document::new(&text);
    let lines = document.text().len_lines();
    let map = RowMap::plain(lines);
    let mut shaper = TextShaper::new();
    let mut viewport = Viewport {
        first: 0,
        count: lines,
        width: 224.0,
        scale: 1.0,
        wrap: Some(200.0),
        clip: Some(ide_app::shaped_text::Clip {
            top: 0.0,
            bottom: 90_000.0,
        }),
    };
    let tall = shaper.prepare_rows(&document, viewport, &[], &map);
    let cut = tall.viewport.clip.expect("a clip");
    assert!(cut.bottom - cut.top < 90_000.0);
    assert!(stack_fits(u64::from(tall.width), u64::from(tall.height)));
    let small = ide_app::shaped_text::Clip {
        top: 240.0,
        bottom: 1_200.0,
    };
    viewport.clip = Some(small);
    let kept = shaper.prepare_rows(&document, viewport, &[], &map);
    assert_eq!(kept.viewport.clip, Some(small));
}
