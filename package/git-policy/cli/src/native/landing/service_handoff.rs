//! What:
//!  The wrapper's side of after-commit work:
//! leave the push and upkeep records for a commit the service landed and wake
//!  the service.
//! Why:
//!  The commit returns as soon as it is saved.
//! The push (which can take as long as the network does) and the upkeep run
//!  in the service afterwards;
//! a push that fails is reported by the person's next wrapper command (`push_failure_notice.rs`)
//!  (`doc/planning/cli-git-resident-service.md`,
//! "After-commit work").
//! The wrapper starts no program and waits for no
//!  one:
//! it writes whole files and sends one message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, push); await enqueue(workDir, maintenance); await wake(commonDir);
//! ```
//!
//! The lines this worktree says about the pushes it holds and the pushes it queued
//! are declared beside this file,
//! in `service_handoff_push_notices.rs`.

use crate::held_push_release_wakes_the_service::WakeTheService;
use crate::iso_time::now_iso;
use crate::service_client::{
    ClientConfig,
    wake,
};
use crate::service_messages::ClientFailure;
use crate::service_messages_of_a_work_that_was_not_handed_over::not_handed_over;
use crate::service_review_key::{
    ReviewKey,
    key_from_environment,
};
use crate::service_work::work_directory;
use crate::standards_review_input::has_shadow_log;
use crate::work_item::{
    WorkItem,
    WorkKind,
    enqueue,
    enqueue_all,
    state_directory,
};
use std::io::Write;
use std::path::PathBuf;

/// What:
///  The lines about the pushes this worktree holds,
///  and the line about the pushes it queued,
/// declared beside this file and served by it,
/// so the paths its callers name stay the ones they named.
pub use crate::service_handoff_push_notices::{
    PUSH_QUEUED_AFTER,
    held_push_notices,
    queued_push_notices,
};

/// What the wrapper knows about a commit the service landed.
#[derive(Clone, Debug)]
pub struct ServiceHandoff {
    /// How to reach the service.
    pub config: ClientConfig,
    /// The worktree root.
    pub repository: PathBuf,
    /// The worktree's own Git directory,
    /// which holds the work records and names the service instance.
    pub git_dir: PathBuf,
    /// The branch the commit went to,
    /// by the bytes Git printed for its name.
    pub reference: crate::ref_name::RefName,
    /// The branch value the commit was made on.
    pub parent: String,
}

/// The kinds of work a landed commit leaves for the service.
pub const HANDED_OVER: &[WorkKind] = &[WorkKind::Maintenance];

/// What:
///  The value of an environment variable,
/// nothing when it is unset or not text.
/// Why:
///  `hand_over` reads the review key from the process environment through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environmentVariable = (name: string) => process.env[name];
/// ```
fn environment_variable(name: &str) -> Option<String> {
    return std::env::var(name).ok();
}

/// What:
///  Writes the work records for a landed commit and wakes the service.
/// Why:
///   The records are written before the message is sent,
/// so a service that wakes always finds them;
/// records that were
///  written but not announced stay pending and are run by the next wake for the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOver(handoff, landed)
/// ```
pub fn hand_over(
    handoff: &ServiceHandoff,
    landed: &str,
    notices: &mut dyn Write,
) -> Result<(), ClientFailure> {
    let key: Option<ReviewKey> = key_from_environment(&environment_variable);
    return hand_over_with_key(handoff, landed, key.as_ref(), notices);
}

