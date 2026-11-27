//! The caret's side of a wrap offset,
//!  and the Home and End rules on rows a test hands over as character ranges:
//!  a line in three rows,
//!  a line of one row,
//!  and the places each press moves to.
//! The rules are the same ones the shaped-row module checks against the shaper.

use crate::fixtures::place;
use ide_app::document::{
    Document,
    ReadingAffinity,
};
/// The pure Home and End rules.
use ide_app::line_boundary::{
    CaretPlace,
    end,
    home,
    row_containing,
};
/// Source character ranges of visual rows.
use std::ops::Range;

/// The caret at the end of an upper row belongs to that row,
/// and at the start of the lower row to the lower one.
#[test]
fn a_wrap_offset_belongs_to_the_row_the_side_names() {
    let rows = vec![0..5, 5..9, 9..12];
    assert_eq!(row_containing(&rows, 5, ReadingAffinity::Downstream), 1);
    assert_eq!(row_containing(&rows, 5, ReadingAffinity::Upstream), 0);
    assert_eq!(row_containing(&rows, 4, ReadingAffinity::Upstream), 0);
    assert_eq!(
        row_containing(&rows, 0, ReadingAffinity::Upstream),
        0,
        "no row above the first"
    );
    assert_eq!(
        row_containing(&rows, 12, ReadingAffinity::Downstream),
        2,
        "the end of the line is on the last row"
    );
    assert_eq!(row_containing(&rows, 12, ReadingAffinity::Upstream), 2);
}

/// A line "  first row  second row  third" in three rows,
/// to exercise the rules with exact characters.
fn sample() -> (ide_app::document::Document, Vec<Range<usize>>) {
    // Characters: 2 blanks, "first" 2..7, blank 7, "row" 8..11, 2 blanks 11..13 | "second" 13..19, blank 19,
    // "row" 20..23, 2 blanks 23..25 | "third" 25..30.
    let document = Document::new("  first row  second row  third");
    return (document, vec![0..13, 13..25, 25..30]);
}

/// A line of one visual row that holds `length` characters.
fn one_row(length: usize) -> Vec<Range<usize>> {
    return vec![Range {
        start: 0,
        end: length,
    }];
}

/// What:
///  One line's text and its row ranges,
///  so the tests name the line once and ask for a Home or End press at a caret.
/// Why:
///  Every assertion presses from a different caret in the same line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const from = { home: (head, affinity) => home(text, rows, head, affinity), end: ... };
/// ```
struct Navigator<'a> {
    /// The line's text.
    text: helix_core::RopeSlice<'a>,
    /// The visual rows of the line,
    ///  as character ranges.
    rows: &'a [Range<usize>],
}

impl Navigator<'_> {
    /// Press Home with the caret at `head` and `affinity`.
    fn home(
        &self,
        head: usize,
        affinity: ReadingAffinity,
    ) -> CaretPlace {
        return home(self.text, self.rows, head, affinity);
    }

    /// Press End with the caret at `head` and `affinity`.
    fn end(
        &self,
        head: usize,
        affinity: ReadingAffinity,
    ) -> CaretPlace {
        return end(self.text, self.rows, head, affinity);
    }
}

