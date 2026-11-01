//! What:
//!  Owned Markdown byte-to-line and UTF-16-column indexes.
//! Why:
//!  Diagnostics keep original byte addresses while columns follow the established Markdown consumer contract.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Build line starts and sparse Unicode-width adjustments once per source.
//! ```

/// Import the common diagnostic wire span.
use crate::diagnostic::Span;
/// Import the parser's CR,
///  LF and CRLF boundary iterator instead of inventing another newline grammar.
use satteri_arena::line_ending_iter;

/// What:
///  Cumulative difference between UTF-8 bytes and UTF-16 units at a multibyte character end.
/// Why:
///  Sparse width changes let each position lookup use binary search without rescanning a long line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WidthChange = { end: number; extraBytes: number };
/// ```
struct WidthChange {
    /// Original byte offset immediately after this character.
    end: usize,
    /// Extra UTF-8 bytes accumulated before this offset.
    extra_bytes: usize,
}

/// What:
///  An owned position index with no references into its source string.
/// Why:
///  The parsed document can own its source and index without a self-referential lifetime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MarkdownPositions { starts: number[]; widths: WidthChange[]; length: number }
/// ```
pub(crate) struct MarkdownPositions {
    /// Original byte offset of each line start;
    ///  a leading BOM precedes the first content position.
    starts: Vec<usize>,
    /// Sparse cumulative Unicode-width changes.
    widths: Vec<WidthChange>,
    /// Original source byte length.
    length: usize,
}

/// What:
///  The original byte offset immediately after one width change,
///  the key `binary_search_by_key` compares the searched boundary against.
/// Why:
///  A named key function replaces the closure that would borrow the searched boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function widthEnd(entry: WidthChange): number;
/// ```
fn width_end(entry: &WidthChange) -> usize {
    return entry.end;
}

/// What:
///  How many width changes end at or before a byte boundary.
/// Why:
///  A named search over the sparse width index keeps `extra_at` free of a closure borrowing `offset`.
///  `binary_search_by_key` reports `Ok(found)` for the entry ending exactly at the boundary,
///  so that entry and everything before it count,
///  while `Err(insertion)` is already the count of earlier entries;
///  the ends are distinct because each one closes a different character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function widthsEndedAtOrBefore(widths: readonly WidthChange[], offset: number): number;
/// ```
fn widths_ended_at_or_before(widths: &[WidthChange], offset: usize) -> usize {
    return match widths.binary_search_by_key(&offset, width_end) {
        // What: `Ok(found)` means one line of widths ends exactly at `offset`:
        //  that entry and every earlier one count.
        Ok(found) => found + 1,
        // What: `Err(insertion)` is already how many entries end earlier.
        Err(insertion) => insertion,
    };
}

/// What:
///  How many line starts lie at or before a byte boundary.
/// Why:
///  A named search over the sorted line starts keeps `span` free of a closure borrowing `offset`.
///  The fold of `binary_search` counts an exact line start itself,
///  and the starts are distinct positions by construction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startsAtOrBefore(starts: readonly number[], offset: number): number;
/// ```
fn starts_at_or_before(starts: &[usize], offset: usize) -> usize {
    return match starts.binary_search(&offset) {
        // What: `Ok(found)` means one line starts exactly at `offset`:
        //  that start and every earlier one count.
        Ok(found) => found + 1,
        // What: `Err(insertion)` is already how many starts lie earlier.
        Err(insertion) => insertion,
    };
}

/// What:
///  Construct and query Markdown positions.
/// Why:
///  A leading BOM is excluded from columns but retained in original-source byte offsets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // span(offset, length) resolves original-source coordinates.
/// ```
impl MarkdownPositions {
    /// Build newline and Unicode-width indexes from the exact input.
    pub(crate) fn new(source: &str) -> MarkdownPositions {
        let bom = if source.starts_with('\u{feff}') { 3 } else { 0 };
        let mut starts = vec![bom];
        for ending in line_ending_iter(source.as_bytes()) {
            starts.push(ending + 1);
        }
        let mut widths = Vec::new();
        let mut extra_bytes = 0;
        for (offset, character) in source.char_indices() {
            if offset < bom {
                continue;
            }
            let extra = character.len_utf8() - character.len_utf16();
            if extra != 0 {
                extra_bytes += extra;
                widths.push(WidthChange {
                    end: offset + character.len_utf8(),
                    extra_bytes,
                });
            }
        }
        return MarkdownPositions {
            starts,
            widths,
            length: source.len(),
        };
    }

    /// Read cumulative width adjustment through this byte boundary.
    fn extra_at(&self, offset: usize) -> usize {
        // What: `count` is how many width changes end at or before `offset`;
        //  `as_slice` lends the owned list to the named search.
        let count = widths_ended_at_or_before(self.widths.as_slice(), offset);
        if count == 0 {
            return 0;
        }
        return self.widths[count - 1].extra_bytes;
    }

    /// Resolve a range without converting its byte length into character units.
    pub(crate) fn span(&self, requested_offset: usize, length: usize) -> Span {
        let offset: usize = requested_offset.min(self.length);
        // What: `count` is how many line starts lie at or before `offset`.
        let count = starts_at_or_before(self.starts.as_slice(), offset);
        let index = count.saturating_sub(1);
        let start = self.starts[index];
        let byte_column = offset.saturating_sub(start);
        let extra = self.extra_at(offset).saturating_sub(self.extra_at(start));
        return Span {
            offset,
            length: length.min(self.length - offset),
            line: index + 1,
            column: byte_column.saturating_sub(extra) + 1,
        };
    }
}

/// Position regressions are verification-only.
#[cfg(test)]
#[path = "markdown_positions_tests.rs"]
mod tests;
