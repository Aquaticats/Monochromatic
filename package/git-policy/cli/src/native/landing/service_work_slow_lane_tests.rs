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
use crate::push_retry::{PLAN_DIRECTORY, PushRetries};
use crate::service_claim_watch::ClaimWatch;
use crate::service_round::{Assignment, RoundDrain, drain_directories};
use crate::service_server_work_loop_awaiting::work_loop_awaiting;
use crate::service_work::{GitHandlers, Verdict, work_directory};
use crate::service_work_handlers_replanning_and_acceptance::{PUSH_KINDS, SERVICE_KINDS, WorkHandlers};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository, wait_until};
use crate::work_item::{DONE, PENDING, WorkItem, WorkKind, enqueue, state_directory};
use crate::worktree_trash_dispose_awaiting::INSIDE_A_SLOW_STEP;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
        .map_or_else(|_no_directory| Vec::new(), |listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); });
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
    runtime.block_on(drain_the_slow_work_awaiting(world.common.as_path(), &counted));
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
