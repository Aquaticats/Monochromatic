//! Annotation pixels in the source tile,
//! as the GPU draws them:
//!  one underline style per severity in its ink,
//!  hints and messages on
//! virtual rows above their line and never in a code row,
//!  and inks that follow the scheme.
//! The scene of the shaped rows is rendered on the hardware device and read back.

/// The tile the GPU draws,
/// read back without a window,
/// and how far two renders may differ.
use super::tile_reads::{RENDER_NOISE, rendered_tile};
/// The production layout and packing,
///  underline constants,
///  vertical mapping,
///  shaper,
///  and palette.
use ide_app::{
    annotation::{Label, Mark, Visible},
    annotation_layout::{AnnotationColors, lay_out, pack},
    annotation_paint::DROP,
    code_colors::CodeColors,
    document::Document,
    language::diagnostics::Severity,
    row_map::{CODE_ROW, RowMap},
    shaped_text::{ShapedView, TextShaper, Viewport},
    virtual_row::{BLOCK_GAP, Block, MessageRow, ROW_HEIGHT},
};
/// The pixels the GPU drew.
use slint::{Rgba8Pixel, SharedPixelBuffer};
/// Blocks are shared between the window state and the frames that paint them.
use std::sync::Arc;

/// Dark-scheme annotation inks for the tests;
///  each differs from the source ink.
const DARK: AnnotationColors = AnnotationColors {
    hint: [150, 160, 170, 255],
    error: [255, 110, 120, 255],
    warning: [240, 200, 40, 255],
    information: [90, 200, 250, 255],
    suggestion: [170, 170, 170, 255],
};

/// Light-scheme annotation inks for the tests.
const LIGHT: AnnotationColors = AnnotationColors {
    hint: [90, 96, 104, 255],
    error: [190, 30, 30, 255],
    warning: [140, 90, 0, 255],
    information: [0, 90, 180, 255],
    suggestion: [100, 100, 100, 255],
};

