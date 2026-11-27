//! What:
//!  The add's scan of `<common>/worktrees` after Git's own add: what each entry
//!  is, which of them is this add's, and what the resume does with it.
//! Why:
//!  A kill inside Git's own `worktree add` can leave ten shapes, and only a scan
//!  that reads each entry's lock,
//!  its `gitdir` back-link and the slot's pointer can tell them apart
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resume after a kill between Git's add and the registered write").
//!  The finder and the rows are the registration half's; this file is the
//!  decision that applies their answers, so the reads are
//!  `add_registration_reads.rs`'s,
//!  the adopted entry's lock is `add_registration_lock.rs`'s,
//!  and the incomplete entry's removal is `add_registration_clearing.rs`'s.
//!  A complete registration is then adopted through the one classification
//!  `doctor` and the removal share.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = scanAndSettle({ commonDir, home, slot, slotPath, nonce });
//! if (found.kind === 'adopt') return found.registration;
//! ```

use super::add_registration_clearing::clear_the_entry;
use super::add_registration_lock::settle_the_lock;
use super::add_registration_reads::{
    RECORD_LIMIT,
    other_slots_with_bindings,
    read_optional,
    read_the_entries,
    read_the_gitdir,
};
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::record_publication::RecordRefusalAtTheFile;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use crate::worktree_clone_binding_record::registration_resume::{
    EntryDecision,
    Finding as RegistrationFinding,
    ListingDecision,
    find_registrations,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Everything the scan reads.
/// Why:
///  The slot and its path are the finder's key, and the home answers which
///  other slots own a candidate name,
///  so the value is named here where the scan's decision is made and read
///  nowhere else.
pub use super::add_registration_reads::ScanInput;

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

/// A record refusal's sentence, which the scan reports with its path.
pub fn refusal_sentence(refusal: &RecordRefusalAtTheFile) -> String {
    return refusal.message(Path::new("the record"));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_registration_tests.rs"]
mod tests;
