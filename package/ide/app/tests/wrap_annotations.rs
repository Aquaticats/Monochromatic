//! Hint rows and message rows on a source view that wraps:
//!  hints are packed against the line as it is broken and held inside the text area,
//!  a label wider than the text area breaks into rows,
//!  message rows break to the text area and keep their row cap,
//!  and the block sits above the first visual row of its line with underlines still on their own rows.
//! No window is opened.

/// The store under test,
/// the marks that become underlines,
/// and the records the store takes.
use ide_app::{
    annotation::{Annotations, Mark, Visible},
    annotation_layout::{AnnotationColors, lay_out, pack},
    code_colors::CodeColors,
    document::Document,
    language::{
        diagnostics::{Diagnostic, DiagnosticsSnapshot, Freshness, Severity, SourceGroup},
        hints::{HintKind, HintsSnapshot, InlayHint},
        identity::{DocumentStamp, ServerIdentity},
    },
    row_map::{CODE_ROW, RowMap},
    shaped_text::{TextShaper, Viewport},
    tile_scene::{
        read::{center_y, drawn_strokes},
        tile_items,
    },
    virtual_row::{BLOCK_GAP, Block, CONTINUATION_INDENT, MESSAGE_ROWS, ROW_HEIGHT},
};
/// The pointer snapshots travel in,
/// and the range of lines near the view.
use std::{ops::Range, sync::Arc};

/// A line that wraps to several rows at the widths used here,
/// with a clear place for hints on its later rows.
const CODE: &str = "let value = compute_total(first_item, second_item); // a long comment that keeps going and going";

/// The stamp of the displayed text.
const SHOWN: DocumentStamp = DocumentStamp {
    file: 1,
    revision: 1,
};

/// Palette of the painted checks;
/// the hint ink is dark gray so it cannot be taken for the error ink.
const COLORS: AnnotationColors = AnnotationColors {
    hint: [20, 20, 20, 255],
    error: [200, 0, 0, 255],
    warning: [200, 100, 0, 255],
    information: [0, 0, 200, 255],
    suggestion: [90, 90, 90, 255],
};

/// The character offset where line `line` of `text` starts.
fn start_of(text: &str, line: usize) -> usize {
    return Document::new(text).text().line_to_char(line);
}

/// The block that stands above `line`.
fn block_at(blocks: &[Arc<Block>], line: usize) -> Arc<Block> {
    for block in blocks {
        if block.line == line {
            return Arc::clone(block);
        }
    }
    panic!("block");
}

/// The words of all message rows,
///  in order.
fn words(rows: &[ide_app::virtual_row::MessageRow]) -> Vec<String> {
    let mut texts: Vec<&str> = Vec::new();
    for row in rows {
        texts.push(row.text.as_str());
    }
    let mut found: Vec<String> = Vec::new();
    for word in texts.join(" ").split_whitespace() {
        found.push(word.to_string());
    }
    return found;
}

/// The process every record names.
fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted".to_string(),
        instance: 1,
    };
}

/// One hint at `position`.
fn hint(position: usize, label: &str) -> InlayHint {
    return InlayHint {
        position,
        label: label.to_string(),
        kind: Some(HintKind::Type),
        padding_left: false,
        padding_right: false,
        server: server(),
    };
}

/// One error over `start..end`.
fn error(start: usize, end: usize, message: &str) -> Diagnostic {
    return Diagnostic {
        start,
        end,
        severity: Some(Severity::Error),
        code: None,
        message: message.to_string(),
        server: server(),
        freshness: Freshness::Versioned,
    };
}

/// A store holding `hints` and `diagnostics` for the shown text.
fn store(
    text: &str,
    hints: Vec<InlayHint>,
    diagnostics: Vec<Diagnostic>,
) -> (Annotations, Document) {
    let document = Document::new(text);
    let mut annotations = Annotations::default();
    let lines = document.text().len_lines();
    let snapshot = HintsSnapshot {
        stamp: SHOWN,
        first_line: 0,
        last_line: lines,
        hints,
    };
    let group = DiagnosticsSnapshot {
        stamp: SHOWN,
        groups: vec![SourceGroup {
            source: "scripted".to_string(),
            items: diagnostics,
        }],
    };
    annotations.replace(
        SHOWN,
        document.text(),
        Some(Arc::new(snapshot)),
        Some(Arc::new(group)),
    );
    return (annotations, document);
}

