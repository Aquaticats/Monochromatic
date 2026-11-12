//! Controls for the service's slow lane:
//! the round that answers a wake claims the pushes alone,
//! the lane claims the kinds that can take minutes,
//! and a push is taken while the service is inside a trash.
//! Why:
//!  On `2026-11-10` the round of the main checkout emptied trashes for 18 minutes
//! (`18:16:11` to `18:34:47`) and seven landings of `refs/heads/main` waited for it;
//! the day holds four more such intervals,
//! the longest 4 946 s
//!  (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! round(gitDir); expect(trash).not.toHaveRun(); lane(gitDir); expect(trash).toHaveRun();
//! ```

use super::*;
use crate::claim_time::ClaimTime;
use crate::parallel_budget::HostResources;
use crate::push_retry::{PLAN_DIRECTORY, PushRetries, Retrying};
use crate::service_claim_renewal_running::Renewal;
use crate::service_claim_watch::ClaimWatch;
use crate::service_round::{Assignment, RoundDrain, drain_directories};
use crate::service_server_work_loop_awaiting::work_loop_awaiting;
use crate::service_work::{GitHandlers, Verdict, work_directory};
use crate::service_work_handlers_replanning_and_acceptance::{PUSH_KINDS, SERVICE_KINDS, WorkHandlers};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository, wait_until};
use crate::work_item::{CLAIMED, DONE, FAILED, PENDING, WorkItem, WorkKind, enqueue, state_directory};
use crate::worktree_trash_dispose_awaiting::INSIDE_A_SLOW_STEP;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

/// The branch every push item of these controls names.
const BRANCH: &str = "refs/heads/main";

/// The pause before a push of these controls is tried again;
/// no control here waits for one.
const PAUSE: Duration = Duration::from_millis(300);

/// A host large enough that neither the CPUs nor the memory bound a round of one repository.
const LARGE_HOST: HostResources = HostResources { parallelism: 8, memory_available: None };

/// A repository directory standing in for a Git directory,
/// with a worktree directory beside it and a work directory.
struct World {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The Git directory the records name.
    common: PathBuf,
    /// The work directory the service drains.
    work: PathBuf,
}

/// One record of `kind` for `world`,
/// acceptable to the service and leaving nothing behind when it runs.
fn record(world: &World, kind: WorkKind, oid: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: world.root.clone(),
        git_dir: world.common.clone(),
        branch_ref: if kind == WorkKind::Push { BRANCH.to_string() } else { String::from("refs/cli-git/trash") },
        oid: oid.to_string(),
        parent: String::new(),
        created: String::from("2026-11-10T18:16:51.000Z"),
        tries: 0,
    };
}

/// A directory standing for a repository,
/// with the records of the kinds given written for it.
fn world(name: &str, kinds: &[(WorkKind, &str)]) -> World {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir_all(&common).expect("the Git directory");
    std::fs::create_dir_all(&worktree).expect("the worktree directory");
    let work: PathBuf = work_directory(common.as_path());
    let world: World = World { root, common, work };
    for (kind, oid) in kinds {
        let item: WorkItem = record(&world, *kind, oid);
        enqueue(world.work.as_path(), &item).expect("the record is written");
    }
    return world;
}

/// The names of the records of `world` in `state`,
/// sorted.
fn names_in(world: &World, state: &str) -> Vec<String> {
    return names_in_at(world.work.as_path(), state);
}

/// The names of the records of the work directory `work` in `state`,
/// sorted.
fn names_in_at(work: &Path, state: &str) -> Vec<String> {
    let mut names: Vec<String> = state_directory(work, state)
        .read_dir()
        .map_or_else(|_no_directory| return Vec::new(), |listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); });
    names.sort();
    return names;
}

/// Handlers that count what ran and answer done,
/// holding only what several threads may share.
#[derive(Default)]
struct Counted {
    /// How many pushes ran.
    pushes: AtomicUsize,
    /// How many trashes ran.
    trashes: AtomicUsize,
    /// How many reviews ran.
    reviews: AtomicUsize,
}

