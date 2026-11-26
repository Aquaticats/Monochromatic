//! What:
//!  The binding record one managed slot's add wrote,
//!  as the lines a person reads:
//!  the phase,
//!  the claim,
//!  the clone and its GUID,
//!  the origin the add recorded,
//!  the Git registration,
//!  and the settle reason of an add that stopped.
//! Why:
//!  The removal's own refusals send a person to the binding record when the
//!  clone, the origin or the registration a removal is about cannot be proved
//!  (`RemovalStop::NoBinding`, `RemovalStop::BindingIncomplete`),
//!  and plan step 18 is where that record is printed
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds").
//!  The record is a file a person may read at any time and a reader that
//!  refuses it proves nothing,
//!  so a record that cannot be read is unknown rather than a finding:
//!  a record filed under the wrong name is the store's own listing check and
//!  not this one.
//!  Every field printed here is one the record's grammar proved at its phase,
//!  and a field the phase does not prove is printed as absent,
//!  because an empty line would read as one nobody filled in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkBindingRecord(reading); // pass | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    pass,
    unknown,
};
use crate::worktree_clone_slot_state::slot_state_readings::BindingReading;

/// The words that stand where a field's phase has not proved it yet.
pub const NOT_RECORDED: &str = "not recorded at this phase";

/// What:
///  One field's recorded value,
///  or the words for a phase that has not proved it.
/// Why:
///  A record's fields appear phase by phase,
///  so the same line prints a value for a filled clone and the absent words for
///  one whose add stopped earlier,
///  and a person can tell "no value" from "the value is empty".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = shown(record.clone_guid?.fileName()); // '4821193048571903102' | 'not recorded at this phase'
/// ```
fn shown<T>(
    value: Option<T>,
    render: impl Fn(T) -> String,
) -> String {
    return match value {
        Some(held) => render(held),
        None => String::from(NOT_RECORDED),
    };
}

/// What:
///  One slot's binding record as the line a person reads,
///  or unknown when the record cannot be read.
/// Why:
///  The record that cannot be read may name this very slot,
///  so nothing may be concluded from its absence;
///  a slot with no record at all is the store's own arm and passes here,
///  because this check reports a record and never the slot's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkBindingRecord(BindingReading.Absent); // pass
/// ```
pub fn check_binding_record(reading: &BindingReading) -> CheckOutcome {
    return match reading {
        BindingReading::Absent => pass(String::from(
            "no binding record is filed under this slot's name, so no add claims it",
        )),
        BindingReading::Unreadable {
            reason,
        } => unknown(format!(
            "the binding record filed under this slot's name cannot be read ({reason}), so the clone, the origin and the registration it holds are unknown"
        )),
        BindingReading::Record(record) => {
            let registration: String = match record.registration_path() {
                Some(path) => path.display().to_string(),
                None => String::from(NOT_RECORDED),
            };
            let settling: String = match record.settle_reason {
                Some(reason) => format!(", settle reason `{}`", reason.word()),
                None => String::new(),
            };
            let clone_guid: String = shown(record.clone_guid, |guid| return guid.file_name());
            let clone_createtxg: String =
                shown(record.clone_createtxg, |group| return group.to_string());
            let origin: String = match (&record.origin_dataset, record.origin_snapshot_guid) {
                (Some(dataset), Some(guid)) => format!(
                    "the origin dataset `{dataset}` and the origin snapshot `{}` (GUID {})",
                    record.origin_snapshot,
                    guid.file_name()
                ),
                _ => format!(
                    "the origin snapshot `{}` and no origin dataset or GUID",
                    record.origin_snapshot
                ),
            };
            return pass(format!(
                "the binding record filed under `{}` stands at `phase={}`{settling}: claim {}, the clone `{}` with GUID {clone_guid} and creation group {clone_createtxg}, {origin}, and the Git registration {registration}",
                record
                    .slot_name()
                    .unwrap_or_else(|| return String::from("its slot")),
                record.phase.word(),
                record.claim,
                record.clone_dataset,
            ));
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_record_checks_tests.rs"]
mod tests;
