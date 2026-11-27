//! What:
//! The removal of one qualified tree under the protocol every lean-folder tool uses:
//! take the profile directories' locks,
//! read the activity again,
//! scan every process,
//! claim the tree into the trash,
//! and delete the claim.
//! Why:
//! A tree that could not be emptied whole is reported as partly removed,
//! never as skipped,
//! because part of it is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function collectOne(tree: Tree, reason: string, trash: string, env: GuardEnvironment, now: number, needed: number, deadline: Deadline, log: string[]): TreeVerdict
//! ```
/// Imports the verdict this removal answers with.
use super::idle_policy_and_verdicts::TreeVerdict;
/// Imports the tree being removed,
/// the profile directories to lock and the activity read again under the locks.
use crate::lean_build_trees::{
    Tree,
    last_activity,
    profile_dirs_of,
};
/// Imports the locks a profile directory yields,
/// the environment the guard reads and the guarded handle the tree is held under.
use crate::lean_guard::{
    Guard,
    GuardEnvironment,
    Guarded,
    guard_profile,
};
/// Imports the run's deadline.
use crate::lean_identity::Deadline;
/// Imports the record of the process scan and the scan itself.
use crate::lean_proc_scan::{
    ScanReport,
    scan_processes,
};
/// Imports the claim and the deletion that finish a removal,
/// the words the proof line is made of,
/// and the rule the tool counts bytes by.
use crate::lean_removal::{
    Refusal,
    allocated_bytes_of,
    claim_into_trash,
    delete_claimed,
    item_of,
    modes_named,
};
/// Imports the streaming log line.
use crate::lean_run_report::emit;
/// Imports the borrowed and owned path types the item's name is built from.
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `pub(super) fn collect_one(tree, reason, trash, environment, now, needed, deadline, log) -> TreeVerdict`
/// removes one qualified tree under the protocol.
/// Why:
/// Unlinking what a directory holds needs write and search on it,
/// so a directory this user owns and may not write is made writable first and the removal's proof line says so;
/// a directory of another user stays as it is and is named.
/// A tree that could not be emptied whole is reported as partly removed,
/// never as skipped,
/// because part of it is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectOne(tree: Tree, reason: string, trash: string, env: GuardEnvironment, now: number, needed: number, deadline: Deadline, log: string[]): TreeVerdict
/// ```
pub(super) fn collect_one(
    tree: &Tree,
    reason: &str,
    trash: &Path,
    environment: &GuardEnvironment,
    now: i64,
    needed: i64,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> TreeVerdict {
    let mut held: Vec<Guarded> = Vec::new();
    for profile in profile_dirs_of(&tree.path) {
        match guard_profile(&profile, environment) {
            Guard::Ready(ready) => held.push(ready),
            Guard::Skip(why) => return TreeVerdict::Skip(format!("{}: {why}", profile.display())),
        }
    }
    match last_activity(&tree.path, deadline) {
        Some(activity) if now - activity > needed => {}
        Some(_) => {
            return TreeVerdict::Skip(String::from(
                "a write happened while the locks were being taken",
            ));
        }
        None => {
            return TreeVerdict::Skip(String::from("the deadline passed while reading the tree"));
        }
    }
    let scan: ScanReport = scan_processes(
        &environment.proc_root,
        &[tree.path.as_path()],
        &[],
        environment.own_pid,
    );
    if let Some(used) = scan.blocks(&tree.path) {
        return TreeVerdict::Skip(format!(
            "pid {} uses {} as {}",
            used.pid,
            used.path.display(),
            used.how
        ));
    }
    let mut proofs: Vec<String> = Vec::new();
    for guarded in &held {
        proofs.push(guarded.proof.clone());
    }
    proofs.push(scan.describe());
    let item = match item_of(&tree.path) {
        Ok(found) => found,
        Err(error) => return TreeVerdict::Skip(error.to_string()),
    };
    let claimed: PathBuf = match claim_into_trash(&item, trash, 0) {
        Ok(target) => target,
        Err(refusal) => return TreeVerdict::Skip(refusal.to_string()),
    };
    drop(held);
    return match delete_claimed(&claimed, item.identity, allocated_bytes_of) {
        Ok(deleted) => {
            emit(
                log,
                format!(
                    "removed tree {} (device {} inode {}): {reason}; proof: {}{}",
                    tree.path.display(),
                    item.identity.device,
                    item.identity.inode,
                    proofs.join(" | "),
                    modes_named(&deleted.modes_added)
                ),
            );
            TreeVerdict::Removed {
                reason: reason.to_string(),
                bytes: deleted.bytes,
            }
        }
        Err(Refusal::Gone(_)) => {
            TreeVerdict::Skip(String::from("another owner removed the claimed entry"))
        }
        Err(Refusal::PartlyRemoved {
            deleted,
            ..
        }) => {
            emit(
                log,
                format!(
                    "partly removed tree {} (device {} inode {}): {reason}; {} disk bytes are gone and {} part(s) stay{}; each stayed because: {}; proof: {}",
                    tree.path.display(),
                    item.identity.device,
                    item.identity.inode,
                    deleted.bytes,
                    deleted.remaining.len(),
                    modes_named(&deleted.modes_added),
                    deleted.remaining.join("; "),
                    proofs.join(" | ")
                ),
            );
            TreeVerdict::PartlyRemoved {
                reason: reason.to_string(),
                bytes: deleted.bytes,
                remaining: deleted.remaining.join("; "),
            }
        }
        Err(refusal) => TreeVerdict::Skip(refusal.to_string()),
    };
}