impl WorkHandlers for Counted {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        self.pushes.fetch_add(1, Ordering::SeqCst);
        return Verdict::Done;
    }

    fn trash(&self, _item: &WorkItem) -> Verdict {
        self.trashes.fetch_add(1, Ordering::SeqCst);
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        self.reviews.fetch_add(1, Ordering::SeqCst);
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// The oids of the records the round's control writes,
/// each named once so an assertion says which record it read.
const THE_ROUNDS_PUSH: &str = "1111111111111111111111111111111111111111";

const THE_ROUNDS_TRASH: &str = "2222222222222222222222222222222222222222";

const THE_ROUNDS_REVIEW: &str = "3333333333333333333333333333333333333333";

/// The oids of the records the lane's control writes.
const THE_LANES_PUSH: &str = "4444444444444444444444444444444444444444";

const THE_LANES_TRASH: &str = "5555555555555555555555555555555555555555";

const THE_LANES_REVIEW: &str = "6666666666666666666666666666666666666666";

/// Whether the record of `kind` and `oid` is in `state` of `world`.
/// Why:
///   The name a record is stored under carries the hash of the destination it is for
/// (`push-<oid>-<destination hash>`),
/// which a control does not write and cannot name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesIn(world, state).some((name) => name.startsWith(`${kind}-${oid}`))
/// ```
fn record_of(world: &World, state: &str, kind: WorkKind, oid: &str) -> bool {
    let wanted: String = format!("{}-{oid}", kind.name());
    return names_in(world, state).iter().any(|name| { return name.starts_with(wanted.as_str()); });
}

/// What:
///  The round that answers a wake takes the push and leaves every slow kind where it is,
///  so no step that can take minutes stands between a landing and its push.
/// Why:
///   A round holds the after-commit work until it ends,
/// and the push a landing leaves is claimed by the round its wake starts:
/// a round that claimed a trash goes on emptying it for as long as that takes,
///  and the push of the next landing waits for the whole of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// round(world); expect(counted.pushes).toBe(1); expect(namesIn(world, 'pending')).toEqual(['review-…', 'trash-…'])
/// ```
#[test]
fn the_round_that_serves_the_person_takes_the_push_and_leaves_the_slow_kinds() {
    let world: World = world(
        "slow-lane-the-round",
        &[(WorkKind::Push, THE_ROUNDS_PUSH), (WorkKind::Trash, THE_ROUNDS_TRASH), (WorkKind::Review, THE_ROUNDS_REVIEW)],
    );
    let counted: Arc<Counted> = Arc::new(Counted::default());
    let assignments: Vec<Assignment> = vec![Assignment::plain(world.common.clone())];
    let _waits: Vec<(PathBuf, Duration)> =
        drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::clone(&counted), now: &ClaimTime::read(), host: &LARGE_HOST }).waits;
    assert_eq!(counted.pushes.load(Ordering::SeqCst), 1, "the round took the push");
    assert_eq!(counted.trashes.load(Ordering::SeqCst), 0, "the round claims no trash");
    assert_eq!(counted.reviews.load(Ordering::SeqCst), 0, "the round claims no review");
    assert!(record_of(&world, DONE, WorkKind::Push, THE_ROUNDS_PUSH), "the round settled the push");
    assert_eq!(names_in(&world, DONE).len(), 1, "the round settled nothing else");
    assert!(record_of(&world, PENDING, WorkKind::Trash, THE_ROUNDS_TRASH), "the trash waits for the lane");
    assert!(record_of(&world, PENDING, WorkKind::Review, THE_ROUNDS_REVIEW), "the review waits for the lane");
    remove(world.root.as_path());
}

