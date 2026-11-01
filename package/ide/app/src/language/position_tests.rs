//! Position conversion in the three column units a language server can negotiate.

use super::{from_lsp_position, from_lsp_range, to_lsp_position, to_lsp_range};
use helix_core::Rope;
use helix_lsp::{OffsetEncoding, lsp};

/// Line 0 has a tab and CRLF;
///  line 1 has a combining mark and an astral character;
///  line 2 has no line break.
const SOURCE: &str = "a\tb\r\ne\u{301}\u{1F600}x\r\nlast";

/// Character offset of `x` on line 1:
///  five characters on line 0,
///  then `e`,
///  the mark,
///  and the emoji.
const X_OFFSET: usize = 8;

#[test]
fn offsets_become_columns_in_each_negotiated_unit() {
    let text = Rope::from_str(SOURCE);
    // `e` is one byte, U+0301 two bytes, U+1F600 four bytes.
    assert_eq!(
        to_lsp_position(&text, X_OFFSET, OffsetEncoding::Utf8),
        Some(lsp::Position::new(1, 7))
    );
    // U+1F600 needs a surrogate pair, so it counts as two UTF-16 units.
    assert_eq!(
        to_lsp_position(&text, X_OFFSET, OffsetEncoding::Utf16),
        Some(lsp::Position::new(1, 4))
    );
    assert_eq!(
        to_lsp_position(&text, X_OFFSET, OffsetEncoding::Utf32),
        Some(lsp::Position::new(1, 3))
    );
}

#[test]
fn columns_return_to_the_same_offset_in_each_unit() {
    let text = Rope::from_str(SOURCE);
    for encoding in [
        OffsetEncoding::Utf8,
        OffsetEncoding::Utf16,
        OffsetEncoding::Utf32,
    ] {
        for offset in 0..=text.len_chars() {
            // Offsets inside a CRLF pair have no protocol position; Helix reports the line end for them.
            if offset == 4 || offset == 10 {
                continue;
            }
            let position =
                to_lsp_position(&text, offset, encoding).expect("offset inside the text");
            assert_eq!(
                from_lsp_position(&text, position, encoding),
                Some(offset),
                "offset {offset} did not survive {encoding:?}"
            );
        }
    }
}

#[test]
fn tab_counts_as_one_column_in_every_unit() {
    let text = Rope::from_str(SOURCE);
    for encoding in [
        OffsetEncoding::Utf8,
        OffsetEncoding::Utf16,
        OffsetEncoding::Utf32,
    ] {
        assert_eq!(
            to_lsp_position(&text, 2, encoding),
            Some(lsp::Position::new(0, 2))
        );
    }
}

#[test]
fn column_past_a_crlf_line_stops_before_the_line_break() {
    let text = Rope::from_str(SOURCE);
    for encoding in [
        OffsetEncoding::Utf8,
        OffsetEncoding::Utf16,
        OffsetEncoding::Utf32,
    ] {
        assert_eq!(
            from_lsp_position(&text, lsp::Position::new(0, 99), encoding),
            Some(3),
            "a column past the line must stop before CRLF in {encoding:?}"
        );
    }
}

#[test]
fn line_past_the_end_is_rejected() {
    let text = Rope::from_str(SOURCE);
    assert_eq!(text.len_lines(), 3);
    for encoding in [
        OffsetEncoding::Utf8,
        OffsetEncoding::Utf16,
        OffsetEncoding::Utf32,
    ] {
        assert_eq!(
            from_lsp_position(&text, lsp::Position::new(3, 0), encoding),
            None,
            "a position on a line past the end was accepted"
        );
        assert_eq!(
            from_lsp_position(&text, lsp::Position::new(99, 0), encoding),
            None,
            "a position on a line past the end was accepted"
        );
    }
}

#[test]
fn offset_past_the_end_has_no_position() {
    let text = Rope::from_str(SOURCE);
    let end = text.len_chars();
    assert!(to_lsp_position(&text, end, OffsetEncoding::Utf16).is_some());
    assert_eq!(to_lsp_position(&text, end + 1, OffsetEncoding::Utf16), None);
    assert_eq!(to_lsp_range(&text, 0, end + 1, OffsetEncoding::Utf8), None);
}

#[test]
fn range_start_must_exist_and_range_end_clamps_to_the_text_end() {
    let text = Rope::from_str(SOURCE);
    let through_the_end = lsp::Range::new(lsp::Position::new(2, 1), lsp::Position::new(9, 0));
    assert_eq!(
        from_lsp_range(&text, through_the_end, OffsetEncoding::Utf16),
        Some((12, text.len_chars()))
    );
    let starts_past_the_end = lsp::Range::new(lsp::Position::new(3, 0), lsp::Position::new(9, 0));
    assert_eq!(
        from_lsp_range(&text, starts_past_the_end, OffsetEncoding::Utf16),
        None
    );
    let reversed = lsp::Range::new(lsp::Position::new(1, 1), lsp::Position::new(0, 1));
    assert_eq!(
        from_lsp_range(&text, reversed, OffsetEncoding::Utf32),
        Some((6, 6))
    );
}

#[test]
fn reversed_character_range_is_sent_in_document_order() {
    let text = Rope::from_str(SOURCE);
    assert_eq!(
        to_lsp_range(&text, X_OFFSET, 5, OffsetEncoding::Utf8),
        Some(lsp::Range::new(
            lsp::Position::new(1, 0),
            lsp::Position::new(1, 7)
        ))
    );
}
