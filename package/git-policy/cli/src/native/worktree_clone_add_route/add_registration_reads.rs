//! What:
//!  Everything the add's scan of `<common>/worktrees` reads: the entries of
//!  that folder, a registration's `gitdir`, one record that may be absent, and
//!  the slots whose bindings already stand.
//! Why:
//!  The finder decides from these facts and nothing else, so they are read once,
//!  under the claim, and each entry is read through the one reader the lock's own
//!  classification uses
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resume after a kill between Git's add and the registered write").
//!  `add_registration.rs` would go over the file-size limit carrying the reads
//!  beside the decision it makes from them,
//!  `add_registration_lock.rs` settles the lock of the entry it adopts,
//!  and `add_registration_clearing.rs` clears the entry it refuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = readTheEntries(scan); // one EntryFacts per folder
//! const slots = otherSlotsWithBindings(scan.home, scan.slot);
//! const gitdir = readTheGitdir(registration); // resolved against the folder
//! ```

use crate::private_storage::read_regular_file;
use crate::worktree_add_binding::resolve_lexically;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use crate::worktree_clone_binding_record::registration_home::record_name_of;
use crate::worktree_clone_managed_lock::dataset_guid_reads::DatasetGuidReads;
use crate::worktree_clone_managed_lock::registration_entry_reading::read_one_entry;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Everything the scan reads.
/// Why:
///  The slot and its path are the finder's key, and the home answers which
///  other slots own a candidate name.
pub struct ScanInput<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The wrapper's record home,
    /// which holds the other slots' bindings.
    pub home: &'a Path,
    /// The slot's name.
    pub slot: &'a str,
    /// The slot's whole path.
    pub slot_path: &'a Path,
    /// This add's nonce,
    /// which recomposes the sentence.
    pub nonce: &'a RequestNonce,
    /// The pool's GUID read,
    /// which the lock's mounted arm compares the record's `cloneGuid=` with.
    pub guids: &'a dyn DatasetGuidReads,
}

/// The most bytes of a `gitdir` or pointer record this scan reads,
/// the path bound plus the pointer's prefix and its newline.
pub(super) const RECORD_LIMIT: u64 = 8 + 4096 + 2;

/// Read every entry of `<common>/worktrees`, in the listing's order,
/// each through the one entry reader the classification uses.
pub(super) fn read_the_entries(input: &ScanInput<'_>) -> Result<Vec<EntryFacts>, String> {
    let parent: PathBuf = input.common_dir.join("worktrees");
    let listing: std::fs::ReadDir = match std::fs::read_dir(parent.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "the registrations in {} cannot be listed ({error})",
                parent.display()
            ));
        }
    };
    let mut names: Vec<OsString> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| {
            return format!(
                "the registrations in {} cannot be listed ({error})",
                parent.display()
            );
        })?;
        names.push(entry.file_name());
    }
    names.sort();
    let mut facts: Vec<EntryFacts> = Vec::with_capacity(names.len());
    for name in names {
        let Some(text) = name.to_str() else { continue };
        let registration: PathBuf = parent.join(&name);
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(registration.as_path()) {
            Ok(found) => found,
            Err(_gone) => continue,
        };
        if !metadata.is_dir() {
            continue;
        }
        facts.push(read_one_entry(parent.as_path(), text, input.slot_path));
    }
    return Ok(facts);
}

/// The path a registration's `gitdir` names,
/// relative paths resolved against the registration.
pub(super) fn read_the_gitdir(registration: &Path) -> Result<Option<PathBuf>, String> {
    let Some(bytes) = read_optional(&registration.join("gitdir"), RECORD_LIMIT)? else {
        return Ok(None);
    };
    let line: &[u8] = bytes.strip_suffix(b"\n").unwrap_or(bytes.as_slice());
    return match resolve_lexically(registration, line) {
        Ok(resolved) => Ok(Some(resolved)),
        Err(()) => Err(format!(
            "the registration {} names a place outside the file system root; Ways forward: inspect its `gitdir`, then remove what is not a worktree by hand",
            registration.display()
        )),
    };
}

/// One record that may be absent,
/// for the reads this scan makes outside the entries.
pub(super) fn read_optional(
    path: &Path,
    limit: u64,
) -> Result<Option<Vec<u8>>, String> {
    return match read_regular_file(path, limit) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(crate::private_storage::ReadRefusal::Missing) => Ok(None),
        Err(refusal) => Err(format!(
            "the record {} cannot be read ({refusal:?})",
            path.display()
        )),
    };
}

/// Every slot whose binding stands in the home, except this one.
pub(super) fn other_slots_with_bindings(
    home: &Path,
    slot: &str,
) -> Result<Vec<String>, String> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(home) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "the record home {} cannot be listed ({error})",
                home.display()
            ));
        }
    };
    let mut slots: Vec<String> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| {
            return format!(
                "the record home {} cannot be listed ({error})",
                home.display()
            );
        })?;
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        let Some(record) = record_name_of(name.as_str()) else {
            continue;
        };
        if record.slot != slot
            && record.kind
                == crate::worktree_clone_binding_record::registration_home::RecordKind::Binding
        {
            slots.push(record.slot);
        }
    }
    return Ok(slots);
}