/// What:
///  `hand_over` with the standards-review credential given.
/// The records written to disk never hold it;
/// the wake message
///  carries it to the service,
/// which keeps it in memory for the review.
/// Why:
///   A systemd user service does not inherit the wrapper's environment,
/// and the credential must not be stored in a work
///  item,
/// a unit file or the manager's environment.
/// The wrapper therefore sends it over the user-only socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWithKey(handoff, landed, key)
/// ```
pub fn hand_over_with_key(
    handoff: &ServiceHandoff,
    landed: &str,
    review_key: Option<&ReviewKey>,
    notices: &mut dyn Write,
) -> Result<(), ClientFailure> {
    // A work item spells its branch in text, and a spelling with replacement characters would push another branch. A commit on a
    //  branch whose name is not text therefore leaves no item, and the person is told once: the commit is on the branch, and a
    //  push they make themselves sends it.
    let Some(reference) = handoff.reference.text() else {
        let _said = writeln!(
            notices,
            "{}",
            unqueued_branch_note(handoff.reference.as_os_str(), landed)
        );
        return Ok(());
    };
    let work_dir: PathBuf = work_directory(handoff.git_dir.as_path());
    // A review is left only where the repository keeps a shadow log; the file is the repository's own switch.
    let review: &[WorkKind] = if has_shadow_log(handoff.repository.as_path()) {
        &[WorkKind::Review]
    } else {
        &[]
    };
    // One round for the commit's records: each state directory the queue asks about is listed once, however many kinds the
    //  commit leaves.
    let mut items: Vec<WorkItem> = Vec::new();
    for kind in HANDED_OVER.iter().chain(review.iter()).copied() {
        items.push(WorkItem {
            kind,
            repository: handoff.repository.clone(),
            git_dir: handoff.git_dir.clone(),
            branch_ref: reference.to_string(),
            oid: landed.to_string(),
            parent: handoff.parent.clone(),
            created: now_iso(),
            tries: 0,
            transport: false,
        });
    }
    enqueue_all(work_dir.as_path(), items.as_slice()).map_err(ClientFailure::WorkNotHandedOver)?;
    // The records this wake is about are the commit's upkeep and review, never a push of its own: the push record of a landed
    //  commit comes from the landing, which holds the request's programs in the service's memory itself.
    return wake(
        &handoff.config,
        handoff.git_dir.as_path(),
        review_key,
        &[],
        notices,
    )
    .map_err(not_handed_over);
}

/// What:
///  Promotes the push record the service held for a commit it landed,
/// once the post-commit gate approves the backup.
/// Why:
///   The record must outlive a wrapper killed between the answer and the gate,
/// and it must not run before the gate:
///  the service writes no record of its own,
/// so nothing pushes a commit whose gate blocked it,
/// and a record the gate
///  then allows is the one the hand-off wakes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await leaveThePush(handoff, landed)
/// ```
///
/// # Errors
///
/// The work queue's own words when the record cannot be written.
pub fn promote_the_push(
    handoff: &ServiceHandoff,
    landed: &str,
) -> Result<(), ClientFailure> {
    // Nothing was held for a branch whose name is not text (`hand_over_with_key`), so there is nothing to promote.
    let Some(reference) = handoff.reference.text() else {
        return Ok(());
    };
    let item: WorkItem = push_item(handoff, landed, reference);
    let work_dir: PathBuf = work_directory(handoff.git_dir.as_path());
    // A record an earlier release held is named by the kind and the commit alone, and it is this push's.
    for name in [item.id(), item.id_before_the_branch()] {
        // The pass is written into the record before it runs: a command killed in this gap leaves a record whose own words say the
        //  checks passed, and a later command releases it from them (`held_push_holder_and_verdict.rs`). The record then names this
        //  command, which is the retry's own when a retry is what finishes the commit.
        if crate::work_item::record_pass(work_dir.as_path(), name.as_str()) {
            crate::service_fault::reach(crate::service_fault::AFTER_THE_VERDICT);
        }
        if crate::work_item::promote(work_dir.as_path(), name.as_str()) {
            return Ok(());
        }
    }
    // Nothing was held for this commit: the record is written here instead, so the push is not lost.
    return match enqueue(work_dir.as_path(), &item) {
        Ok(_enqueued) => Ok(()),
        Err(reason) => Err(ClientFailure::WorkNotHandedOver(reason)),
    };
}

