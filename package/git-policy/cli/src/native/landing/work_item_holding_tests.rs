//! Controls for the three steps of a work item that waits for its producer's approval,
//! and for the verdict a producer's refusal is kept as.
//! Why:
//!   The held record is the only statement that a commit may not be published,
//! so a refusal whose verdict cannot be written must leave that record where it is:
//! a record forgotten beside a verdict that was never written leaves nothing at all,
//! and the next hold then publishes the commit the gate refused.

use super::*;
use crate::test_support::{fixture, remove, restore_owner_access};
use std::path::PathBuf;

/// The commit the gate of these controls refused.
const OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// The words the gate of these controls said.
const REASON: &str = "final-newline: the file does not end with a newline";

/// What:
///  The push record of one landed commit of a worktree,
/// as the hand-over writes it.
/// Why:
///   A push is the item whose hold keeps a gate-refused commit local,
/// and its id carries the branch,
/// so the record a control reads is the one the product writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = { kind: 'push', repository: root, gitDir: root, branchRef: 'refs/heads/main', oid: OID, parent: '', created: '', tries: 0 };
/// ```
fn push_item(root: &Path) -> WorkItem {
    return WorkItem {
        kind: crate::work_item::WorkKind::Push,
        repository: root.to_path_buf(),
        git_dir: root.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from(OID),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
}

/// What:
///  The `blocked` directory of a work tree,
/// made to refuse every new name.
/// Why:
///   A verdict record is published by a create-only rename of a staging file made inside that directory,
/// so a directory this user may not write is the storage failure the control needs,
/// and it needs no hook in the product.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const closed = mkdir(join(work, 'blocked')); chmod(closed, 0o500);
/// ```
#[cfg(unix)]
fn a_blocked_directory_that_refuses_every_name(work: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let blocked: PathBuf = state_directory(work, BLOCKED);
    create_private_directory_all(blocked.as_path()).expect("the blocked directory");
    std::fs::set_permissions(blocked.as_path(), std::fs::Permissions::from_mode(0o500)).expect("a directory no name can be made in");
    return blocked;
}

/// What a refusal that cannot be written does:
/// keeps the held record,
/// so the commit stays local,
/// and answers that the verdict is not durable.
/// Why:
///   A record forgotten beside a verdict that was never written leaves neither,
/// and `is_blocked` then reads false,
/// so a later hold and push publish the commit the gate refused.
#[test]
#[cfg(unix)]
fn a_refusal_that_cannot_be_written_keeps_the_held_record() {
    let root: PathBuf = fixture("work-item-holding-refusal-not-written");
    let work: PathBuf = root.join("work");
    let item: WorkItem = push_item(root.as_path());
    assert_eq!(hold(work.as_path(), &item, "4321"), Ok(()), "the push is held while the gate decides");
    let blocked: PathBuf = a_blocked_directory_that_refuses_every_name(work.as_path());
    assert!(!block(work.as_path(), &item, REASON), "the verdict could not be written, so the answer says it is not durable");
    assert!(
        state_directory(work.as_path(), HELD).join(item.id()).exists(),
        "the held record is the only statement that this commit may not be published, so it stays"
    );
    assert!(blocked_verdicts(work.as_path()).is_empty(), "no verdict was written, so none is read");
    // The next hold of the same commit asks nothing of a blocked id, and this one is not blocked: the hold stands again, which is
    //  what keeps the push local.
    assert_eq!(hold(work.as_path(), &item, "4321"), Ok(()), "the hold is written again");
    restore_owner_access(blocked.as_path());
    remove(root.as_path());
}

/// What:
///  A push record the queue has already settled is not held again,
///  so no command names a commit as held once the push that carried it was made.
/// Why:
///   A batch is replayed after a restart,
///  and a recovery that re-holds the push of a commit whose record already reached `done` writes a second record nothing can
///  promote:
/// the command that would have promoted it answered long before,
///  so the line stays on every command in the worktree while the commit is on the remote
///  (`held_push_line` in `service_handoff.rs`).
/// The queue's own states are the proof that the work was done,
/// so the hold answers that nothing is owed.
#[test]
fn a_settled_push_is_not_held_again() {
    let root: PathBuf = fixture("work-item-holding-settled-push");
    let work: PathBuf = root.join("work");
    let item: WorkItem = push_item(root.as_path());
    let done: PathBuf = state_directory(work.as_path(), DONE);
    create_private_directory_all(done.as_path()).expect("the done directory");
    std::fs::write(done.join(item.id()), item.encode()).expect("the settled record");
    assert_eq!(hold(work.as_path(), &item, "4321"), Ok(()), "the hold answers that nothing is owed");
    assert!(
        !state_directory(work.as_path(), HELD).join(item.id()).exists(),
        "a push that was made is not held again: a hold here would name a published commit on every command"
    );
    assert!(done.join(item.id()).exists(), "the settled record is left where it is");
    remove(root.as_path());
}

/// What a refusal that is written does:
/// keeps the verdict with the gate's words,
/// gives the held record up,
/// and answers that the verdict is durable.
/// Why:
///   This is the answer the control above departs from:
/// the verdict is the refusal,
/// so the hold that waited for it has nothing left to say.
#[test]
fn a_refusal_that_is_written_gives_the_held_record_up() {
    let root: PathBuf = fixture("work-item-holding-refusal-written");
    let work: PathBuf = root.join("work");
    let item: WorkItem = push_item(root.as_path());
    assert_eq!(hold(work.as_path(), &item, "4321"), Ok(()), "the push is held while the gate decides");
    assert!(block(work.as_path(), &item, REASON), "the verdict is durable");
    assert!(!state_directory(work.as_path(), HELD).join(item.id()).exists(), "the verdict replaces the hold");
    assert_eq!(
        blocked_verdicts(work.as_path()),
        vec![BlockedVerdict { branch_ref: String::from("refs/heads/main"), commit: String::from(OID), reason: String::from(REASON) }],
        "the verdict names the branch, the commit and the gate's words"
    );
    remove(root.as_path());
}

/// What a second refusal of the same commit does:
/// keeps the verdict that is already there,
/// and still gives the held record up.
/// Why:
///   A verdict is never replaced,
/// so the write of a second refusal answers that it already exists,
/// and a hold that waited for a refusal has one either way.
#[test]
fn a_refusal_whose_verdict_is_already_there_gives_the_held_record_up() {
    let root: PathBuf = fixture("work-item-holding-refusal-already-there");
    let work: PathBuf = root.join("work");
    let item: WorkItem = push_item(root.as_path());
    assert!(block(work.as_path(), &item, REASON), "the first verdict is durable");
    assert_eq!(hold(work.as_path(), &item, "4321"), Ok(()), "a hold an earlier release wrote is written again");
    assert!(block(work.as_path(), &item, REASON), "a verdict that is already there is a refusal that stands");
    assert!(!state_directory(work.as_path(), HELD).join(item.id()).exists(), "the verdict replaces the hold");
    assert_eq!(blocked_verdicts(work.as_path()).len(), 1, "one verdict for one commit");
    remove(root.as_path());
}
