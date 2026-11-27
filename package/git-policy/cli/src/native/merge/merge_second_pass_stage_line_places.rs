//! What:
//!  Where a run of lines sits in one stage of a file:
//!  the byte offset of a line's text at or after a place,
//!  and the lines counted between two places.
//! Why:
//!   A region's place in its stage feeds the report's line numbers only,
//!  never the merged bytes,
//!  and a report over many parts costs the stage's lines once;
//!  one file owns the line ending both readings turn on,
//!  so the cut of an answer and the places of its regions cannot disagree about a line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const at = offsetFromLine(text, needle, from);
//! ```

/// The byte that ends a line.
pub(super) const LINE_END: u8 = b'\n';

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
