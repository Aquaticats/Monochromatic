//! Controls of the words a kind that answers with words is settled with:
//! what each verdict leaves behind in the queue.

use crate::claim_time::ClaimTime;
use crate::service_claim_renewal_running::Renewal;
use crate::service_work::{CLAIM_LEASE, SERVICE_KINDS, Verdict};
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::{MAX_TRIES, Replanning, WorkHandlers, tell_about_given_up, work_directory};
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, DONE, FAILED, PENDING, WorkItem, enqueue, id_of_entry, state_directory};
use crate::work_item_recovery::requeue_expired;
use crate::worktree_remove_hand_over::removal_item;
use std::sync::atomic::{AtomicU32, Ordering};
use std::path::{Path, PathBuf};

/// A handler whose subvolume delete answers one verdict every time it is asked,
/// and counts the times it was asked.
struct OneVerdict {
    /// What the delete says,
    /// by the attempt's number counting from one.
    said: Box<dyn Fn(u32) -> Verdict + Send + Sync>,
    /// How many times the delete was asked.
    asked: AtomicU32,
}

impl WorkHandlers for OneVerdict {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn subvolume_delete(&self, _item: &WorkItem) -> Verdict {
        let attempt: u32 = self.asked.fetch_add(1, Ordering::SeqCst) + 1;
        return (self.said)(attempt);
    }
}

/// One queued removal,
/// its record's id,
/// and the handler that answers each attempt with `said`,
/// in a fresh fixture.
struct Queued {
    /// The fixture's root,
    /// which a control removes.
    root: PathBuf,
    /// The common directory the record is queued under.
    common: PathBuf,
    /// The work directory the record's states are in.
    work: PathBuf,
    /// The record's id.
    id: String,
    /// The handler every attempt is asked of.
    handlers: OneVerdict,
}

/// What:
///  A fixture with one removal of a subvolume id in `pending`,
/// and the handler that answers each attempt with `said`.
/// Why:
///   Every control below asks the same question of a different verdict,
/// so the queue is built here once and drained by whichever control needs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queued = (name, said) => { fixture(name); enqueue(removalItem(53)); return { common, work, id, handlers }; }
/// ```
fn queued(name: &str, said: impl Fn(u32) -> Verdict + Send + Sync + 'static) -> Queued {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let parent_path: PathBuf = root.join("parent");
    std::fs::create_dir(common.as_path()).expect("common");
    std::fs::create_dir(parent_path.as_path()).expect("parent");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), 53);
    let work: PathBuf = work_directory(common.as_path());
    enqueue(work.as_path(), &item).expect("the record");
    return Queued { root, common, work, id: item.id(), handlers: OneVerdict { said: Box::new(said), asked: AtomicU32::new(0) } };
}

/// Drain whatever `queued` holds,
/// through the service's own drain.
fn drained(queued: &Queued) {
    let mut replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    crate::lazy_runtime::run_on_runtime(drain_pending_awaiting(
        queued.common.as_path(),
        &queued.handlers,
        &ClaimTime::read(),
        CLAIM_LEASE,
        Renewal::Timed,
        &mut replanning,
        &SERVICE_KINDS,
        crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
    ))
    .expect("the pending drain runs");
}

/// Where the record `id` stands now:
/// a state its name is in,
/// or `claimed`,
/// whose file names carry the reading of the claim as well as the id.
fn settled_in(work: &Path, id: &str) -> &'static str {
    for state in [PENDING, DONE, FAILED] {
        if state_directory(work, state).join(id).is_file() {
            return state;
        }
    }
    let claimed: bool = std::fs::read_dir(state_directory(work, CLAIMED)).is_ok_and(|entries| {
        return entries.flatten().any(|entry| { return id_of_entry(entry.file_name().to_string_lossy().as_ref()) == id; });
    });
    return if claimed { CLAIMED } else { "no state" };
}

/// What:
///  A deletion the handler answers with a retryable failure is settled by nothing:
/// its record keeps its claim,
/// which a later round takes back with one more try.
/// Why:
///  `Verdict::Retry` says the failure may pass,
/// so settling it where a failure the service gave up on goes ends every attempt that would
///  have succeeded:
/// nothing scans `failed`,
/// and an id that reached it can never be queued again,
/// which leaves the directory the record names on the disk for ever.
/// A claim's lease runs out only when nothing renews it,
/// so the record is still there for the recovery to find (`work_item_recovery.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queued = queued('settling-retry-keeps-its-claim', () => Retry('a transient failure')); drained(queued);
/// assert(queued.handlers.asked === 1); assert(state(queued) === CLAIMED);
/// ```
#[test]
fn a_retryable_deletion_keeps_its_claim_and_is_not_settled_as_a_failure() {
    let queued: Queued = queued("settling-retry-keeps-its-claim", |_attempt: u32| { return Verdict::Retry(String::from("a transient failure of the destroy")); });
    drained(&queued);
    assert_eq!(queued.handlers.asked.load(Ordering::SeqCst), 1, "the attempt the verdict answered is the one that ran");
    assert_eq!(settled_in(queued.work.as_path(), queued.id.as_str()), CLAIMED, "a retryable answer is not a failure and is not done, so the claim is kept");
    remove(queued.root.as_path());
}

