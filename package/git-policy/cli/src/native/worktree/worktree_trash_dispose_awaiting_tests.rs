//! What:
//!  A trash that needs more rounds than the tries a failure is given is emptied and settled `done`,
//!  with no round spending a try.
//! Why:
//!  The review of `af0b824df3` reproduced a trash of a removed worktree left in `failed` after three bounded rounds,
//! where nothing reads it again
//! (`/home/user/temp/agent/change-reviews/git-policy-cli-landing-never-waits-answers-after-its-third-review/`,
//! finding 1).
//!  The emptying of a large trash is finished over many rounds by design (`TRASH_ITEM_CEILING`), so a round that removed
//! entries must be counted apart from a round that could not work:
//! the first is queued again for a later look with the tries it had,
//! the second is a retry the claim's lease and `MAX_TRIES` bound.
//! This control drives the rounds one at a time through the shipped emptying, the shipped settling and the shipped
//! recovery, holding each round's wave past its ceiling with the emptying's own hold point, so the rounds are states of
//! the work and not a size of tree or a slow disk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claimed = claimAllOf(work, [Trash]); const verdict = await emptyTrashAwaiting(claimed[0].item); settle(verdict); if (!done) requeueExpired(...)
//! expect(rounds).toBeGreaterThan(MAX_TRIES); expect(state).toBe('done'); expect(tries).toBe(0)
//! ```

use super::*;
use crate::claim_time::ClaimTime;
use crate::fs_beneath::{ObjectId, identity_at, identity_of, open_directory_at, open_directory_beneath, open_root, plain_components};
use crate::service_claim_renewal::held_claims;
use crate::service_hold_a_step_for_a_control::{BEFORE_A_TRASH_WAVE, hold_for_a_control, let_go_file, reached_file};
use crate::service_work::{CLAIM_LEASE, work_directory};
use crate::service_work_handlers_replanning_and_acceptance::MAX_TRIES;
use crate::service_work_settling_with_words::settle_with_the_words;
use crate::service_work_storage_awaited::{claim_all_awaiting, requeue_expired_awaiting};
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, Claimed, DONE, FAILED, PENDING, WorkItem, WorkKind, id_of_entry, state_directory};
use crate::worktree_trash::{PendingTrash, TRASH_PARENT, TrashManifest, entry_name, record_the_trash};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How many entries the trash holds:
/// more than the rounds a trash may need, at one wave an entry, than the tries a failure is given.
/// How many rounds the trash must need to prove the point:
/// one more than the tries a failure is given.
const ROUNDS_NEEDED: usize = MAX_TRIES as usize + 1;

/// One trash item's ceiling, as this control states it:
/// a second, so a wave held past it ends that round in a moment.
const THE_CEILING: Duration = Duration::from_secs(1);

/// The longest this control waits for a state it expects within a moment.
const PATIENCE: Duration = Duration::from_secs(30);

/// A worktree with a planted trash of as many entries as the rounds need at this host's wave width,
/// the record the leave-out would have written,
/// and the directory a hold of this control goes in.
struct Staged {
    /// The fixture's root, removed at the end.
    root: PathBuf,
    /// The worktree that holds the trash.
    worktree: PathBuf,
    /// The common Git directory the record sits under.
    common: PathBuf,
    /// The work directory the record's states are in.
    work: PathBuf,
    /// The record's id.
    id: String,
    /// The trash's recorded name.
    name: String,
    /// Where the hold of this control says it was reached and is let go.
    holds: PathBuf,
    /// How many entries the trash holds.
    entries: usize,
}