/// What:
///  Takes the push record back when the post-commit gate blocks the backup,
/// and keeps what the gate said.
/// Why:
///   A blocked gate keeps the commit local,
/// and a record left behind would push it at the next wake.
/// A record another round already
///  claimed is not reached here:
/// the gate is what says whether the push may run.
///   The verdict keeps the gate's words,
/// because the queue pushes the branch and not the commit:
/// the next push of
///  that branch must stop before this commit and say why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetThePush(handoff, landed, reason) // unlink the pending record, keep the verdict; words when the verdict could not be kept
/// ```
/// The words a command says when the verdict could not be kept,
/// or nothing when it was.
pub fn forget_the_push(
    handoff: &ServiceHandoff,
    landed: &str,
    reason: &str,
    wake: &dyn WakeTheService,
) -> Option<String> {
    // A branch whose name is not text held no record (`hand_over_with_key`), so there is none to take back and no verdict to keep.
    let reference: &str = handoff.reference.text()?;
    let work_dir: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let item: WorkItem = push_item(handoff, landed, reference);
    // The refusal is written into the record before it is settled: a command killed in this gap leaves a record whose own words say
    //  the gate refused this commit, and a later command settles it into the same verdict instead of leaving it held
    //  (`held_push_holder_and_verdict.rs`).
    for name in [item.id(), item.id_before_the_branch()] {
        if crate::work_item::record_refusal(work_dir.as_path(), name.as_str(), reason) {
            crate::service_fault::reach(crate::service_fault::AFTER_THE_VERDICT);
        }
    }
    // The verdict is durable: a later recovery of the same batch must not hold the push again, and the next push of the branch
    //  reads the same verdict to know where it must stop. A verdict that could not be written says so, and the held records then
    //  stay: a record with a verdict keeps the commit local, and a record without one does too (`work_item_holding.rs`).
    let durable: bool = crate::work_item::block(work_dir.as_path(), &item, reason);
    // A record an earlier release named by the kind and the commit alone is this push's too, so the gate forgets it as well:
    //  left in `pending`, the next wake would push a commit the gate kept local, and left in `held`, every command would name a push
    //  nothing may make. The held record is kept while the verdict could not be, because it is then the only copy of the refusal.
    for id in [item.id(), item.id_before_the_branch()] {
        let _removed: std::io::Result<()> = std::fs::remove_file(
            state_directory(work_dir.as_path(), crate::work_item::PENDING).join(id.as_str()),
        );
        if durable {
            crate::work_item::forget_held(work_dir.as_path(), id.as_str());
        }
    }
    if durable {
        // What: The service is told that these records can no longer run, so it gives up the programs and transport settings it
        //  holds against them.
        // Why:  They live in the service's memory alone, and the service learns nothing else from a record this command removed:
        //  one of the values can hold a password (`service_push_programs_held_in_memory`). A record kept because its verdict could
        //  not be written keeps its values too, so this runs only when the record went.
        let gone: [String; 2] = [item.id(), item.id_before_the_branch()];
        wake.forget_programs(handoff.git_dir.as_path(), gone.as_slice());
        return None;
    }
    return Some(format!(
        "the commit {landed} stays local, because its post-commit checks refused it ({reason}), but the verdict that stops this branch's automatic push could not be written to {}. The held record keeps the refusal, so no push this repository makes sends the commit, and every later command tries to keep that verdict again; a push you make of the branch still sends it.",
        work_directory(handoff.git_dir.as_path())
            .join(crate::work_item_holding::BLOCKED)
            .display()
    ));
}

/// What:
///  The one line a commit on a branch whose name is not text leaves instead of its work records.
/// Why:
///   A work item names its branch in text,
/// so none is written for such a branch;
///  saying so keeps the person from believing a push was queued,
/// and names the commit and the branch that hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const note = `the commit ${landed} landed on ${shellQuote(ref)} ...`
/// ```
pub fn unqueued_branch_note(
    reference: &std::ffi::OsStr,
    landed: &str,
) -> String {
    return format!(
        "monochromatic-git-policy-cli: the commit {landed} landed on {}, whose name is not text. Its automatic push was not queued, \
         because a push record names a branch in text and a spelling with replacement characters would push another branch; the commit \
         is on the branch, and a push you make yourself sends it.",
        crate::shell_quote::shell_quote(crate::path_bytes::argument_bytes(reference))
    );
}

/// The push record of one landed commit of a worktree.
fn push_item(
    handoff: &ServiceHandoff,
    landed: &str,
    reference: &str,
) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: handoff.repository.clone(),
        git_dir: handoff.git_dir.clone(),
        branch_ref: reference.to_string(),
        oid: landed.to_string(),
        parent: handoff.parent.clone(),
        created: now_iso(),
        tries: 0,
        // The record a landing held already says whether the request's command chose a transport; this one is written only when
        //  nothing was held, and then this command is what asked for the push, so its own environment answers.
        transport: crate::service_push_programs_held_in_memory::of_this_process().transport,
    };
}

/// Handoff controls stay out of the release executable.
#[cfg(test)]
#[path = "service_handoff_tests.rs"]
mod tests;