/// Every line is near the view.
fn everywhere() -> Range<usize> {
    return 0..usize::MAX;
}

/// A viewport of `count` lines wrapped at `wrap`.
fn viewport(count: usize, wrap: f32) -> Viewport {
    return Viewport {
        first: 0,
        count,
        width: wrap + 24.0,
        scale: 1.0,
        wrap: Some(wrap),
        clip: None,
    };
}

/// A hint on the second visual row of a wrapped line is packed at the x that row draws it,
/// not at the x of the
///  unbroken line,
/// and the frame paints it where the packing put it.
#[test]
fn hints_are_packed_against_the_line_as_it_is_broken() {
    let document = Document::new(CODE);
    let mut shaper = TextShaper::new();
    let wrapped = shaper.wrapped_row(&document, 0, 1.0, 300.0);
    assert!(wrapped.visual_rows() >= 3, "the fixture wraps at 300 px");
    let position = wrapped.visual_row_sources(1).expect("second row").start + 4;
    let label = ide_app::annotation::Label {
        position,
        text: ": u32".to_string(),
    };
    let (rows, places) = pack(
        &wrapped,
        std::slice::from_ref(&label),
        &mut shaper,
        1.0,
        Some(300.0),
    );
    let unbroken = shaper.row(&document, 0, 1.0).caret_x(position, 1.0);
    assert_eq!(rows, 1);
    assert_eq!(places[0].x, wrapped.caret_x(position, 1.0));
    assert!(
        (places[0].x - unbroken).abs() > 20.0,
        "the unbroken line puts the character at {unbroken}, the broken line at {}",
        places[0].x
    );
    // The frame paints the hint at the packed x.
    let block = Arc::new(Block {
        line: 0,
        hint_rows: rows,
        hints: places,
        messages: Vec::new(),
        held: (0.0, 0.0),
    });
    let map = RowMap::new(1, &[(0, block.height())]);
    let view = shaper.prepare_rows(&document, viewport(1, 300.0), &[], &map);
    let visible = Visible {
        blocks: vec![Arc::clone(&block)],
        marks: Vec::new(),
    };
    let frame = lay_out(&view, &visible, &mut shaper, COLORS);
    assert_eq!(frame.texts.len(), 1);
    assert_eq!(frame.texts[0].x, block.hints[0].x);
}

/// A hint at the end of a long line would pass the right edge of the text area;
/// it ends at the edge instead,
///  and a hint that fits stays exactly at its character.
#[test]
fn a_hint_that_would_pass_the_text_area_ends_at_its_edge() {
    let text = "x".repeat(60);
    let document = Document::new(&text);
    let mut shaper = TextShaper::new();
    // 60 characters of 9 px (540 px) in a 560 px text area leave no room for a label at the line end.
    let row = shaper.wrapped_row(&document, 0, 1.0, 560.0);
    assert_eq!(row.visual_rows(), 1);
    let labels = [
        ide_app::annotation::Label {
            position: 2,
            text: ": a".to_string(),
        },
        ide_app::annotation::Label {
            position: 60,
            text: ": a_label_that_does_not_fit".to_string(),
        },
    ];
    let (_, places) = pack(&row, &labels, &mut shaper, 1.0, Some(560.0));
    assert_eq!(
        places[0].x,
        row.caret_x(2, 1.0),
        "a hint that fits stays at its character"
    );
    assert!(
        (places[1].x + places[1].width - 560.0).abs() < 0.01,
        "the last hint ends at the right edge: starts {} width {}",
        places[1].x,
        places[1].width
    );
    assert!(places[1].x < row.caret_x(60, 1.0));
}

