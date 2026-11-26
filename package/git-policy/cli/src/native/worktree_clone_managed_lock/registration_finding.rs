//! What:
//!  Find the one `worktrees/<name>` entry a slot's add made,
//!  by the names Git could derive from the slot and the finder's own decision.
//! Why:
//!  A registration is read by the record's `registration=` when the add wrote it,
//!  and otherwise by the finder,
//!  whose candidates are Git's sanitiser's name and its counters,
//!  and whose rows are the binding design's
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The finder, and the lock's part in it",
//!  and `doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The states Git's source can leave").
//!  This route calls the bindings' rules and states none of its own,
//!  so a slot whose registration the add's resume would adopt,
//!  clear or refuse reads the same here.
//!  Two entries that both attribute themselves are the binding's refusal and never a choice:
//!  only a copy of the sentence could make two.
//!  The names another slot's binding owns are excluded first,
//!  and those are read from the home folder rather than supplied,
//!  so `doctor` and the removal see one answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = findTheSlotRegistration(commonDir, slot, slotPath, nonce); // { registration, namesSeen }
//! ```

use super::registration_entry_reading::read_one_entry;
use crate::worktree_clone_binding_record::RecordKind;
use crate::worktree_clone_binding_record::record_name_of;
use crate::worktree_clone_binding_record::registration_finder::candidate_names;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_binding_record::registration_resume::{
    EntryDecision,
    decide,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::{
    Path,
    PathBuf,
};

/// The folder Git keeps one directory per registration in.
pub const WORKTREES_FOLDER: &str = "worktrees";

/// What:
///  What one slot's candidates held.
/// Why:
///  A caller that found one entry needs its decision,
///  and one that found none needs the names the walk read,
///  so both travel in one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Registration = { kind: 'one', name, path, decision } | { kind: 'two', names } | { kind: 'none' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheRegistration {
    /// One entry attributed itself to this add.
    One {
        /// The entry's name below `worktrees`.
        name: String,
        /// The entry's whole path.
        path: PathBuf,
        /// What the finder decided for it.
        decision: EntryDecision,
    },
    /// Two entries attributed themselves,
    /// which only a copied sentence makes.
    Two {
        /// Their names.
        names: Vec<String>,
    },
    /// No entry attributed itself.
    None,
}

/// What one walk of this slot's candidate names found.
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheWalkFound = { registration: Registration; namesSeen: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatTheWalkFound {
    /// What the walk found.
    pub registration: TheRegistration,
    /// Every candidate name the walk read,
    /// so a refusal can print them.
    pub names_seen: Vec<String>,
}

/// What:
///  The slots the home folder's binding names hold.
/// Why:
///  The finder excludes a name another slot's binding owns,
///  and only the listing knows every slot,
///  so the caller does not have to carry them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slots = readdir(home).flatMap((name) => recordNameOf(name)?.slot ?? []);
/// ```
pub fn the_slots_the_home_names(common_dir: &Path) -> Vec<String> {
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

/// Whether one decision attributed itself to this add,
/// which is the finder's own count of the two-match refusal.
fn attributes_itself(decision: &EntryDecision) -> bool {
    return matches!(
        decision,
        EntryDecision::Adopt | EntryDecision::Clear { .. } | EntryDecision::Refuse { .. }
    );
}

/// What:
///  Walk one slot's candidate names and read what each entry decides.
/// Why:
///  Every name Git's sanitiser and its counter could have made is read,
///  because a stale registration makes Git name the new one `<name>1`,
///  and the ownership exclusion runs inside the finder's own decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of candidateNames(slot)) { const decisions = decide(readOneEntry(...), ...); }
/// ```
pub fn find_the_slot_registration(
    common_dir: &Path,
    slot: &str,
    slot_path: &Path,
    nonce: &RequestNonce,
) -> WhatTheWalkFound {
    let worktrees: PathBuf = common_dir.join(WORKTREES_FOLDER);
    let other_slots: Vec<String> = the_slots_the_home_names(common_dir)
        .into_iter()
        .filter(|other| return other != slot)
        .collect();
    let names: Vec<String> = candidate_names(slot);
    let mut names_seen: Vec<String> = Vec::new();
    let mut attributing: Vec<(String, EntryDecision)> = Vec::new();
    for name in names.clone() {
        if !worktrees.join(name.as_str()).exists() {
            continue;
        }
        names_seen.push(name.clone());
        let entry = read_one_entry(worktrees.as_path(), name.as_str(), slot_path);
        let decision: EntryDecision =
            decide(&entry, slot, slot_path, nonce, other_slots.as_slice());
        if attributes_itself(&decision) {
            attributing.push((name, decision));
        }
    }
    if attributing.len() > 1 {
        let found: Vec<String> = attributing
            .into_iter()
            .map(|(name, _decision)| return name)
            .collect();
        return WhatTheWalkFound {
            registration: TheRegistration::Two {
                names: found,
            },
            names_seen,
        };
    }
    let Some((name, decision)) = attributing.pop() else {
        return WhatTheWalkFound {
            registration: TheRegistration::None,
            names_seen,
        };
    };
    let path: PathBuf = worktrees.join(name.as_str());
    return WhatTheWalkFound {
        registration: TheRegistration::One {
            name,
            path,
            decision,
        },
        names_seen,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_finding_tests.rs"]
mod tests;
