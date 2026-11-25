//! What:
//!  The settling command's one entry:
//!  the reads the design orders,
//!  the two modes and the refusal,
//!  and the removals it makes once the person's flags confirm the record.
//! Why:
//!  A binding whose recorded clone is gone and whose absence cannot be proved
//!  keeps its slot refusing the next add,
//!  and the project chose a command that removes the record after a person
//!  confirms it
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A slot whose recorded clone is gone and cannot be proved gone";
//!  `doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command",
//!  and its veto entry V23).
//!  The reads are the design's six and they run in its order:
//!  the claim,
//!  the binding,
//!  the pinned parent,
//!  the slot,
//!  the origin cleanup record,
//!  and the plan folder.
//!  It is built ahead of the plan's order and no command reaches it:
//!  the round that switches removal on adds it to the management grammar and
//!  names it in every sentence that names this command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = settleTheRecordsOfASlot(commonDir, request, reads);
//! console.log(done.lines.join('\n'));
//! ```

use crate::lock::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_record_settling::settling_absence::{TheClonesPresence, the_clones_presence};
use crate::worktree_clone_record_settling::settling_lines::{TheEvidence, evidence_lines, last_line, removal_lines};
use crate::worktree_clone_record_settling::settling_record::{TheRecord, confirm_the_record, read_the_record};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_removals::{RemovalsGone, RemovalsToMake, make_the_removals, the_claim_prefix};
use crate::worktree_clone_record_settling::settling_request::SettlingRequest;
use crate::worktree_clone_record_settling::settling_slot::{TheSlot, read_the_slot};
use crate::worktree_clone_record_settling::settling_standing_records::{a_standing_origin_cleanup, a_standing_plan};
use crate::worktree_clone_record_settling::settling_steps::{SettlingStep, TheOrderFacts, the_steps};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::path::{Path, PathBuf};

/// What:
///  The two live sources one run reads besides the files:
///  the pool and this process's mount table.
/// Why:
///  A control answers both without a pool and without the host's mount table,
///  and the release path supplies the binding's own device and this process's
///  table,
///  so no control on a person's machine opens `/dev/zfs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reads = { probe: DatasetReader; mounts: MountedDatasets };
/// ```
pub struct SettlingReads<'a> {
    /// The pool's children and identities.
    pub probe: &'a dyn DatasetReader,
    /// This process's mount table.
    pub mounts: &'a dyn MountedDatasets,
}

/// What:
///  What a completed settling run did,
///  and the lines it prints.
/// Why:
///  The wiring round prints these lines and exits zero,
///  and a test asserts the steps and the names that went rather than reading
///  three folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Done = { lines: string[]; steps: SettlingStep[]; files: string[]; checkoutBegun: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlingDone {
    /// The report, in the order it prints.
    pub lines: Vec<String>,
    /// The steps the run made,
    /// in order.
    pub steps: Vec<SettlingStep>,
    /// The files that went,
    /// in order.
    pub files: Vec<String>,
    /// Whether a checkout had begun,
    /// which is what left the registration and the slot standing.
    pub checkout_begun: bool,
    /// Whether the lost lock was written again.
    pub lock_written: bool,
}

/// What:
///  The slot's claim,
///  or the stop a slot another command holds reads.
/// Why:
///  Two settling commands on one slot would remove each other's evidence,
///  so the claim is the design's first read and comes before the record.
///
/// # Throws
/// A live owner,
/// a claim that cannot be taken,
/// or a record home that is not there,
/// which is no record rather than a busy slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = takeTheSlotsClaim(commonDir, 'topic');
/// ```
fn take_the_slots_claim(common_dir: &Path, slot: &str) -> Result<OwnerClaim, SettlingStop> {
    let home: PathBuf = home_folder(common_dir);
    if !home.is_dir() {
        return Err(SettlingStop::NoRecord { path: home.join(binding_name(slot)) });
    }
    return match take_claim(home.as_path(), the_claim_prefix(slot).as_os_str()) {
        Ok(ClaimOutcome::Held(claim)) => Ok(claim),
        Ok(ClaimOutcome::Busy(owner)) => Err(SettlingStop::TheClaimIsHeld { slot: String::from(slot), pid: owner.pid }),
        Err(error) => Err(SettlingStop::TheClaimCannotBeTaken { slot: String::from(slot), said: format!("{error:?}") }),
    };
}

/// What:
///  The lines a completed run prints:
///  the evidence,
///  the removals,
///  what went or what was left,
///  and the cost the person confirmed.
/// Why:
///  One list keeps the order the design states,
///  so a person reads what was read before what was removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = doneLines(evidence, gone, steps, checkoutBegun, lockWritten);
/// ```
fn done_lines(TheDoneFacts { evidence, gone, checkout_begun, lock_written }: TheDoneFacts<'_>, steps: &[SettlingStep], slot: &str) -> Vec<String> {
    let mut lines: Vec<String> = evidence_lines(evidence);
    if lock_written {
        lines.push(String::from(
            "the registration's managed lock was gone or cut short, so it was written again with this add's sentence before the recipe removed it.",
        ));
    }
    lines.extend(removal_lines(steps));
    lines.push(what_went(gone, checkout_begun, slot));
    lines.push(last_line(slot));
    return lines;
}