/// A label wider than the text area breaks into pieces on rows of their own,
/// the later pieces at the left edge,
///  and no piece is wider than the text area.
#[test]
fn a_label_wider_than_the_text_area_breaks_into_rows() {
    let document = Document::new("let a = 1;");
    let mut shaper = TextShaper::new();
    let row = shaper.wrapped_row(&document, 0, 1.0, 120.0);
    let label = ide_app::annotation::Label {
        position: 4,
        text: ": HashMap<String, Vec<(usize, usize)>> with a long tail of words".to_string(),
    };
    let (rows, places) = pack(&row, &[label], &mut shaper, 1.0, Some(120.0));
    assert!(rows >= 3, "the label needs several rows: {rows}");
    assert_eq!(rows, places.len());
    for (index, place) in places.iter().enumerate() {
        assert_eq!(place.row, index);
        assert!(
            place.x + place.width <= 120.01,
            "piece {index} passes the text area: x {} width {}",
            place.x,
            place.width
        );
        if index > 0 {
            assert_eq!(place.x, 0.0, "later pieces start at the left edge");
        }
    }
    let mut joined: Vec<&str> = Vec::new();
    for place in places.iter() {
        joined.push(place.text.as_str());
    }
    assert_eq!(
        joined
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(""),
        ": HashMap<String, Vec<(usize, usize)>> with a long tail of words"
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(""),
        "the pieces hold all of the label"
    );
}

/// Message rows break to the text area:
/// the same message takes more rows at a narrow width,
/// the block is as tall as
///  its rows,
/// a wide text area changes nothing,
/// and no row is wider than the area.
#[test]
fn message_rows_break_to_the_text_area() {
    let message =
        "expected `u32`, found `&str` because the argument named first_item is a string slice here";
    let (mut annotations, document) = store(CODE, Vec::new(), vec![error(4, 9, message)]);
    let mut shaper = TextShaper::new();
    let plain = annotations.assemble(SHOWN, &document, &mut shaper, 1.0, None, &everywhere());
    let wide = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(1400.0),
        &everywhere(),
    );
    let narrow = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(240.0),
        &everywhere(),
    );
    assert_eq!(plain.len(), 1);
    assert_eq!(
        wide[0].messages, plain[0].messages,
        "a wide area breaks nothing more"
    );
    assert!(
        narrow[0].messages.len() > plain[0].messages.len(),
        "a narrow area takes more rows: {} against {}",
        narrow[0].messages.len(),
        plain[0].messages.len()
    );
    assert_eq!(
        narrow[0].height(),
        BLOCK_GAP + narrow[0].messages.len() as f32 * ROW_HEIGHT,
        "the block is as tall as its rows"
    );
    assert!(!narrow[0].messages[0].continued);
    for row in narrow[0].messages[1..].iter() {
        assert!(row.continued);
    }
    for row in &narrow[0].messages {
        let indent = if row.continued {
            CONTINUATION_INDENT
        } else {
            0.0
        };
        let width = shaper.row_layout(&row.text, 1.0).full_width();
        assert!(
            indent + width <= 240.5,
            "`{}` is {width} px wide after an indent of {indent}",
            row.text
        );
    }
    assert_eq!(
        words(&narrow[0].messages),
        words(&plain[0].messages),
        "no word is lost"
    );
}

/// A message that breaks into more than the cap at a narrow width keeps the cap and says how many rows it left out.
#[test]
fn a_message_keeps_its_row_cap_at_a_narrow_width() {
    let message = "word ".repeat(120);
    let (mut annotations, document) = store(CODE, Vec::new(), vec![error(4, 9, &message)]);
    let mut shaper = TextShaper::new();
    let narrow = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(90.0),
        &everywhere(),
    );
    let rows = &narrow[0].messages;
    assert_eq!(
        rows.len(),
        MESSAGE_ROWS,
        "the cap holds after the rows are broken"
    );
    assert!(
        rows.last().expect("last row").text.ends_with("more lines"),
        "the last row counts what is left out: {}",
        rows.last().expect("last row").text
    );
}

