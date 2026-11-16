//! Controls of the claim one command holds on one worktree's relinking.

use super::*;
use crate::landing::service_work::CLAIM_LEASE;
use crate::service_claim_renewal::renewal_interval;
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, DONE, Enqueued, PENDING, WorkItem, enqueue, state_directory};

/// The work directory of a fixture repository's common Git directory.
fn work_dir_of(root: &Path) -> PathBuf {
    return crate::service_work::work_directory(root.join("common").as_path());
}

/// A relink record of `destination`,
/// written into the work directory of `root` as the add's hand-over writes it.
fn a_relink_record(root: &Path, destination: &Path) -> WorkItem {
    let item: WorkItem = crate::package_repair_relink::item_of(
        root.join("common").as_path(),
        destination,
        &crate::package_repair_relink::RelinkEntry {
            root: destination.to_path_buf(),
            search_path: crate::package_repair_relink::SearchPathOfTheAdd::NotRecorded,
            tools: vec![crate::package_repair_relink::RelinkTool::Pnpm],
        },
    )
    .expect("a relink record");
    assert_eq!(enqueue(work_dir_of(root).as_path(), &item).expect("the queue takes the record"), Enqueued::Added);
    return item;
}

/// What:
///  Of two parties about to relink one worktree, only one takes the record,
///  and the other is told the work is not its own.
/// Why:
///   The rename out of `pending` is the one step only one consumer of a directory wins,
///  which is what keeps two installers out of one `node_modules`
///  (the review of 2026-11-13,
///  finding 1:
///  two runs of one worktree 1.5 s apart).
#[test]
fn only_one_of_the_two_takes_the_record_of_one_worktrees_relink() {
    let root: PathBuf = fixture("relink-claim-one");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let item: WorkItem = a_relink_record(root.as_path(), destination.as_path());
    let first: HeldRelinkClaim = HeldRelinkClaim::take(work_dir_of(root.as_path()).as_path(), item.id().as_str()).expect("the first party takes the record");
    assert_eq!(first.item().id(), item.id(), "the claim carries the record it took");
    assert!(
        HeldRelinkClaim::take(work_dir_of(root.as_path()).as_path(), item.id().as_str()).is_none(),
        "the second party finds nothing to take, so it starts no second run"
    );
    let claimed: PathBuf = state_directory(work_dir_of(root.as_path()).as_path(), CLAIMED);
    assert_eq!(std::fs::read_dir(claimed.as_path()).expect("the claimed directory").count(), 1, "one claim, by one party");
    first.kept_fresh().give_back_on_the_calling_thread();
    assert_eq!(std::fs::read_dir(claimed.as_path()).expect("the claimed directory").count(), 0, "giving the claim back settles it");
    let done: PathBuf = state_directory(work_dir_of(root.as_path()).as_path(), DONE);
    assert_eq!(std::fs::read_dir(done.as_path()).expect("the done directory").count(), 1, "the record is settled as done, so nothing runs it again");
    remove(root.as_path());
}

/// What:
///  A record this command took is settled as done when its run ends,
///  so the queue does not run the same install again after the lease.
/// Why:
///   A claim left behind is requeued by its lease and run a second time,
///  which is the work repeated; the settle is what says this command finished it.
#[test]
fn a_claim_this_command_held_is_settled_as_done() {
    let root: PathBuf = fixture("relink-claim-settled");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let item: WorkItem = a_relink_record(root.as_path(), destination.as_path());
    let claim: HeldRelinkClaim = HeldRelinkClaim::take(work_dir_of(root.as_path()).as_path(), item.id().as_str()).expect("the record is taken");
    claim.kept_fresh().give_back_on_the_calling_thread();
    assert_eq!(
        crate::work_item::find_claim(work_dir_of(root.as_path()).as_path(), item.id().as_str()),
        None,
        "nothing is left claimed, so nothing requeues it"
    );
    let done: PathBuf = state_directory(work_dir_of(root.as_path()).as_path(), DONE).join(item.id());
    assert!(done.is_file(), "the record is settled as done at {}", done.display());
    remove(root.as_path());
}

/// What:
///  A record no longer in `pending` is no claim to take,
///  which is how a command learns that the work is the service's.
/// Why:
///   The service claims a record by the same rename,
///  so a command that finds none takes nothing and starts no tools.
#[test]
fn a_record_the_service_already_claimed_is_no_claim_to_take() {
    let root: PathBuf = fixture("relink-claim-taken");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let item: WorkItem = a_relink_record(root.as_path(), destination.as_path());
    let service_claim: crate::work_item::Claimed = crate::work_item::claim_next_of(work_dir_of(root.as_path()).as_path(), &ClaimTime::read(), &[WorkKind::Relink])
        .expect("the queue is readable")
        .expect("the service claims the record");
    assert_eq!(service_claim.id, item.id());
    assert!(
        HeldRelinkClaim::take(work_dir_of(root.as_path()).as_path(), item.id().as_str()).is_none(),
        "the record is the service's, so this command does not run the tools"
    );
    let pending: PathBuf = state_directory(work_dir_of(root.as_path()).as_path(), PENDING);
    assert_eq!(std::fs::read_dir(pending.as_path()).expect("the pending directory").count(), 0, "nothing was written back to pending");
    remove(root.as_path());
}

/// What:
///  The renewal keeps the claim fresh for longer than one lease.
/// Why:
///   A tool may run past the lease,
///  and a claim that expired mid-run would be requeued and claimed by the service,
///  which is the second run the claim exists to prevent.
///  The reading is driven here, so the control does not depend on how this host schedules a task.
#[test]
fn the_claim_is_kept_fresh_past_one_lease() {
    let root: PathBuf = fixture("relink-claim-renewed");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let item: WorkItem = a_relink_record(root.as_path(), destination.as_path());
    let claim: HeldRelinkClaim = HeldRelinkClaim::take(work_dir_of(root.as_path()).as_path(), item.id().as_str()).expect("the record is taken");
    let (held, renewer) = crate::service_claim_renewal::held_claims(work_dir_of(root.as_path()).as_path());
    held.hold(&claim.claimed);
    let before: String = {
        let found: PathBuf = crate::work_item::find_claim(work_dir_of(root.as_path()).as_path(), item.id().as_str()).expect("the claim is on disk");
        found.file_name().expect("a name").to_string_lossy().into_owned()
    };
    // A reading one lease and a half later, then a renewal at it.
    let later: ClaimTime = ClaimTime::read().advanced(CLAIM_LEASE + renewal_interval(CLAIM_LEASE));
    let mut renewer: Renewer = renewer;
    renewer.renew(&later);
    let after: String = {
        let found: PathBuf = crate::work_item::find_claim(work_dir_of(root.as_path()).as_path(), item.id().as_str()).expect("the claim is still on disk");
        found.file_name().expect("a name").to_string_lossy().into_owned()
    };
    assert_ne!(before, after, "the renewal writes a new reading into the claim's name, so no reader takes a live claim");
    assert_eq!(held.settle(work_dir_of(root.as_path()).as_path(), item.id().as_str(), DONE), Ok(()), "a renewed claim is still this command's to settle");
    remove(root.as_path());
}
