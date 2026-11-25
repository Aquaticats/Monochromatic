//! What:
//!  The pass one `git worktree prune` runs over the wrapper's own records
//!  before Git starts:
//!  every bound slot whose registration stands has its pointer compared with
//!  the bytes the add recorded,
//!  the recorded bytes are written back when the pointer is gone or changed,
//!  and the slots nothing can restore are handed back to the caller,
//!  which refuses the prune.
//! Why:
//!  An explicit `git worktree prune` drops a registration whose pointer is
//!  missing at once,
//!  a `git gc` waits three months,
//!  and the clone dataset a dropped registration named is then reachable by
//!  nobody
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The record holds the exact bytes Git wrote,
//!  so the pointer the add kept for prune can be put back rather than lost,
//!  and a slot with nothing to restore stops the prune instead of losing its
//!  registration silently.
//!  Nothing here reads the pool,
//!  changes a record or unmounts anything:
//!  a restore writes one file the record itself proves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guard = guardThePrune(commonDir, liveMounts); // { lines, refused }
//! ```

use crate::worktree_clone_binding_record::registration_home::{home_folder, record_name_of, slot_of, RecordKind};
use crate::worktree_clone_binding_record::record_publication::read_binding;
use crate::worktree_clone_prune_pointer::link_facts::{LinkFacts, link_facts_of, recorded_link_of};
use crate::worktree_clone_prune_pointer::pointer_restore::{RestoreOutcome, restore_the_slot_pointer};
use crate::worktree_clone_prune_pointer::pointer_state::PointerState;
use crate::worktree_clone_slot_state::slot_state_mounts::{clone_mount_of, read_mount_rows, slot_mount_of};
use crate::worktree_clone_slot_state::slot_state_mounts::{CloneMount, MountRows};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::path::{Path, PathBuf};

/// What:
///  Why one prune must not reach Git.
/// Why:
///  A registration whose pointer nothing restores would be dropped,
///  and a record no reader may act on may name such a registration,
///  so both stop the prune and each names what to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PruneRefusal = { kind: 'record'; slot; message } | { kind: 'link'; facts: LinkFacts };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PruneRefusal {
    /// A record filed under a slot's name cannot be read,
    /// so its registration and pointer are unproved.
    Record {
        /// The slot the record is filed under.
        slot: String,
        /// The record reader's own sentence.
        message: String,
    },
    /// A registration that stands whose pointer nothing here can restore.
    Link(LinkFacts),
}

/// What:
///  What the pass found:
///  the lines a person reads,
///  and the reasons a prune must not reach Git.
/// Why:
///  A restore is reported and the prune goes on,
///  while a slot nothing restores must stop the prune before Git runs,
///  so the command needs both answers from one pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PruneGuard = { lines: string[]; refused: PruneRefusal[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneGuard {
    /// One line per slot this pass restored or could not check,
    /// in the order the records were listed.
    pub lines: Vec<String>,
    /// Why the prune must not reach Git,
    /// one entry per slot.
    pub refused: Vec<PruneRefusal>,
}

/// What:
///  Every slot the record home names,
///  in the order the folder lists them.
/// Why:
///  A prune serves the whole repository,
///  so every bound slot is checked rather than one the command names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slots = boundSlots(commonDir);
/// ```
fn bound_slots(common_dir: &Path) -> Vec<String> {
    let home: PathBuf = home_folder(common_dir);
    let Ok(entries) = std::fs::read_dir(home) else {
        return Vec::new();
    };
    let mut slots: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        let Some(record) = record_name_of(name.as_str()) else {
            continue;
        };
        if record.kind == RecordKind::Binding && !slots.contains(&record.slot) {
            slots.push(record.slot);
        }
    }
    return slots;
}

/// What:
///  Run the pass over every bound slot's pointer before a prune.
/// Why:
///  The pointer is the file Git's prune reads,
///  so it is put back whenever the record proves it,
///  and a slot whose registration stands with no restorable pointer is named
///  for the caller's refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guard = guardThePrune(commonDir, mounts);
/// ```
pub fn guard_the_prune(common_dir: &Path, mounts: &dyn MountedDatasets) -> PruneGuard {
    let slots: Vec<String> = bound_slots(common_dir);
    // A repository with no record folder bounds no slot, so the pass is empty and the mount table is never read: this is
    // every prune in a repository the route has never served.
    if slots.is_empty() {
        return PruneGuard::default();
    }
    let rows: MountRows = read_mount_rows(mounts);
    let mut guard: PruneGuard = PruneGuard::default();
    for slot in slots {
        let record = match read_binding(home_folder(common_dir).as_path(), slot.as_str()) {
            Ok(found) => found,
            Err(refusal) => {
                let message: String = refusal.message(home_folder(common_dir).join(crate::worktree_clone_binding_record::registration_home::binding_name(slot.as_str())).as_path());
                guard.lines.push(format!("the record filed under `{slot}` cannot be read, so its registration and pointer are unproved: {message}"));
                guard.refused.push(PruneRefusal::Record { slot, message });
                continue;
            }
        };
        let Some(slot_path) = record.slot_path() else {
            guard.lines.push(format!("the record filed under `{slot}` names no slot path, so its pointer was not checked"));
            continue;
        };
        if slot_of(slot_path.as_path()).as_deref() != Ok(slot.as_str()) {
            guard.lines.push(format!(
                "the record filed under `{slot}` names the slot at {}, so its pointer was not checked",
                slot_path.display()
            ));
            continue;
        }
        let slot_mount = slot_mount_of(mounts, slot_path.as_path());
        let mount: CloneMount = clone_mount_of(&rows, &slot_mount, record.clone_dataset.as_str(), slot_path.as_path());
        let link = recorded_link_of(&record);
        let facts: LinkFacts = link_facts_of(slot.as_str(), &link, &mount);
        if !facts.registration_stands {
            // Git's prune has no registration here to drop, so the pointer cannot lose one.
            continue;
        }
        match &facts.pointer {
            PointerState::Intact | PointerState::Present { .. } => {}
            PointerState::Missing | PointerState::Differs { .. } => {
                let outcome: RestoreOutcome = restore_the_slot_pointer(slot_path.as_path(), &facts.site, record.pointer.as_deref());
                match outcome {
                    RestoreOutcome::Written { .. } | RestoreOutcome::AlreadyIntact => guard.lines.push(format!(
                        "{}; `git worktree prune` keeps the registration of `{slot}` because the pointer stands",
                        outcome.line(slot_path.as_path(), &facts.site)
                    )),
                    RestoreOutcome::NothingToWrite { .. } | RestoreOutcome::Refused { .. } => {
                        guard.lines.push(outcome.line(slot_path.as_path(), &facts.site));
                        guard.refused.push(PruneRefusal::Link(facts));
                    }
                }
            }
            PointerState::Unprovable { .. } => guard.refused.push(PruneRefusal::Link(facts)),
        }
    }
    return guard;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "prune_guard_tests.rs"]
mod tests;