/// Message rows stand inside the text area in the frame:
/// a diagnostic far to the right moves its whole message left.
#[test]
fn a_message_far_to_the_right_moves_left_to_stay_inside_the_text_area() {
    let text = format!("{}tail", "y".repeat(50));
    let message = "expected something quite different here";
    let (mut annotations, document) = store(&text, Vec::new(), vec![error(50, 54, message)]);
    let mut shaper = TextShaper::new();
    let blocks = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(560.0),
        &everywhere(),
    );
    let map = RowMap::new(1, &[(0, blocks[0].height())]);
    let view = shaper.prepare_rows(&document, viewport(1, 560.0), &[], &map);
    let visible = Visible {
        blocks: blocks.clone(),
        marks: Vec::new(),
    };
    let frame = lay_out(&view, &visible, &mut shaper, COLORS);
    assert!(!frame.texts.is_empty());
    let own_x = view.rows[0].caret_x(50, 1.0);
    for shown in &frame.texts {
        assert!(
            shown.x + shown.width <= 560.01,
            "a message row passes the text area: x {} width {}",
            shown.x,
            shown.width
        );
    }
    assert!(
        frame.texts[0].x < own_x,
        "the message moved left of its diagnostic at {own_x}"
    );
    // Without a wrap width nothing moves.
    let unwrapped_view = shaper.prepare_rows(
        &document,
        Viewport {
            wrap: None,
            clip: None,
            ..viewport(1, 560.0)
        },
        &[],
        &map,
    );
    let unwrapped = lay_out(&unwrapped_view, &visible, &mut shaper, COLORS);
    assert_eq!(unwrapped.texts[0].x, own_x);
}

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

/// A new width packs the lines that have hints again and measures no message row twice;
///  the same width again does no shaping,
/// and only rows wider than the new width are broken by shaping.
#[test]
fn a_new_width_does_only_the_work_it_needs() {
    let mut text = String::new();
    for number in 0..40 {
        text.push_str(&format!("{CODE} {number}\n"));
    }
    let hints = vec![
        hint(start_of(&text, 3) + 10, ": a"),
        hint(start_of(&text, 9) + 10, ": b"),
    ];
    let diagnostics = vec![
        error(
            start_of(&text, 5) + 4,
            start_of(&text, 5) + 9,
            "short message",
        ),
        error(
            start_of(&text, 7) + 4,
            start_of(&text, 7) + 9,
            "expected `u32`, found `&str` because the argument named first_item is a string slice here",
        ),
    ];
    let (mut annotations, document) = store(&text, hints, diagnostics);
    let mut shaper = TextShaper::new();
    assert_eq!(annotations.work().packed, 0);
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(900.0),
        &everywhere(),
    );
    let first = annotations.work();
    assert_eq!(first.packed, 2, "one packing for each line that has hints");
    assert!(first.measured >= 2, "every message row was measured once");
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(900.0),
        &everywhere(),
    );
    assert_eq!(
        annotations.work(),
        first,
        "the same width again does no shaping"
    );
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(260.0),
        &everywhere(),
    );
    let narrow = annotations.work();
    assert_eq!(
        narrow.packed,
        first.packed + 2,
        "a new width packs the two hinted lines again"
    );
    assert_eq!(
        narrow.measured, first.measured,
        "no message row is measured twice"
    );
    assert!(
        narrow.broken >= 1 && narrow.broken <= first.measured,
        "only rows wider than the width are broken by shaping: {}",
        narrow.broken
    );
    // A width nothing is wider than breaks nothing at all.
    let before = annotations.work();
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(5000.0),
        &everywhere(),
    );
    assert_eq!(annotations.work().broken, before.broken);
    assert_eq!(annotations.work().measured, before.measured);
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