/// What:
///  The lane runs the slow kinds the round left,
/// and takes no push.
/// Why:
///   The two claim lists partition the kinds,
/// so no record is run twice and none is left behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lane(world); expect(counted.trashes).toBe(1); expect(namesIn(world, 'pending')).toEqual(['push-…'])
/// ```
#[test]
fn the_lane_runs_the_slow_kinds_the_round_left() {
    let world: World = world(
        "slow-lane-the-lane",
        &[(WorkKind::Push, THE_LANES_PUSH), (WorkKind::Trash, THE_LANES_TRASH), (WorkKind::Review, THE_LANES_REVIEW)],
    );
    let counted: Counted = Counted::default();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    let _left: Option<Duration> = runtime.block_on(drain_the_slow_work_awaiting(
        world.common.as_path(),
        &counted,
        HOUSEKEEPING_OF_ONE_ROUND,
        crate::service_work::CLAIM_LEASE,
    ));
    assert_eq!(counted.trashes.load(Ordering::SeqCst), 1, "the lane took the trash");
    assert_eq!(counted.reviews.load(Ordering::SeqCst), 1, "the lane took the review");
    assert_eq!(counted.pushes.load(Ordering::SeqCst), 0, "the lane takes no push");
    assert_eq!(names_in(&world, PENDING).len(), 1, "only the push waits");
    assert!(record_of(&world, PENDING, WorkKind::Push, THE_LANES_PUSH), "the push is left to the round");
    remove(world.root.as_path());
}

/// What:
///  The round's kinds and the lane's are the kinds the service runs,
/// each of them in exactly one list.
/// Why:
///   The two lists are written apart,
/// and a kind the service gained but one list did not would wait in `pending` for ever
/// with no step that fails to say so:
/// the round claims the pushes and the lane the rest
///  (`landing/service_work_slow_lane.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect([...PUSH_KINDS, ...SLOW_KINDS].sort()).toEqual([...SERVICE_KINDS].sort())
/// ```
#[test]
fn the_two_claim_lists_partition_the_kinds_the_service_runs() {
    let mut claimed: Vec<&str> = Vec::new();
    for kind in PUSH_KINDS.iter().chain(SLOW_KINDS.iter()) {
        claimed.push(kind.name());
    }
    let mut service: Vec<&str> = Vec::new();
    for kind in &SERVICE_KINDS {
        service.push(kind.name());
    }
    claimed.sort_unstable();
    service.sort_unstable();
    assert_eq!(claimed, service, "the round's kinds and the lane's are the service's kinds, each named once");
}

/// A repository tracking a bare remote,
/// with a commit landed on its branch and no push made for it.
struct Stage {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The commit to deliver.
    landed: String,
}

/// Build the stage:
/// an initial commit pushed,
/// then a commit landed that no push has carried.
fn stage(name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let staged: Stage = Stage { root, repo, remote, landed };
    // The reading of the remote's ref file is what the control waits on, so it is shown to see a push before the control runs.
    assert_ne!(remote_tip(&staged), staged.landed, "no push has carried the landed commit yet");
    return staged;
}

/// The branch tip the remote holds,
/// read from the bare remote's own ref file.
fn remote_tip(stage: &Stage) -> String {
    return std::fs::read_to_string(stage.remote.join("refs").join("heads").join("main")).map(|text| { return text.trim().to_owned(); }).unwrap_or_default();
}

/// The trash record one of these controls writes into a repository's work directory.
fn a_trash_record(staged: &Stage, oid: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Trash,
        repository: staged.repo.clone(),
        git_dir: staged.repo.join(".git"),
        branch_ref: String::from("refs/cli-git/trash"),
        oid: oid.to_string(),
        parent: String::new(),
        created: String::from("2026-11-10T18:16:51.000Z"),
        tries: 0,
    };
}

/// A push record for the stage's landed commit.
fn a_push_record(staged: &Stage) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: staged.repo.clone(),
        git_dir: staged.repo.join(".git"),
        branch_ref: String::from(BRANCH),
        oid: staged.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-10T18:20:53.000Z"),
        tries: 0,
    };
}

/// What:
///  Start the after-commit work as a task of the process's runtime,
/// as the service does,
/// and give the way to wake it and the task itself.
/// Why:
///   The loop is a task,
/// so what reaches it is a channel of wakes,
/// and dropping the sender ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [wake, running] = startTheLoop(); wake.send([gitDir, null]);
/// ```
fn start_the_loop(staged: &Stage) -> (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::task::JoinHandle<()>) {
    let plans: PathBuf = staged.root.join("journal").join(PLAN_DIRECTORY);
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let (control, taken): (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::sync::mpsc::UnboundedReceiver<Wake>) = tokio::sync::mpsc::unbounded_channel::<Wake>();
    let running: tokio::task::JoinHandle<()> = crate::lazy_runtime::spawn_on_runtime(async move {
        let mut receiver = taken;
        work_loop_awaiting(&mut receiver, &handlers, &busy, PushRetries::restored(PAUSE, plans.as_path()), ClaimWatch::in_memory()).await;
    })
    .expect("the runtime starts the after-commit work");
    return (control, running);
}

