//! What:
//!  Whether a Git registration stands for one plan's clone,
//!  and the settle of that registration with the proof the pass read from the pool.
//! Why:
//!  The list settles the registration at its step 11,
//!  before the binding's files go at step 12,
//!  and only for a registration whose directory identity is the one the plan recorded
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!  A registration with another identity is another removal's,
//!  so the pass leaves it;
//!  the settle itself refuses a registration whose identity differs from the plan's,
//!  which is why this file reads the identity first,
//!  and releases the managed lock the way every command that holds the slot does.
//!  The plan is the caller's proof that the clone is gone:
//!  the pass reads the pool before it reaches this file,
//!  and the destruction proof is made from that absence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = theRegistrationStandingFor(commonDir, plan);
//! const settled = settleThePlansRegistration(commonDir, slot, plan, settle);
//! ```

use crate::fs_beneath::{ObjectId, identity_of};
use crate::worktree_clone_managed_lock::clone_destruction_proof::{CloneDestructionProof, TheLiveCloneAnswer, prove_the_recorded_clone_is_gone};
use crate::worktree_clone_managed_lock::recorded_lock_facts::{RecordedClone, RecordedLockFacts, read_the_recorded_facts};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{RegistrationSettle, SettleAnswer, SettleRequest};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  Which registration the plan's own registration name holds.
/// Why:
///  The plan records the directory's name and its identity,
///  so a directory of another identity is a registration a later add made,
///  and this plan's removal has nothing to settle there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegistrationStanding = 'absent' | 'this-plans' | 'anothers' | { unreadable; path; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistrationStanding {
    /// Nothing stands under the registration's name.
    Absent,
    /// The directory's identity is the plan's `admin`.
    ThisPlans,
    /// A directory of another identity stands there.
    Anothers,
    /// The directory could not be read,
    /// or the plan names no directory name,
    /// so nothing proves whose registration it is.
    Unreadable {
        /// The registration's path.
        path: PathBuf,
        /// What the call answered.
        said: String,
    },
}

/// What:
///  The registration a plan's removal would settle,
///  read from the plan's own name and identity.
/// Why:
///  The reuse arm and the settle both ask this one question,
///  and one answer keeps them from disagreeing about who owns the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = theRegistrationStandingFor(commonDir, plan);
/// ```
pub fn the_registration_standing_for(common_dir: &Path, plan: &DestructionPlan) -> RegistrationStanding {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let path: PathBuf = worktrees.join(OsStr::from_bytes(plan.facts.registration_id.as_slice()));
    if plan.facts.registration_id.is_empty() {
        return RegistrationStanding::Unreadable { path, said: String::from("the plan names no registration directory name, so no registration can be matched to it") };
    }
    let opened: std::fs::File = match std::fs::File::open(path.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return RegistrationStanding::Absent,
        Err(error) => return RegistrationStanding::Unreadable { path, said: error.to_string() },
    };
    let live: ObjectId = match identity_of(&opened) {
        Ok(found) => found,
        Err(error) => return RegistrationStanding::Unreadable { path, said: error.to_string() },
    };
    return match live.encode() == plan.facts.admin {
        true => RegistrationStanding::ThisPlans,
        false => RegistrationStanding::Anothers,
    };
}

/// What:
///  The Git settle of one plan's registration,
///  with the proof the pass made from the pool's own absence.
/// Why:
///  The settle is the one step that removes a registration,
///  a pointer and a folder the next add reuses,
///  so the proof travels with the request and the live settle compares it with the record it settles.
///  The empty answer is the state a kill after step 11 leaves:
///  the binding records no registration, so there is nothing to settle.
///
/// # Throws
/// A binding whose facts cannot be read,
/// a proof that cannot be made,
/// or the settle's own refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = settleThePlansRegistration(commonDir, slot, plan, settle);
/// ```
pub fn settle_the_plans_registration(common_dir: &Path, slot: &str, plan: &DestructionPlan, settle: &dyn RegistrationSettle) -> Result<SettleAnswer, RemovalStop> {
    let facts: RecordedLockFacts = read_the_recorded_facts(common_dir, slot).map_err(|refusal| {
        return RemovalStop::TheSettleRefused { step: String::from("the binding's lock facts"), said: format!("{refusal:?}") };
    })?;
    let Some(_registration) = facts.registration.clone() else {
        return Ok(SettleAnswer { wrote_the_lost_lock: false, released_the_lock: false, removed_the_registration: false, removed_the_underlay: false, lines: Vec::new() });
    };
    let clone: RecordedClone = RecordedClone { dataset: plan.facts.clone_dataset.clone(), guid: Some(plan.facts.clone_guid) };
    let proof: CloneDestructionProof = prove_the_recorded_clone_is_gone(slot, &clone, &TheLiveCloneAnswer::Absent).map_err(|refusal| {
        return RemovalStop::TheSettleRefused { step: String::from("the destruction proof"), said: refusal.words() };
    })?;
    return settle.settle_the_registration(&SettleRequest {
        common_dir,
        slot,
        slot_path: plan.facts.target.as_path(),
        registration_id: plan.facts.registration_id.as_slice(),
        admin: plan.facts.admin.as_str(),
        slot_identity: facts.slot_identity,
        proof: &proof,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_plans_registration_tests.rs"]
mod tests;
