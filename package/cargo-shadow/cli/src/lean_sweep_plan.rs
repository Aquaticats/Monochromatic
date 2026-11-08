//! What:
//! The decision of mechanism B,
//! the in-place sweep of units that an older compiler built.
//! Why:
//! Cargo does not hash a nightly's date into a unit's directory names,
//! so a nightly update dirties every unit in place and the units no later build rebuilt stay behind with an older compiler
//! recorded and no reuse value under the newer one (`doc/planning/lean-build-dirs.md`, "Which units and incremental state were
//! built by an older compiler").
//! The newest compiler group of a profile directory is kept,
//! and so is any group invoked within the grace period (a second toolchain may still be in use).
//! A profile directory with one compiler group is left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const planOlderCompilerSweep = (units: ProfileUnits, now: number, graceSeconds: number): Removals
//! ```

/// The decision types.
use crate::lean_unit_reader::{ProfileUnits, Unit};
use crate::lean_unit_removal::{Planner, Removals};

/// The sweep with its grace period.
#[derive(Clone, Copy, Debug)]
pub struct OlderCompilerSweep {
    /// Units invoked within this many seconds of now are kept whatever compiler built them.
    pub grace_seconds: i64,
}

/// What:
/// `fn newest_unit(units: &[Unit]) -> Option<&Unit>` is the unit invoked last.
/// Why:
/// Its compiler is the current one of the profile directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const newestUnit = (units: Unit[]): Unit | undefined => maxBy(units, (u) => u.invoked);
/// ```
fn newest_unit(units: &[Unit]) -> Option<&Unit> {
    let mut newest: Option<&Unit> = None;
    for unit in units {
        if newest.is_none_or(|best| { return unit.invoked > best.invoked; }) {
            newest = Some(unit);
        }
    }
    return newest;
}

impl Planner for OlderCompilerSweep {
    fn plan(&self, found: &ProfileUnits, now: i64) -> Removals {
        let mut removals: Removals = Removals::default();
        let Some(newest) = newest_unit(&found.units) else { return removals };
        let mut other_compiler: bool = false;
        let mut first_of_newest: i64 = newest.invoked;
        for unit in &found.units {
            if unit.compiler == newest.compiler {
                first_of_newest = first_of_newest.min(unit.invoked);
            } else {
                other_compiler = true;
            }
        }
        if !other_compiler {
            return removals;
        }
        let grace_start: i64 = now - self.grace_seconds;
        for unit in &found.units {
            if unit.compiler != newest.compiler && unit.invoked < grace_start {
                removals.units.push(unit.clone());
            }
        }
        let incremental_before: i64 = first_of_newest.min(grace_start);
        for directory in &found.incremental {
            if directory.modified < incremental_before {
                removals.incremental.push(directory.clone());
            }
        }
        removals.reason = format!("built by a compiler older than the newest ({}) and invoked before the grace period", newest.compiler);
        return removals;
    }
}