/// A new width packs and breaks only the lines near the view;
/// the others keep what they had until the view comes
///  near,
/// and `stale_in` tells the window when that is.
#[test]
fn a_new_width_applies_to_the_lines_near_the_view_first() {
    let mut text = String::new();
    for number in 0..40 {
        text.push_str(&format!("{CODE} {number}\n"));
    }
    let hints = vec![
        hint(start_of(&text, 3) + 90, ": a"),
        hint(start_of(&text, 30) + 90, ": b"),
    ];
    let long =
        "expected `u32`, found `&str` because the argument named first_item is a string slice here";
    let diagnostics = vec![
        error(start_of(&text, 5) + 4, start_of(&text, 5) + 9, long),
        error(start_of(&text, 33) + 4, start_of(&text, 33) + 9, long),
    ];
    let (mut annotations, document) = store(&text, hints, diagnostics);
    let mut shaper = TextShaper::new();

    let wide = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(900.0),
        &everywhere(),
    );
    assert_eq!(annotations.work().packed, 2);
    assert!(!annotations.stale_in(Some(900.0), &everywhere()));
    let (near, far) = (0..10, 20..40);
    let narrow = annotations.assemble(SHOWN, &document, &mut shaper, 1.0, Some(260.0), &near);
    assert_eq!(
        annotations.work().packed,
        3,
        "only the hinted line near the view is packed again"
    );
    assert!(
        !annotations.stale_in(Some(260.0), &near),
        "the lines near the view are exact"
    );
    assert!(
        annotations.stale_in(Some(260.0), &far),
        "the lines far from it are stale"
    );
    assert_ne!(
        block_at(&narrow, 3).hints[0].x,
        block_at(&wide, 3).hints[0].x
    );
    assert_ne!(block_at(&narrow, 5).messages, block_at(&wide, 5).messages);
    assert_eq!(
        block_at(&narrow, 30).hints,
        block_at(&wide, 30).hints,
        "a far line keeps its packing"
    );
    assert_eq!(
        block_at(&narrow, 33).messages,
        block_at(&wide, 33).messages,
        "a far line keeps its rows"
    );
    // The view comes near the other lines.
    let after = annotations.assemble(SHOWN, &document, &mut shaper, 1.0, Some(260.0), &far);
    assert_eq!(
        annotations.work().packed,
        4,
        "the far hinted line is packed when the view comes near"
    );
    assert!(!annotations.stale_in(Some(260.0), &everywhere()));
    assert_ne!(block_at(&after, 30).hints, block_at(&wide, 30).hints);
    assert_ne!(block_at(&after, 33).messages, block_at(&wide, 33).messages);
    // Hints of a snapshot that no longer describes the text leave no stale packing behind.
    let other = DocumentStamp {
        file: 9,
        revision: 9,
    };
    annotations.assemble(
        other,
        &document,
        &mut shaper,
        1.0,
        Some(500.0),
        &everywhere(),
    );
    assert!(!annotations.stale_in(Some(500.0), &everywhere()));
}

/// A message line cut at 80 columns is joined and broken again as a whole,
/// so no row but its last is a short tail,
///  and a word that was cut is joined without a blank.
#[test]
fn a_message_line_is_broken_again_as_a_whole() {
    let long = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega alpha beta gamma delta";
    let word = "x".repeat(150);
    let (mut annotations, document) = store(
        CODE,
        Vec::new(),
        vec![error(4, 9, long), error(10, 15, &word)],
    );
    let mut shaper = TextShaper::new();
    let limit = 340.0;
    let blocks = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(limit),
        &everywhere(),
    );
    let rows = &blocks[0].messages;
    // The first message: every row but its last holds most of the width.
    let mut first_end = rows.len();
    for (index, row) in rows.iter().enumerate().skip(1) {
        if !row.continued {
            first_end = index;
            break;
        }
    }
    assert!(
        first_end >= 3,
        "the message takes several rows: {first_end}"
    );
    for row in &rows[..first_end - 1] {
        let indent = if row.continued {
            CONTINUATION_INDENT
        } else {
            0.0
        };
        let width = shaper.row_layout(&row.text, 1.0).full_width();
        assert!(
            indent + width >= limit * 0.6,
            "`{}` is a short row of {width} px before the end of its message",
            row.text
        );
    }
    // The cut word stays whole: joined with a blank inside, the second message would break at that blank and leave a
    // short row before its end, as the pieces of a word cut at 80 columns are one run of characters.
    let second = &rows[first_end..];
    // The first row stays the label before the word, which does not fit beside it; the rows after it hold the word.
    assert!(second.len() >= 4, "the second message takes several rows");
    for row in &second[1..second.len() - 1] {
        let indent = if row.continued {
            CONTINUATION_INDENT
        } else {
            0.0
        };
        let width = shaper.row_layout(&row.text, 1.0).full_width();
        assert!(
            indent + width >= limit * 0.9,
            "`{}` is a short row of {width} px before the end of the cut word's message: the word was split by a blank",
            row.text
        );
    }
}
