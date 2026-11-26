//! What:
//!  What the readings of one slot saw,
//!  one line per source:
//!  the binding record,
//!  the Git registration,
//!  the removal's plan,
//!  the clone's mount intent,
//!  and the mount table.
//! Why:
//!  A state without its evidence cannot be checked,
//!  and the design's report exists to stop a reader treating an unmounted clone
//!  as deleted files,
//!  which only the named sources can rule out
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees").
//!  Each line names its source and prints what the reader said,
//!  so the report's state and the evidence it came from are one read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seen = evidence(readings);
//! ```

use crate::worktree_clone_slot_state::slot_state_mounts::{
    CloneMount,
    MountRows,
    SlotMount,
    clone_mount_of,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
};

/// What:
///  What the readings saw,
///  one line per source,
///  so a person can check the state against the evidence it came from.
/// Why:
///  A state without its evidence cannot be checked,
///  and the design's report exists to stop a reader treating an unmounted clone
///  as deleted files,
///  which only the named sources can rule out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seen = evidence(readings);
/// ```
pub fn evidence(readings: &SlotReadings) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    match &readings.binding {
        BindingReading::Absent => lines.push(format!(
            "binding: no record is filed under `{}`",
            readings.slot
        )),
        BindingReading::Unreadable {
            reason,
        } => lines.push(format!("binding: cannot be read: {reason}")),
        BindingReading::Record(record) => {
            let settling: String = record.settle_reason.map_or_else(String::new, |reason| {
                return format!(" settleReason={}", reason.word());
            });
            lines.push(format!(
                "binding: phase={} cloneDataset=`{}` pool=`{}`{settling}",
                record.phase.word(),
                record.clone_dataset,
                record.pool
            ));
        }
    }
    match (&readings.binding, readings.registration_stands) {
        (BindingReading::Record(record), Some(stands)) => {
            let registration: String = record
                .registration_path()
                .map_or_else(String::new, |path| return path.display().to_string());
            lines.push(format!(
                "registration: {}{registration}",
                if stands {
                    "stands at "
                } else {
                    "gone, and the record names "
                }
            ));
        }
        (BindingReading::Record(record), None) => match record.registration_path() {
            Some(path) => lines.push(format!(
                "registration: not read, and the record names {}",
                path.display()
            )),
            None => lines.push(String::from("registration: the record names none yet")),
        },
        (
            BindingReading::Absent
            | BindingReading::Unreadable {
                ..
            },
            _,
        ) => {
            lines.push(String::from(
                "registration: no record was read, so no registration was looked for",
            ));
        }
    }
    match &readings.plan {
        PlanReading::NotAsked => lines.push(String::from(
            "plan: not asked, because no record was read to name the clone with",
        )),
        PlanReading::NoPlan => {
            lines.push(String::from("plan: no destruction plan names this clone"))
        }
        PlanReading::NamesTheClone {
            plan,
        } => lines.push(format!("plan: {} names this clone", plan.display())),
        PlanReading::Unreadable {
            plan,
            reason,
        } => lines.push(format!("plan: {} cannot be read: {reason}", plan.display())),
    }
    match &readings.intent {
        IntentFacts::Unasked => lines.push(String::from(
            "intent: not asked, because the record names no clone dataset yet",
        )),
        IntentFacts::Absent => lines.push(String::from("intent: no request of the clone's own")),
        IntentFacts::Mounting {
            nonce,
        } => lines.push(format!("intent: mount requested under nonce `{nonce}`")),
        IntentFacts::Unmounting {
            nonce,
        } => lines.push(format!("intent: unmount requested under nonce `{nonce}`")),
        IntentFacts::Inherited {
            from,
        } => lines.push(format!(
            "intent: inherited from `{from}`, which authorizes nothing"
        )),
        IntentFacts::Unreadable {
            reason,
        } => lines.push(format!("intent: cannot be read: {reason}")),
    }
    let clone_dataset: Option<&str> = match &readings.binding {
        BindingReading::Record(record) => Some(record.clone_dataset.as_str()),
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => None,
    };
    match clone_dataset {
        Some(dataset) => lines.extend(mount_evidence(readings, dataset)),
        None => lines.push(slot_mount_evidence(
            &readings.slot_mount,
            readings.slot_path.as_path(),
        )),
    }
    return lines;
}

/// What:
///  What the mount table and the slot's own mount say,
///  read against the clone dataset.
/// Why:
///  The class turns on this reading,
///  so the report prints the clone's rows and the slot's own mount instead of a
///  summary,
///  and a person can see whether the slot is the clone,
///  another mount,
///  or empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = mount_evidence(readings, cloneDataset);
/// ```
fn mount_evidence(
    readings: &SlotReadings,
    clone_dataset: &str,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mount: CloneMount = clone_mount_of(
        &readings.mounts,
        &readings.slot_mount,
        clone_dataset,
        readings.slot_path.as_path(),
    );
    match &readings.mounts {
        MountRows::Unreadable {
            reason,
        } => lines.push(format!("mount table: cannot be read: {reason}")),
        MountRows::Rows(rows) => {
            let clone_rows: Vec<
                &crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance,
            > = rows
                .iter()
                .filter(|row| return row.dataset == clone_dataset)
                .collect();
            if clone_rows.is_empty() {
                lines.push(format!("mount table: no row mounts `{clone_dataset}`"));
            }
            lines.extend(
                clone_rows
                    .iter()
                    .map(|row| return format!("mount table: clone mounted as {}", row.described())),
            );
        }
    }
    lines.push(slot_mount_evidence(
        &readings.slot_mount,
        readings.slot_path.as_path(),
    ));
    match mount {
        CloneMount::AtItsSlot => lines.push(String::from(
            "mount: the clone's own root stands at its slot",
        )),
        CloneMount::Nowhere => lines.push(String::from(
            "mount: the clone is not mounted anywhere this table shows",
        )),
        CloneMount::Elsewhere {
            described,
        } => lines.push(format!("mount: the clone is elsewhere ({described})")),
        CloneMount::SomethingElseAtTheSlot {
            described,
        } => lines.push(format!("mount: {described} stands at the slot")),
        CloneMount::Unreadable {
            reason,
        } => lines.push(format!("mount: cannot be read: {reason}")),
    }
    return lines;
}

/// What:
///  One line about the mount at the slot's own path.
/// Why:
///  An empty underlay and a mount a person put there are different states,
///  and only this reading tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = slot_mount_evidence(slotMount, slotPath);
/// ```
fn slot_mount_evidence(
    slot_mount: &SlotMount,
    slot_path: &std::path::Path,
) -> String {
    return match slot_mount {
        SlotMount::Nothing => format!("slot mount: nothing is mounted at {}", slot_path.display()),
        SlotMount::Something {
            described,
        } => format!("slot mount: {described}"),
        SlotMount::Unreadable {
            reason,
        } => format!("slot mount: cannot be read: {reason}"),
    };
}
