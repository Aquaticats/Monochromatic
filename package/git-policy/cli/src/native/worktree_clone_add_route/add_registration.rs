//! What:
//!  The add's scan of `<common>/worktrees` after Git's own add: what each entry
//!  is, which of them is this add's, and what the resume does with it.
//! Why:
//!  A kill inside Git's own `worktree add` can leave ten shapes, and only a scan
//!  that reads each entry's lock,
//!  its `gitdir` back-link and the slot's pointer can tell them apart
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resume after a kill between Git's add and the registered write").
//!  The finder and the rows are the registration half's; this file is the I/O
//!  that feeds them and applies their answers through the recipe, so the pure
//!  rules stay controlled on any machine and the reads happen once, under the
//!  claim.
//!  The entry read is the managed lock's own
//!  (`worktree_clone_managed_lock::read_one_entry`),
//!  so the add's resume and the classification of a lock read one entry the same way.
//!  A complete registration is then adopted through the one classification
//!  `doctor` and the removal share:
//!  the whole lock is taken as it stands,
//!  a cut-short one is completed in place,
//!  a lost one is written again and the one line printed,
//!  and words this add did not write refuse with the words a person acts on
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "A lost lock is written again, and which commands do it").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = scanTheRegistration({ commonDir, home, slot, slotPath, nonce });
//! if (found.kind === 'adopt') return found.registration;
//! ```

use crate::private_storage::read_regular_file;
use crate::worktree_add_binding::resolve_lexically;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::record_publication::RecordRefusalAtTheFile;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use crate::worktree_clone_binding_record::registration_home::record_name_of;
use crate::worktree_clone_binding_record::registration_recipe::{
    NoSeams,
    RecipeRefusal,
    ToClear,
    clear_registration,
    complete_a_cut_short_lock,
};
use crate::worktree_clone_binding_record::registration_resume::{
    EntryDecision,
    Finding as RegistrationFinding,
    ListingDecision,
    find_registrations,
};
use crate::worktree_clone_managed_lock::dataset_guid_reads::DatasetGuidReads;
use crate::worktree_clone_managed_lock::lock_classification::classify_with_a_known_slot_identity;
use crate::worktree_clone_managed_lock::lock_installation::{
    InstallAnswer,
    write_the_lost_lock,
};
use crate::worktree_clone_managed_lock::lock_state::{
    Classification,
    ManagedLockState,
};
use crate::worktree_clone_managed_lock::lock_state_words::the_lines;
use crate::worktree_clone_managed_lock::registration_entry_reading::read_one_entry;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};

/// The most bytes of a `gitdir` or pointer record this scan reads,
/// the path bound plus the pointer's prefix and its newline.
const RECORD_LIMIT: u64 = 8 + 4096 + 2;

/// What:
///  The registration this add owns, proved from the five facts.
/// Why:
///  The binding records the registration's path,
///  its identity and the pointer's bytes,
///  so every later step has the two facts without a scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Registration = { path; identity; pointer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    /// The registration's directory below `<common>/worktrees`.
    pub path: PathBuf,
    /// Its identity,
    /// the inode with its birth time.
    pub identity: FolderIdentity,
    /// The bytes of the slot's `.git` file.
    pub pointer: Vec<u8>,
}

/// What:
///  What one scan settles.
/// Why:
///  The caller either has the registration,
///  or must run Git's own add,
///  or has just cleared an incomplete one and runs Git next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanOutcome = { kind: 'adopt'; registration } | { kind: 'runGit' } | { kind: 'cleared'; names: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanOutcome {
    /// A complete registration this add owns,
    /// with the one line a lock put back owes a person when it wrote one.
    Adopt {
        /// The registration the scan proved.
        registration: Registration,
        /// The line the command prints once,
        /// present only when this command wrote a lost lock again.
        wrote_the_lock: Option<String>,
    },
    /// Nothing stands for this slot, or the incomplete entry was cleared.
    RunGit {
        /// What the recipe removed,
        /// in the order it removed it,
        /// empty when nothing stood.
        cleared: Vec<String>,
    },
}

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

