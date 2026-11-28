//! What:
//! The longest name the exposure folder's file system holds,
//! which a build made for controls can answer with
//! a number of its own.
//! Why:
//! The bin-name check asks the file system,
//! so that a folder on eCryptfs (143 bytes) and one on ext4 (255 bytes) each answer for themselves;
//! a control can only show that the answer comes from the file system if it can make the answer differ
//! from this machine's 255 bytes,
//! and no mount-free file system here holds another limit.
//! A shipped build holds no way to plant one:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose,
//! as the removal's own plant does (`exposure`,
//! `a_control_plants_a_removal`).
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can plant it:
//! a control plants one for its own thread and gives the file system back when it ends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-plants-a-name-limit.ts
//! function plantingTheFoldersNameLimit(limit: number): Guard;  // tests only
//! function aPlantedNameLimit(): number | undefined;  // always undefined in a shipped build
//! ```

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    /// The name limit this thread's check answers with,
    /// in bytes.
    static THE_PLANTED_NAME_LIMIT: Cell<Option<u64>> = const { Cell::new(None) };
}

/// What:
/// Forgets a plant,
/// so a check after a control's own asks the file system again.
/// Why:
/// A plant that outlived its control would decide another control's refusal on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetThePlant(): void
/// ```
#[cfg(test)]
fn forget_the_plant() {
    THE_PLANTED_NAME_LIMIT.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// The release of a plant,
/// taken by the guard `planting_the_folders_name_limit` answers.
/// Why:
/// A control's failing assertion raises a panic,
/// which skips the line that would forget the plant,
/// and a limit left planted would decide the next check on this thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlantGuard { [Symbol.dispose]() { forgetThePlant(); } }
/// ```
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct NameLimitPlantGuard;

/// The release of a plant.
#[cfg(test)]
impl Drop for NameLimitPlantGuard {
    /// What:
    /// Forgets the limit this thread planted,
    /// which is the one release a control's body must reach however it leaves.
    /// Why:
    /// The drop is the only place that runs on the way out of a panic.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { forgetThePlant(); }
    /// ```
    fn drop(&mut self) {
        forget_the_plant();
    }
}

/// What:
/// Plants `limit` bytes as the longest name the exposure folder holds for this thread's next check,
/// and answers the guard that forgets it when the control's body leaves,
/// however it leaves.
/// Why:
/// The check's whole claim is that the limit is the file system's,
/// and a control that cannot make the answer differ from this machine's cannot hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantingTheFoldersNameLimit(limit: number): PlantGuard {
///   THE_PLANTED_NAME_LIMIT = limit;
///   return new PlantGuard();
/// }
/// ```
#[cfg(test)]
pub(crate) fn planting_the_folders_name_limit(limit: u64) -> NameLimitPlantGuard {
    THE_PLANTED_NAME_LIMIT.with(|cell| {
        cell.set(Some(limit));
    });
    return NameLimitPlantGuard;
}

/// What:
/// The limit a control planted for this thread,
/// or nothing when this is a real question for a file system.
/// Why:
/// The check returns the planted answer in place of the one `statvfs` gives,
/// and every other build answers nothing so the file system is asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aPlantedNameLimit(): number | undefined
/// ```
#[cfg(test)]
pub(crate) fn a_planted_name_limit() -> Option<u64> {
    return THE_PLANTED_NAME_LIMIT.with(|cell| return cell.get());
}

/// The answer a shipped build gives:
/// no limit is planted,
/// because nothing can plant one.
#[cfg(not(test))]
pub(crate) fn a_planted_name_limit() -> Option<u64> {
    return None;
}
