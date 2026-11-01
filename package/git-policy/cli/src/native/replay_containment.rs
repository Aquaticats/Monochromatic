//! What:
//!  The context-tolerant half of replay's subsumption:
//!       whether each prepared change is the landed change extended by the prepared commit's own edit on one side.
//! Why:
//!  Strict reverse application rejects a prepared file holding the landed edit plus an own edit next to it,
//!       because that own edit sits in the landed hunk's context.
//!       Reading both changes as zero-context hunks in base coordinates accepts exactly the one-sided extensions
//!       and still rejects own edits on both sides and modify/delete conflicts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! containsLandedChange({ landed, prepared, base }): boolean
//! ```

/// Zero-context hunks and base lines from `replay_patch`.
use super::replay_patch::{PatchHunk, PatchLine};

/// What:
///  One zero-context hunk in base line coordinates:
///       the first replaced base line (zero-based), the line after the last replaced one, and the new lines.
/// Why:
///   Covering and extension comparisons happen in base coordinates, not in each hunk's own numbering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BaseChange = { start: number; end: number; added: readonly string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct BaseChange {
    /// First replaced base line, zero-based; for an insertion, the line it goes before.
    start: usize,
    /// Line after the last replaced base line.
    end: usize,
    /// New lines, each with its newline.
    added: Vec<Vec<u8>>,
}

/// What:
///  Converts a zero-context hunk to base coordinates.
/// Why:
///   A range with count `0` names the line after which lines are inserted,
///       so an insertion's start is the header's start and not one less.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// baseChange(hunk);
/// ```
fn base_change(hunk: &PatchHunk) -> BaseChange {
    let removed: usize = hunk.lines.iter().filter(|line| { return line.op == b'-'; }).count();
    let start: usize = if removed == 0 {
        usize::try_from(hunk.old_start).unwrap_or(0)
    } else {
        usize::try_from(hunk.old_start - 1).unwrap_or(0)
    };
    return BaseChange {
        start,
        end: start + removed,
        added: hunk
            .lines
            .iter()
            .filter(|line: &&PatchLine| { return line.op == b'+'; })
            .map(|line| { return line.text.clone(); })
            .collect(),
    };
}

/// What:
///  Whether `lines` appear in order, not necessarily adjacent, within `within`:
///       one greedy pass.
/// Why:
///   Own lines may sit between landed lines when the prepared hunk covers exactly the landed range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isSubsequence({ lines, within });
/// ```
fn is_subsequence(lines: &[Vec<u8>], within: &[Vec<u8>]) -> bool {
    // `mut` lets the greedy pass advance over `within` once.
    let mut matched: usize = 0;
    for line in within {
        if matched < lines.len() && lines[matched] == *line {
            matched += 1;
        }
    }
    return matched == lines.len();
}

/// What:
///  Whether one prepared hunk's new text is the covered landed hunks extended on at most one side.
/// Why:
///   The landed text of the region is the landed lines joined by the base lines between them;
///       the prepared text must hold it as a prefix or suffix, or the exact range with own lines between.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// regionContains({ prepared, landed, base });
/// ```
fn region_contains(prepared: &BaseChange, landed: &[BaseChange], base: &[Vec<u8>]) -> bool {
    let (Some(first), Some(last)) = (landed.first(), landed.last()) else {
        return false;
    };
    // `mut` lets the expected text be collected in order.
    let mut expected: Vec<Vec<u8>> = Vec::new();
    for (index, change) in landed.iter().enumerate() {
        expected.extend(change.added.iter().cloned());
        if let Some(next) = landed.get(index + 1) {
            // An inverted or out-of-range gap is empty, as JavaScript's `slice` returns it.
            expected.extend(base.get(change.end..next.start).unwrap_or_default().iter().cloned());
        }
    }
    let anchored_start: bool = prepared.start == first.start;
    let anchored_end: bool = prepared.end == last.end;
    // A landed deletion replaced by other text over exactly its lines is a modify/delete conflict, not an extension.
    if expected.is_empty() && anchored_start && anchored_end {
        return prepared.added.is_empty();
    }
    if expected.len() > prepared.added.len() {
        return false;
    }
    // Over exactly the landed range, own lines may also sit between landed lines,
    //  as long as one end stays the landed one.
    if anchored_start && anchored_end {
        let ends_match: bool = prepared.added.first() == expected.first()
            || prepared.added.last() == expected.last();
        return ends_match && is_subsequence(&expected, &prepared.added);
    }
    let prefix_matches: bool = anchored_start
        && prepared.added[..expected.len()] == expected[..];
    let suffix_start: usize = prepared.added.len() - expected.len();
    let suffix_matches: bool = anchored_end && prepared.added[suffix_start..] == expected[..];
    return prefix_matches || suffix_matches;
}

/// What:
///  Whether the prepared change contains the landed change region by region:
///       every landed hunk lies inside one prepared hunk that extends it on at most one side.
/// Why:
///   This is the one-sided extension of subsumption, which strict reverse application cannot express.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// containsLandedChange({ landed, prepared, base });
/// ```
pub fn contains_landed_change(
    landed: &[PatchHunk],
    prepared: &[PatchHunk],
    base: &[Vec<u8>],
) -> bool {
    let prepared_changes: Vec<BaseChange> = prepared.iter().map(base_change).collect();
    // Each landed change with the index of the prepared change covering it, if any.
    let covered: Vec<(Option<usize>, BaseChange)> = landed
        .iter()
        .map(base_change)
        .map(|change| {
            let cover: Option<usize> = prepared_changes
                .iter()
                .position(|candidate| { return candidate.start <= change.start && change.end <= candidate.end; });
            return (cover, change);
        })
        .collect();
    if covered.iter().any(|(cover, _)| { return cover.is_none(); }) {
        return false;
    }
    return prepared_changes.iter().enumerate().all(|(index, change)| {
        let inside: Vec<BaseChange> = covered
            .iter()
            .filter(|(cover, _)| { return *cover == Some(index); })
            .map(|(_, landed_change)| { return landed_change.clone(); })
            .collect();
        return inside.is_empty() || region_contains(change, &inside, base);
    });
}

#[cfg(test)]
#[path = "replay_containment_tests.rs"]
mod tests;