/// What:
///  Scan the registrations and settle what stands for this add.
/// Why:
///  The finder decides from the scan's facts, and this function performs the one
///  decision it gives:
///  adopt the complete entry,
///  clear the incomplete one through the recipe,
///  or refuse and change nothing.
///
/// # Errors
/// A record cannot be read,
/// the finder refused,
/// or the recipe stopped at a name it would not remove.
pub fn scan_and_settle(input: &ScanInput<'_>) -> Result<ScanOutcome, String> {
    let entries: Vec<EntryFacts> = read_the_entries(input)?;
    let other_slots: Vec<String> = other_slots_with_bindings(input.home, input.slot)?;
    let slot_pointer: Option<Vec<u8>> =
        read_optional(input.slot_path.join(".git").as_path(), RECORD_LIMIT)?;
    let finding: RegistrationFinding = find_registrations(
        entries.as_slice(),
        input.slot,
        input.slot_path,
        input.nonce,
        other_slots.as_slice(),
        slot_pointer.is_some(),
    );
    match finding.decision {
        ListingDecision::Entry(EntryDecision::Adopt) => {
            let entry: &EntryFacts = entries
                .iter()
                .find(|entry| {
                    return entry.gitdir.as_deref() == Some(input.slot_path.join(".git").as_path());
                })
                .ok_or_else(|| {
                    return String::from(
                        "the finder adopted a registration the scan does not name",
                    );
                })?;
            let registration: PathBuf =
                input.common_dir.join("worktrees").join(entry.name.as_str());
            let wrote_the_lock: Option<String> = settle_the_lock(input, registration.as_path())?;
            return Ok(ScanOutcome::Adopt {
                registration: prove_the_registration(
                    &registration,
                    input.slot_path,
                    slot_pointer.as_deref(),
                )?,
                wrote_the_lock,
            });
        }
        ListingDecision::Entry(EntryDecision::Clear {
            row: _,
        }) => {
            let entry: &EntryFacts = entries
                .iter()
                .find(|entry| {
                    return entry.gitdir.as_deref() == Some(input.slot_path.join(".git").as_path())
                        || (entry.gitdir.is_none()
                            && entry.slot_git_pointer.is_none()
                            && entry.slot_other_entries == 0);
                })
                .ok_or_else(|| {
                    return String::from("the finder cleared an entry the scan does not name");
                })?;
            let cleared: Vec<String> = clear_the_entry(input, entry)?;
            return Ok(ScanOutcome::RunGit {
                cleared,
            });
        }
        ListingDecision::Entry(EntryDecision::Refuse {
            sentence,
            ..
        }) => return Err(with_a_way_forward(sentence)),
        ListingDecision::Entry(EntryDecision::Leave {
            ..
        })
        | ListingDecision::RerunTheGitAdd => {
            return Ok(ScanOutcome::RunGit {
                cleared: Vec::new(),
            });
        }
        ListingDecision::Refuse {
            sentence,
        } => return Err(with_a_way_forward(sentence)),
    }
}

/// A refusal's sentence with the way forward the route's own failures carry.
fn with_a_way_forward(sentence: String) -> String {
    return format!(
        "{sentence} The add left the claim, the slot and the registration as they are, and running it again resumes them."
    );
}

/// What:
///  Settle the lock of the registration the scan has just adopted:
///  take the whole one,
///  complete the cut-short one in place,
///  write a lost one again and answer the one line,
///  and refuse words this add did not write.
/// Why:
///  The project decided on 2026-11-22 that any wrapped command which holds the
///  slot's claim and has proved the registration writes a lost lock again and says
///  so,
///  and the lock's own state is the classification's answer rather than a second
///  reading here
///  (`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`;
///  `doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
///  "The states, the checks and the words").
///  A lock the classification calls a person's is never overwritten,
///  and the sentence a person reads is the classification's own.
fn settle_the_lock(
    input: &ScanInput<'_>,
    registration: &Path,
) -> Result<Option<String>, String> {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    // A record whose add was killed before its own identity write stands at phase `claimed`,
    // and this command made the folder, so the identity the proof compares is its own reading.
    let known: Option<FolderIdentity> = FolderIdentity::of_folder(input.slot_path).ok();
    let classification: Classification = classify_with_a_known_slot_identity(
        input.common_dir,
        input.slot,
        input.slot_path,
        &mounts,
        input.guids,
        known,
    );
    match &classification.state {
        ManagedLockState::Managed {
            ..
        } => return Ok(None),
        ManagedLockState::CutShort {
            ..
        } => {
            complete_a_cut_short_lock(registration, input.nonce).map_err(|error| {
                return format!(
                    "the cut-short lock in {} cannot be completed ({error})",
                    registration.display()
                );
            })?;
            return Ok(None);
        }
        ManagedLockState::NoLock {
            ..
        } => {
            return match write_the_lost_lock(registration, input.slot, input.nonce) {
                // A lock the classification read as absent and this write found standing:
                // the state is named by the classification rather than replaced by a write.
                InstallAnswer::ALockStands => Err(format!(
                    "the lock of `{}` stands and this command cannot attribute it, so nothing was rewritten. Ways forward: run `git worktree unlock {}` if no person holds that lock, then run the add again",
                    registration.display(),
                    input.slot
                )),
                InstallAnswer::Refused {
                    reason,
                } => Err(format!(
                    "the lost managed Git lock of `{}` cannot be written again ({reason}), so nothing beyond the claim was changed. Ways forward: inspect the registration's folder and run the add again",
                    registration.display()
                )),
                InstallAnswer::Wrote {
                    line,
                } => Ok(Some(line.clone())),
            };
        }
        _refusing => {}
    }
    return Err(format!(
        "{} Nothing was removed or rewritten. Ways forward: read the line above, settle the lock it names, then run the add again",
        the_lines(&classification, input.slot).join(" ")
    ));
}

