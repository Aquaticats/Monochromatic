//! What:
//!  The owner a claim file names and whether that process still runs:
//!  its pid,
//!  its birth identity and its PID namespace,
//!  and the standing of an owner in another namespace,
//!  whose claim is trusted by its age because it cannot be probed.
//! Why:
//!  A number alone proves nothing,
//!  because the kernel hands it out again;
//!  a claim is taken over only when the owner it records is gone with that birth,
//!  or gave the item back,
//!  and a command must be able to tell "cannot be decided" from "gone"
//!  so that a live owner is never taken from.
//!  This process's own record is made here too,
//!  so the identity a claim is written with and read by is one pair of calls.
//!
//! Files beside this one,
//!  and why the claim is not one file:
//!  `single_owner_claim_record.rs` holds the text of one claim file,
//!  `single_owner_claim_files.rs` holds the numbered files a claim is kept in,
//!  and `single_owner_claim.rs` takes the claim and gives it back.
//!  `rust/max-lines` splits the claim by role,
//!  and deciding whether an owner still runs is the role a reader can hold alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const liveOwner = (record: ClaimRecord, path: string): number | undefined => { /* ... */ };
//! const ownRecord = (): ClaimRecord => ({ kind: 'owned', pid: process.pid, birth: birthOf(process.pid), namespace: currentNamespace() });
//! ```

/// The namespace an owner record names,
///  and the standing of an owner of another one.
use crate::pid_namespace::{
    ForeignStanding,
    current_pid_namespace,
    foreign_owner_standing,
};
/// The birth identity that proves a number still names the process it named,
///  read for this process and for an owner.
use crate::process_identity::{
    current_birth_identity,
    process_birth_identity,
};
/// The record a claim file holds,
///  whose owner is what this file judges.
use crate::single_owner_claim_record::ClaimRecord;
/// The refusal a claim that cannot be judged fails with.
use crate::subvolume_error::MigrateError;
/// The path of the claim file a refusal names.
use std::path::Path;

/// The process id of the owner a record names when that process is alive,
/// nothing when the claim is released or its owner is
///  gone,
/// and an error when that cannot be decided.
pub(crate) fn live_owner(
    record: &ClaimRecord,
    path: &Path,
) -> Result<Option<i64>, MigrateError> {
    let ClaimRecord::Owned {
        pid,
        birth,
        namespace,
    } = record
    else {
        return Ok(None);
    };
    match foreign_owner_standing(namespace.as_deref(), path) {
        Some(ForeignStanding::Standing) => return Ok(Some(*pid)),
        // The claim of another namespace that its owner removed is a finished owner's: nothing is held, so nothing is taken
        //  from a live owner either (cross-vendor review of 2026-11-09, finding 3).
        Some(ForeignStanding::Finished) => return Ok(None),
        // Owner, 2026-11-04, "Refuse and name it (Recommended)": an old claim of another namespace is neither taken nor trusted.
        Some(ForeignStanding::Refused(refused)) => {
            return Err(MigrateError::Journal(refused.describe(*pid)));
        }
        None => {}
    }
    return match process_birth_identity(*pid) {
        Ok(Some(current)) => Ok((&current == birth).then_some(*pid)),
        Ok(None) => Ok(None),
        Err(error) => Err(MigrateError::Journal(format!(
            "{} names process {pid}, and whether it still runs cannot be decided ({error}); nothing was changed. If it does not, remove the file and run the command again",
            path.display()
        ))),
    };
}

/// This process's claim record.
pub(crate) fn own_record() -> Result<ClaimRecord, MigrateError> {
    let birth: String = match current_birth_identity() {
        Ok(identity) => identity,
        Err(error) => {
            return Err(MigrateError::Journal(format!(
                "this process cannot be identified for a claim ({error}); nothing was changed"
            )));
        }
    };
    return Ok(ClaimRecord::Owned {
        pid: i64::from(std::process::id()),
        birth,
        namespace: current_pid_namespace(),
    });
}