/// What:
///  Shape `source`,
///  lay out `visible` with `inks`,
///  and paint the tile at scale one.
/// Why:
///  The pixels come from exactly the path the native renderer takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function painted(source: string, visible: Visible, inks: AnnotationColors): [ShapedView, Pixels];
/// ```
fn painted(
    source: &str,
    visible: &Visible,
    inks: AnnotationColors,
) -> (ShapedView, SharedPixelBuffer<Rgba8Pixel>) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: 2,
        width: 700.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    // Rows are placed by the vertical mapping the visible blocks call for.
    let mut raised = Vec::new();
    for block in &visible.blocks {
        raised.push((block.line, block.height()));
    }
    let map = RowMap::new(document.text().len_lines(), &raised);
    let mut view = shaper.prepare_rows(&document, viewport, &[], &map);
    let frame = lay_out(&view, visible, &mut shaper, inks);
    // `Some(frame)` attaches the positioned annotations to the frame the raster paints.
    view.annotations = Some(frame);
    let colors = CodeColors {
        foreground: [230, 230, 230, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
    let pixels = rendered_tile(&view, colors, 0.0);
    return (view, pixels);
}

/// What:
///  The block above line `line` of `source` with one hint `hint` (position and text) and one message row
///       per entry of `messages` (start,
///  severity,
///  text),
///  packed by the production shaper.
/// Why:
///  The paint tests need real placements,
///  not invented ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function block(source: string, line: number, hint: [number, string], messages: [number, Severity, string][]): Block;
/// ```
pub(super) fn block(
    source: &str,
    line: usize,
    hint: (usize, &str),
    messages: &[(usize, Severity, &str)],
) -> Arc<Block> {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let row = shaper.row(&document, line, 1.0);
    let labels = [Label {
        position: hint.0,
        text: hint.1.to_string(),
    }];
    let (hint_rows, hints) = pack(&row, &labels, &mut shaper, 1.0, None);
    let mut rows = Vec::new();
    for (start, severity, text) in messages {
        rows.push(MessageRow {
            start: *start,
            continued: false,
            severity: *severity,
            gap: None,
            text: (*text).to_string(),
        });
    }
    return Arc::new(Block {
        line,
        hint_rows,
        hints,
        messages: rows,
        held: (0.0, 0.0),
    });
}

/// How many pixels of rows `rows.0..rows.1` of the tile carry `ink`.
fn count(pixels: &SharedPixelBuffer<Rgba8Pixel>, rows: (usize, usize), ink: [u8; 4]) -> usize {
    let mut found = 0;
    for y in rows.0..rows.1 {
        for x in 0..pixels.width() as usize {
            if inked(pixels, x, y, ink) {
                found += 1;
            }
        }
    }
    return found;
}

/// What:
///  Whether the pixel at (`x`,
///  `y`) is mostly covered and has `ink`.
/// Why:
///  Partly covered edge pixels keep the ink's hue (the texture holds straight alpha);
///  the source ink is never close to an annotation ink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inked(pixels: Pixels, x: number, y: number, ink: Rgba): boolean;
/// ```
fn inked(pixels: &SharedPixelBuffer<Rgba8Pixel>, x: usize, y: usize, ink: [u8; 4]) -> bool {
    let pixel = pixels.as_slice()[y * pixels.width() as usize + x];
    if pixel.a < 96 {
        return false;
    }
    for (found, expected) in [pixel.r, pixel.g, pixel.b].iter().zip(ink.iter()) {
        if found.abs_diff(*expected) > 16 {
            return false;
        }
    }
    return true;
}

/// What:
///  For each pixel column of `left..right`,
///  the mean row of its `ink` pixels near the underline,
///       or nothing when the column has none;
///  `Vec<Option<f32>>` is a list of maybe-numbers.
/// Why:
///  Gaps show a broken line style;
///  varying rows show a wave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function columns(view: ShapedView, pixels: Pixels, left: number, right: number, ink: Rgba): (number | undefined)[];
/// ```
fn columns(
    view: &ShapedView,
    pixels: &SharedPixelBuffer<Rgba8Pixel>,
    span: (usize, usize),
    ink: [u8; 4],
) -> Vec<Option<f32>> {
    let center = view.rows[0].baseline + DROP;
    // Rows below the baseline only: capital letters end there, so selected glyph ink is never counted.
    let top = view.rows[0].baseline.ceil() as usize;
    let bottom = ((center + 5.0) as usize).min(23);
    let mut result = Vec::new();
    for x in span.0..span.1 {
        let mut sum = 0.0;
        let mut count = 0.0;
        for y in top..=bottom {
            if inked(pixels, x, y, ink) {
                sum += y as f32;
                count += 1.0;
            }
        }
        // `Some` holds the mean row of a column with underline pixels; `None` marks a gap.
        if count > 0.0 {
            result.push(Some(sum / count));
        } else {
            result.push(None);
        }
    }
    return result;
}

/// What:
///  The weakest column of an underline over `span`:
///  the largest alpha any pixel of the column below the baseline reaches,
///  smallest over the columns.
/// Why:
///  A thin wave that straddles two pixel rows is pale in both,
///  as the first GPU wave was (half the ink of the CPU wave);
///  the strongest pixel of every column must carry most of the ink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function weakestColumn(view: ShapedView, pixels: Pixels, span: [number, number]): number;
/// ```
fn weakest_column(
    view: &ShapedView,
    pixels: &SharedPixelBuffer<Rgba8Pixel>,
    span: (usize, usize),
) -> u8 {
    let top = view.rows[0].baseline.ceil() as usize;
    let bottom = ((view.rows[0].baseline + DROP + 5.0) as usize).min(23);
    let mut weakest = u8::MAX;
    for x in span.0..span.1 {
        let mut strongest = 0;
        for y in top..=bottom {
            strongest = strongest.max(pixels.as_slice()[y * pixels.width() as usize + x].a);
        }
        weakest = weakest.min(strongest);
    }
    return weakest;
}

/// An error wave keeps most of its ink in the strongest pixel of every column,
///  not half of it in each of two rows,
///  because a pale wave on dark text reads as a smudge.
#[test]
fn an_error_wave_is_not_paler_than_the_line_it_replaced() {
    let source = "ABCDEFHIKLMNOTUVWXZABCDEFHIKLMNOTUVWXZ";
    let length = source.chars().count();
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![Mark {
            start: 0,
            end: length,
            severity: Severity::Error,
        }],
    };
    let (view, pixels) = painted(source, &visible, DARK);
    let right = view.rows[0].caret_x(length, 1.0) as usize;
    let weakest = weakest_column(&view, &pixels, (2, right - 2));
    println!("error wave: the weakest column peaks at alpha {weakest} of 255");
    assert!(weakest >= 178, "the wave peaks at alpha {weakest} of 255");
}