/// The end that says the service is inside a trash and the one that lets it go.
struct HeldTrash {
    /// Set once the service is inside the trash.
    arrived: Arc<AtomicBool>,
    /// Sending on it lets the trash go on.
    let_go: tokio::sync::oneshot::Sender<()>,
}

/// What:
///  Hold the service inside a trash on whichever task runs it,
/// without holding the worker of the runtime that task sits on.
/// Why:
///   The hold is a state and not a duration,
/// so "while the trash still runs" is what the control asserts about;
/// the reading that the state was reached is a task of the process's runtime,
///  so the test's own thread needs no runtime.
/// A hold that blocked the thread it is reached on would stop a worker of the runtime,
/// and the push this control watches for would then have nowhere to run:
/// no step of the shipped service blocks a worker while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { arrived, letGo } = holdInsideATrash(); // the service is inside the trash once arrived holds
/// ```
fn hold_inside_a_trash() -> HeldTrash {
    let (reached, reached_wait) = tokio::sync::oneshot::channel::<()>();
    let (let_go, held) = tokio::sync::oneshot::channel::<()>();
    crate::service_fault::hooks::hold_awaiting_on_any_thread(crate::service_fault::hooks::HoldAwaitingOnAnyThread { point: String::from(INSIDE_A_SLOW_STEP), reached, let_go: held });
    let arrived: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let told: Arc<AtomicBool> = Arc::clone(&arrived);
    let _watching = crate::lazy_runtime::spawn_on_runtime(async move {
        let _said = reached_wait.await;
        told.store(true, Ordering::SeqCst);
    })
    .expect("the runtime says the trash was reached");
    return HeldTrash { arrived, let_go };
}

/// What:
///  A landing's push is taken while the service is inside a trash,
/// so no step that can take minutes holds a person's backup.
/// Why:
///   This is the state the journal of `2026-11-10` shows:
/// the round of the main checkout was inside trashes from `18:16:11` to `18:34:47`
/// while seven landings waited for it,
/// and the push that carried them went out only when that round ended
///  (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// The trash is written first and woken for,
/// the service is held inside it,
/// and only then is the landing's push record written and woken for:
/// a service that runs both in one round pushes nothing until the trash ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wake.send([gitDir, null]); await held.arrived; enqueue(pushItem); wake.send([gitDir, null]); expect(remoteTip()).toBe(landed)
/// ```
#[test]
fn a_push_is_taken_while_the_service_is_inside_a_trash() {
    let staged: Stage = stage("slow-lane-inside-a-trash");
    let git_dir: PathBuf = staged.repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &a_trash_record(&staged, "7777777777777777777777777777777777777777")).expect("the trash is written");
    let held: HeldTrash = hold_inside_a_trash();
    let (wake, running): (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::task::JoinHandle<()>) = start_the_loop(&staged);
    wake.send((git_dir.clone(), None)).expect("the trash's wake");
    assert!(wait_until(&|| { return held.arrived.load(Ordering::SeqCst); }), "the service is inside the trash");
    enqueue(work.as_path(), &a_push_record(&staged)).expect("the push is written");
    wake.send((git_dir, None)).expect("the push's wake");
    assert!(wait_until(&|| { return remote_tip(&staged) == staged.landed; }), "the push landed while the trash was still running");
    let _let_go = held.let_go.send(());
    drop(wake);
    assert!(wait_until(&|| { return running.is_finished(); }), "the after-commit work ends when its queue is closed");
    remove(staged.root.as_path());
}

