//! Diagnostic message rows:
//!  broken to the text area,
//!  held to the row cap at a narrow width,
//!  moved left to stay inside the area,
//!  and broken again as a whole after a new width.

use crate::fixtures::{
    CODE,
    COLORS,
    SHOWN,
    error,
    everywhere,
    store,
    viewport,
    words,
};
use ide_app::{
    annotation::Visible,
    annotation_layout::lay_out,
    row_map::RowMap,
    shaped_text::{
        TextShaper,
        Viewport,
    },
    virtual_row::{
        BLOCK_GAP,
        CONTINUATION_INDENT,
        MESSAGE_ROWS,
        ROW_HEIGHT,
    },
};

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
