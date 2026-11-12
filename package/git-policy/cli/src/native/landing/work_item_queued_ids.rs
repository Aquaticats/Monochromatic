//! What:
//!  The ids the queue still owes work for:
//! every record in `pending`,
//! `claimed`,
//! `recovering` and `held`,
//! from one listing of each.
//! Why:
//!  A producer asks whether the work it is about to write is already on its way,
//! and a consumer asks whether a
//!  name a record would act on is already being taken away;
//! a record in `done` or `failed` is one nothing scans,
//! so it owes
//!  no work and is no part of the answer.
//! The file sits beside `work_item.rs`,
//! which re-exports it,
//! so the record's own
//!  file stays within the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owed = new Set([PENDING, CLAIMED, RECOVERING, HELD].flatMap((state) => entries(state).map(idOf)));
//! ```

use crate::work_item::{CLAIMED, HELD, PENDING, RECOVERING, entries_oldest_first, id_of_entry};
use std::collections::HashSet;
use std::path::Path;

/// What:
///  The ids the queue still owes work for,
/// from one listing of each state a consumer acts on.
/// Why:
///   A record in `done` or `failed` owes no work,
/// so a caller asking what is on its way must not count one,
/// and a
///  caller asking whether a name is on its way out must see the record a retry returned to `pending` as well
///  as the one a round is running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owed = new Set([PENDING, CLAIMED, RECOVERING, HELD].flatMap((state) => entries(workDir, state).map(idOf)));
/// ```
pub fn queued_ids(work_dir: &Path) -> HashSet<String> {
    let mut owed: HashSet<String> = HashSet::new();
    for state in [PENDING, CLAIMED, RECOVERING, HELD] {
        for (name, _modified) in entries_oldest_first(work_dir, state) {
            owed.insert(String::from(id_of_entry(name.as_str())));
        }
    }
    return owed;
}