/// Errors are a continuous wave,
///  warnings flat dashes,
///  information flat dots,
///  and hint-severity sparse dots,
/// each in its own ink,
///  so severity is readable without color.
#[test]
fn each_severity_has_its_own_line_style_in_its_ink() {
    // Capital letters have no descenders, so only underline ink can appear below the baseline.
    let source = "ABCDEFHIKLMNOTUVWXZABCDEFHIKLMNOTUVWXZ";
    let length = source.chars().count();
    let mut found = Vec::new();
    for severity in [
        Severity::Error,
        Severity::Warning,
        Severity::Information,
        Severity::Hint,
    ] {
        let visible = Visible {
            blocks: Vec::new(),
            marks: vec![Mark {
                start: 0,
                end: length,
                severity,
            }],
        };
        let (view, pixels) = painted(source, &visible, DARK);
        let right = view.rows[0].caret_x(length, 1.0) as usize;
        // Columns two pixels inside each end avoid partial coverage at the run's edges.
        let measured = columns(&view, &pixels, (2, right - 2), DARK.severity(severity));
        let lit: Vec<f32> = measured.iter().flatten().copied().collect();
        let fraction = lit.len() as f32 / measured.len() as f32;
        let lowest = lit.iter().copied().fold(f32::MAX, f32::min);
        let highest = lit.iter().copied().fold(f32::MIN, f32::max);
        let mut longest = 0;
        let mut current = 0;
        for column in &measured {
            if column.is_some() {
                current += 1;
                longest = longest.max(current);
            } else {
                current = 0;
            }
        }
        println!(
            "underline {severity:?}: {:.2} of columns lit, rows {lowest:.2} to {highest:.2}, longest run {longest}",
            fraction
        );
        found.push((severity, fraction, highest - lowest, longest));
    }
    let (_, error_lit, error_spread, _) = found[0];
    let (_, warning_lit, warning_spread, warning_run) = found[1];
    let (_, information_lit, information_spread, information_run) = found[2];
    let (_, hint_lit, hint_spread, _) = found[3];
    assert!(
        error_lit > 0.95 && error_spread >= 0.75,
        "the error wave is missing"
    );
    assert!(
        warning_lit > 0.5 && warning_lit < 0.85 && warning_spread < 1.0,
        "warning dashes"
    );
    assert!(
        information_lit > 0.3 && information_lit < 0.7 && information_spread < 1.0,
        "information dots"
    );
    assert!(
        information_run < warning_run,
        "information dots are as long as warning dashes"
    );
    assert!(
        hint_lit > 0.1 && hint_lit < 0.4 && hint_spread < 1.0,
        "hint dots"
    );
    assert!(
        information_lit > hint_lit + 0.1,
        "hint dots are as dense as information dots"
    );
}

/// Hints and messages are painted on their own rows above the code row,
///  each in its ink;
///  the gap above the
/// block stays empty;
///  and the code rows carry exactly the pixels they carry without annotations,
///  so nothing
/// that annotates a line is drawn on the line itself or after its end.
#[test]
fn virtual_rows_paint_above_the_code_row_and_never_in_it() {
    let source = "let total = area(2, 3);\nnext";
    let shown = block(
        source,
        0,
        (9, ": u32"),
        &[
            (12, Severity::Error, "Error E0308 (rustc): mismatched types"),
            (4, Severity::Information, "Information: a remark"),
        ],
    );
    let height = shown.height() as usize;
    assert_eq!(shown.height(), BLOCK_GAP + 3.0 * ROW_HEIGHT);
    let annotated = Visible {
        blocks: vec![Arc::clone(&shown)],
        marks: Vec::new(),
    };
    let (view, pixels) = painted(source, &annotated, DARK);
    let (_, bare) = painted(source, &Visible::default(), DARK);
    assert_eq!(view.rows[0].top, shown.height());
    assert_eq!(pixels.height() as usize, bare.height() as usize + height);
    let gap = BLOCK_GAP as usize;
    let row = ROW_HEIGHT as usize;
    // The gap above the block is empty.
    for y in 0..gap {
        for x in 0..pixels.width() as usize {
            let pixel = pixels.as_slice()[y * pixels.width() as usize + x];
            assert_eq!(pixel.a, 0, "ink in the gap at {x},{y}");
        }
    }
    // One ink per row: the hint row, then the error row, then the information row.
    let bands = [
        (DARK.hint, gap, gap + row),
        (DARK.error, gap + row, gap + 2 * row),
        (DARK.information, gap + 2 * row, gap + 3 * row),
    ];
    for (ink, top, bottom) in bands {
        let own = count(&pixels, (top, bottom), ink);
        assert!(own > 20, "row {top}..{bottom} has {own} pixels of its ink");
        for (other, _, _) in bands {
            if other != ink {
                assert_eq!(
                    count(&pixels, (top, bottom), other),
                    0,
                    "row {top}..{bottom} carries another row's ink"
                );
            }
        }
    }
    // The code rows are the rows of the text without annotations, moved down by the block.
    // Two renders of one scene may differ by one level in a few texels, and no more.
    let width = pixels.width() as usize;
    let rows = 2 * CODE_ROW as usize;
    for y in 0..rows {
        let with = (y + height) * width;
        let without = y * width;
        for x in 0..width {
            let (a, b) = (pixels.as_slice()[with + x], bare.as_slice()[without + x]);
            let premultiplied =
                |value: u8, alpha: u8| return (u32::from(value) * u32::from(alpha) / 255) as u8;
            let level = premultiplied(a.r, a.a)
                .abs_diff(premultiplied(b.r, b.a))
                .max(premultiplied(a.g, a.a).abs_diff(premultiplied(b.g, b.a)))
                .max(premultiplied(a.b, a.a).abs_diff(premultiplied(b.b, b.a)))
                .max(a.a.abs_diff(b.a));
            assert!(
                level <= RENDER_NOISE,
                "code row pixels differ by {level} at {x},{y} in tile row {y}"
            );
        }
    }
    // The texts start where their positions are: the hint above character 9, the error above character 12.
    let frame = view.annotations.as_ref().expect("annotation frame");
    assert_eq!(frame.texts[0].x, view.rows[0].caret_x(9, 1.0));
    assert_eq!(frame.texts[1].x, view.rows[0].caret_x(12, 1.0));
    let hint_left = frame.texts[0].x as usize;
    for y in gap..gap + row {
        for x in 0..hint_left.saturating_sub(1) {
            assert!(
                !inked(&pixels, x, y, DARK.hint),
                "hint ink left of its position at {x},{y}"
            );
        }
    }
}

