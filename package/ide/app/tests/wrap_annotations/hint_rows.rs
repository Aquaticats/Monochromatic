//! Hints packed against the text area:
//!  against the line as it is broken,
//!  a hint that would pass the right edge ending at it,
//!  and a label wider than the text area broken into rows.

use crate::fixtures::{
    CODE,
    COLORS,
    viewport,
};
use ide_app::{
    annotation::Visible,
    annotation_layout::{
        lay_out,
        pack,
    },
    document::Document,
    row_map::RowMap,
    shaped_text::TextShaper,
    virtual_row::Block,
};
use std::sync::Arc;

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