/// The line naming what the removals left standing.
fn what_went(gone: &RemovalsGone, checkout_begun: bool, slot: &str) -> String {
    if checkout_begun {
        return format!(
            "the records are gone and the registration and the slot stay, because a checkout had begun there: run `git worktree remove {slot}`, and `--force` if Git refuses it because the worktree holds files it did not index."
        );
    }
    let registration: &str = match gone.registration_removed {
        true => "the registration and its lock are gone",
        false => "the registration was already gone",
    };
    return format!("{registration}, the records are gone, and the slot folder stays for a person: it is the underlay the worktree's files were written into, and nothing of it is this command's to remove.");
}

/// What:
///  The values the done lines print,
///  together so the line order and the values cannot drift apart.
/// Why:
///  A refusal prints nothing and a completed run prints everything,
///  so the two sets of values travel as one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DoneFacts = { evidence; gone; checkoutBegun; lockWritten };
/// ```
struct TheDoneFacts<'a> {
    /// What the reads answered.
    evidence: &'a TheEvidence<'a>,
    /// What the removals did.
    gone: &'a RemovalsGone,
    /// Whether a checkout had begun.
    checkout_begun: bool,
    /// Whether the lost lock was written again.
    lock_written: bool,
}

/// What:
///  Settle one slot's records,
///  after the two flags confirm the record this slot holds.
/// Why:
///  This is the whole command,
///  and its six reads run in the design's order so a refusal always names the
///  first thing it could not prove.
///
/// # Throws
/// The first read that refused,
/// with the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = settleTheRecordsOfASlot(commonDir, request, reads);
/// ```
pub fn settle_the_records_of_a_slot(common_dir: &Path, request: &SettlingRequest, reads: &SettlingReads<'_>) -> Result<SettlingDone, SettlingStop> {
    let mut claim: OwnerClaim = take_the_slots_claim(common_dir, request.slot.as_str())?;
    let record: TheRecord = read_the_record(common_dir, request.slot.as_str())?;
    confirm_the_record(&record.recorded, request)?;
    let presence: TheClonesPresence = the_clones_presence(reads.probe, record.recorded.clone_dataset.as_str(), record.recorded.clone_guid);
    if let Some(stop) = presence.stop_sentence() {
        return Err(stop);
    }
    let cleanup: Option<OriginCleanupRecord> = a_standing_origin_cleanup(common_dir, &record.recorded)?;
    let plan: Option<DestructionPlan> = a_standing_plan(common_dir, &record.recorded)?;
    let registration: PathBuf = registration_path(common_dir, &record);
    let slot: TheSlot = read_the_slot(record.recorded.slot_path.as_path(), registration.as_path(), record.recorded.registration_identity, reads.mounts)?;
    let facts: TheOrderFacts = TheOrderFacts { checkout_begun: slot.a_checkout_has_begun(), plan_stands: plan.is_some() };
    let steps: Vec<SettlingStep> = the_steps(facts);
    let to: RemovalsToMake<'_> = RemovalsToMake {
        common_dir,
        slot: request.slot.as_str(),
        slot_path: record.recorded.slot_path.as_path(),
        registration: registration.as_path(),
        nonce: &record.nonce,
        clone_guid: record.recorded.clone_guid,
    };
    let gone: RemovalsGone = make_the_removals(&to, steps.as_slice(), &mut claim)?;
    let evidence: TheEvidence<'_> = TheEvidence { record: &record, presence: &presence, slot: &slot, cleanup: cleanup.as_ref(), plan: plan.as_ref() };
    let lines: Vec<String> = done_lines(TheDoneFacts { evidence: &evidence, gone: &gone, checkout_begun: facts.checkout_begun, lock_written: gone.lock_written }, steps.as_slice(), request.slot.as_str());
    return Ok(SettlingDone { lines, steps: gone.steps, files: gone.files, checkout_begun: facts.checkout_begun, lock_written: gone.lock_written });
}

/// What:
///  The registration the record names,
///  built from Git's `worktrees` folder and the name the record holds.
/// Why:
///  The record holds the registration's last component and its identity,
///  and the recipe removes names below Git's own folder,
///  so one path serves the slot's pointer check and the recipe's parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(commonDir, 'worktrees', registrationName);
/// ```
fn registration_path(common_dir: &Path, record: &TheRecord) -> PathBuf {
    let name: std::ffi::OsString = std::ffi::OsString::from(String::from_utf8_lossy(record.recorded.registration_id.as_slice()).into_owned());
    return common_dir.join("worktrees").join(name);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_command_tests.rs"]
mod tests;
