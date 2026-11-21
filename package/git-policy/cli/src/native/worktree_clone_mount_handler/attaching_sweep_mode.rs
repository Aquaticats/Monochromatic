//! What:
//!  The clone-mount sweep:
//!  what the installed ZED event runs when it starts the program with no argument,
//!  and what `boot-reconcile` runs after its own checks.
//! Why:
//!  The sweep enumerates the pinned parent's clone datasets,
//!  reads each clone's durable mount intent and attaches only what that intent asks for
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  "Requests,
//! results and ordering").
//!  It is plan step 8,
//! so this build refuses the mode instead of attaching anything:
//!  a partial sweep at root would be a mount nobody designed.
//!  Until that step lands the mode does one thing and no more:
//!  it removes the staging names a killed writer left in the store folder,
//!  which is the remover the accepted design's decision adds beside the administrator's adoption,
//!  and it lists nothing else and reads no record.
//!  Its one listing per run is the cost of that removal,
//!  and it refuses a store folder that is not there rather than making one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = run(); // { refused: 'step 8' } until the sweep lands
//! ```

use super::mode_awaiting_its_step::AwaitingItsStep;
use crate::diagnostic_log::warn;
use crate::worktree_clone_mount::{SLOT_IDENTITY_STORE_PATH, StoreRefusal, remove_staging_entries_in};
use std::path::Path;

/// The mode this build refuses,
/// its plan step and the work that step adds.
pub const AWAITING: AwaitingItsStep = AwaitingItsStep {
    mode: "sweep (no argument)",
    step: 8,
    does: "enumerates the pinned parent's clone datasets, reads each clone's durable mount intent and attaches only the clones that ask for a mount",
};

/// Module tag for diagnostics.
const TAG: &str = "attachingSweep";

/// Refuse the sweep until its step lands,
/// after removing the staging names a killed writer left in the store folder.
///
/// # Errors
/// Always,
/// with the refusal naming the step that builds it.
pub fn run() -> Result<String, AwaitingItsStep> {
    return sweep_at(Path::new(SLOT_IDENTITY_STORE_PATH));
}

/// What:
///  The sweep against a named store folder:
///  one removal of the staging names a killed writer left,
///  and then the step's own refusal.
/// Why:
///  The store's path is what a control cannot set,
///  so it is an argument here:
///  a control drives the removal on a fixture folder,
///  and the program passes the folder the design fixes.
///  A missing folder is the setup run's to make and says nothing on a boot that never had one,
///  while any other refusal is said rather than swallowed,
///  because the litter a killed writer leaves is what this run exists to shed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sweep = sweepAt(path); // the staging names are gone, and step 8 is still refused
/// ```
///
/// # Errors
/// Always,
/// with the refusal naming the step that builds it.
fn sweep_at(store: &Path) -> Result<String, AwaitingItsStep> {
    match remove_staging_entries_in(store) {
        Ok(removed) => {
            if !removed.is_empty() {
                warn(TAG, format!("the sweep removed {} staging entries a killed writer left in {}", removed.len(), store.display()).as_str());
            }
        }
        // A store folder the setup run never made holds no staging name to remove.
        Err(StoreRefusal::Missing { .. }) => {}
        Err(refusal) => warn(TAG, refusal.message().as_str()),
    }
    return Err(AWAITING);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attaching_sweep_mode_tests.rs"]
mod tests;
