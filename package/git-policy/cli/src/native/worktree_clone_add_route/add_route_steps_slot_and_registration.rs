//! What:
//!  The add's first two steps:
//!  the empty slot and the identity it is recorded with,
//!  and Git's registration with the scan that adopts it or clears it.
//! Why:
//!  `add_route_steps.rs` runs every step and would go over the file-size limit
//!  carrying these too,
//!  and these two are the steps that write before any clone exists,
//!  so a reader meets what a kill at the add's start leaves in one place.
//!  Both answer with the refusal and write through the record functions the
//!  steps' shared file holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await stepSlot(steps); await stepGit(steps);
//! ```

use crate::worktree_clone_add_route::add_git_words::the_locked_add_words;
use crate::worktree_clone_add_route::add_registration::{Registration, ScanInput, ScanOutcome, scan_and_settle};
use super::facts_and_refusal::{StepRefusal, Steps, write_the_record};
use crate::forwarding::{ChildOutcome, run_real_git};
use crate::hooks_off::StartOwner;
use crate::private_storage::create_private_directory_all;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use std::ffi::OsString;
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _};
use std::path::Path;

/// What:
///  Make or verify the empty slot and record its identity.
/// Why:
///  Git accepts an existing empty slot and nothing else,
///  and the identity is recorded before any mount covers the path.
pub fn step_slot(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    let slot: &Path = steps.route.destination.as_path();
    let existing: Option<FolderIdentity> = match std::fs::symlink_metadata(slot) {
        Ok(metadata) => Some(verify_the_slot(&metadata, slot)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(StepRefusal::refused(format!("the slot {} cannot be examined ({error})", slot.display()))),
    };
    let identity: FolderIdentity = match existing {
        Some(found) => found,
        None => make_the_slot(slot)?,
    };
    if record.slot_identity == Some(identity) {
        return Ok(());
    }
    record.slot_identity = Some(identity);
    record.phase = Phase::Slot;
    write_the_record(steps, record)?;
    steps.stops.after("slot-identity");
    return Ok(());
}

/// Verify one existing slot: a directory of this account, empty, on the
/// checkout's own dataset.
fn verify_the_slot(metadata: &std::fs::Metadata, slot: &Path) -> Result<FolderIdentity, StepRefusal> {
    if !metadata.is_dir() {
        return Err(StepRefusal::refused(format!("the slot {} holds something that is not a folder; Ways forward: remove it, then run the add again", slot.display())));
    }
    // SAFETY: `geteuid` reads this process's own identity and has no preconditions.
    let ours: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != ours {
        return Err(StepRefusal::refused(format!("the slot {} belongs to account {}, and this is account {ours}; Ways forward: remove it as its owner, then run the add again", slot.display(), metadata.uid())));
    }
    let mut entries = std::fs::read_dir(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be listed ({error})", slot.display())))?;
    if entries.next().is_some() {
        return Err(StepRefusal::refused(format!(
            "the slot {} holds entries, and the clone route registers a worktree only in an empty slot; Ways forward: remove the entries or name another worktree",
            slot.display()
        )));
    }
    return FolderIdentity::of_folder(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", slot.display())));
}

/// Make the empty slot with this account's ownership.
fn make_the_slot(slot: &Path) -> Result<FolderIdentity, StepRefusal> {
    let parent: &Path = slot.parent().ok_or_else(|| return StepRefusal::refused(format!("the slot {} names no parent", slot.display())))?;
    create_private_directory_all(parent).map_err(|error| return StepRefusal::refused(format!("the slot's folder {} cannot be made ({error})", parent.display())))?;
    let mut builder: std::fs::DirBuilder = std::fs::DirBuilder::new();
    builder.mode(0o755);
    builder.create(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be made ({error})", slot.display())))?;
    steps_note_after_making(slot)?;
    return FolderIdentity::of_folder(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", slot.display())));
}

/// Make sure the slot Git receives is a plain empty folder on the checkout.
fn steps_note_after_making(slot: &Path) -> Result<(), StepRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be examined ({error})", slot.display())))?;
    if !metadata.is_dir() {
        return Err(StepRefusal::refused(format!("the slot {} is no folder after it was made", slot.display())));
    }
    return Ok(());
}

/// What:
///  Scan for the registration,
///  adopt the complete one,
///  clear the incomplete one and run Git,
///  or run Git when nothing stands.
/// Why:
///  Git's own add is the only writer of the lock and the registration,
///  so the scan's answer decides whether it runs again at all,
///  and the line a lock put back owes a person is printed here,
///  once,
///  where the command's other lines go.
pub fn step_git(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    for _round in 0..3 {
        let outcome: ScanOutcome = scan_and_settle(&ScanInput { common_dir: steps.route.common_dir.as_path(), home: steps.home_path, slot: steps.route.slot.as_str(), slot_path: steps.route.destination.as_path(), nonce: &record.nonce, guids: steps.device })
            .map_err(StepRefusal::refused)?;
        match outcome {
            ScanOutcome::Adopt { registration, wrote_the_lock } => {
                if let Some(line) = wrote_the_lock {
                    steps.out.borrow_mut().push(format!("monochromatic-git-policy-cli: {line}\n"));
                }
                return adopt(steps, record, &registration);
            }
            ScanOutcome::RunGit { cleared } => {
                if !cleared.is_empty() {
                    steps.out.borrow_mut().push(format!("monochromatic-git-policy-cli: cleared an incomplete registration at {} ({}).\n", steps.route.slot, cleared.join(", ")));
                }
                // Git is about to run, so the slot must be the empty folder its
                // own add requires, with its identity recorded before the mount
                // can cover it. A slot Git already wrote to is the scan's
                // business above and never this step's.
                step_slot(steps, record)?;
                run_the_git_add(steps, &record.nonce)?;
                steps.stops.after("git-add");
            }
        }
    }
    return Err(StepRefusal::refused(format!(
        "the registration for {} kept changing while the add ran; the claim, the slot and whatever Git left stand, and running the add again resumes them",
        steps.route.slot
    )));
}

/// Record the adopted registration's facts and move to `registered`.
fn adopt(steps: &Steps<'_>, record: &mut BindingRecord, registration: &Registration) -> Result<(), StepRefusal> {
    // A registration adopted at phase `claimed` was made without A4's write, and
    // every later reader needs the identity of the folder it names.
    if record.slot_identity.is_none() {
        record.slot_identity = Some(FolderIdentity::of_folder(steps.route.destination.as_path()).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", steps.route.destination.display())))?);
    }
    record.registration = Some(registration.path.as_os_str().as_encoded_bytes().to_vec());
    record.registration_identity = Some(registration.identity);
    record.pointer = Some(registration.pointer.clone());
    record.phase = Phase::Registered;
    write_the_record(steps, record)?;
    steps.stops.after("registered");
    return Ok(());
}

/// Run Git's own add with the managed lock, keeping Git's status.
fn run_the_git_add(steps: &Steps<'_>, nonce: &crate::worktree_clone_mount_protocol::RequestNonce) -> Result<(), StepRefusal> {
    let words: Vec<OsString> = the_locked_add_words(steps.arguments, steps.plan.insert_at, nonce).map_err(StepRefusal::refused)?;
    let ran: ChildOutcome = run_real_git(steps.real_git, words.as_slice(), steps.overlay, StartOwner::Person).map_err(|error| {
        return StepRefusal::refused(format!("Git's own add could not be started ({error}); the claim, the slot and the records stand, and running the add again resumes them"));
    })?;
    return match ran {
        ChildOutcome::Exited(0) => Ok(()),
        ChildOutcome::Exited(code) => Err(StepRefusal::git(code)),
        ChildOutcome::Signaled(signal) => Err(StepRefusal::git(128 + signal)),
    };
}
