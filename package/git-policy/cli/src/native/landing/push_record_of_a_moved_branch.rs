//! What:
//!  The push record a command that moves a branch leaves,
//!  and the hand-off that wakes the service to run it with this command's programs.
//! Why:
//!  A merge,
//! a cherry-pick,
//! a revert,
//! an applied patch or a pull's merge adds commits that the next commit would otherwise be the first to push,
//! and the record of that push must outlive a command killed before its values travel
//! (`service_push_programs_held_in_memory`).
//! The record is written into the queue before anything asks whether a service can be reached,
//! so a context with no route leaves the push on disk for the next wake,
//! and the values that push must run with go with the wake that follows the record.
//! A branch whose name is not text has no record,
//! and the person is told that instead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = pushRecordItem(repo, gitDir, 'refs/heads/main', tip, parent); enqueue(workDir, record); await wake({ pushPrograms: [[record.id, programsOfThisProcess()]] });
//! ```

use crate::iso_time::now_iso;
use crate::service_client::wake;
use crate::service_handoff::{
    ServiceHandoff,
    unqueued_branch_note,
};
use crate::service_messages::ClientFailure;
use crate::service_messages_of_a_work_that_was_not_handed_over::not_handed_over;
use crate::service_work::work_directory;
use crate::work_item::{
    WorkItem,
    WorkKind,
    enqueue,
};
use std::io::Write;

/// What:
///  The push record of one landed commit of a worktree,
/// named by the item's own id:
/// the kind,
/// the commit,
/// the branch,
/// and whether this command chose a transport.
/// Why:
///  The hand-off names the record's id in the wake that carries the values the push must run with,
/// so the id is made here,
/// once,
/// and the record is enqueued from it,
/// and a record written here is written only for a branch whose name is text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pushRecordItem = (repository, gitDir, reference, tip, parent) => ({ kind: 'push', repository, gitDir, branchRef: reference, oid: tip, parent, created: nowIso(), tries: 0, transport: ofThisProcess().transport });
/// ```
pub(crate) fn push_record_item(
    repository: &std::path::Path,
    git_dir: &std::path::Path,
    reference: &str,
    tip: &str,
    parent: &str,
) -> WorkItem {
    // The command that moved the branch is what asks for this push, so its own environment answers whether a transport was
    //  chosen; the values themselves travel with the wake and are written nowhere (`service_push_programs_held_in_memory`).
    return WorkItem {
        kind: WorkKind::Push,
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: reference.to_string(),
        oid: tip.to_string(),
        parent: parent.to_string(),
        created: now_iso(),
        tries: 0,
        transport: crate::service_push_programs_held_in_memory::of_this_process().transport,
    };
}

/// What:
///  Writes the push record for a branch tip that a command other than a commit moved,
/// before anything asks whether a service can be reached.
/// Why:
///   A merge,
/// a cherry-pick,
/// a revert,
/// an applied patch or a pull's merge adds commits that the next commit would otherwise
///  be the first to push.
/// Only the push is left:
/// the review and the upkeep belong to a commit.
///   The record is a directory of this worktree's own Git directory,
/// so writing it needs no service,
/// no socket and no session bus,
/// and a context with no route to the service still leaves the push on disk for the next wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePushRecord({ repository, gitDir, reference, tip, parent })
/// ```
pub fn write_push_record(
    repository: &std::path::Path,
    git_dir: &std::path::Path,
    reference: &crate::ref_name::RefName,
    tip: &str,
    parent: &str,
) -> Result<Option<String>, ClientFailure> {
    // A push record names its branch in text, and a spelling with replacement characters would push another branch, so a branch
    //  whose name is not text has none; `hand_over_push` tells the person that instead.
    let Some(name) = reference.text() else {
        return Ok(None);
    };
    let item: WorkItem = crate::push_record_of_a_moved_branch::push_record_item(
        repository, git_dir, name, tip, parent,
    );
    let id: String = item.id();
    enqueue(work_directory(git_dir).as_path(), &item).map_err(ClientFailure::WorkNotHandedOver)?;
    return Ok(Some(id));
}

/// What:
///  `write_push_record` for a handoff,
/// and then the wake of the service.
/// Why:
///   The caller that already knows how to reach the service leaves both in one call;
/// a caller that may find no route writes the record with `write_push_record` and wakes only when it has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverPush(handoff, tip)
/// ```
pub fn hand_over_push(
    handoff: &ServiceHandoff,
    tip: &str,
    notices: &mut dyn Write,
) -> Result<(), ClientFailure> {
    if !handoff.reference.is_text() {
        let _said = writeln!(
            notices,
            "{}",
            unqueued_branch_note(handoff.reference.as_os_str(), tip)
        );
        return Ok(());
    }
    let record: Option<String> = write_push_record(
        handoff.repository.as_path(),
        handoff.git_dir.as_path(),
        &handoff.reference,
        tip,
        handoff.parent.as_str(),
    )?;
    // What: The programs of this command, and whether it chose the transport the push speaks through, go with the wake.
    // Why:  Nothing of them may be written (the two command lines can hold a password), and the push runs after this command has
    //  gone, so the message that announces the record is the last moment they can reach the service that runs it, which holds them
    //  in its memory alone (`service_push_programs_held_in_memory`).
    let programs: Vec<(
        String,
        crate::service_push_programs_held_in_memory::PushPrograms,
    )> = match record {
        Some(id) => vec![(
            id,
            crate::service_push_programs_held_in_memory::of_this_process(),
        )],
        None => Vec::new(),
    };
    return wake(
        &handoff.config,
        handoff.git_dir.as_path(),
        None,
        programs.as_slice(),
        notices,
    )
    .map_err(not_handed_over);
}