/// What:
///  A deletion whose retryable attempt ran out its claim's lease is queued again with one more try,
/// and the attempt that follows deletes the subvolume.
/// Why:
///   The claim a retryable answer keeps is what the recovery reads:
/// a claim nothing renews has a lease that runs out,
/// and the recovery returns the record to `pending` with one more try
/// (`work_item_recovery.rs`),
/// which is the retry the answer asked for.
/// The bound is that recovery's own:
/// a record that reaches `MAX_TRIES` is given up on there,
/// so nothing here can keep a round busy for the life of the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queued = queued('settling-retry-then-done', (attempt) => attempt === 1 ? Retry('a transient failure') : Done);
/// drained(queued); requeueExpired(now + 2 * LEASE); drained(queued);
/// assert(queued.handlers.asked === 2); assert(state(queued) === DONE);
/// ```
#[test]
fn a_retryable_deletion_whose_claim_ran_out_is_queued_again_and_the_next_attempt_deletes_it() {
    let queued: Queued = queued("settling-retry-then-done", |attempt: u32| {
        return if attempt == 1 { Verdict::Retry(String::from("a transient failure of the destroy")) } else { Verdict::Done };
    });
    drained(&queued);
    assert_eq!(settled_in(queued.work.as_path(), queued.id.as_str()), CLAIMED, "the first attempt keeps its claim");
    let after_the_lease: ClaimTime = ClaimTime::read().advanced(CLAIM_LEASE * 2);
    assert_eq!(
        requeue_expired(queued.work.as_path(), &after_the_lease, CLAIM_LEASE, MAX_TRIES),
        1,
        "the claim whose lease ran out is taken back, which is the retry the answer asked for"
    );
    assert_eq!(settled_in(queued.work.as_path(), queued.id.as_str()), PENDING, "the record waits for the round that follows");
    drained(&queued);
    assert_eq!(queued.handlers.asked.load(Ordering::SeqCst), 2, "the record is tried again after a retryable verdict");
    assert_eq!(settled_in(queued.work.as_path(), queued.id.as_str()), DONE, "the attempt that followed deleted the subvolume and the record is done");
    remove(queued.root.as_path());
}

/// What:
///  The other verdicts keep the places they had:
/// a done one goes to `done`,
/// a failure the handler gave up on goes to `failed`,
/// and a skip is done.
/// Why:
///   Only the retry may change;
/// a settlement that sent every verdict to `pending` would pass the control above and fail the queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(stateAfter(Done) === DONE); assert(stateAfter(Failed('x')) === FAILED); assert(stateAfter(Skipped('x')) === DONE);
/// ```
#[test]
fn the_other_verdicts_keep_the_state_they_had() {
    let cases: [(Verdict, &str); 3] = [
        (Verdict::Done, DONE),
        (Verdict::Failed(String::from("this mount does not allow the destroy")), FAILED),
        (Verdict::Skipped(String::from("the record names no subvolume")), DONE),
    ];
    for (verdict, state) in cases {
        let answered: Verdict = verdict.clone();
        let queued: Queued = queued("settling-other-verdicts", move |_attempt: u32| { return answered.clone(); });
        drained(&queued);
        assert_eq!(queued.handlers.asked.load(Ordering::SeqCst), 1, "{verdict:?} is settled by the attempt that answered it");
        assert_eq!(settled_in(queued.work.as_path(), queued.id.as_str()), state, "{verdict:?} is settled in {state}");
        remove(queued.root.as_path());
    }
}

/// What:
///  A relink the service gives up on leaves a notice in the worktree's own Git directory,
///  and the notice names the command that finishes the job.
/// Why:
///   A record that reached `failed` is scanned by nothing,
///   so without the notice the person keeps a worktree whose package links still name the checkout the copy came from,
///   and learns of it only when a tool fails in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(tellAboutGivenUp(gitDir, [relinkItem])); assert(takeNotices(gitDir)[0].includes('pnpm --dir '));
/// ```
#[test]
fn a_given_up_relink_leaves_a_notice_naming_the_command() {
    let root: PathBuf = fixture("settling-given-up-relink");
    let git_dir: PathBuf = root.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    let worktree: PathBuf = root.join("the worktree");
    let entry: crate::package_repair_relink::RelinkEntry = crate::package_repair_relink::RelinkEntry {
        root: worktree.clone(),
        search_path: crate::package_repair_relink::SearchPathOfTheAdd::Unset,
        tools: vec![crate::package_repair_relink::RelinkTool::Pnpm],
    };
    let item: WorkItem = crate::package_repair_relink::item_of(git_dir.as_path(), worktree.as_path(), &entry).expect("the record");
    assert!(tell_about_given_up(git_dir.as_path(), std::slice::from_ref(&item)), "the person was not told");
    let lines: Vec<String> = crate::package_repair_relink_notice::take_notices(git_dir.as_path());
    assert_eq!(lines.len(), 1, "one notice per record given up on: {lines:?}");
    assert!(lines[0].contains("pnpm --dir "), "the notice names no command: {lines:?}");
    remove(root.as_path());
}

