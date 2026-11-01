//! Relate accepted in-file matches to the reading selection without owning either.

/// Match ranges and their shared list come from the single matching function.
use crate::find::{FindMatches, FindRange, FindRanges};
/// Accepted results keep the tag of the request that produced them.
use crate::find_worker::FindIdentity;
/// What:
///  `Arc` shares one immutable allocation between owners.
/// Why:
///  A stale result returns an empty list without copying or exposing its outdated ranges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const empty: ReadonlyArray<FindRange> = [];
/// ```
use std::sync::Arc;

/// What:
///  Whether `range` starts before `limit`.
/// Why:
///  A sorted list of ranges is searched for the first one that does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startsBefore = (range: FindRange, limit: number) => range.start < limit;
/// ```
fn starts_before(range: &FindRange, limit: usize) -> bool {
    return range.start < limit;
}

/// What:
///  Whether `range` ends at or before `limit`.
/// Why:
///  A sorted list of ranges is searched for the first one that does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endsBy = (range: FindRange, limit: number) => range.end <= limit;
/// ```
fn ends_by(range: &FindRange, limit: usize) -> bool {
    return range.end <= limit;
}

/// What:
///  The index of the first range for which `before` is false,
///  or the length when `before` holds for all of them.
///  The ranges are sorted,
///  so `before` holds for a prefix and a halving search finds the end of it.
///  `fn(&FindRange, usize) -> bool` is a plain function pointer that receives `limit` explicitly.
/// Why:
///  Up to ten thousand ranges are consulted on every selection change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstNotBefore(ranges: FindRange[], limit: number, before: (range: FindRange, limit: number) => boolean): number;
/// ```
fn first_not_before(
    ranges: &[FindRange],
    limit: usize,
    before: fn(&FindRange, usize) -> bool,
) -> usize {
    let mut low = 0;
    let mut high = ranges.len();
    // Each step halves the candidates between `low` and `high`.
    while low < high {
        let middle = low + (high - low) / 2;
        if before(&ranges[middle], limit) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low;
}

/// What:
///  `&[FindRange]` borrows a sorted list;
///  `Option<usize>` is an index or nothing.
/// Why:
///  The active match is derived from the selection,
///  so reloads and clicks need no second cursor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function active(ranges: FindRange[], start: number, end: number): number | undefined;
/// ```
pub fn active(ranges: &[FindRange], start: usize, end: usize) -> Option<usize> {
    // What: `first_not_before` binary-searches a sorted list; `starts_before` is the named test it applies.
    // Why: Up to ten thousand ranges are consulted on every selection change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = lowerBound(ranges, range => range.start < start);
    // ```
    let index = first_not_before(ranges, start, starts_before);
    // What: `get` returns `Some(&range)` inside the list or `None` past its end.
    // Why: A selection after the last match must not index out of bounds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const candidate = ranges[index]; if (candidate === undefined) return undefined;
    // ```
    let Some(candidate) = ranges.get(index) else {
        // None means the selection is not exactly one match.
        return None;
    };
    if candidate.start == start && candidate.end == end {
        // Some wraps the present index.
        return Some(index);
    }
    return None;
}

/// First match starting at or after `origin`,
///  wrapping to the first match of the document.
/// Incremental typing passes the selection start;
///  Enter passes the selection end.
pub fn at_or_after(ranges: &[FindRange], origin: usize) -> Option<usize> {
    if ranges.is_empty() {
        return None;
    }
    let index = first_not_before(ranges, origin, starts_before);
    if index == ranges.len() {
        return Some(0);
    }
    return Some(index);
}

/// Last match ending at or before `origin`,
///  wrapping to the last match of the document.
/// Shift+Enter passes the selection start.
pub fn at_or_before(ranges: &[FindRange], origin: usize) -> Option<usize> {
    if ranges.is_empty() {
        return None;
    }
    let index = first_not_before(ranges, origin, ends_by);
    if index == 0 {
        return Some(ranges.len() - 1);
    }
    return Some(index - 1);
}

/// What:
///  The returned `&[FindRange]` borrows a contiguous part of the input list.
/// Why:
///  Painting shapes rectangles only for matches intersecting the materialized rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function visible(ranges: FindRange[], start: number, end: number): FindRange[];
/// ```
pub fn visible(ranges: &[FindRange], start: usize, end: usize) -> &[FindRange] {
    let first = first_not_before(ranges, start, ends_by);
    let last = first_not_before(ranges, end, starts_before);
    if last <= first {
        // What: `&ranges[0..0]` borrows an empty part of the same list.
        // Why: An inverted or empty window has no visible matches.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [];
        // ```
        return &ranges[0..0];
    }
    return &ranges[first..last];
}

/// Visible count text and its spoken form;
///  the two strings always describe the same state.
/// The default is empty text:
///  no find text,
///  so nothing to count.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FindStatus {
    /// Compact "active/total" text,
    ///  or "No matches".
    pub label: String,
    /// Full sentence for accessibility tools.
    pub detail: String,
    /// True only for a completed search without matches.
    pub no_match: bool,
}

