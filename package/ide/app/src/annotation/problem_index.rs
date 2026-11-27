//! What one diagnostics snapshot is flattened into:
//!  one list ordered by start and end,
//!  the furthest end so far beside it,
//!  the sort keys both searches order by,
//!  and the halving search that skips a range ending before a window.

/// The severity order that puts the worst problem first.
use super::severity::rank;
/// An omitted severity is shown as a warning.
use super::severity::shown;
/// The flattened records the index is built over.
use super::{
    Mark,
    Problem,
};
/// The diagnostics snapshot whose groups are flattened.
use crate::language::diagnostics::DiagnosticsSnapshot;

/// What:
///  The sort key that orders problems by start,
///  then end.
/// Why:
///  Window and caret searches binary-search by start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const positionOrder = (problem: Problem): [number, number] => [problem.mark.start, problem.mark.end];
/// ```
fn position_order(problem: &Problem) -> (usize, usize) {
    return (problem.mark.start, problem.mark.end);
}

/// What:
///  The sort key that puts the worst problem first.
///  `&&Problem` is what a sort over a list of lent problems hands its key function.
/// Why:
///  The stable sort keeps position order among equal severities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const severityOrder = (problem: Problem): number => rank(problem.mark.severity);
/// ```
pub(super) fn severity_order(problem: &&Problem) -> u8 {
    return rank(problem.mark.severity);
}

/// What:
///  The index of the first entry of `reach` that is not below `threshold`.
///  `reach` never decreases,
///  so a halving search finds it.
/// Why:
///  Window and caret searches skip every diagnostic that ends before the window without reading each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstReaching(reach: number[], threshold: number): number;
/// ```
pub(super) fn first_reaching(
    reach: &[usize],
    threshold: usize,
) -> usize {
    let mut low = 0;
    let mut high = reach.len();
    // Each step halves the candidates between `low` and `high`.
    while low < high {
        let middle = low + (high - low) / 2;
        if reach[middle] < threshold {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low;
}

/// What:
///  Flatten every group of `diagnostics` into one list ordered by start,
///  then end,
///  with the furthest end so
///       far beside it;
///  `Option<&DiagnosticsSnapshot>` lends a snapshot or nothing;
///  the answer is a pair (tuple).
/// Why:
///  Groups arrive per source,
///  so their union is sorted once per snapshot instead of on every frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexed(diagnostics?: DiagnosticsSnapshot): [Problem[], number[]];
/// ```
pub(super) fn indexed(diagnostics: Option<&DiagnosticsSnapshot>) -> (Vec<Problem>, Vec<usize>) {
    let mut problems = Vec::new();
    // What: `if let Some(snapshot) = diagnostics` runs only when a snapshot is present.
    // Why: Absent diagnostics leave the index empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (diagnostics) { for (const group of diagnostics.groups) ... }
    // ```
    if let Some(snapshot) = diagnostics {
        for group in &snapshot.groups {
            for item in &group.items {
                problems.push(Problem {
                    mark: Mark {
                        start: item.start,
                        // A range the server sent backwards still marks its characters.
                        end: item.end.max(item.start),
                        severity: shown(item.severity),
                    },
                    // `clone` copies the text so the index owns it independently of the snapshot.
                    source: group.source.clone(),
                    code: item.code.clone(),
                    message: item.message.clone(),
                });
            }
        }
    }
    // What: `sort_by_key` orders by the pair `(start, end)` that `position_order` builds.
    // Why: Window and caret searches binary-search by start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // problems.sort((a, b) => a.mark.start - b.mark.start || a.mark.end - b.mark.end);
    // ```
    problems.sort_by_key(position_order);
    let mut reach = Vec::new();
    let mut furthest = 0;
    for problem in &problems {
        furthest = furthest.max(problem.mark.end);
        reach.push(furthest);
    }
    return (problems, reach);
}
