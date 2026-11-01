//! Keyboard caret movement over source characters that needs no pixel geometry.
//!
//! Positions are Unicode scalar indices into the rope,
//!  the unit of selection and copying.
//! Every result lies on a grapheme boundary,
//!  so a caret never lands inside a combining sequence,
//! a surrogate-free astral character,
//!  or a CRLF pair.

/// What:
///  Import Helix's borrowed rope view,
///  character classes,
///  grapheme steps,
///  and line-end lookup.
/// Why:
///  The document is a Helix rope;
///  reusing its Unicode tables avoids a second,
///  different notion of a character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RopeSlice, categorizeChar, nextGraphemeBoundary, prevGraphemeBoundary, lineEndCharIndex } from 'helix-core';
/// ```
use helix_core::{
    RopeSlice,
    chars::{CharCategory, categorize_char},
    graphemes::{next_grapheme_boundary, prev_grapheme_boundary},
    line_ending::line_end_char_index,
};

/// What:
///  `enum` lists the allowed movements as named variants;
///  `derive` generates copying and comparison.
/// Why:
///  The key handler names a movement once and the same function serves the plain and the Shift form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Motion = 'left' | 'right' | 'wordLeft' | 'wordRight' | 'lineStart' | 'lineEnd'
///   | 'documentStart' | 'documentEnd';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// One grapheme towards the start of the text.
    Left,
    /// One grapheme towards the end of the text.
    Right,
    /// To the start of the current or previous word.
    WordLeft,
    /// To the end of the current or next word.
    WordRight,
    /// To the first character of the logical line,
    /// whichever visual row the caret is on.
    LineStart,
    /// To the end of the logical line,
    /// whichever visual row the caret is on,
    ///  before its terminator.
    LineEnd,
    /// To the first character of the text.
    DocumentStart,
    /// Past the last character of the text.
    DocumentEnd,
}

/// What:
///  The classes a word movement distinguishes;
///  a run of one class is one stop.
/// Why:
///  Line terminators and blanks are skipped together,
///  so they share one class here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Class = 'blank' | 'word' | 'punctuation' | 'other';
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    /// Spaces,
    ///  tabs,
    ///  and line terminators.
    Blank,
    /// Letters of any script,
    ///  digits,
    ///  and underscore.
    Word,
    /// Punctuation and operators.
    Punctuation,
    /// Everything else,
    ///  such as emoji.
    Other,
}

/// Class of the grapheme that starts at `position`,
///  decided by its first character.
/// A combining mark therefore belongs to the class of its base letter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classAt(text: RopeSlice, position: number): Class;
/// ```
fn class_at(text: RopeSlice, position: usize) -> Class {
    let category = categorize_char(text.char(position));
    if category == CharCategory::Whitespace || category == CharCategory::Eol {
        return Class::Blank;
    }
    if category == CharCategory::Word {
        return Class::Word;
    }
    if category == CharCategory::Punctuation {
        return Class::Punctuation;
    }
    return Class::Other;
}

/// Position of the first character of the line containing `position`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineStart(text: RopeSlice, position: number): number;
/// ```
pub fn line_start(text: RopeSlice, position: usize) -> usize {
    return text.line_to_char(text.char_to_line(position.min(text.len_chars())));
}

/// Position after the last visible character of the line containing `position`,
///  before LF or CRLF.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineEnd(text: RopeSlice, position: number): number;
/// ```
pub fn line_end(text: RopeSlice, position: usize) -> usize {
    let row = text.char_to_line(position.min(text.len_chars()));
    // What: `&text` lends the rope view to Helix's lookup, which takes it by reference.
    // Why: Helix knows which terminators the rope treats as line breaks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return lineEndCharIndex(text, row);
    // ```
    return line_end_char_index(&text, row);
}

