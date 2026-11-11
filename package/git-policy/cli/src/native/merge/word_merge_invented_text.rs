//! What:
//!  Which text a word merge of one line region may not write:
//! a word written twice in a row that no side writes twice,
//! and,
//!  in a region that spans more than one line on some side,
//! a line no side holds.
//! Why:
//!   A word merge takes an insertion from each side,
//! and two sides inserting at one place is the shape that writes text neither side wrote in either sense:
//!  merge `ef43afe9f9` shipped `return return without_variable == *selected;`
//! and the record merge `31f72ecf12` left ran one body into the other mid-sentence.
//! A conflict a person resolves beats wrong text that lands silently,
//! so a region that would write such text keeps both sides instead
//! (`doc/planning/cli-git-merge-auto-resolution.md`,
//! "Measured again on 2026-11-10").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refused = inventsDoubledWord(...) || inventsLine(...); // one line region of a three-way merge
//! ```

use crate::merge::word_merge::{LINE_END, Level, is_line_end, token_ranges};
/// A byte range of a text.
use std::ops::Range;

/// What:
///  Whether two equal words stand next to each other somewhere in `text`.
/// Why:
///   A word written twice in a row is the shape a word merge leaves when it takes an insertion
/// from each side at one place;
/// a side that writes it itself is a side that meant it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repeatsWord(text: Uint8Array, word: Uint8Array): boolean;
/// ```
fn repeats_word(text: &[u8], word: &[u8]) -> bool {
    let ranges: Vec<Range<usize>> = token_ranges(text, Level::Word);
    for pair in ranges.windows(2) {
        if text[pair[0].clone()] != *word || text[pair[1].clone()] != *word {
            continue;
        }
        let between: &[u8] = &text[pair[0].end..pair[1].start];
        if !between.is_empty() && between.iter().all(u8::is_ascii_whitespace) {
            return true;
        }
    }
    return false;
}

/// What:
///  The words `text` writes twice in a row,
/// each once.
/// Why:
///   They are the only words of a merge that could be one side's insertion of a word the other
/// side also inserted,
/// so a merge is checked against a side for these words alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function doubledWords(text: Uint8Array): Uint8Array[];
/// ```
fn doubled_words(text: &[u8]) -> Vec<&[u8]> {
    let ranges: Vec<Range<usize>> = token_ranges(text, Level::Word);
    let mut found: Vec<&[u8]> = Vec::new();
    for pair in ranges.windows(2) {
        let first: &[u8] = &text[pair[0].clone()];
        if first != &text[pair[1].clone()] {
            continue;
        }
        let between: &[u8] = &text[pair[0].end..pair[1].start];
        if between.is_empty() || !between.iter().all(u8::is_ascii_whitespace) || found.contains(&first) {
            continue;
        }
        found.push(first);
    }
    return found;
}

/// What:
///  Whether a merged region writes a word twice in a row that no side writes twice in a row.
/// Why:
///   Merge `ef43afe9f9` shipped `return return without_variable == *selected;`
/// (`package/git-policy/cli/src/native/policy/policy_forbidden_strings.rs`):
/// ours began a line with `return`,
/// theirs inserted `return` inside the closure of the line it kept,
/// and the word pass wrote both.
/// No side writes two words in a row that the other side's text can explain,
/// so a doubled word no side wrote is the pass's own invention and keeps the region in conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inventsDoubledWord(base, ours, theirs, merged: Uint8Array): boolean;
/// ```
fn invents_a_doubled_word(base: &[u8], ours: &[u8], theirs: &[u8], merged: &[u8]) -> bool {
    for word in doubled_words(merged) {
        if repeats_word(base, word) || repeats_word(ours, word) || repeats_word(theirs, word) {
            continue;
        }
        return true;
    }
    return false;
}

/// What:
///  The lines of `text`,
/// each without its line end.
/// Why:
///   Lines are compared as a person reads them,
/// so a side that ends its last line and a merge that does not (or the reverse)
/// are the same line here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linesWithoutLineEnd(text: Uint8Array): Uint8Array[];
/// ```
fn lines_without_line_end(text: &[u8]) -> Vec<&[u8]> {
    let mut found: Vec<&[u8]> = Vec::new();
    for line in text.split_inclusive(is_line_end) {
        found.push(match line.split_last() {
            Some((last, head)) if *last == LINE_END => head,
            _ => line,
        });
    }
    return found;
}

/// What:
///  Whether a merged region that spans lines on some side writes a line no side holds.
/// Why:
///   A region of one line on every side is the case the pass exists for:
/// two edits of one line become one line,
/// which no side wrote and which is the answer.
/// A region that spans lines is where an edit of whole clauses sits:
/// writing a line no side holds there joins one side's sentence to the other side's,
/// the shape the owner called "one body ran into the other mid-sentence".
/// A line that is only whitespace is not judged,
/// because a merge of two blank-line layouts has no meaning to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inventsLine(base, ours, theirs, merged: Uint8Array): boolean;
/// ```
fn invents_a_line(base: &[u8], ours: &[u8], theirs: &[u8], merged: &[u8]) -> bool {
    let sides: [&[u8]; 3] = [base, ours, theirs];
    let mut widest: usize = 0;
    for text in sides {
        widest = widest.max(lines_without_line_end(text).len());
    }
    if widest <= 1 {
        return false;
    }
    let mut known: Vec<&[u8]> = Vec::new();
    for text in sides {
        known.extend(lines_without_line_end(text));
    }
    for line in lines_without_line_end(merged) {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if !known.contains(&line) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a word merge of one line region wrote text the pass may not write,
/// which keeps the region in conflict.
/// Why:
///   See the two checks;
/// a conflict a person resolves beats wrong text that lands silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = inventsDoubledWord(...) || inventsLine(...);
/// ```
pub(super) fn refuses_word_merge(base: &[u8], ours: &[u8], theirs: &[u8], merged: &[u8]) -> bool {
    return invents_a_doubled_word(base, ours, theirs, merged) || invents_a_line(base, ours, theirs, merged);
}