/// What:
///  A linked worktree of a common directory,
///  with a trash of as many entries as the rounds need,
///  and the record of that trash in the common directory's work directory.
/// Why:
///   The emptying proves the worktree's `.git` pointer, the trash's identity and each entry's identity, and this is the
/// shape `git worktree add` leaves behind, written by hand so a control needs no snapshot.
fn stage(label: &str) -> Staged {
    let root: PathBuf = fixture(label);
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = common.join("worktrees").join("topic");
    std::fs::create_dir_all(git_dir.as_path()).expect("the administrative directory");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect("the .git file");
    let name: String = "7".repeat(40);
    // What:
    //  Enough entries that the trash needs more rounds than the tries a failure is given at the wave width this process
    //  has.
    // Why:
    //  A wave is `places_a_round_may_take()` entries at most and that width follows the CPUs a run is given,
    // so a fixed count would prove nothing on a host whose wave is wide: the count is the width times the rounds.
    let entries: usize = crate::blocking_jobs_within_the_budget::places_a_round_may_take().max(1) * ROUNDS_NEEDED;
    for index in 0..entries {
        let entry: PathBuf = worktree.join(TRASH_PARENT).join(name.as_str()).join(entry_name(index));
        std::fs::create_dir_all(entry.join("payload")).expect("a trash entry");
        std::fs::write(entry.join("payload/credentials"), b"another worktree's token\n").expect("an entry file");
    }
    let held: std::fs::File = open_root(worktree.as_path()).expect("the worktree");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("a plain parent");
    let parent: std::fs::File = open_directory_beneath(&held, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = open_directory_at(&parent, OsStr::new(name.as_str())).expect("the trash");
    let identities: Vec<ObjectId> = (0..entries)
        .map(|index: usize| { return identity_at(&trash, OsStr::new(entry_name(index).as_str())).expect("an entry identity"); })
        .collect();
    let manifest: TrashManifest = TrashManifest { name: name.clone(), trash: identity_of(&trash).expect("the trash identity"), entries: identities };
    let pending: PendingTrash = PendingTrash { destination: worktree.clone(), common_dir: common.clone(), manifest, taken_over: 0, quiet: true };
    record_the_trash(&pending).expect("the record the add writes before it returns");
    let work: PathBuf = work_directory(common.as_path());
    let holds: PathBuf = root.join("holds");
    std::fs::create_dir_all(holds.as_path()).expect("the directory the hold goes in");
    hold_for_a_control(BEFORE_A_TRASH_WAVE, holds.as_path());
    // One trash item's bound, as this control states it: the emptying reads it once for the life of the process.
    // SAFETY: the control's own process, before the runtime exists and before any thread reads this variable.
    unsafe {
        std::env::set_var(crate::wait_ceilings::TRASH_ITEM_CEILING_VARIABLE, THE_CEILING.as_secs().to_string());
    }
    return Staged { root, worktree, work, common, id: format!("trash-{name}"), name, holds, entries };
}

impl Staged {
    /// How many of the trash's entries are still there.
    fn entries_present(&self) -> usize {
        let trash: PathBuf = self.worktree.join(TRASH_PARENT).join(self.name.as_str());
        return (0..self.entries).filter(|index| { return trash.join(entry_name(*index)).exists(); }).count();
    }

    /// Where the record stands now,
    /// a state its name is in,
    /// or `claimed` when one of its claims is.
    fn state(&self) -> &'static str {
        for state in [PENDING, DONE, FAILED] {
            if state_directory(self.work.as_path(), state).join(self.id.as_str()).is_file() {
                return state;
            }
        }
        let claimed: bool = std::fs::read_dir(state_directory(self.work.as_path(), CLAIMED)).is_ok_and(|entries| {
            return entries.flatten().any(|entry| {
                return id_of_entry(entry.file_name().to_string_lossy().as_ref()) == self.id;
            });
        });
        return if claimed { CLAIMED } else { "no state" };
    }

    /// The tries the record in `state` carries.
    fn tries(&self, state: &str) -> Option<u32> {
        let text: String = std::fs::read_to_string(state_directory(self.work.as_path(), state).join(self.id.as_str())).ok()?;
        return WorkItem::decode(text.as_str()).ok().map(|item| { return item.tries; });
    }

    /// The record in `state`,
    /// read back.
    fn record(&self, state: &str) -> Option<WorkItem> {
        let text: String = std::fs::read_to_string(state_directory(self.work.as_path(), state).join(self.id.as_str())).ok()?;
        return WorkItem::decode(text.as_str()).ok();
    }

    /// What:
    ///  Hold the next round's wave past the ceiling,
    ///  and let it go when the round has been inside it for longer than the ceiling.
    /// Why:
    ///  The ceiling stops an emptying only between two waves, so a wave held past it is the one state that makes a round
    /// end at its bound: no size of tree and no slow disk is needed.
    /// The files are taken away first, so each round is held on its own and not on the one before it.
    fn hold_one_round(&self, ceiling_reached: &std::sync::Arc<std::sync::atomic::AtomicUsize>) -> std::thread::JoinHandle<()> {
        let counted: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::clone(ceiling_reached);
        let reached: PathBuf = reached_file(self.holds.as_path(), BEFORE_A_TRASH_WAVE);
        let let_go: PathBuf = let_go_file(self.holds.as_path(), BEFORE_A_TRASH_WAVE);
        let _armed = std::fs::remove_file(reached.as_path());
        let _disarmed = std::fs::remove_file(let_go.as_path());
        return std::thread::spawn(move || {
            let deadline: Instant = Instant::now() + PATIENCE;
            while !reached.exists() {
                assert!(Instant::now() < deadline, "the emptying never reached the wave the control holds");
                std::thread::sleep(Duration::from_millis(5));
            }
            std::thread::sleep(THE_CEILING + Duration::from_millis(200));
            counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::fs::write(let_go.as_path(), b"").expect("the file that lets the held wave go");
        });
    }
}

