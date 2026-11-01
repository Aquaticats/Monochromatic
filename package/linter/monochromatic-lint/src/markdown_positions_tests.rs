//! What:
//!  Markdown coordinate controls independent from rule logic.
//! Why:
//!  Emoji,
//!  BOMs and CR-only input must not shift fixes or reported columns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('Markdown positions', () => { /* original bytes versus UTF-16 columns */ });
//! ```

/// Import the production index and common range representation.
use super::MarkdownPositions;
use crate::diagnostic::Span;

/// Astral and multibyte BMP characters change columns without changing source byte ranges.
#[test]
fn utf16_columns_are_distinct_from_byte_offsets() {
    let index = MarkdownPositions::new("a🚀éz");
    assert_eq!(
        index.span(1, 4),
        Span {
            offset: 1,
            length: 4,
            line: 1,
            column: 2
        }
    );
    assert_eq!(
        index.span(5, 2),
        Span {
            offset: 5,
            length: 2,
            line: 1,
            column: 4
        }
    );
    assert_eq!(
        index.span(7, 1),
        Span {
            offset: 7,
            length: 1,
            line: 1,
            column: 5
        }
    );
}

/// CR,
///  LF and CRLF each advance exactly one line.
#[test]
fn newline_variants_share_the_parser_convention() {
    let index = MarkdownPositions::new("a\rb\r\nc\nd");
    assert_eq!(index.span(2, 1).line, 2);
    assert_eq!(index.span(5, 1).line, 3);
    assert_eq!(index.span(7, 1).line, 4);
    assert_eq!(index.span(7, 1).column, 1);
}

/// A leading BOM is still present in offsets but does not shift the first visible column.
#[test]
fn bom_and_empty_inputs_keep_visible_columns() {
    let index = MarkdownPositions::new("\u{feff}🚀x\n");
    assert_eq!(
        index.span(3, 4),
        Span {
            offset: 3,
            length: 4,
            line: 1,
            column: 1
        }
    );
    assert_eq!(index.span(7, 1).column, 3);
    assert_eq!(
        index.span(9, 0),
        Span {
            offset: 9,
            length: 0,
            line: 2,
            column: 1
        }
    );
    assert_eq!(
        MarkdownPositions::new("").span(0, 0),
        Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1
        }
    );
}

/// Width changes on previous lines must not leak into the next line's column.
#[test]
fn unicode_adjustments_reset_per_line() {
    let index = MarkdownPositions::new("🚀\néx");
    assert_eq!(
        index.span(7, 1),
        Span {
            offset: 7,
            length: 1,
            line: 2,
            column: 2
        }
    );
    assert_eq!(index.span(usize::MAX, usize::MAX).length, 0);
}
