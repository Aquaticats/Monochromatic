//! What:
//!  Every fact a damaged link is judged from,
//!  read once:
//!  the record's slot and registration,
//!  whether the registration still stands,
//!  whether its `gitdir` names this slot,
//!  the pointer at the slot against the recorded bytes,
//!  whether the record keeps those bytes,
//!  and which copy of the pointer the slot shows.
//! Why:
//!  The repair report and the guard before a prune decide from the same facts,
//!  and two readers that gathered them separately could word one state two
//!  ways
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  A registration that stands with no pointer is the state a repair can put
//!  right,
//!  a registration Git already pruned is not,
//!  and only the registration's own files and the slot's `.git` tell them
//!  apart.
//!  The facts are read from the few values a record proves,
//!  so a control builds them without a pool or an add.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = linkFactsOf('one', recordedLinkOf(record), cloneMountOf(rows, slotMount, clone, slot));
//! ```

use crate::hooks_path_values::without_trailing_newlines;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_managed_lock::reading_a_bounded_file::{Reading, read_a_bounded_file};
use crate::worktree_clone_managed_lock::registration_proof_two_facts::{GITDIR_FILE_NAME, REGISTRATION_RECORD_MAXIMUM_BYTES};
use crate::worktree_clone_prune_pointer::pointer_state::{PointerSite, PointerState, pointer_site_of, read_pointer_state_at};
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  The three facts a record holds about one link:
///  the slot's path,
///  the registration it names,
///  the pointer bytes Git was given,
///  and the clone dataset the worktree lives on.
/// Why:
///  The facts are read from a record whatever its phase,
///  and a control that needs no pool builds them directly,
///  so the report's rules never depend on a binding record's other fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedLink = { slotPath; registration?; pointer?; cloneDataset };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedLink<'a> {
    /// The slot's whole path.
    pub slot_path: PathBuf,
    /// The registration the record names,
    /// and nothing when it names none yet.
    pub registration: Option<PathBuf>,
    /// The exact pointer bytes the add recorded,
    /// and nothing when the record keeps none.
    pub pointer: Option<&'a [u8]>,
    /// The clone dataset the worktree lives on.
    pub clone_dataset: &'a str,
}

/// What:
///  What one slot's registration and its two pointer copies say.
/// Why:
///  The class follows from these fields alone,
///  so a control scripts them and the release path reads exactly the same
///  shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkFacts = { slot; slotPath; cloneDataset; registration?; registrationStands; registrationNamesTheSlot?; pointer; backup; site };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkFacts {
    /// The slot's name,
    /// which is the record's key.
    pub slot: String,
    /// The slot's whole path.
    pub slot_path: PathBuf,
    /// The clone dataset the record names.
    pub clone_dataset: String,
    /// The registration the record names,
    /// and nothing when it names none yet.
    pub registration: Option<PathBuf>,
    /// Whether the registration's directory stands,
    /// which is what a pruned entry no longer does.
    pub registration_stands: bool,
    /// Whether the registration's `gitdir` names this slot's `.git`,
    /// and nothing when there was no registration to read.
    pub registration_names_the_slot: Option<bool>,
    /// The pointer at the slot against the recorded bytes.
    pub pointer: PointerState,
    /// Whether the record keeps the exact pointer bytes,
    /// which is the backup a restore uses.
    pub backup: bool,
    /// Which copy of the pointer the slot shows.
    pub site: PointerSite,
}

/// What:
///  The values a record proves about one link.
/// Why:
///  One adapter reads a record's fields,
///  so no caller of the facts reaches into the record's grammar itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const link = recordedLinkOf(record);
/// ```
pub fn recorded_link_of(record: &BindingRecord) -> RecordedLink<'_> {
    return RecordedLink {
        slot_path: record.slot_path().unwrap_or_default(),
        registration: record.registration_path(),
        pointer: record.pointer.as_deref(),
        clone_dataset: record.clone_dataset.as_str(),
    };
}

/// What:
///  Whether a registration's `gitdir` names one slot's `.git`.
/// Why:
///  `git worktree repair` rewrites the worktree's pointer from this back-link,
///  so a `gitdir` naming another path would rebuild the wrong link,
///  and only this reading tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = registrationNamesTheSlot(registration, slotPath);
/// ```
pub fn registration_names_the_slot(registration: &Path, slot_path: &Path) -> bool {
    let reading: Reading = read_a_bounded_file(registration.join(GITDIR_FILE_NAME).as_path(), REGISTRATION_RECORD_MAXIMUM_BYTES);
    let Reading::Bytes(bytes) = reading else {
        return false;
    };
    let named: &[u8] = without_trailing_newlines(bytes.as_slice());
    let wanted: PathBuf = slot_path.join(OsStr::new(".git"));
    return named == wanted.as_os_str().as_encoded_bytes();
}

/// What:
///  Whether a registration's directory stands.
/// Why:
///  A record can outlive its registration,
///  and the answer is what tells a repair that can rebuild a link from one
///  that cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = registration !== undefined && isDirectory(registration);
/// ```
pub fn the_registration_stands(registration: Option<&Path>) -> bool {
    let Some(path) = registration else {
        return false;
    };
    return std::fs::symlink_metadata(path).is_ok_and(|found| return found.is_dir());
}

/// What:
///  Every fact one slot's damaged link is judged from,
///  read from the registration's own files and the slot's pointer.
/// Why:
///  One call fills the shape the report and the guard read,
///  so the pointer is compared once against the bytes the record holds and
///  the site is decided from the same mount reading the classification used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = linkFactsOf('one', recordedLinkOf(record), cloneMountOf(rows, slotMount, clone, slot));
/// ```
pub fn link_facts_of(slot: &str, link: &RecordedLink<'_>, mount: &CloneMount) -> LinkFacts {
    let stands: bool = the_registration_stands(link.registration.as_deref());
    let names: Option<bool> = match link.registration.as_deref() {
        Some(path) if stands => Some(registration_names_the_slot(path, link.slot_path.as_path())),
        _ => None,
    };
    return LinkFacts {
        slot: String::from(slot),
        slot_path: link.slot_path.clone(),
        clone_dataset: String::from(link.clone_dataset),
        registration: link.registration.clone(),
        registration_stands: stands,
        registration_names_the_slot: names,
        pointer: read_pointer_state_at(link.slot_path.as_path(), link.pointer),
        backup: link.pointer.is_some(),
        site: pointer_site_of(mount),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "link_facts_tests.rs"]
mod tests;