/// Read every entry of `<common>/worktrees`, in the listing's order,
/// each through the one entry reader the classification uses.
fn read_the_entries(input: &ScanInput<'_>) -> Result<Vec<EntryFacts>, String> {
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
fn read_the_gitdir(registration: &Path) -> Result<Option<PathBuf>, String> {
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
fn read_optional(
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
fn other_slots_with_bindings(
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

/// Prove the five facts of one registration and read the facts the binding
/// records from it.
fn prove_the_registration(
    registration: &Path,
    slot_path: &Path,
    pointer: Option<&[u8]>,
) -> Result<Registration, String> {
    let gitdir: Option<PathBuf> = read_the_gitdir(registration)?;
    if gitdir.as_deref() != Some(slot_path.join(".git").as_path()) {
        return Err(format!(
            "the registration {} does not name {}/.git, so nothing proves it belongs to this slot",
            registration.display(),
            slot_path.display()
        ));
    }
    if !registration.join("commondir").is_file() || !registration.join("HEAD").is_file() {
        return Err(format!(
            "the registration {} holds no `commondir` or no `HEAD`, so it is no complete registration",
            registration.display()
        ));
    }
    let Some(found_pointer) = pointer else {
        return Err(format!(
            "the slot {} holds no `.git` pointer, and the registration names it; Ways forward: inspect the slot and run the add again",
            slot_path.display()
        ));
    };
    let identity: FolderIdentity = FolderIdentity::of_folder(registration).map_err(|error| {
        return format!(
            "the registration {} cannot be identified ({error})",
            registration.display()
        );
    })?;
    return Ok(Registration {
        path: registration.to_path_buf(),
        identity,
        pointer: found_pointer.to_vec(),
    });
}

/// Clear one incomplete entry through the recipe and answer what went.
fn clear_the_entry(
    input: &ScanInput<'_>,
    entry: &EntryFacts,
) -> Result<Vec<String>, String> {
    let slot_parent_path: PathBuf = input
        .slot_path
        .parent()
        .ok_or_else(|| return format!("the slot {} names no parent", input.slot_path.display()))?
        .to_path_buf();
    let worktrees_parent_path: PathBuf = input.common_dir.join("worktrees");
    let slot_parent: std::fs::File = crate::fs_beneath::open_root(slot_parent_path.as_path())
        .map_err(|error| {
            return format!(
                "the worktrees folder {} cannot be opened ({error})",
                slot_parent_path.display()
            );
        })?;
    let worktrees_parent: std::fs::File =
        crate::fs_beneath::open_root(worktrees_parent_path.as_path()).map_err(|error| {
            return format!(
                "the registrations folder {} cannot be opened ({error})",
                worktrees_parent_path.display()
            );
        })?;
    let registration: PathBuf = worktrees_parent_path.join(entry.name.as_str());
    let to: ToClear<'_> = ToClear {
        slot_parent: &slot_parent,
        slot_pointer_name: input
            .slot_path
            .file_name()
            .ok_or_else(|| return String::from("the slot names no folder"))?,
        slot: input.slot_path,
        worktrees_parent: &worktrees_parent,
        registration_name: OsStr::new(entry.name.as_str()),
        registration: registration.as_path(),
    };
    clear_registration(&to, &mut NoSeams)
        .map_err(|refusal: RecipeRefusal| return refusal.message())?;
    return Ok(names_of_the_removed(entry));
}

/// The names one cleared row had,
/// in the recipe's order,
/// for the line the person reads.
fn names_of_the_removed(entry: &EntryFacts) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if entry.slot_git_pointer.is_some() {
        names.push(String::from("the slot's `.git`"));
    }
    if entry.gitdir.is_some() {
        names.push(String::from("gitdir"));
    }
    if entry.commondir {
        names.push(String::from("commondir"));
    }
    if entry.head {
        names.push(String::from("HEAD"));
    }
    if entry.locked.is_some() {
        names.push(String::from("locked"));
    }
    names.push(format!("worktrees/{}", entry.name));
    return names;
}

/// A record refusal's sentence, which the scan reports with its path.
pub fn refusal_sentence(refusal: &RecordRefusalAtTheFile) -> String {
    return refusal.message(Path::new("the record"));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_registration_tests.rs"]
mod tests;
