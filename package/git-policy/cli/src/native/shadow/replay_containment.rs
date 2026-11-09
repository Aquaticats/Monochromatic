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
use crate::replay_patch::PatchHunk;

/// What:
///  One zero-context hunk in base line coordinates:
///       the first replaced base line (zero-based),
/// the line after the last replaced one,
/// and the new lines.
/// Why:
///   Covering and extension comparisons happen in base coordinates,
/// not in each hunk's own numbering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BaseChange = { start: number; end: number; added: readonly string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct BaseChange {
    /// First replaced base line,
    /// zero-based;
    /// for an insertion,
    /// the line it goes before.
    start: usize,
    /// Line after the last replaced base line.
    end: usize,
    /// New lines,
    /// each with its newline.
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
    let mut removed: usize = 0;
    let mut added: Vec<Vec<u8>> = Vec::new();
    for line in &hunk.lines {
        if line.op == b'-' {
            removed += 1;
        }
        if line.op == b'+' {
            added.push(line.text.clone());
        }
    }
    let start: usize = if removed == 0 {
        usize::try_from(hunk.old_start).unwrap_or(0)
    } else {
        usize::try_from(hunk.old_start - 1).unwrap_or(0)
    };
    return BaseChange {
        start,
        end: start + removed,
        added,
    };
}

/// What:
///  Whether `lines` appear in order,
/// not necessarily adjacent,
/// within `within`:
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
///       the prepared text must hold it as a prefix or suffix,
/// or the exact range with own lines between.
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
///  Whether a prepared change ends before a landed change begins,
///  so it cannot cover it and no earlier prepared change can either.
/// Why:
///   A prepared change that stops where a landed change of one line or more begins holds none
///  of its lines;
///  a prepared insertion at that point holds all of a landed insertion there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// candidate.end < change.start || (candidate.end === change.start && change.end > change.start)
/// ```
fn ends_before(candidate: &BaseChange, change: &BaseChange) -> bool {
    return candidate.end < change.start || (candidate.end == change.start && change.end > change.start);
}

/// What:
///  Whether the prepared change contains the landed change region by region:
///       every landed hunk lies inside one prepared hunk that extends it on at most one side.
/// Why:
///   This is the one-sided extension of subsumption,
/// which strict reverse application cannot express.
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
    // The landed changes grouped under the prepared change covering each, in the order the
    //  landed list gave them.
    let mut grouped: Vec<Vec<BaseChange>> = vec![Vec::new(); prepared_changes.len()];
    // Both lists ascend by start coordinate and the hunks of one diff do not overlap, so one
    //  walk gives every landed change its cover: a prepared change that ends before the landed
    //  change begins cannot cover it,
    //  nor can a prepared change ending exactly where a landed change of at least one line
    //  begins,
    //  and no earlier one can cover it either (`AGENTS.md` `RG2`).
    let mut prepared_index: usize = 0;
    for hunk in landed {
        let change: BaseChange = base_change(hunk);
        while prepared_index < prepared_changes.len() && ends_before(&prepared_changes[prepared_index], &change) {
            crate::collected_elements_looked_at::note_looked_at(1);
            prepared_index += 1;
        }
        crate::collected_elements_looked_at::note_looked_at(1);
        let cover: Option<usize> = match prepared_changes.get(prepared_index) {
            Some(candidate) if candidate.start <= change.start && change.end <= candidate.end => Some(prepared_index),
            _ => None,
        };
        let Some(index) = cover else {
            return false;
        };
        grouped[index].push(change);
    }
    for (index, change) in prepared_changes.iter().enumerate() {
        if !(grouped[index].is_empty() || region_contains(change, grouped[index].as_slice(), base)) {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
#[path = "replay_containment_tests.rs"]
mod tests;
