//! Where a block and its hints stand:
//!  a block above the first row with its underlines on their own rows,
//!  and assembled hints where the broken line draws them.

use crate::fixtures::{
    CODE,
    COLORS,
    SHOWN,
    everywhere,
    hint,
    store,
    viewport,
};
use ide_app::{
    annotation::{
        Mark,
        Visible,
    },
    annotation_layout::lay_out,
    code_colors::CodeColors,
    document::Document,
    language::diagnostics::Severity,
    row_map::{
        CODE_ROW,
        RowMap,
    },
    shaped_text::TextShaper,
    tile_scene::{
        read::{
            center_y,
            drawn_strokes,
        },
        tile_items,
    },
};

/// With a block above a wrapped line,
/// the block ends where the line's first row begins,
/// the line's rows follow it
///  and fill the rows it was measured at,
/// the next line starts under the last row,
/// and an underline across the
///  wrap break is painted on exactly the two visual rows it crosses,
/// none of it in the block.
#[test]
fn a_block_sits_above_the_first_row_and_underlines_stay_on_their_rows() {
    let text = format!("{CODE}\nnext line\n");
    let (mut annotations, document) = store(&text, vec![hint(4, ": u32")], Vec::new());
    let mut shaper = TextShaper::new();
    let blocks = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(200.0),
        &everywhere(),
    );
    assert_eq!(blocks.len(), 1);
    let block_height = blocks[0].height();
    let plain = RowMap::new(3, &[(0, block_height)]);
    let view = shaper.prepare_rows(&document, viewport(2, 200.0), &[], &plain);
    let rows = view.rows[0].visual_rows();
    assert!(rows >= 3, "the fixture wraps at 200 px");
    let map = &view.map;
    assert_eq!(map.block_top(0), 0.0);
    assert_eq!(
        map.code_top(0),
        block_height,
        "the first row starts under the block"
    );
    assert_eq!(map.code_bottom(0), block_height + rows as f32 * CODE_ROW);
    assert_eq!(
        map.block_top(1),
        map.code_bottom(0),
        "the next line starts under the last visual row"
    );
    // A mark across the break between the first and second rows.
    let second = view.rows[0]
        .visual_row_sources(1)
        .expect("second row")
        .start;
    let mark = Mark {
        start: second - 3,
        end: second + 3,
        severity: Severity::Error,
    };
    let visible = Visible {
        blocks: blocks.clone(),
        marks: vec![mark],
    };
    let mut framed = shaper.prepare_rows(&document, viewport(2, 200.0), &[], &plain);
    let frame = lay_out(&framed, &visible, &mut shaper, COLORS);
    let mut offsets: Vec<f32> = Vec::new();
    for run in frame.underlines.iter() {
        offsets.push(run.offset);
    }
    offsets.sort_by(f32::total_cmp);
    assert_eq!(
        offsets,
        vec![0.0, CODE_ROW],
        "one run for each row the mark crosses"
    );
    framed.annotations = Some(frame);
    let items = tile_items(
        &framed,
        CodeColors {
            foreground: [0, 0, 0, 255],
            selected: [255, 255, 255, 255],
            dark: false,
        },
        0.0,
    )
    .expect("items");
    let mut bands = Vec::new();
    for drawn in drawn_strokes(&items) {
        assert_eq!(
            drawn.stroke.ink, COLORS.error,
            "the stroke is in the error ink"
        );
        bands.push(((center_y(&drawn.stroke) - block_height) / CODE_ROW).floor() as i32);
    }
    bands.sort_unstable();
    bands.dedup();
    assert_eq!(
        bands,
        vec![0, 1],
        "error ink stands on the two visual rows the mark crosses and nowhere in the block"
    );
}

/// The store packs a hint against the line as it is broken at the wrap width,
/// and packs again for another width.
#[test]
fn assembled_hints_stand_where_the_broken_line_draws_them() {
    let document = Document::new(CODE);
    let mut shaper = TextShaper::new();
    let wrapped = shaper.wrapped_row(&document, 0, 1.0, 300.0);
    let position = wrapped.visual_row_sources(1).expect("second row").start + 4;
    let (mut annotations, _) = store(CODE, vec![hint(position, ": u32")], Vec::new());
    let narrow = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(300.0),
        &everywhere(),
    );
    assert_eq!(narrow[0].hints[0].x, wrapped.caret_x(position, 1.0));
    let wide = shaper.wrapped_row(&document, 0, 1.0, 700.0);
    let assembled = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(700.0),
        &everywhere(),
    );
    assert_eq!(
        assembled[0].hints[0].x,
        wide.caret_x(position, 1.0),
        "another width packs the hint again"
    );
    assert_ne!(narrow[0].hints[0].x, assembled[0].hints[0].x);
}