/// A handler whose relink answers one retryable failure every time it is asked.
struct RelinkRetries {
    /// What the relink says failed.
    said: String,
}

impl WorkHandlers for RelinkRetries {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn relink(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Retry(self.said.clone());
    }
}

/// Drain whatever the common directory holds once.
fn drained_once(common: &Path, handlers: &RelinkRetries) {
    let mut replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    crate::lazy_runtime::run_on_runtime(drain_pending_awaiting(
        common,
        handlers,
        &ClaimTime::read(),
        CLAIM_LEASE,
        Renewal::Timed,
        &mut replanning,
        &SERVICE_KINDS,
        crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
    ))
    .expect("the pending drain runs");
}

/// What:
///  A relink whose tries run out quotes the reason the last try gave,
///  and that notice waits in the relinked worktree's own Git directory.
/// Why:
///   The step that gives a record up is the recovery,
///   which holds the record and never the answer the tool gave,
///   so without the reason kept beside the record the person reads a count of tries where the reason is owed.
///   The notice is written where a command run in that worktree reads it:
///   a notice left in the Git directory of the worktree whose command added it is shown by a command about something
///   else, which is what the person complained about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; attempt < MAX_TRIES; attempt += 1) { drainOnce(common); requeueExpired(work, after, LEASE, MAX_TRIES, skipNotice(common)); }
/// assert(takeNotices(admin)[0].includes('the last try failed with: pnpm: exit status 1: ERR_PNPM_OUTDATED_LOCKFILE'));
/// ```
#[test]
fn a_given_up_relink_quotes_the_reason_the_last_try_gave_and_lands_in_its_own_worktree() {
    let root: PathBuf = fixture("settling-given-up-relink-reason");
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("the worktree");
    let admin: PathBuf = common.join("worktrees/the-worktree");
    std::fs::create_dir_all(common.as_path()).expect("common");
    std::fs::create_dir_all(worktree.as_path()).expect("worktree");
    std::fs::create_dir_all(admin.as_path()).expect("registration");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    let entry: crate::package_repair_relink::RelinkEntry = crate::package_repair_relink::RelinkEntry {
        root: worktree.clone(),
        search_path: crate::package_repair_relink::SearchPathOfTheAdd::Unset,
        tools: vec![crate::package_repair_relink::RelinkTool::Pnpm],
    };
    let item: WorkItem = crate::package_repair_relink::item_of(common.as_path(), worktree.as_path(), &entry).expect("the record");
    let work: PathBuf = work_directory(common.as_path());
    enqueue(work.as_path(), &item).expect("the record is queued");
    let handlers: RelinkRetries = RelinkRetries { said: String::from("pnpm: exit status 1: ERR_PNPM_OUTDATED_LOCKFILE") };
    // Every attempt is tried and the claim it keeps is taken back until the record's tries run out, which is where the
    //  service gives it up and tells the person.
    let mut given_up: Vec<WorkItem> = Vec::new();
    for _attempt in 1..=MAX_TRIES {
        drained_once(common.as_path(), &handlers);
        let after_the_lease: ClaimTime = ClaimTime::read().advanced(CLAIM_LEASE * 2);
        let recovered = crate::work_item_recovery::requeue_expired_reporting_telling(work.as_path(), &after_the_lease, CLAIM_LEASE, MAX_TRIES, &crate::service_work_handlers_replanning_and_acceptance::SkipNotice(common.as_path()));
        given_up.extend(recovered.given_up);
    }
    assert_eq!(given_up.len(), 1, "the record whose tries ran out is given up on once");
    let notices: Vec<String> = crate::package_repair_relink_notice::take_notices(admin.as_path());
    assert_eq!(notices.len(), 1, "one notice waits in the relinked worktree's own Git directory: {notices:?}");
    assert!(notices[0].contains("the last try failed with: pnpm: exit status 1: ERR_PNPM_OUTDATED_LOCKFILE"), "the notice does not quote the reason: {notices:?}");
    assert!(crate::package_repair_relink_notice::take_notices(common.as_path()).is_empty(), "the notice was left in the Git directory of the worktree whose command added it");
    // The reason the notice quoted is not kept once the record it belongs to has settled.
    assert!(!work_directory(common.as_path()).parent().expect("the wrapper's folder").join("relink-failures").join(item.id()).is_file(), "the reason of a settled record is left behind");
    remove(root.as_path());
}
