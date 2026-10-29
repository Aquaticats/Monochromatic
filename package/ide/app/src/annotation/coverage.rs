//! The lines of one text revision that answers for hints have asked about.
//!
//! The reader asks for hints around the visible lines only.
//!  A later view whose request would ask lines that answers already covered
//!  gets no new request,
//!  because the store holds the hints of those lines already.

/// The text revision an answer describes.
use crate::language::identity::DocumentStamp;
/// Line ranges are half-open:
///  the first line asked,
///  and the line the request ended before.
use std::ops::Range;

/// What:
///  The revision the ranges belong to and the lines answers asked for in it,
///  as sorted ranges that neither overlap nor touch.
///  `Option<DocumentStamp>` is "a revision,
///  or none yet".
/// Why:
///  Only an answer that was stored counts,
///  so a request that is still pending,
///  failed,
///  or was cancelled adds nothing.
///  Lines asked for in another revision say nothing about this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Coverage = { stamp?: DocumentStamp; ranges: [number, number][] };
/// ```
#[derive(Debug, Default)]
pub(super) struct Coverage {
    /// The revision `ranges` describes.
    stamp: Option<DocumentStamp>,
    /// Asked lines,
    ///  ordered by start,
    ///  with overlapping and touching ranges joined.
    ranges: Vec<Range<usize>>,
}

/// The first line of a range,
///  the order in which covered ranges are kept.
fn first_line_of(range: &Range<usize>) -> usize {
    return range.start;
}

/// Add asked lines,
///  and ask whether given lines were asked.
impl Coverage {
    /// What:
    ///  Take the lines of a stored answer for `stamp` into the covered lines;
    ///  an answer for another revision first forgets what the earlier revision covered.
    ///  `Range<usize>` is a half-open line range.
    /// Why:
    ///  Edits,
    ///  reloads,
    ///  and other files change what a line holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// note(stamp: DocumentStamp, lines: [number, number]): void;
    /// ```
    pub(super) fn note(&mut self, stamp: DocumentStamp, lines: Range<usize>) {
        if self.stamp != Some(stamp) {
            self.ranges.clear();
            self.stamp = Some(stamp);
        }
        if lines.is_empty() {
            return;
        }
        self.ranges.push(lines);
        self.ranges.sort_by_key(first_line_of);
        let mut joined: Vec<Range<usize>> = Vec::new();
        for range in self.ranges.drain(..) {
            match joined.last_mut() {
                Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
                _ => joined.push(range),
            }
        }
        self.ranges = joined;
    }

    /// What:
    ///  Whether one covered range holds all of `lines` in the revision `stamp`;
    ///  an empty range is never covered.
    /// Why:
    ///  A request for lines that are asked in pieces by two answers is covered too,
    ///  because the pieces were joined.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// covers(stamp: DocumentStamp, lines: [number, number]): boolean;
    /// ```
    pub(super) fn covers(&self, stamp: DocumentStamp, lines: Range<usize>) -> bool {
        if self.stamp != Some(stamp) || lines.is_empty() {
            return false;
        }
        for range in &self.ranges {
            if range.start <= lines.start && lines.end <= range.end {
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  Take the lines `lines` out of the covered lines.
    /// Why:
    ///  A stored answer replaces the hints of the lines it asked about,
    ///  so an answer that put no hint on them leaves nothing there that an earlier answer covered.
    ///  Lines of an earlier revision lose them too,
    ///  which changes nothing because that revision is never asked about again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// uncover(lines: [number, number]): void;
    /// ```
    pub(super) fn uncover(&mut self, lines: Range<usize>) {
        let mut kept: Vec<Range<usize>> = Vec::new();
        for range in self.ranges.drain(..) {
            if range.end <= lines.start || lines.end <= range.start {
                kept.push(range);
                continue;
            }
            if range.start < lines.start {
                kept.push(range.start..lines.start);
            }
            if lines.end < range.end {
                kept.push(lines.end..range.end);
            }
        }
        self.ranges = kept;
    }

    /// Forget every covered line.
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}
