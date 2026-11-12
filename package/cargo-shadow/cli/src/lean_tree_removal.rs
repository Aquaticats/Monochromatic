//! What:
//! The removal of one tree of the host build base,
//! under the proof that nothing uses it.
//! Why:
//! Mechanism C decides what qualifies and this file does what the decision allows:
//! every profile directory yields its build locks without waiting,
//! a scan of every process finds none using anything under the tree,
//! the tree root is renamed into the trash while the locks are held,
//! and the trash is deleted under the identity it was judged by.
//! A deletion the deadline stopped says so and leaves the rest in the trash,
//! which is one claimed entry the next pass's sweep finishes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function collectOne(tree: Tree, reason: string, trash: string, env: GuardEnvironment, now: number, needed: number, deadline: Deadline, log: string[]): TreeVerdict
//! ```

/// Paths.
use std::path::{Path, PathBuf};

/// The deadline of every walk and of the deletion.
use crate::lean_identity::Deadline;
/// The tree and the answer to when it was last written,
/// and the profile directories the locks are taken on.
use crate::lean_build_trees::{Tree, last_activity, profile_dirs_of};
/// The proof that nothing uses the tree.
use crate::lean_guard::{Guard, GuardEnvironment, Guarded, guard_profile};
/// The scan of the processes.
use crate::lean_proc_scan::scan_processes;
/// The claim and the deletion.
use crate::lean_removal::{Refusal, claim_into_trash, delete_claimed, item_of, modes_named};
/// The verdict the removal answers with.
use crate::lean_idle_trees::TreeVerdict;
/// The line each removal leaves in the log.
use crate::lean_run_report::emit;

/// What:
/// `pub(crate) fn collect_one(tree, reason, trash, environment, now, needed, deadline, log) -> TreeVerdict`
/// removes one qualified tree under the protocol.
/// Why:
/// Unlinking what a directory holds needs write and search on it,
/// so a directory this user owns and may not write is made writable first and the removal's proof line says so;
/// a directory of another user stays as it is and is named.
/// A tree that could not be emptied whole is reported as partly removed,
/// never as skipped,
/// because part of it is gone,
/// and a deletion the deadline stopped is reported as left in the trash,
/// because that is what a person has to know to trust the numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectOne(tree: Tree, reason: string, trash: string, env: GuardEnvironment, now: number, needed: number, deadline: Deadline, log: string[]): TreeVerdict
/// ```
pub(crate) fn collect_one(tree: &Tree, reason: &str, trash: &Path, environment: &GuardEnvironment, now: i64, needed: i64, deadline: &Deadline, log: &mut Vec<String>) -> TreeVerdict {
    let Some(profiles) = profile_dirs_of(&tree.path, deadline) else {
        return TreeVerdict::Skip(String::from("the deadline passed while looking for its profile directories"));
    };
    let mut held: Vec<Guarded> = Vec::new();
    for profile in profiles {
        match guard_profile(&profile, environment, deadline) {
            Guard::Ready(ready) => held.push(ready),
            Guard::Skip(why) => return TreeVerdict::Skip(format!("{}: {why}", profile.display())),
        }
    }
    match last_activity(&tree.path, deadline) {
        Some(activity) if now - activity > needed => {}
        Some(_) => return TreeVerdict::Skip(String::from("a write happened while the locks were being taken")),
        None => return TreeVerdict::Skip(String::from("the deadline passed while reading the tree")),
    }
    let Some(scan) = scan_processes(&environment.proc_root, &[tree.path.as_path()], &[], environment.own_pid, deadline) else {
        return TreeVerdict::Skip(String::from("the deadline passed while scanning the processes, so nothing was removed"));
    };
    if let Some(used) = scan.blocks(&tree.path) {
        return TreeVerdict::Skip(format!("pid {} uses {} as {}", used.pid, used.path.display(), used.how));
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
    return match delete_claimed(&claimed, item.identity, deadline) {
        Ok(deleted) => {
            emit(log, format!(
                "removed tree {} (device {} inode {}): {reason}; proof: {}{}",
                tree.path.display(),
                item.identity.device,
                item.identity.inode,
                proofs.join(" | "),
                modes_named(&deleted.modes_added)
            ));
            TreeVerdict::Removed { reason: reason.to_string(), bytes: deleted.bytes }
        }
        Err(Refusal::Gone(_)) => TreeVerdict::Skip(String::from("another owner removed the claimed entry")),
        Err(Refusal::PartlyRemoved { deleted, .. }) if deleted.cut => {
            emit(log, format!(
                "left tree {} in {} (device {} inode {}): {reason}; {} disk bytes are gone and the deadline stopped the rest, which the next pass finishes; proof: {}",
                tree.path.display(),
                claimed.display(),
                item.identity.device,
                item.identity.inode,
                deleted.bytes,
                proofs.join(" | ")
            ));
            TreeVerdict::Left { reason: reason.to_string(), bytes: deleted.bytes }
        }
        Err(Refusal::PartlyRemoved { deleted, .. }) => {
            emit(log, format!(
                "partly removed tree {} (device {} inode {}): {reason}; {} disk bytes are gone and {} part(s) stay{}; each stayed because: {}; proof: {}",
                tree.path.display(),
                item.identity.device,
                item.identity.inode,
                deleted.bytes,
                deleted.remaining.len(),
                modes_named(&deleted.modes_added),
                deleted.remaining.join("; "),
                proofs.join(" | ")
            ));
            TreeVerdict::PartlyRemoved { reason: reason.to_string(), bytes: deleted.bytes, remaining: deleted.remaining.join("; ") }
        }
        Err(refusal) => TreeVerdict::Skip(refusal.to_string()),
    };
}
