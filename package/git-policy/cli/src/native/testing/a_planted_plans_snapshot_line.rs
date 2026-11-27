//! What:
//!  One line of a planted plan's own snapshot:
//!  the name and GUID a fixture gives it,
//!  and the line a control records at the attempt and with the outcome it
//!  names.
//! Why:
//!  A control that says what a plan witnesses needs a line whose outcome left
//!  `planned`,
//!  and the grammar refuses a line whose name is not a snapshot of the clone the
//!  plan records,
//!  so the name is the planted plan's own dataset and nothing a control spells.
//!  The binary controls plant a removal's first write and never a history,
//!  so this file stays out of that crate
//!  (`the_destruction_plan_a_control_plants.rs` is the half both crates take).
//! Gotcha:
//!  The snapshot a line names was never made by a kernel,
//!  so its GUID is a stand-in and its group is the one after the clone's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = aLineOf(plan, 0, 'planned'); // <clone>@worktree-own
//! ```

use super::the_destruction_plan_a_control_plants::{
    APlanAControlPlants,
    a_guid,
};
use super::the_destruction_plans_writer::{
    PlanAttempt,
    PlanSnapshot,
    SnapshotOutcome,
};

/// The name a planted plan gives the clone's own first snapshot,
/// which ends the line's `name=` field.
const THE_OWN_SNAPSHOT: &str = "worktree-own";

/// The GUID a planted plan records for the clone's own first snapshot.
///
/// Why:
///  The kernel reports a GUID when it makes a snapshot,
///  and a fixture's snapshot was never made,
///  so this is a stand-in for that GUID.
const THE_OWN_SNAPSHOTS_GUID: u64 = 17_157_080_670_377_245_572;

/// What:
///  One line of the planted plan's own first snapshot,
///  at the attempt number and with the outcome the control names.
/// Why:
///  The grammar holds one line per attempt of one name,
///  and a name's group is above the clone's,
///  so a control that plants a history plants these lines and no others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = aLineOf(plan, 1, 'destroyed');
/// ```
pub(crate) fn a_line_of(
    plan: &APlanAControlPlants,
    attempt: u64,
    outcome: SnapshotOutcome,
) -> PlanAttempt {
    return PlanAttempt {
        snapshot: PlanSnapshot {
            name: format!("{}@{THE_OWN_SNAPSHOT}", plan.clone.clone_dataset),
            guid: a_guid(THE_OWN_SNAPSHOTS_GUID),
            createtxg: plan.clone.clone_createtxg + 1,
        },
        attempt,
        outcome,
        errno: None,
    };
}