/// Home on a row goes to its start,
/// from the start of a continuation row to the first non-blank character,
///  and on the first row it alternates between the line start and the first non-blank character.
#[test]
fn home_goes_to_the_row_start_then_the_first_non_blank() {
    let (document, rows) = sample();
    let from = Navigator {
        text: document.text().slice(..),
        rows: &rows,
    };
    use ReadingAffinity::{
        Downstream,
        Upstream,
    };
    assert_eq!(
        from.home(28, Downstream),
        place(25, Downstream),
        "inside the last row"
    );
    assert_eq!(
        from.home(25, Downstream),
        place(2, Downstream),
        "at a continuation row start: the first non-blank"
    );
    assert_eq!(
        from.home(13, Downstream),
        place(2, Downstream),
        "the same from the middle row start"
    );
    assert_eq!(
        from.home(13, Upstream),
        place(2, Downstream),
        "an end of the first row, past the first non-blank"
    );
    assert_eq!(from.home(24, Downstream), place(13, Downstream));
    assert_eq!(
        from.home(5, Downstream),
        place(2, Downstream),
        "past the indent on the first row"
    );
    assert_eq!(
        from.home(2, Downstream),
        place(0, Downstream),
        "at the first non-blank character"
    );
    assert_eq!(
        from.home(1, Downstream),
        place(0, Downstream),
        "inside the indent"
    );
    assert_eq!(
        from.home(0, Downstream),
        place(2, Downstream),
        "from the line start"
    );
    // A first row of blanks only has no first non-blank character.
    let blanks = Document::new("      then text");
    let blank_rows = vec![0..6, 6..15];
    let blank_text = blanks.text().slice(..);
    assert_eq!(
        home(blank_text, &blank_rows, 10, Downstream),
        place(6, Downstream)
    );
    assert_eq!(
        home(blank_text, &blank_rows, 6, Downstream),
        place(0, Downstream),
        "no first non-blank on row 0"
    );
    assert_eq!(
        home(blank_text, &blank_rows, 3, Downstream),
        place(0, Downstream)
    );
    assert_eq!(
        home(blank_text, &blank_rows, 0, Downstream),
        place(0, Downstream)
    );
}

/// End goes to the end of the row without its trailing blanks,
/// then to the row end,
///  then from the row end to the logical line end,
/// which again leaves its trailing blanks out first.
#[test]
fn end_goes_to_the_row_end_then_the_line_end() {
    let (document, rows) = sample();
    let from = Navigator {
        text: document.text().slice(..),
        rows: &rows,
    };
    use ReadingAffinity::{
        Downstream,
        Upstream,
    };
    assert_eq!(
        from.end(0, Downstream),
        place(11, Downstream),
        "before the two trailing blanks of the first row"
    );
    assert_eq!(
        from.end(11, Downstream),
        place(13, Upstream),
        "the end of the upper row is drawn on it"
    );
    assert_eq!(
        from.end(13, Upstream),
        place(30, Downstream),
        "from the end of an upper row to the line end"
    );
    assert_eq!(
        from.end(13, Downstream),
        place(23, Downstream),
        "the middle row, before its trailing blanks"
    );
    assert_eq!(from.end(23, Downstream), place(25, Upstream));
    assert_eq!(from.end(25, Upstream), place(30, Downstream));
    assert_eq!(
        from.end(27, Downstream),
        place(30, Downstream),
        "the last row has no trailing blanks"
    );
    assert_eq!(
        from.end(30, Downstream),
        place(30, Downstream),
        "at the end of the text"
    );
    // A row of blanks only keeps its whole end.
    let gap = Document::new("aaaa    bb");
    let gap_rows = vec![0..4, 4..8, 8..10];
    let gap_text = gap.text().slice(..);
    assert_eq!(
        end(gap_text, &gap_rows, 5, Downstream),
        place(8, Upstream),
        "a row of blanks only keeps its whole end"
    );
    // Trailing blanks on the last row: End goes before them, then back after them.
    let trailing = Document::new("ab   ");
    let trailing_rows = one_row(5);
    let trailing_text = trailing.text().slice(..);
    assert_eq!(
        end(trailing_text, &trailing_rows, 0, Downstream),
        place(2, Downstream)
    );
    assert_eq!(
        end(trailing_text, &trailing_rows, 2, Downstream),
        place(5, Downstream)
    );
    assert_eq!(
        end(trailing_text, &trailing_rows, 5, Downstream),
        place(2, Downstream)
    );
    // A line of blanks only.
    let blank = Document::new("   ");
    let blank_rows = one_row(3);
    assert_eq!(
        end(blank.text().slice(..), &blank_rows, 0, Downstream),
        place(3, Downstream)
    );
    assert_eq!(
        end(blank.text().slice(..), &blank_rows, 3, Downstream),
        place(0, Downstream)
    );
}
