//! How hint answers become rows:
//!  grouped by the line that ends them,
//!  accumulated per revision over the lines each asked about,
//!  and never holding a hint outside the lines an answer asked about.

use crate::fixtures::{
    SHOWN,
    TEXT,
    block_lines,
    blocks,
    hint,
    hint_texts,
    hints,
    ranged,
    unwrapped,
};
use helix_core::Rope;
use ide_app::{
    annotation::Annotations,
    language::identity::DocumentStamp,
};

/// Hints belong to the line their position lies on:
///  a hint at a line end stays on its line,
///  the end of the
/// text belongs to the last line,
///  labels lose their padding spaces,
///  and blank labels are dropped.
#[test]
fn hints_are_grouped_by_line_and_line_ends() {
    let text = Rope::from(TEXT);
    // Line starts: 0, 11, 24, 25, 37; the text has 38 characters.
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &text,
        hints(
            SHOWN,
            vec![
                hint(5, " : i32 "),
                hint(10, "end-of-line-0"),
                hint(11, "start-of-line-1"),
                hint(24, "   "),
                hint(38, "end-of-text"),
            ],
        ),
        None,
    );
    let assembled = blocks(&mut annotations, SHOWN);
    let lines: Vec<usize> = block_lines(&assembled);
    assert_eq!(
        lines,
        [0, 1, 4],
        "the blank label on line 2 must own no block"
    );
    assert_eq!(hint_texts(&assembled, 0), [": i32", "end-of-line-0"]);
    assert_eq!(hint_texts(&assembled, 1), ["start-of-line-1"]);
    assert_eq!(hint_texts(&assembled, 4), ["end-of-text"]);
    assert_eq!(assembled[2].hints[0].position, 38);
}

/// A hint snapshot replaces the hints of the lines it asked about and keeps those of other lines of the same
/// revision;
///  hints of another revision are dropped as a whole.
#[test]
fn hints_of_one_revision_accumulate_by_line_range() {
    let text = Rope::from(TEXT);
    let mut store = Annotations::default();
    // The view first shows lines 0 and 1, then lines 3 and 4.
    let upper = ranged(SHOWN, (0, 2), vec![hint(5, ": i32"), hint(17, ": i64")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(upper)));
    let lower = ranged(SHOWN, (3, 5), vec![hint(37, "end")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(lower)));
    let both = blocks(&mut store, SHOWN);
    let lines: Vec<usize> = block_lines(&both);
    assert_eq!(
        lines,
        [0, 1, 4],
        "an answer for other lines removed hints it did not ask about"
    );
    // A second answer for lines 0 and 1 without the hint on line 0 removes exactly that one.
    let again = ranged(SHOWN, (0, 2), vec![hint(17, ": u8")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(again)));
    let replaced = blocks(&mut store, SHOWN);
    let kept: Vec<usize> = block_lines(&replaced);
    assert_eq!(
        kept,
        [1, 4],
        "an answer did not replace the hints of its own lines"
    );
    assert_eq!(hint_texts(&replaced, 1), [": u8"]);
    // Hints of the next revision start from nothing.
    let next = DocumentStamp {
        file: 3,
        revision: 8,
    };
    let fresh = ranged(next, (0, 2), vec![hint(5, ": new")]);
    assert!(store.accept_hints(next, &text, unwrapped(fresh)));
    let renewed = blocks(&mut store, next);
    assert_eq!(renewed.len(), 1, "hints of the previous revision were kept");
    assert_eq!(hint_texts(&renewed, 0), [": new"]);
}

/// A server answers for every node of the text that reaches into the lines it was asked about,
///  so an answer for later lines can hold hints that stand on earlier ones
///  (TypeScript 7 answers a request for lines 100 to 200 of a test file with the four hints of the
///  `const integrationChildren = ...map(...)` statement on line 96,
///  whose body spans the whole file).
///  Those hints are not part of the answer:
///  the store replaces the hints of the lines asked about and keeps every other line as it was,
///  so an answer that repeats a hint of an unasked line adds nothing to that line.
#[test]
fn hints_outside_the_asked_lines_are_not_part_of_an_answer() {
    let text = Rope::from(TEXT);
    let mut store = Annotations::default();
    let upper = ranged(SHOWN, (0, 2), vec![hint(5, ": i32"), hint(17, ": i64")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(upper)));
    // The next answers ask for lines 3 and 4 and repeat the hint of line 0 and add one of line 1, as a server does.
    for _ in 0..3 {
        let lower = ranged(
            SHOWN,
            (3, 5),
            vec![hint(5, ": i32"), hint(20, ": extra"), hint(37, "end")],
        );
        assert!(store.accept_hints(SHOWN, &text, unwrapped(lower)));
    }
    let assembled = blocks(&mut store, SHOWN);
    assert_eq!(
        block_lines(&assembled),
        [0, 1, 4],
        "an answer put hints on lines it did not ask about"
    );
    assert_eq!(
        hint_texts(&assembled, 0),
        [": i32"],
        "an answer that repeated the hint of an unasked line added it again"
    );
    assert_eq!(
        hint_texts(&assembled, 1),
        [": i64"],
        "an answer added a hint to a line it did not ask about"
    );
    assert_eq!(hint_texts(&assembled, 4), ["end"]);
    // Asking for the lines again replaces them whole, as before.
    let again = ranged(
        SHOWN,
        (0, 2),
        vec![hint(5, ": i32"), hint(17, ": u8"), hint(18, ": u16")],
    );
    assert!(store.accept_hints(SHOWN, &text, unwrapped(again)));
    let replaced = blocks(&mut store, SHOWN);
    assert_eq!(hint_texts(&replaced, 0), [": i32"]);
    assert_eq!(hint_texts(&replaced, 1), [": u8", ": u16"]);
}
