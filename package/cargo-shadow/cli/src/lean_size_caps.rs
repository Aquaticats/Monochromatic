//! What:
//! Mechanism E,
//! the size cap of a profile directory with oldest-first eviction.
//! Why:
//! The sweep of older compilers frees nothing when one compiler's units alone outgrow the bound
//! (superseded units of a dependency or feature change stay in the newest compiler's group).
//! The cap makes the bound true:
//! units and incremental directories are evicted oldest first,
//! by the time of their last invocation or write,
//! until the profile directory is under the cap.
//! Entries touched within the grace period are never evicted,
//! so a cap set too low costs a rebuild of old units and never of the build in progress.
//! Final artifacts outside the units are not touched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const planSizeCap = (units: ProfileUnits, capBytes: number, graceSeconds: number): Removals
//! ```

/// Sets of identities.
use std::collections::HashSet;

/// The decision types,
/// sizes and the deadline.
use crate::lean_identity::{Deadline, Identity};
use crate::lean_sizes::add_tree_bytes;
use crate::lean_unit_reader::ProfileUnits;
use crate::lean_unit_removal::{Planner, Removals};

/// The cap of one profile directory.
#[derive(Clone, Copy, Debug)]
pub struct SizeCap {
    /// The most disk bytes the units and incremental directories of one profile directory may occupy.
    pub cap_bytes: u64,
    /// Entries touched within this many seconds of now are never evicted.
    pub grace_seconds: i64,
    /// The deadline of the size walks.
    pub deadline: Deadline,
}

/// One thing that can be evicted.
enum Entry {
    /// The unit at this index.
    Unit(usize),
    /// The incremental directory at this index.
    Incremental(usize),
}

impl Planner for SizeCap {
    fn plan(&self, found: &ProfileUnits, now: i64) -> Removals {
        let mut seen: HashSet<Identity> = HashSet::new();
        let mut sized: Vec<(i64, u64, Entry)> = Vec::new();
        let mut total: u64 = 0;
        for (index, unit) in found.units.iter().enumerate() {
            let mut bytes: u64 = 0;
            for path in &unit.paths {
                bytes += add_tree_bytes(path, &mut seen, &self.deadline).bytes;
            }
            total += bytes;
            sized.push((unit.invoked, bytes, Entry::Unit(index)));
        }
        for (index, directory) in found.incremental.iter().enumerate() {
            let bytes: u64 = add_tree_bytes(&directory.path, &mut seen, &self.deadline).bytes;
            total += bytes;
            sized.push((directory.modified, bytes, Entry::Incremental(index)));
        }
        let mut removals: Removals = Removals::default();
        if total <= self.cap_bytes || self.deadline.passed() {
            return removals;
        }
        sized.sort_by_key(|entry| { return entry.0; });
        let grace_start: i64 = now - self.grace_seconds;
        let over: u64 = total;
        for (time, bytes, entry) in sized {
            if total <= self.cap_bytes || time >= grace_start {
                break;
            }
            match entry {
                Entry::Unit(index) => removals.units.push(found.units[index].clone()),
                Entry::Incremental(index) => removals.incremental.push(found.incremental[index].clone()),
            }
            total = total.saturating_sub(bytes);
        }
        removals.reason = format!("the profile directory held {over} bytes, over its cap of {} bytes; oldest first until under it", self.cap_bytes);
        return removals;
    }
}