/// What:
///  One round of the emptying as the slow lane runs it:
/// take back the claims whose holder stopped renewing them,
/// claim the record,
///  empty as much as the bound allows,
///  and settle what the emptying answered.
/// Why:
///   This is the shipped path a trash item takes:
/// the recovery, the claim, `empty_trash_awaiting`, the settling and the claim's lease,
/// so a control of it proves the answer and the settling together and not one of them alone.
fn a_round(staged: &Staged, now: &ClaimTime, ceiling_reached: &std::sync::Arc<std::sync::atomic::AtomicUsize>) -> bool {
    let requeued: bool = crate::lazy_runtime::run_on_runtime(requeue_expired_awaiting(staged.work.as_path(), staged.common.as_path(), now, CLAIM_LEASE))
        .expect("the runtime takes back the claims whose lease ran out");
    assert!(requeued, "the recovery ran");
    let (held, _renewer) = held_claims(staged.work.as_path());
    let claimed: Vec<Claimed> = crate::lazy_runtime::run_on_runtime(claim_all_awaiting(staged.work.as_path(), &[WorkKind::Trash]))
        .expect("the runtime claims the record")
        .expect("the claim of the record");
    let Some(item) = claimed.into_iter().next() else { return false };
    held.hold(&item);
    let holding: std::thread::JoinHandle<()> = staged.hold_one_round(ceiling_reached);
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(empty_trash_awaiting(&item.item)).expect("the runtime empties the trash");
    holding.join().expect("the control that held the wave");
    // The hold's files are taken away, so the round after this one is held on its own.
    let _cleared = std::fs::remove_file(let_go_file(staged.holds.as_path(), BEFORE_A_TRASH_WAVE).as_path());
    let settled = crate::lazy_runtime::run_on_runtime(settle_with_the_words(held, staged.work.as_path(), item, verdict, "trash"))
        .expect("the runtime settles the answer");
    assert!(settled.is_some(), "the settling of an answer that answers in words does not take a thread that is refused");
    return true;
}

/// What:
///  A trash whose emptying needs more rounds than the tries a failure gets is emptied and settled `done`,
///  and no round spends a try.
/// Why:
///  This is the review's finding 1 as a control:
/// before the repair every round answered `Retry`, the recovery gave the record one more try each time it ran out its
/// lease, and the record reached `failed` at `MAX_TRIES` with entries still on the disk, where nothing reads it again.
/// The control drives a trash of more entries than the tries cover, holding each round past its ceiling, and asserts the three
/// things the repair makes true: the rounds were more than the tries, the record is `done`, and its tries stayed zero.
#[test]
fn a_trash_of_more_rounds_than_the_tries_is_emptied_and_never_given_up_on() {
    let staged: Staged = stage("trash-more-rounds-than-the-tries");
    let mut rounds: usize = 0;
    let ceiling_reached: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut now: ClaimTime = ClaimTime::read();
    while rounds <= MAX_TRIES as usize * 3 && staged.state() != DONE {
        let claimed: bool = a_round(&staged, &now, &ceiling_reached);
        assert!(claimed, "the record was claimable while it stood in {}", staged.state());
        rounds += 1;
        // A round that kept its claim is queued again only when its lease runs out, which is the recovery's own pause.
        now = ClaimTime::read().advanced(CLAIM_LEASE + Duration::from_secs(1));
    }
    assert_ne!(staged.state(), FAILED, "a trash whose rounds remove entries may not be given up on");
    assert_eq!(staged.state(), DONE, "the trash of more rounds than the tries is emptied and done");
    assert_eq!(staged.entries_present(), 0, "every entry of the trash is gone");
    assert_eq!(staged.tries(DONE), Some(0), "no round spent a try of a failure: a round that removed entries is queued again for a later look");
    assert!(
        rounds > MAX_TRIES as usize,
        "the trash was emptied in {rounds} rounds, which is not more than the {MAX_TRIES} tries a failure is given"
    );
    assert_eq!(
        ceiling_reached.load(std::sync::atomic::Ordering::SeqCst),
        rounds,
        "every round was held past its ceiling, so every one of them ended at its bound"
    );
    let done: WorkItem = staged.record(DONE).expect("the record that is done");
    assert_eq!(done.kind, WorkKind::Trash);
    let _closed = std::fs::remove_file(staged.holds.join(format!("{BEFORE_A_TRASH_WAVE}.reached")));
    remove(staged.root.as_path());
    let _ = staged.common;
}
