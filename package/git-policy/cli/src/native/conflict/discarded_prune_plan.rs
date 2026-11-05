//! What:
//!  Which saved-copy directories a pass removes, and why: the ones past the age limit, then the oldest until the folder fits its size.
//! Why:
//!  The choice is a pure function of the listing, the clock and the limits, so it lives in a file of its own that `discarded_prune.rs` imports and that imports nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const doomed = plan(entries, nowMillis, limits, "the-new-one"); // [[0, "age"], [3, "size"]]
//! ```

use crate::discarded_prune_limits::Limits;
use crate::discarded_prune_stamps::{DAY_MILLISECONDS, STAMP_RESOLUTION_MILLISECONDS, Stamped, YOUNG_FLOOR_MILLISECONDS};

/// Why a directory goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    /// Older than the age limit.
    Age,
    /// The folder is over the size limit and this is among the oldest.
    Size,
}

impl Reason {
    /// The word in the audit line.
    pub fn word(self) -> &'static str {
        return match self {
            Reason::Age => "age",
            Reason::Size => "size",
        };
    }
}

/// Milliseconds from the end of the named second of a directory to now, in a width no clock or limit can overflow.
fn age_of(entry: &Stamped, now_millis: i64) -> i128 {
    return i128::from(now_millis) - (i128::from(entry.millis) + i128::from(STAMP_RESOLUTION_MILLISECONDS));
}

/// Whether the plan may remove the directory: it is past the young floor and is not the one just written.
fn is_reachable(entry: &Stamped, now_millis: i64, keep: Option<&str>) -> bool {
    return age_of(entry, now_millis) >= i128::from(YOUNG_FLOOR_MILLISECONDS) && keep != Some(entry.name.as_str());
}

/// Whether the entry at `index` is already chosen for removal.
fn is_doomed(doomed: &[(usize, Reason)], index: usize) -> bool {
    for (gone, _reason) in doomed {
        if *gone == index {
            return true;
        }
    }
    return false;
}

/// What:
///  Which directories to remove, as indexes into `entries` with the reason, from the time, the limits and the name to keep.
/// Why:
///   Pure, so the rule is tested without a disk: first every copy past the age limit; then, while the rest is over the
///  size limit, the oldest of those that remain. A directory younger than a day, and the one named `keep`, is never chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doomed = plan(entries, nowMillis, limits, 'the-new-one') // [[0, 'age'], [3, 'size']]
/// ```
pub fn plan(entries: &[Stamped], now_millis: i64, limits: Limits, keep: Option<&str>) -> Vec<(usize, Reason)> {
    let age_limit: i128 = i128::from(limits.max_age_days) * i128::from(DAY_MILLISECONDS);
    let mut doomed: Vec<(usize, Reason)> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if is_reachable(entry, now_millis, keep) && age_of(entry, now_millis) > age_limit {
            doomed.push((index, Reason::Age));
        }
    }
    let mut total: u128 = 0;
    for (index, entry) in entries.iter().enumerate() {
        if !is_doomed(&doomed, index) {
            total += entry.bytes;
        }
    }
    // The oldest first, a tie in time broken by name, and a tie in both by position, as a stable sort keeps it.
    let mut candidates: Vec<(i64, &String, usize)> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if is_reachable(entry, now_millis, keep) && !is_doomed(&doomed, index) {
            candidates.push((entry.millis, &entry.name, index));
        }
    }
    candidates.sort();
    for (_millis, _name, index) in candidates {
        if total <= u128::from(limits.max_bytes) {
            break;
        }
        total = total.saturating_sub(entries[index].bytes);
        doomed.push((index, Reason::Size));
    }
    return doomed;
}
