//! Keyboard caret movement over graphemes,
//!  words,
//!  lines,
//!  and the whole text,
//!  at Unicode and line-end boundaries.

/// What:
///  Import the production movement functions and Helix's rope through the library's public interface.
/// Why:
///  The native key handler calls exactly these functions on the document's rope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Motion, lineRange, moved, wordRange } from 'ide-app/caret-motion';
/// import { Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The movements under test.
use ide_app::caret_motion::{Motion, line_range, moved, word_range};

/// What:
///  Apply one movement repeatedly and collect every position it visits;
/// `Vec<usize>` is a growable list of address-sized character positions (siblings:
///  `[usize; N]`,
///  `&[usize]`).
/// Why:
///  A whole walk shows skipped or split graphemes in one readable assertion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(source: string, start: number, motion: Motion): number[];
/// ```
fn walk(source: &str, start: usize, motion: Motion) -> Vec<usize> {
    // What: `Rope::from_str` copies the borrowed text into an owned rope; `slice(..)` lends all of it.
    // Why: Movement reads the same rope type the document owns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rope = Rope.from(source);
    // ```
    let rope = Rope::from_str(source);
    let mut visited = Vec::new();
    let mut head = start;
    loop {
        let next = moved(rope.slice(..), head, motion);
        if next == head {
            return visited;
        }
        visited.push(next);
        head = next;
    }
}

/// One movement from one position.
fn step(source: &str, head: usize, motion: Motion) -> usize {
    let rope = Rope::from_str(source);
    return moved(rope.slice(..), head, motion);
}

/// Left and Right step over a combining sequence,
///  an astral character,
///  and CRLF as single units.
#[test]
fn grapheme_steps_never_split_combining_astral_or_crlf_sequences() {
    // a(0) 猫(1) e(2) U+0301(3) 𝒳(4) CR(5) LF(6) b(7), eight characters.
    let source = "a猫e\u{301}𝒳\r\nb";
    assert_eq!(walk(source, 0, Motion::Right), [1, 2, 4, 5, 7, 8]);
    assert_eq!(walk(source, 8, Motion::Left), [7, 5, 4, 2, 1, 0]);
    // A woman-woman-girl family is five characters joined by zero-width joiners; a flag is two.
    let emoji = "👩\u{200d}👩\u{200d}👧🇯🇵x";
    assert_eq!(walk(emoji, 0, Motion::Right), [5, 7, 8]);
    assert_eq!(walk(emoji, 8, Motion::Left), [7, 5, 0]);
}

/// Home and End stay on the line and stop before LF and CRLF;
///  the last line needs no terminator.
#[test]
fn line_start_and_end_respect_terminators_and_the_end_of_text() {
    // a(0) b(1) CR(2) LF(3) c(4) d(5) LF(6) LF(7) e(8) f(9).
    let source = "ab\r\ncd\n\nef";
    assert_eq!(
        step(source, 0, Motion::LineEnd),
        2,
        "End must stop before CRLF"
    );
    assert_eq!(step(source, 2, Motion::LineEnd), 2);
    assert_eq!(step(source, 1, Motion::LineStart), 0);
    assert_eq!(
        step(source, 4, Motion::LineEnd),
        6,
        "End must stop before LF"
    );
    assert_eq!(
        step(source, 7, Motion::LineStart),
        7,
        "an empty line starts where it ends"
    );
    assert_eq!(step(source, 7, Motion::LineEnd), 7);
    assert_eq!(
        step(source, 8, Motion::LineEnd),
        10,
        "the last line ends at the end of text"
    );
    assert_eq!(step(source, 10, Motion::LineStart), 8);
    assert_eq!(step(source, 5, Motion::DocumentStart), 0);
    assert_eq!(step(source, 5, Motion::DocumentEnd), 10);
    // After a final terminator the caret can stand on the empty last line.
    assert_eq!(step("ab\n", 3, Motion::LineStart), 3);
    assert_eq!(step("ab\n", 3, Motion::LineEnd), 3);
    assert_eq!(step("", 0, Motion::LineEnd), 0);
    assert_eq!(step("", 0, Motion::DocumentEnd), 0);
}