/// Inside a selection the underline keeps its line style but takes the selected-text ink,
///  as selected glyphs do;
/// outside the selection it keeps the severity ink.
#[test]
fn selected_underlines_take_the_selected_ink() {
    let source = "ABCDEFHIKLMNOTUVWXZ";
    let mut document = Document::new(source);
    // `ReadingPosition` selects the first nine characters, as a drag or Shift+Right would.
    document.select(ide_app::document::ReadingPosition {
        anchor: 0,
        head: 9,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: 2,
        width: 700.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    let mut view = shaper.prepare(&document, viewport, &[]);
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![Mark {
            start: 0,
            end: 19,
            severity: Severity::Error,
        }],
    };
    let frame = lay_out(&view, &visible, &mut shaper, LIGHT);
    view.annotations = Some(frame);
    let selected = [255, 255, 255, 255];
    let colors = CodeColors {
        foreground: [30, 30, 30, 255],
        selected,
        dark: false,
    };
    let pixels = rendered_tile(&view, colors, 0.0);
    let boundary = view.rows[0].caret_x(9, 1.0) as usize;
    let end = view.rows[0].caret_x(19, 1.0) as usize;
    let inside = columns(&view, &pixels, (2, boundary - 2), selected);
    let inside_severity = columns(&view, &pixels, (2, boundary - 2), LIGHT.error);
    let outside = columns(&view, &pixels, (boundary + 2, end - 2), LIGHT.error);
    println!(
        "selected underline: {} selected-ink columns and {} error-ink columns inside, {} error-ink columns outside",
        lit(&inside),
        lit(&inside_severity),
        lit(&outside)
    );
    assert!(
        lit(&inside) > 40,
        "the selected underline is not in the selected ink"
    );
    assert_eq!(
        lit(&inside_severity),
        0,
        "the severity ink shows on the selection fill"
    );
    assert!(
        lit(&outside) > 40,
        "the unselected underline lost its severity ink"
    );
}

/// The same annotations painted with the light inks contain the light inks and none of the dark ones,
///  and back.
#[test]
fn annotation_inks_follow_the_scheme() {
    let source = "ABCDEFHIK";
    let visible = Visible {
        blocks: vec![block(
            source,
            0,
            (3, "hint"),
            &[(0, Severity::Error, "Error: wrong")],
        )],
        marks: vec![Mark {
            start: 0,
            end: 9,
            severity: Severity::Error,
        }],
    };
    for (inks, other) in [(DARK, LIGHT), (LIGHT, DARK)] {
        let (_, pixels) = painted(source, &visible, inks);
        let all = (0, pixels.height() as usize);
        let own_error = count(&pixels, all, inks.error);
        let own_hint = count(&pixels, all, inks.hint);
        let foreign = count(&pixels, all, other.error) + count(&pixels, all, other.hint);
        assert!(
            own_error > 40 && own_hint > 20 && foreign == 0,
            "own error {own_error}, own hint {own_hint}, foreign {foreign}"
        );
    }
}

/// How many columns of a probed list hold ink.
fn lit(list: &[Option<f32>]) -> usize {
    let mut count = 0;
    for column in list {
        if column.is_some() {
            count += 1;
        }
    }
    return count;
}
