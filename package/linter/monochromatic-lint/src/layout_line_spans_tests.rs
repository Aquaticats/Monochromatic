//! What:
//!  Controls of the bounded line reader.
//! Why:
//!  A finding's span is what points the reader at the line,
//!  and one read per file is what keeps a list of dead lines from reading its file once per finding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the span of a line starts at that line', () => expect(spans.span(file, 2)).toEqual({...}));
//! ```

use super::{
    LineSpans,
    MAX_SPAN_BYTES,
};
use crate::{
    diagnostic::Span,
    layout_test_support::package_with,
};

/// The span of a finding's line starts at that line,
///  covers it without its line break,
///  and falls back to the file start for line zero,
///  a line the file does not have and a file that cannot be read.
///  One file read answers every line it holds,
///  which is what keeps a list full of dead lines from reading the list once per finding,
///  so a line read before the file disappears is still answered after it is gone.
#[test]
fn layout_the_span_of_a_line_starts_at_that_line() {
    let text: &str = "first\nsecond line\r\nthird";
    let fixture = package_with(&[("lines.txt", text)]);
    let file = fixture.path.join("lines.txt");
    let second_start: usize = text.find("second").expect("second line");
    let third_start: usize = text.find("third").expect("third line");
    let mut spans: LineSpans = LineSpans::default();
    assert_eq!(
        spans.span(&file, 1),
        Span {
            offset: 0,
            length: "first".len(),
            line: 1,
            column: 1
        }
    );
    assert_eq!(
        spans.span(&file, 2),
        Span {
            offset: second_start,
            length: "second line".len(),
            line: 2,
            column: 1
        }
    );
    // The file is gone from here on, so the third line can only be answered from the one read above.
    std::fs::remove_file(&file).expect("remove the fixture file");
    assert_eq!(
        spans.span(&file, 3),
        Span {
            offset: third_start,
            length: "third".len(),
            line: 3,
            column: 1
        },
        "a second read would answer the file start here"
    );
    let at_start: Span = spans.span(&file, 0);
    assert_eq!(spans.span(&file, 4), at_start);
    assert_eq!(spans.span(&fixture.path.join("missing.txt"), 2), at_start);
}

/// A file over the bound answers the file start for every line it holds,
///  so a path that turns out to be a device or a file that grew is never read whole.
#[test]
fn layout_a_file_over_the_span_bound_answers_the_file_start() {
    // One line as long as the bound, then a line the bound never reaches.
    let text: String = format!("{}\nafter the bound\n", "a".repeat(MAX_SPAN_BYTES as usize));
    let fixture = package_with(&[("long.txt", text.as_str())]);
    let file = fixture.path.join("long.txt");
    let mut spans: LineSpans = LineSpans::default();
    let at_start: Span = spans.span(&file, 0);
    assert_eq!(spans.span(&file, 1), at_start);
    assert_eq!(spans.span(&file, 2), at_start);
}
