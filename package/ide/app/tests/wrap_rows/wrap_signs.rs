//! The gutter's wrap indicator:
//!  a tab correction that settles at every width,
//!  the sign chosen as the first displayable pair or an arrow,
//!  and the coverage read from the shaped glyphs.

use crate::fixtures::Coverage;
use ide_app::{
    document::{
        Document,
        ReadingAffinity,
    },
    shaped_text::TextShaper,
    wrap_sign::{
        SIGN_PAIRS,
        Sign,
        choose_sign,
    },
};

/// Tabs end on their stops at every width and scale,
/// on every row they end on.
#[test]
fn tab_correction_settles_at_every_width() {
    let text = "a\tb\tc\td\te\tf\tg\th\ti\tj\tk\tl\tm\tn\to\tp\tq\tr\ts\tt\tu\tv\tw\tx\ty\tz\t\t\tlong_identifier_after_tabs\t\tx";
    let mixed = "\t\tfn\tcompute(\tvalue:\tu32)\t->\tu32\t{\t\tlet\ttotal\t=\tvalue\t*\t2;\t\treturn\ttotal;\t}";
    let mut broken = Vec::new();
    for scale in [1.0_f32, 2.0] {
        let mut shaper = TextShaper::new();
        let space = {
            let probe = Document::new(" ");
            shaper.row(&probe, 0, scale).caret_x(1, scale)
        };
        for source in [text, mixed] {
            let document = Document::new(source);
            let mut width = 40.0;
            while width < 700.0 {
                let row = shaper.wrapped_row(&document, 0, scale, width);
                for byte in row.projection.tabs.clone() {
                    let source_after = row.projection.byte_to_source[byte + 1];
                    let end = row.caret_at(
                        row.source_start + source_after,
                        ReadingAffinity::Upstream,
                        scale,
                    );
                    let start_row = {
                        let mut found = 0;
                        for index in 0..row.visual_rows() {
                            if row.visual_row_bytes(index).expect("row").start <= byte {
                                found = index;
                            }
                        }
                        found
                    };
                    if end.row != start_row {
                        continue;
                    }
                    let stop = space * 2.0;
                    let remainder = (end.x / stop).fract();
                    if !(remainder.abs() < 0.02 || (1.0 - remainder).abs() < 0.02) {
                        broken.push((scale, width, byte, end.x));
                    }
                }
                width += 7.0;
            }
        }
    }
    assert!(
        broken.is_empty(),
        "{} tabs ended off their stops: {broken:?}",
        broken.len()
    );
}

/// The wrap indicator is the first pair both of whose symbols the fonts display,
/// else an arrow.
#[test]
fn the_sign_is_the_first_displayable_pair_or_an_arrow() {
    assert_eq!(choose_sign(Coverage::All), Sign::Glyph('\u{2925}'));
    assert_eq!(
        choose_sign(Coverage::Without('\u{2926}')),
        Sign::Glyph('\u{21B3}')
    );
    assert_eq!(
        choose_sign(Coverage::Pair(SIGN_PAIRS[3])),
        Sign::Glyph('\u{21AA}')
    );
    assert_eq!(
        choose_sign(Coverage::Only('\u{2925}')),
        Sign::Arrow,
        "a pair needs both of its symbols, as in IntelliJ's TextBasedSoftWrapPainter"
    );
    assert_eq!(choose_sign(Coverage::None), Sign::Arrow);
}

/// The shaper tells a displayable character from one no font has.
#[test]
fn coverage_is_read_from_the_shaped_glyphs() {
    let mut shaper = TextShaper::new();
    assert!(shaper.can_display('A', 1.0));
    assert!(shaper.can_display('\u{2192}', 1.0) || !shaper.can_display('\u{10FFFE}', 1.0));
    assert!(
        !shaper.can_display('\u{10FFFE}', 1.0),
        "a noncharacter has no glyph in any font"
    );
}