/// What:
///  The loop's wake settles a trash through the lane,
/// so a repository is not looked at for the same trash for ever.
/// Why:
///   The lane claims the trash a wake names,
/// and the record leaves `pending` with the answer of the step,
/// which is where a service that restarts finds it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wake.send([gitDir, null]); await settled('trash-…'); expect(namesInAt(work, 'pending')).not.toContain('trash-…')
/// ```
#[test]
fn the_lane_settles_the_trash_the_loop_was_woken_for() {
    let staged: Stage = stage("slow-lane-settles-a-trash");
    let git_dir: PathBuf = staged.repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &a_trash_record(&staged, "8888888888888888888888888888888888888888")).expect("the trash is written");
    let (wake, running): (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::task::JoinHandle<()>) = start_the_loop(&staged);
    wake.send((git_dir, None)).expect("the trash's wake");
    // The step settles the record for what it holds, which is what the control reads: the record leaves `pending` without the
    //  round having run any slow kind.
    let settled = |state: &str| -> bool {
        return names_in_at(work.as_path(), state).iter().any(|name| { return name.starts_with("trash-8888888888888888888888888888888888888888"); });
    };
    assert!(wait_until(&|| { return settled("failed") || settled(DONE); }), "the lane settled the trash");
    drop(wake);
    assert!(wait_until(&|| { return running.is_finished(); }), "the after-commit work ends when its queue is closed");
    remove(staged.root.as_path());
}

/// The slice one look of the lane's control may start housekeeping for:
/// nothing,
/// so a look has spent it before it reaches the housekeeping it finds and every item after the first of a look asks to be
///  looked at again.
const A_SLICE_ONE_LOOK_SPENDS: Duration = Duration::ZERO;

/// The trash the lane's control holds its look inside.
const THE_HELDS_TRASH: &str = "9999999999999999999999999999999999999999";

/// The trash that look does not start,
/// which only the look after it can settle.
const THE_LEFT_TRASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// What:
///  A look that could not start every housekeeping item it found leaves the rest `pending`,
/// and the lane looks at that repository again by itself after the pause the drain asked for,
/// with no second wake.
/// Why:
///   A trash of one removed worktree is hundreds of thousands of removals,
/// and a look that ran every one of them in turn would keep the lane from every other repository's work for as long as it
///  took
/// (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
/// The slice bounds what one look starts
/// (`housekeeping_of_one_round.rs`),
/// so a record it did not start goes back to `pending` and the drain names the pause it wants;
/// the lane is the only thing that will look again,
/// so a lane that dropped that answer would leave the trash waiting for the next wake of any kind.
/// The slice here is nothing rather than a duration to wait out,
/// because the state the control needs is "this look has spent its slice before the housekeeping",
/// which a bound of nothing reaches at every look.
/// The first item of a look starts all the same,
/// and these records are all trashes,
/// so a look that spent its first start on the empty deletion step would defer both of them for ever and the control would
///  never see one settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lane.ask([gitDir, null]); await held; release(); expect(await settled(first)).toBe(true); expect(await settled(second)).toBe(true)
/// ```
#[test]
fn a_look_that_ran_out_of_its_slice_is_looked_at_again() {
    let staged: Stage = stage("slow-lane-a-slice-then-again");
    let git_dir: PathBuf = staged.repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &a_trash_record(&staged, THE_HELDS_TRASH)).expect("the held trash is written");
    enqueue(work.as_path(), &a_trash_record(&staged, THE_LEFT_TRASH)).expect("the left trash is written");
    let held: HeldTrash = hold_inside_a_trash();
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let lane: SlowLane =
        crate::merge_second_pass::tests::await_it(async move { return SlowLane::start_with_slice_and_lease(&handlers, busy, A_SLICE_ONE_LOOK_SPENDS, crate::service_work::CLAIM_LEASE); });
    lane.ask((git_dir, None));
    assert!(wait_until(&|| { return held.arrived.load(Ordering::SeqCst); }), "the lane is inside the first trash");
    // The slice is already spent while that trash is held, so the trash behind it is not started by this look.
    let _let_go = held.let_go.send(());
    // The step settles the record for what it holds, which is what this control reads.
    let settled = |oid: &str| -> bool {
        let wanted: String = format!("trash-{oid}");
        return [DONE, FAILED].iter().any(|state| {
            return names_in_at(work.as_path(), *state).iter().any(|name| { return name.starts_with(wanted.as_str()); });
        });
    };
    assert!(wait_until(&|| { return settled(THE_HELDS_TRASH); }), "the first trash of a look starts however spent its slice is");
    assert!(
        wait_until(&|| { return settled(THE_LEFT_TRASH); }),
        "the trash the first look did not start is run by the look the lane takes by itself, with no second wake"
    );
    drop(lane);
    remove(staged.root.as_path());
}