/// Ctrl+Right stops at the end of each word or punctuation run;
///  Ctrl+Left at each start.
#[test]
fn word_steps_stop_at_word_and_punctuation_runs_across_lines() {
    // foo_bar(0..7) two blanks b a z(9..12) .(12) qux(13..16) ((16) 猫猫(17..19) blank e+accent t(20..23) )(23)
    // LF(24) two blanks x(27), twenty-eight characters.
    let source = "foo_bar  baz.qux(猫猫 e\u{301}t)\n  x";
    assert_eq!(
        walk(source, 0, Motion::WordRight),
        [7, 12, 13, 16, 17, 19, 23, 24, 28]
    );
    assert_eq!(
        walk(source, 28, Motion::WordLeft),
        [27, 23, 20, 17, 16, 13, 12, 9, 0]
    );
    assert_eq!(
        step(source, 1, Motion::WordRight),
        7,
        "from inside a word to its end"
    );
    assert_eq!(
        step(source, 5, Motion::WordLeft),
        0,
        "from inside a word to its start"
    );
    assert_eq!(
        step(source, 19, Motion::WordRight),
        23,
        "the accent stays inside its word"
    );
}

/// The unit a double click selects at a caret boundary.
fn word(source: &str, position: usize) -> (usize, usize) {
    let rope = Rope::from_str(source);
    return word_range(rope.slice(..), position);
}

/// A double click selects the word at the boundary,
///  preferring a word over neighboring blanks or punctuation.
#[test]
fn word_range_prefers_a_word_and_stays_on_its_line() {
    let source = "foo_bar  baz.qux(猫猫 e\u{301}t)\n  x";
    assert_eq!(word(source, 0), (0, 7));
    assert_eq!(word(source, 3), (0, 7), "underscore is part of the word");
    assert_eq!(
        word(source, 7),
        (0, 7),
        "the boundary after a word belongs to that word"
    );
    assert_eq!(
        word(source, 8),
        (7, 9),
        "between two blanks the blank run is selected"
    );
    assert_eq!(word(source, 9), (9, 12));
    assert_eq!(
        word(source, 12),
        (9, 12),
        "a word wins over the punctuation after it"
    );
    assert_eq!(word(source, 16), (13, 16));
    assert_eq!(
        word(source, 17),
        (17, 19),
        "a run of CJK letters is one word"
    );
    assert_eq!(
        word(source, 20),
        (20, 23),
        "a combining mark does not end a word"
    );
    assert_eq!(
        word(source, 24),
        (23, 24),
        "the end of a line selects the last run, not the terminator"
    );
    assert_eq!(
        word(source, 25),
        (25, 27),
        "leading blanks stay on their own line"
    );
    assert_eq!(word(source, 28), (27, 28));
    assert_eq!(word("a == b", 3), (2, 4), "an operator is one run");
    assert_eq!(word("a\n\nb", 2), (2, 2), "an empty line has no word");
    assert_eq!(word("a\r\nb", 1), (0, 1), "CRLF is never part of a word");
    assert_eq!(word("", 0), (0, 0));
}

/// The unit a triple click selects.
fn line(source: &str, position: usize) -> (usize, usize) {
    let rope = Rope::from_str(source);
    return line_range(rope.slice(..), position);
}

/// A triple click selects the whole line with its terminator;
///  the last line ends at the end of text.
#[test]
fn line_range_includes_the_terminator_when_there_is_one() {
    let source = "ab\r\ncd\n\nef";
    assert_eq!(line(source, 1), (0, 4), "CRLF belongs to its line");
    assert_eq!(line(source, 5), (4, 7));
    assert_eq!(line(source, 7), (7, 8), "an empty line is its terminator");
    assert_eq!(
        line(source, 9),
        (8, 10),
        "a last line without terminator stays whole"
    );
    assert_eq!(line(source, 10), (8, 10));
    assert_eq!(
        line("ab\n", 3),
        (3, 3),
        "the empty line after a final terminator"
    );
    assert_eq!(line("", 0), (0, 0));
}

/// Only LF and CRLF end a line of the document;
///  a lone carriage return stays inside its line.
#[test]
fn lone_carriage_return_is_not_a_line_break() {
    // a(0) CR(1) b(2) LF(3) c(4).
    let source = "a\rb\nc";
    assert_eq!(
        Rope::from_str(source).len_lines(),
        2,
        "a lone carriage return split the line"
    );
    assert_eq!(step(source, 0, Motion::LineEnd), 3);
    assert_eq!(line(source, 1), (0, 4));
}