/// End of the word at or after `position`:
///  skip blanks,
///  then one run of a single class.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordRight(text: RopeSlice, position: number): number;
/// ```
pub fn word_right(text: RopeSlice, position: usize) -> usize {
    let end = text.len_chars();
    let mut index = position.min(end);
    while index < end && class_at(text, index) == Class::Blank {
        index = next_grapheme_boundary(text, index);
    }
    if index >= end {
        return end;
    }
    let class = class_at(text, index);
    while index < end && class_at(text, index) == class {
        index = next_grapheme_boundary(text, index);
    }
    return index;
}

/// Start of the word at or before `position`:
///  skip blanks backwards,
///  then one run of a single class.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordLeft(text: RopeSlice, position: number): number;
/// ```
pub fn word_left(text: RopeSlice, position: usize) -> usize {
    let mut index = position.min(text.len_chars());
    // The grapheme before `index` starts at the previous boundary; its first character decides the class.
    while index > 0 && class_at(text, prev_grapheme_boundary(text, index)) == Class::Blank {
        index = prev_grapheme_boundary(text, index);
    }
    if index == 0 {
        return 0;
    }
    let class = class_at(text, prev_grapheme_boundary(text, index));
    while index > 0 && class_at(text, prev_grapheme_boundary(text, index)) == class {
        index = prev_grapheme_boundary(text, index);
    }
    return index;
}

/// New caret position after one geometry-free movement from `head`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moved(text: RopeSlice, head: number, motion: Motion): number;
/// ```
pub fn moved(text: RopeSlice, head: usize, motion: Motion) -> usize {
    if motion == Motion::Left {
        return prev_grapheme_boundary(text, head);
    }
    if motion == Motion::Right {
        return next_grapheme_boundary(text, head);
    }
    if motion == Motion::WordLeft {
        return word_left(text, head);
    }
    if motion == Motion::WordRight {
        return word_right(text, head);
    }
    if motion == Motion::LineStart {
        return line_start(text, head);
    }
    if motion == Motion::LineEnd {
        return line_end(text, head);
    }
    if motion == Motion::DocumentStart {
        return 0;
    }
    return text.len_chars();
}

/// The word,
///  punctuation run,
///  or blank run around `position`,
///  limited to the visible text of its line.
///
/// `position` is a caret boundary,
///  so it touches two characters.
///  A word character on either side wins,
/// the following one first;
///  otherwise the run of the following character is used,
/// or the preceding one at the end of a line.
///  An empty line yields an empty range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordRange(text: RopeSlice, position: number): [number, number];
/// ```
pub fn word_range(text: RopeSlice, position: usize) -> (usize, usize) {
    let bounded = position.min(text.len_chars());
    let first = line_start(text, bounded);
    let last = line_end(text, bounded);
    let caret = bounded.clamp(first, last);
    if first == last {
        return (caret, caret);
    }
    // Start of the grapheme whose class names the unit.
    let mut probe = caret;
    let follows = caret < last;
    let precedes = caret > first;
    if !follows || (precedes && class_at(text, caret) != Class::Word) {
        let before = prev_grapheme_boundary(text, caret);
        if !follows || class_at(text, before) == Class::Word {
            probe = before;
        }
    }
    let class = class_at(text, probe);
    let mut start = probe;
    while start > first && class_at(text, prev_grapheme_boundary(text, start)) == class {
        start = prev_grapheme_boundary(text, start);
    }
    let mut end = probe;
    while end < last && class_at(text, end) == class {
        end = next_grapheme_boundary(text, end);
    }
    return (start, end);
}

/// The whole line containing `position`,
///  including its terminator when it has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineRange(text: RopeSlice, position: number): [number, number];
/// ```
pub fn line_range(text: RopeSlice, position: usize) -> (usize, usize) {
    let bounded = position.min(text.len_chars());
    let row = text.char_to_line(bounded);
    let start = text.line_to_char(row);
    // Past the last line this is the end of the text, so a final line without a terminator stays whole.
    let end = text.line_to_char((row + 1).min(text.len_lines()));
    return (start, end);
}
