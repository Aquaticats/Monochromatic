//! What:
//!  Where a run of lines sits in one stage of a file:
//!  the byte range of the side of a region,
//!  the byte offset of a line's text at or after a place,
//!  and the lines counted between two places.
//! Why:
//!   A region's own text is the stage's own bytes,
//!  so a region that ends a file keeps the line end that file has there,
//!  and a region's place feeds the report's line numbers;
//!  a report over many parts costs the stage's lines once;
//!  one file owns the line ending every one of those readings turns on,
//!  so the cut of an answer and the places of its regions cannot disagree about a line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const span = placeInStage(text, side, from);
//! const at = offsetFromLine(text, needle, from);
//! ```

/// A byte range of a text.
use std::ops::Range;

/// The byte that ends a line.
pub(super) const LINE_END: u8 = b'\n';

/// The byte that comes before the line end of a file whose lines end with both.
pub(super) const CARRIAGE_RETURN: u8 = b'\r';

/// Whether one byte ends a line.
pub(super) fn is_line_end(byte: &u8) -> bool {
    return *byte == LINE_END;
}

/// What:
///  The byte offset of `needle` in `text` at or after `from`,
/// searched only at the start of a line.
/// Why:
///   A region's text is a run of whole lines of the stage it came from,
/// so a search that lands mid-line would name a place the region cannot start at.
/// An empty needle stands for a region with no line on that side,
/// which sits exactly where the search starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function offsetFromLine(text: Uint8Array, needle: Uint8Array, from: number): number | undefined;
/// ```
pub fn offset_from_line(
    text: &[u8],
    needle: &[u8],
    from: usize,
) -> Option<usize> {
    if needle.is_empty() {
        return Some(from);
    }
    if needle.len() > text.len() {
        return None;
    }
    let mut at: usize = 0;
    while at < text.len() {
        if at >= from && text.len() - at >= needle.len() && &text[at..at + needle.len()] == needle {
            return Some(at);
        }
        let offset: usize = text[at..].iter().position(is_line_end)?;
        at += offset + 1;
    }
    return None;
}

/// What:
///  Where the run of lines `side` sits in `text` at or after `from`:
///  the byte range of `text` it holds.
/// Why:
///   A region's side is a run of whole lines of its own stage,
/// and the line merge writes a line end after the last line of every side,
/// so a side whose last line is the stage's last line with no line end of its own
///  holds one byte fewer in the stage than in the answer.
/// Answering with the stage's own bytes there is what keeps a file's ending:
/// a region that ends a file writes no line end the file did not have,
///  and drops none it had.
/// A match that is not at a line start is no place a region can stand,
/// so it is not taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placeInStage(text: Uint8Array, side: Uint8Array, from: number): { start: number; end: number } | undefined;
/// ```
pub fn place_in_stage(
    text: &[u8],
    side: &[u8],
    from: usize,
) -> Option<Range<usize>> {
    if let Some(at) = offset_from_line(text, side, from) {
        return Some(at..at + side.len());
    }
    let without_newline: &[u8] = side.strip_suffix(&[LINE_END])?;
    // Git writes the file's own line ending after the last line of a side,
    // so a stage whose lines end with both bytes has that ending in the answer alone.
    let candidates: [&[u8]; 2] = [
        without_newline,
        without_newline
            .strip_suffix(&[CARRIAGE_RETURN])
            .unwrap_or(without_newline),
    ];
    for candidate in candidates {
        let Some(at): Option<usize> = text.len().checked_sub(candidate.len()) else {
            continue;
        };
        if at < from {
            continue;
        }
        if at != 0 && text[at - 1] != LINE_END {
            continue;
        }
        if text[at..] == *candidate {
            return Some(at..text.len());
        }
    }
    return None;
}

/// What:
///  How many lines of `text` end at or after byte `from` and before byte `to`.
/// Why:
///   A region's place never moves backwards,
/// so the lines above it are the ones counted for the region before it plus the ones between the two places,
/// and a report over many parts costs the stage's lines once:
/// counting them from the top of the stage for every part cost the stage per part.
/// A place past the end counts the lines up to the end,
/// as `lines_before` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesBetween = (text, from, to) => text.subarray(Math.min(from, text.length), Math.max(Math.min(from, text.length), Math.min(to, text.length))).filter((b) => b === 10).length;
/// ```
pub fn lines_between(
    text: &[u8],
    from: usize,
    to: usize,
) -> usize {
    let start: usize = from.min(text.len());
    let end: usize = to.min(text.len()).max(start);
    let mut lines: usize = 0;
    for byte in &text[start..end] {
        if *byte == LINE_END {
            lines += 1;
        }
    }
    return lines;
}

/// What:
///  How many lines of `text` come before byte `start`.
/// Why:
///   A region's line span inside the region,
/// moved to the file,
/// needs the number of lines the region starts after.
/// The report counts that incrementally from region to region (`lines_between`),
/// so this whole-file count is what the controls compare the carried one against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesBefore = (text, start) => text.subarray(0, start).filter((b) => b === 10).length;
/// ```
#[cfg(test)]
pub fn lines_before(
    text: &[u8],
    start: usize,
) -> usize {
    let mut lines: usize = 0;
    for byte in &text[..start.min(text.len())] {
        if *byte == LINE_END {
            lines += 1;
        }
    }
    return lines;
}