/// The lease the controls of a kept claim give it,
/// so one runs out inside a control instead of the shipped 120 seconds.
const A_LEASE_A_CONTROL_WAITS_OUT: Duration = Duration::from_millis(600);

/// How long past an answer a control waits before it looks again,
/// so the claim the answer named has run out.
const A_MARGIN_PAST_THE_ANSWER: Duration = Duration::from_millis(150);

/// A repository directory with one removal of the subvolume `id` queued,
/// and the id of that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [world, id] = worldWithARemoval('name', 61);
/// ```
fn world_with_a_removal(name: &str, id: u64) -> (World, String) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let parent: PathBuf = root.join("parent");
    std::fs::create_dir(common.as_path()).expect("the Git directory");
    std::fs::create_dir(parent.as_path()).expect("the directory the removal renamed out of");
    let work: PathBuf = work_directory(common.as_path());
    let item: WorkItem = crate::worktree_remove_hand_over::removal_item(common.as_path(), parent.as_path(), id);
    enqueue(work.as_path(), &item).expect("the record is written");
    return (World { root, common, work }, item.id());
}

/// Handlers whose delete answers `said` by the attempt's number counting from one,
/// and count the attempts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Said { subvolumeDelete(item) { return this.said(this.asked.increment()) } }
/// ```
struct Said {
    /// What the delete says, by the attempt's number counting from one.
    said: Box<dyn Fn(u32) -> Verdict + Send + Sync>,
    /// How many times the delete was asked.
    asked: Arc<AtomicU32>,
}