/// Describe accepted matches relative to the optional active index.
pub fn status(matches: &FindMatches, active_index: Option<usize>) -> FindStatus {
    let total = matches.ranges.len();
    if total == 0 {
        // What: `to_string` copies a literal into an owned `String`.
        // Why: The status outlives this function and is handed to the window.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { label: 'No matches', detail: 'No matches', noMatch: true };
        // ```
        return FindStatus {
            label: "No matches".to_string(),
            detail: "No matches".to_string(),
            no_match: true,
        };
    }
    let mut suffix = "";
    let mut more = "";
    if matches.truncated {
        suffix = "+";
        more = " or more";
    }
    // What: `if let Some(index)` runs only when an active index is present.
    // Why: A selection away from every match still reports the total.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (activeIndex !== undefined) return { label: `${activeIndex + 1}/${total}`, ... };
    // ```
    if let Some(index) = active_index {
        let number = index + 1;
        return FindStatus {
            label: format!("{number}/{total}{suffix}"),
            detail: format!("Match {number} of {total}{more}"),
            no_match: false,
        };
    }
    return FindStatus {
        label: format!("0/{total}{suffix}"),
        detail: format!("{total}{more} matches, none selected"),
        no_match: false,
    };
}

/// Accepted matches remember the file generation,
///  revision,
///  and query that produced them.
pub struct FindResults {
    /// Tag copied from the accepted worker reply.
    identity: FindIdentity,
    /// Shared ranges and truncation state.
    matches: FindMatches,
}

/// Results are read only through identity checks,
///  so stale positions cannot be painted or navigated.
impl FindResults {
    /// Record matches together with the identity of the reply that carried them.
    pub fn new(identity: FindIdentity, matches: FindMatches) -> Self {
        return Self { identity, matches };
    }

    /// Lend matches while their positions describe the displayed document.
    /// The query may be one edit behind:
    ///  its highlights stay until the newer reply replaces them.
    pub fn positioned(&self, file: u64, revision: u64) -> Option<&FindMatches> {
        if self.identity.file == file && self.identity.revision == revision {
            return Some(&self.matches);
        }
        return None;
    }

    /// Lend matches only for the exact displayed file,
    ///  revision,
    ///  and current find text.
    /// Enter and Shift+Enter use this,
    ///  so they never step through matches of a superseded query.
    pub fn navigable(&self, wanted: FindIdentity) -> Option<&FindMatches> {
        // What: `&wanted` lends the copied tag to the three-part comparison.
        // Why: One comparison function decides staleness for worker replies and for navigation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (isCurrent(this.identity, wanted)) return this.matches;
        // ```
        if self.identity.is_current(&wanted) {
            return Some(&self.matches);
        }
        return None;
    }
}

/// What:
///  `&Option<FindResults>` borrows results that may be absent;
///  the answer borrows from them.
/// Why:
///  Painting and the count text after a reload or file switch must not use outdated positions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function positionedMatches(results: FindResults | undefined, file: number, revision: number): FindMatches | undefined;
/// ```
pub fn positioned_matches(
    results: &Option<FindResults>,
    file: u64,
    revision: u64,
) -> Option<&FindMatches> {
    let Some(accepted) = results else {
        return None;
    };
    return accepted.positioned(file, revision);
}

/// Matches usable for Enter and Shift+Enter;
///  absent while a newer query,
///  revision,
///  or file is pending.
pub fn navigable_matches(
    results: &Option<FindResults>,
    wanted: FindIdentity,
) -> Option<&FindMatches> {
    let Some(accepted) = results else {
        return None;
    };
    return accepted.navigable(wanted);
}

/// Ranges to paint for the displayed document;
///  stale or absent results paint nothing.
pub fn paint_ranges(results: &Option<FindResults>, file: u64, revision: u64) -> FindRanges {
    if let Some(matches) = positioned_matches(results, file, revision) {
        // What: `Arc::clone` copies the pointer, not the ranges.
        // Why: The frame stamp keeps the same allocation and can compare identity cheaply.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return matches.ranges;
        // ```
        return Arc::clone(&matches.ranges);
    }
    return Arc::from([]);
}