impl WorkHandlers for Said {
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

/// The handler of a control and the counter of its attempts.
fn said(answers: impl Fn(u32) -> Verdict + Send + Sync + 'static) -> (Said, Arc<AtomicU32>) {
    let asked: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    return (Said { said: Box::new(answers), asked: Arc::clone(&asked) }, asked);
}

/// The number of tries the record claimed in `world` carries,
/// which a recovery at the claim's end writes as one more than the tries before it.
fn tries_of_the_claimed_record(world: &World) -> u32 {
    let names: Vec<String> = names_in(world, CLAIMED);
    let name: &String = names.first().expect("a record is claimed");
    let text: String = std::fs::read_to_string(state_directory(world.work.as_path(), CLAIMED).join(name)).expect("the claim is a file");
    return WorkItem::decode(text.as_str()).expect("the record decodes").tries;
}

/// What:
///  A slow item claimed after the round of the pushes has already read when the claims it answered run out,
/// and answering `Retry` because the condition can pass,
/// is looked at again when its own claim runs out,
/// with no other wake.
/// Why:
///   This is the interleaving the audit of `0.6.1-alpha.12` reproduced:
/// the server queues slow work and then runs the push round
/// (`service_server_work_loop_awaiting.rs`),
/// the push round reads when the claims it answered run out,
/// a slow item claimed after that reading has no timer,
/// and `Retry` keeps the claim (`service_work_settling_with_words.rs`),
/// so the item waited until an unrelated command woke the service:
/// 130 s after a claim whose lease is 120 s it was still the same claim with one attempt.
/// The service's own words promise it is tried again when its claim runs out,
/// so the look that keeps the claim answers the moment it ends,
/// and the look at that moment takes the claim back,
/// claims the record again with one more try, and runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [world] = worldWithARemoval(...); const [handlers, asked] = said((n) => n === 1 ? retry : done)
/// const wait = await laneLook(world); expect(asked).toBe(1); await sleep(wait); await laneLook(world); expect(asked).toBe(2)
/// ```
#[test]
fn a_slow_item_that_answers_a_retryable_answer_is_tried_again_when_its_claim_runs_out() {
    let (world, id): (World, String) = world_with_a_removal("slow-lane-a-kept-claim", 61);
    let (handlers, asked): (Said, Arc<AtomicU32>) = said(|attempt| {
        return if attempt == 1 { Verdict::Retry(String::from("the volume is busy")) } else { Verdict::Done };
    });
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    let answered: Option<Duration> = runtime.block_on(drain_the_slow_work_awaiting(
        world.common.as_path(),
        &handlers,
        HOUSEKEEPING_OF_ONE_ROUND,
        A_LEASE_A_CONTROL_WAITS_OUT,
    ));
    let waited: Duration = answered.unwrap_or_else(|| panic!("the look that keeps a claim answers when it runs out, or nothing looks at it again"));
    assert!(waited <= A_LEASE_A_CONTROL_WAITS_OUT, "the answer is no later than the claim's end: {waited:?}");
    assert_eq!(asked.load(Ordering::SeqCst), 1, "the first look ran the item once");
    assert_eq!(names_in(&world, CLAIMED).len(), 1, "the retryable answer keeps the claim");
    std::thread::sleep(waited + A_MARGIN_PAST_THE_ANSWER);
    let _again: Option<Duration> = runtime.block_on(drain_the_slow_work_awaiting(
        world.common.as_path(),
        &handlers,
        HOUSEKEEPING_OF_ONE_ROUND,
        A_LEASE_A_CONTROL_WAITS_OUT,
    ));
    assert_eq!(asked.load(Ordering::SeqCst), 2, "the look at the claim's end runs the item again, with no other wake");
    assert_eq!(names_in(&world, DONE), vec![id], "the item the second attempt finished is done");
    remove(world.root.as_path());
}

/// What:
///  A claim this look could not settle,
/// because the step that moves a settled record could not run,
/// is looked at again when its lease runs out.
/// Why:
///   `Retry` is not the only answer that keeps a claim:
/// a settling step the process could not run leaves the record claimed as well
/// (`service_work_storage_awaited.rs`),
/// and a claim nothing looks at again keeps the work behind it for ever.
/// The file where the `done` directory must be makes every settlement fail,
/// which is the state a step that cannot be run leaves.
/// The look answers the claim's end,
/// and the look at that moment takes the claim back, claims the record with one more try, and runs it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [world] = worldWithARemoval(...); blockSettling(world); const wait = await laneLook(world); await sleep(wait); await laneLook(world); expect(asked).toBe(2)
/// ```
#[test]
fn a_claim_whose_settling_could_not_run_is_looked_at_again_when_its_lease_runs_out() {
    let (world, _id): (World, String) = world_with_a_removal("slow-lane-a-claim-that-could-not-be-settled", 62);
    // A file where the `done` directory must be makes every settlement of a finished item fail, so the claim is the look's to
    //  keep and nothing else would ever look at it again.
    let done: PathBuf = state_directory(world.work.as_path(), DONE);
    std::fs::remove_dir(done.as_path()).expect("the empty done directory");
    std::fs::write(done.as_path(), b"in the way").expect("the blocker");
    let (handlers, asked): (Said, Arc<AtomicU32>) = said(|_attempt| { return Verdict::Done; });
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    let answered: Option<Duration> = runtime.block_on(drain_the_slow_work_awaiting(
        world.common.as_path(),
        &handlers,
        HOUSEKEEPING_OF_ONE_ROUND,
        A_LEASE_A_CONTROL_WAITS_OUT,
    ));
    let waited: Duration = answered.unwrap_or_else(|| panic!("the look that could not settle a claim answers when it runs out"));
    assert!(waited <= A_LEASE_A_CONTROL_WAITS_OUT, "the answer is no later than the claim's end: {waited:?}");
    assert_eq!(asked.load(Ordering::SeqCst), 1, "the first look ran the item once");
    assert_eq!(names_in(&world, CLAIMED).len(), 1, "the claim the settling step could not move is kept");
    std::thread::sleep(waited + A_MARGIN_PAST_THE_ANSWER);
    let _again: Option<Duration> = runtime.block_on(drain_the_slow_work_awaiting(
        world.common.as_path(),
        &handlers,
        HOUSEKEEPING_OF_ONE_ROUND,
        A_LEASE_A_CONTROL_WAITS_OUT,
    ));
    assert_eq!(asked.load(Ordering::SeqCst), 2, "the look at the claim's end claims the record again and runs it");
    assert_eq!(tries_of_the_claimed_record(&world), 1, "the recovery at the claim's end queued it with one more try");
    remove(world.root.as_path());
}

/// What:
///  The look at a bare repository's common records answers the end of a claim it keeps,
/// as the look at the directory itself does.
/// Why:
///   A bare repository has no instance whose own Git directory is the common one,
/// so the records its worktrees left there are drained by the look the lane gives the common directory
/// (`drain_the_slow_work_awaiting`),
/// which is a drain of its own.
/// A claim that look keeps is a claim nothing else would look at again,
/// so its answer is what wakes the lane.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [world] = worldWithARemoval(...); makeBare(world.common); const wait = await laneLook(linkedGitDir(world))
/// expect(wait).toBeLessThanOrEqual(lease); expect(asked).toBe(1)
/// ```
#[test]
fn the_lanes_look_at_a_bare_repositorys_common_records_answers_the_end_of_a_claim_it_keeps() {
    let (world, _id): (World, String) = world_with_a_removal("slow-lane-common-kept-claim", 63);
    std::fs::write(world.common.join("config"), "[core]\n\tbare = true\n").expect("the config of a bare repository");
    let linked: PathBuf = world.common.join("worktrees").join("w");
    std::fs::create_dir_all(linked.as_path()).expect("a linked worktree's Git directory");
    std::fs::write(linked.join("commondir"), "../..\n").expect("the commondir the linked Git directory names");
    let (handlers, asked): (Said, Arc<AtomicU32>) = said(|_attempt| { return Verdict::Retry(String::from("the volume is busy")); });
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    let answered: Option<Duration> = runtime.block_on(drain_the_slow_work_awaiting(
        linked.as_path(),
        &handlers,
        HOUSEKEEPING_OF_ONE_ROUND,
        A_LEASE_A_CONTROL_WAITS_OUT,
    ));
    let waited: Duration = answered.unwrap_or_else(|| panic!("the look at a bare repository's common records answers when its claim runs out"));
    assert!(waited <= A_LEASE_A_CONTROL_WAITS_OUT, "the answer is no later than the claim's end: {waited:?}");
    assert_eq!(asked.load(Ordering::SeqCst), 1, "the common directory's record ran");
    assert_eq!(names_in(&world, CLAIMED).len(), 1, "the retryable answer keeps the claim");
    remove(world.root.as_path());
}

/// What:
///  The round's drain answers the end of a claim it keeps,
/// as the look of the lane does,
/// so the loop that serves the pushes is left with a wake for it.
/// Why:
///   A control of the rule, not of a defect:
/// the round already read the claim ends itself after its drain,
/// and this holds that answer still when the drain became the one place that makes it,
/// so the loop that serves the pushes cannot lose the wake for a claim the drain kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [world] = worldWithARemoval(...); blockSettling(world); const drained = await roundDrain(world); expect(drained.wait).toBeLessThanOrEqual(lease)
/// ```
#[test]
fn the_rounds_drain_answers_the_end_of_a_claim_it_keeps() {
    let (world, _id): (World, String) = world_with_a_removal("slow-lane-the-round-keeps-a-claim", 64);
    let done: PathBuf = state_directory(world.work.as_path(), DONE);
    std::fs::remove_dir(done.as_path()).expect("the empty done directory");
    std::fs::write(done.as_path(), b"in the way").expect("the blocker");
    let (handlers, asked): (Said, Arc<AtomicU32>) = said(|_attempt| { return Verdict::Done; });
    let drained = crate::merge_second_pass::tests::await_it(crate::service_work::drain_planned_with_awaiting(
        world.common.as_path(),
        &handlers,
        &ClaimTime::read(),
        A_LEASE_A_CONTROL_WAITS_OUT,
        Renewal::Timed,
        &Retrying::none(),
        &SERVICE_KINDS,
    ));
    let waited: Duration = drained.wait.unwrap_or_else(|| panic!("the round answers when to look again for the claim it keeps"));
    assert!(waited <= A_LEASE_A_CONTROL_WAITS_OUT, "the answer is no later than the claim's end: {waited:?}");
    assert_eq!(asked.load(Ordering::SeqCst), 1, "the round ran the item once");
    assert_eq!(names_in(&world, CLAIMED).len(), 1, "the claim the settling step could not move is kept");
    remove(world.root.as_path());
}
