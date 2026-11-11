//! Controls for the independent items of one work directory run beside each other:
//! two worktrees' slow items overlap in time,
//! two branches of one repository are pushed at once,
//! and a person's push is not held by a queue of slow work.
//! Why:
//!  One repository's work directory holds the records of every worktree of that repository,
//! and the measured push of `main` at `08:54:53` on `2026-11-11` went out when a queue of 22 records of one work
//! directory drained
//! (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
//! Two items of one worktree may stand in each other's way;
//! two items of different worktrees share nothing but the disk,
//! and two branches share only the repository's object store,
//! so those run beside each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(overlapped('trash', twoWorktrees)).toBe(true); expect(overlapped('push', twoBranches)).toBe(true);
//! ```

use crate::claim_time::ClaimTime;
use crate::parallel_budget::HostResources;
use crate::push_retry::{PLAN_DIRECTORY, PushRetries};
use crate::service_claim_watch::ClaimWatch;
use crate::service_round::{Assignment, RoundDrain, drain_directories};
use crate::service_server_work_loop_awaiting::work_loop_awaiting;
use crate::service_work::{GitHandlers, Verdict, Wake, work_directory};
use crate::service_work_handlers_replanning_and_acceptance::WorkHandlers;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository, wait_until};
use crate::work_item::{DONE, WorkItem, WorkKind, enqueue, state_directory};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// The branch a push record of these controls names,
/// and the second branch two pushes are watched on.
const BRANCH: &str = "refs/heads/main";

const OTHER_BRANCH: &str = "refs/heads/topic";

/// How long one item of these controls is inside its work,
/// so an item that starts while it runs is measured beside it.
const HANDOVER: Duration = Duration::from_millis(300);

/// A host large enough that neither the CPUs nor the memory narrow a batch.
const LARGE_HOST: HostResources = HostResources { parallelism: 8, memory_available: None };

/// What:
///  A batch of one kind run as the drain runs it,
/// which counts the most items that were inside the handler at one moment.
/// Why:
///  Two items are beside each other exactly when both are inside their work at one moment.
/// The handler stays inside its work for `HANDOVER`,
/// so an item that starts after another ended sees nothing beside it,
/// and an item that starts while another is inside measures two.
/// A clock would have to be compared against the host's load;
/// a count of the items inside says the one thing the control is about.
struct SideBySide {
    /// How many items are inside the handler now.
    inside: AtomicUsize,
    /// The most that were inside at one moment.
    most: AtomicUsize,
}

impl SideBySide {
    /// A batch whose items record how many of them met.
    fn of() -> SideBySide {
        return SideBySide { inside: AtomicUsize::new(0), most: AtomicUsize::new(0) };
    }

    /// What:
    ///  Be inside the work of one item for `HANDOVER`,
    /// and record how many items were inside with it.
    /// Why:
    ///   The pause is what an item of this kind really does: a trash is emptied and a push is sent,
    /// and both take longer than the trip between two items of a batch.
    /// It is short enough that a batch run beside each other still ends at once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const now = inside.fetchAdd(1) + 1; most.fetchMax(now); await sleep(HANDOVER); inside.fetchSub(1);
    /// ```
    async fn enter(&self) {
        let now: usize = self.inside.fetch_add(1, Ordering::SeqCst) + 1;
        self.most.fetch_max(now, Ordering::SeqCst);
        tokio::time::sleep(HANDOVER).await;
        self.inside.fetch_sub(1, Ordering::SeqCst);
    }
}

/// What the batch's handler answers with,
/// which is done.
impl WorkHandlers for SideBySide {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        return Verdict::Done;
    }

    fn push_awaiting<'a>(&'a self, _items: &'a [WorkItem]) -> Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            self.enter().await;
            return Verdict::Done;
        });
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn trash(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn trash_awaiting<'a>(&'a self, _item: &'a WorkItem) -> Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            self.enter().await;
            return Verdict::Done;
        });
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// A repository directory standing in for a Git directory,
/// with one worktree directory per name given.
struct World {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The Git directory the records name.
    common: PathBuf,
    /// The work directory the service drains.
    work: PathBuf,
}

impl World {
    /// The fixtures of this control.
    fn cleaned(self) {
        remove(self.root.as_path());
    }
}

/// What:
///  A world whose records name one worktree of their own each,
/// so the batch these are written into has as many worktrees as names.
/// Why:
///   The worktree an item names is what two items may stand in each other's way over,
/// so a batch of items of one worktree and a batch of items of different worktrees are the two cases a control
/// tells apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world('name', 2); for (const at of [0, 1]) enqueue(world.work, trashItem(world.worktreeOf(at)));
/// ```
fn world(name: &str, worktrees: usize) -> World {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(&common).expect("the Git directory");
    let work: PathBuf = work_directory(common.as_path());
    for at in 0..worktrees {
        std::fs::create_dir_all(root.join(format!("worktree-{at}"))).expect("the worktree directory");
    }
    return World { root, common, work };
}

/// The record of `kind` for worktree `at` of `world`,
/// on the branch given.
fn record(world: &World, kind: WorkKind, at: usize, branch: &str, oid: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: world.root.join(format!("worktree-{at}")),
        git_dir: world.common.clone(),
        branch_ref: branch.to_string(),
        oid: oid.to_string(),
        parent: String::new(),
        created: String::from("2026-11-11T08:50:59.000Z"),
        tries: 0,
    };
}

/// How many records of `world` settled in `state`.
fn settled(world: &World, state: &str) -> usize {
    return state_directory(world.work.as_path(), state).read_dir().map_or(0, |listing| { return listing.flatten().count(); });
}

/// What:
///  Two worktrees' trash records are run beside each other,
/// so a queue of records of one repository drains in as many steps as its widest worktree's queue
///  and not in as many as its records.
/// Why:
///   The queue of 22 records of one work directory that held the push of `main` back on `2026-11-11`
/// named prewarms, relinks and trashes of many worktrees of one repository
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// Two of them share nothing: each trash is emptied below its own worktree's handle,
/// each is proved to be the directory its own snapshot made,
/// and each removes its own trash parent,
/// so the one-at-a-time order was the drain's shape and not a limit of the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const both = SideBySide.of(); await lane(world.common, both); expect(both.most).toBe(2)
/// ```
#[test]
fn two_worktrees_trash_records_run_beside_each_other() {
    let world: World = world("side-by-side-worktrees", 2);
    enqueue(world.work.as_path(), &record(&world, WorkKind::Trash, 0, "refs/cli-git/trash", "1111111111111111111111111111111111111111")).expect("the first trash");
    enqueue(world.work.as_path(), &record(&world, WorkKind::Trash, 1, "refs/cli-git/trash", "2222222222222222222222222222222222222222")).expect("the second trash");
    let both: SideBySide = SideBySide::of();
    // The trashes are the slow lane's kinds, which the round that answers a push never claims.
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    runtime.block_on(crate::service_work_slow_lane::drain_the_slow_work_awaiting(world.common.as_path(), &both));
    assert_eq!(settled(&world, DONE), 2, "both trashes were emptied");
    assert_eq!(both.most.load(Ordering::SeqCst), 2, "the two worktrees' trashes were inside the handler at the same moment");
    world.cleaned();
}

/// What:
///  One worktree's two trash records are run one after the other,
/// so nothing of one worktree empties beside another of its own.
/// Why:
///   The emptying removes the trash parent when nothing else is in it
/// (`worktree_trash_dispose.rs`),
/// so two trashes of one worktree would race over that directory,
/// and a prewarm fills the Cargo build directories a relink of the same worktree writes its package state beside.
/// The order each worktree's records keep is the order they were claimed in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const both = SideBySide.of(); await lane(world.common, both); expect(both.most).toBe(1)
/// ```
#[test]
fn one_worktrees_two_trash_records_run_one_after_the_other() {
    let world: World = world("side-by-side-one-worktree", 1);
    enqueue(world.work.as_path(), &record(&world, WorkKind::Trash, 0, "refs/cli-git/trash", "3333333333333333333333333333333333333333")).expect("the first trash");
    enqueue(world.work.as_path(), &record(&world, WorkKind::Trash, 0, "refs/cli-git/trash", "4444444444444444444444444444444444444444")).expect("the second trash");
    let both: SideBySide = SideBySide::of();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    runtime.block_on(crate::service_work_slow_lane::drain_the_slow_work_awaiting(world.common.as_path(), &both));
    assert_eq!(settled(&world, DONE), 2, "both trashes of the one worktree were emptied");
    assert_eq!(both.most.load(Ordering::SeqCst), 1, "the one worktree's trashes never ran at the same moment");
    world.cleaned();
}

/// What:
///  Two branches of one repository are pushed at once,
/// so one branch's push does not wait for another's.
/// Why:
///   Each push names its own branch,
/// its own objects are in the repository's store already,
/// and only the reference it updates locally and the branch it sends are its own,
/// so the order the records were written in was the drain's shape and not a limit of the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const both = SideBySide.of(); await round(world.common, both); expect(both.most).toBe(2)
/// ```
#[test]
fn two_branches_of_one_repository_are_pushed_at_once() {
    let world: World = world("side-by-side-branches", 1);
    enqueue(world.work.as_path(), &record(&world, WorkKind::Push, 0, BRANCH, "5555555555555555555555555555555555555555")).expect("the first branch's push");
    enqueue(world.work.as_path(), &record(&world, WorkKind::Push, 0, OTHER_BRANCH, "6666666666666666666666666666666666666666")).expect("the second branch's push");
    let both: Arc<SideBySide> = Arc::new(SideBySide::of());
    let assignments: Vec<Assignment> = vec![Assignment::plain(world.common.clone())];
    let _waits: Vec<(PathBuf, Duration)> =
        drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::clone(&both), now: &ClaimTime::read(), host: &LARGE_HOST }).waits;
    assert_eq!(settled(&world, DONE), 2, "both branches' pushes ran");
    assert_eq!(both.most.load(Ordering::SeqCst), 2, "the two branches were inside the push handler at the same moment");
    world.cleaned();
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
    return Stage { root, repo, remote, landed };
}

/// The branch tip the remote holds,
/// read from the bare remote's own ref file.
fn remote_tip(stage: &Stage) -> String {
    return std::fs::read_to_string(stage.remote.join("refs").join("heads").join("main")).map(|text| { return text.trim().to_owned(); }).unwrap_or_default();
}

/// Write a file,
/// making its directories.
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full: PathBuf = root.join(path);
    std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
    std::fs::write(full, bytes).expect("file");
}

/// Start the after-commit work as a task of the process's runtime,
/// as the service does,
/// and give the way to wake it and the task itself.
fn start_the_loop(staged: &Stage) -> (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::task::JoinHandle<()>) {
    let plans: PathBuf = staged.root.join("journal").join(PLAN_DIRECTORY);
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let (control, taken): (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::sync::mpsc::UnboundedReceiver<Wake>) = tokio::sync::mpsc::unbounded_channel::<Wake>();
    let running: tokio::task::JoinHandle<()> = crate::lazy_runtime::spawn_on_runtime(async move {
        let mut receiver = taken;
        work_loop_awaiting(&mut receiver, &handlers, &busy, PushRetries::restored(Duration::from_millis(300), plans.as_path()), ClaimWatch::in_memory()).await;
    })
    .expect("the runtime starts the after-commit work");
    return (control, running);
}

/// The worktrees whose trashes one measurement queues,
/// and the entries each trash holds.
/// Why:
///   The measured queue of `2026-11-11` held 22 records of one work directory
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`),
/// so four worktrees of one repository keep that shape at a size a measurement can run twice on a loaded host.
const MEASURED_WORKTREES: usize = 4;

const MEASURED_ENTRIES: usize = 500;

/// How many of the queue's trashes are already claimed and running when the push is written.
/// Why:
///   The measured case is a push that arrives while slow work holds the service,
/// so the queue must be in flight before the push exists;
/// half of the queue is a state a drain reaches quickly and that cannot be confused with an empty one.
const INTO_THE_QUEUE: usize = MEASURED_WORKTREES / 2;

/// What:
///  Plant a trash below the worktree `worktree` and answer the manifest a leave-out would have written for it.
/// Why:
///   The service's handler trusts nothing in a record beyond what it can prove:
/// the worktree's `.git` points into the record's Git directory,
/// every directory down to the trash is a real one,
/// and the trash and each entry have the recorded identities.
/// A trash planted any other way is refused and measures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const manifest = plantTrash(worktree, name, count);
/// ```
fn plant_trash(worktree: &Path, name: &str, count: usize) -> crate::worktree_trash_manifest::TrashManifest {
    for index in 0..count {
        put(worktree, format!("{}/{name}/{}/file", crate::worktree_trash::TRASH_PARENT, crate::worktree_trash_manifest::entry_name(index)).as_str(), b"left out\n");
    }
    let root: std::fs::File = crate::fs_beneath::open_root(worktree).expect("the worktree");
    let names: Vec<std::ffi::OsString> = crate::fs_beneath::plain_components(Path::new(crate::worktree_trash::TRASH_PARENT)).expect("plain parent");
    let parent: std::fs::File = crate::fs_beneath::open_directory_beneath(&root, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = crate::fs_beneath::open_directory_at(&parent, std::ffi::OsStr::new(name)).expect("the trash");
    let mut entries: Vec<crate::fs_beneath::ObjectId> = Vec::with_capacity(count);
    for index in 0..count {
        entries.push(crate::fs_beneath::identity_at(&trash, std::ffi::OsStr::new(crate::worktree_trash_manifest::entry_name(index).as_str())).expect("entry identity"));
    }
    return crate::worktree_trash_manifest::TrashManifest { name: String::from(name), trash: crate::fs_beneath::identity_of(&trash).expect("trash identity"), entries };
}

/// The names of the trashes a measurement plants,
/// which are what a work item carries as its object name.
fn trash_name_of(index: usize) -> String {
    return format!("{:0>40}", format!("{index:x}"));
}

/// What:
///  Measure what one queue of one repository's slow work costs and how long a person's push waits behind it.
/// Why:
///   The owner asked "Shouldn't the service run anything that can be ran in parallel in parallel?"
/// and the measured cost of the answer is the wall time to drain a queue of real trashes of real worktrees of one
/// repository,
/// with a real push written behind them and a real bare remote to land it on.
/// The figures this prints are the ones written beside `SLOW_ITEMS_CEILING`
/// (`service_work_items_side_by_side.rs`) and in the handover,
/// so this is a measurement for the record and not a control: it asserts only that the queue drained and the push landed,
/// which every arm must.
/// Run it alone, as the service's own caps and its one process are what the figure is about:
/// `mise run //package/git-policy/cli:native:test:host -- --run-ignored side_by_side_measure --success-output=final`
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drained = await measureQueue(8, 2000); print(`queue_ms=${drained.ms} push_wait_ms=${drained.pushWaitMs}`)
/// ```
#[ignore = "a measurement for the record, not a control"]
#[test]
fn side_by_side_measure_a_queue_of_one_repositorys_slow_work_and_the_push_behind_it() {
    // The service's own bounds, which are what its queue drains under (`service/service_main.rs`).
    crate::lazy_runtime::cap_workers(crate::service_main::SERVICE_WORKER_CAP);
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(crate::service_main::SERVICE_WORKER_CAP);
    let staged: Stage = stage("side-by-side-measure");
    let git_dir: PathBuf = staged.repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    let mut worktrees: Vec<PathBuf> = Vec::new();
    for index in 0..MEASURED_WORKTREES {
        let name: String = format!("measured-{index}");
        // The worktree Git makes sits beside the repository it was added from, so that is where the trash is planted.
        git(staged.repo.as_path(), &["worktree", "add", "--quiet", "--detach", name.as_str()]);
        let worktree: PathBuf = staged.repo.join(name.as_str());
        let manifest: crate::worktree_trash_manifest::TrashManifest = plant_trash(worktree.as_path(), trash_name_of(index).as_str(), MEASURED_ENTRIES);
        let item: WorkItem = WorkItem {
            kind: WorkKind::Trash,
            repository: worktree.clone(),
            git_dir: git_dir.clone(),
            branch_ref: String::from(crate::worktree_trash_dispose::TRASH_REFERENCE),
            oid: manifest.name.clone(),
            parent: manifest.encode(),
            created: String::from("2026-11-11T08:50:59.000Z"),
            tries: 0,
        };
        enqueue(work.as_path(), &item).expect("the trash is written");
        worktrees.push(worktree);
    }
    let push: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: staged.repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from(BRANCH),
        oid: staged.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-11T08:50:59.000Z"),
        tries: 0,
    };
    let (wake, running): (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::task::JoinHandle<()>) = start_the_loop(&staged);
    let began: Instant = Instant::now();
    wake.send((git_dir.clone(), None)).expect("the queue's wake");
    // The push is written once the queue is in flight, which is the case the coordinator measured: slow work claimed and
    //  running, then a landing that must reach the remote anyway.
    assert!(
        wait_until(&|| {
            return claimed(&held_world(&staged, work.as_path()), WorkKind::Trash) >= INTO_THE_QUEUE;
        }),
        "{INTO_THE_QUEUE} trashes of the queue are claimed and running"
    );
    let asked: Instant = Instant::now();
    enqueue(work.as_path(), &push).expect("the push is written");
    wake.send((git_dir.clone(), None)).expect("the push's wake");
    assert!(wait_until(&|| { return remote_tip(&staged) == staged.landed; }), "the push landed");
    let push_wait: Duration = asked.elapsed();
    let claimed_then: usize = claimed(&held_world(&staged, work.as_path()), WorkKind::Trash);
    assert!(wait_until(&|| { return settled(&held_world(&staged, work.as_path()), DONE) >= MEASURED_WORKTREES; }), "every trash of the queue was emptied");
    let drained: Duration = began.elapsed();
    println!(
        "measure queue_worktrees={MEASURED_WORKTREES} entries_each={MEASURED_ENTRIES} queue_ms={} push_wait_ms={} push_written_after_ms={} slow_records_claimed_when_the_push_landed={claimed_then}",
        drained.as_millis(),
        push_wait.as_millis(),
        asked.duration_since(began).as_millis()
    );
    drop(wake);
    assert!(wait_until(&|| { return running.is_finished(); }), "the after-commit work ends when its queue is closed");
    for worktree in &worktrees {
        assert!(std::fs::symlink_metadata(worktree.join(crate::worktree_trash::TRASH_PARENT)).is_err(), "the trash of {} was emptied", worktree.display());
    }
    remove(staged.root.as_path());
}

/// The world whose work directory is `work`,
/// for the counters of the measurement and of the controls.
fn held_world(staged: &Stage, work: &Path) -> World {
    return World { root: staged.root.clone(), common: staged.repo.join(".git"), work: work.to_path_buf() };
}

/// How long the stand-in slow work of one item stays inside its hold.
/// Why:
///   The alpha.10 service the coordinator measured on `2026-11-11T10:38:00-05:00` held two reviews and four trashes
/// claimed while a fresh commit's push sat pending for more than nine minutes,
/// so a hold that outlasts any push by minutes is what the control needs.
const HOLD: Duration = Duration::from_secs(20);

/// The longest a push may take while slow work of another worktree runs.
/// Why:
///   A push to a bare remote in this fixture takes about 0.1 s,
/// and the control asks for the order of magnitude, not for a latency budget:
/// ten seconds is two orders of magnitude under the hold and far above what the push itself takes.
const PUSH_BOUND: Duration = Duration::from_secs(10);

/// What:
///  Handlers whose review and trash work is held open for `HOLD`,
/// and whose push is the real one.
/// Why:
///   The service the coordinator measured held a push behind a review and a trash that had nothing to do with it,
/// and no fault hook of this release can hold a review open
/// (`worktree_trash_dispose_awaiting.rs` is the one that can, and it holds a trash).
/// The slow kinds answer the same four verdicts whatever runs them,
/// so a stand-in that holds them open says what a slow review and a slow trash say about the push beside them,
/// and the push itself is real Git against a real bare remote.
struct StandIns {
    /// The real handlers, which run the push.
    real: GitHandlers,
    /// How many slow items are inside their hold now.
    inside: AtomicUsize,
}

impl StandIns {
    /// The real push and a slow review and trash.
    fn new() -> StandIns {
        return StandIns { real: GitHandlers::new(git_context()), inside: AtomicUsize::new(0) };
    }

    /// What:
    ///  Be inside one slow item's work for `HOLD`.
    /// Why:
    ///   A review of a landed commit and a trash of a removed worktree take minutes on this repository
    /// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`),
    /// and what matters here is only that they are still running when the push is asked for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// inside.fetchAdd(1); await sleep(HOLD); inside.fetchSub(1);
    /// ```
    async fn hold(&self) {
        self.inside.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(HOLD).await;
        self.inside.fetch_sub(1, Ordering::SeqCst);
    }
}

impl WorkHandlers for StandIns {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        return self.real.push(items);
    }

    fn push_awaiting<'a>(&'a self, items: &'a [WorkItem]) -> Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return self.real.push_awaiting(items);
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review_awaiting<'a>(&'a self, _item: &'a WorkItem) -> Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            self.hold().await;
            return Verdict::Done;
        });
    }

    fn trash(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn trash_awaiting<'a>(&'a self, _item: &'a WorkItem) -> Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            self.hold().await;
            return Verdict::Done;
        });
    }
}

/// The record of `kind` for `staged`, naming the commit,
/// which the service accepts whatever runs it.
fn stand_in_record(staged: &Stage, kind: WorkKind, branch: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: staged.repo.clone(),
        git_dir: staged.repo.join(".git"),
        branch_ref: branch.to_string(),
        oid: staged.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-11T10:29:00.000Z"),
        tries: 0,
    };
}

/// What:
///  A push of a fresh commit goes out while a review and a trash of the same repository are claimed and running,
/// so no person's backup waits for work that has nothing to do with it.
/// Why:
///   This is the state the coordinator measured on the installed `0.6.1-alpha.10` at `2026-11-11T10:38:00-05:00`:
/// six slow records were claimed and running while
/// `push-a52a4adaa8…` sat pending from `10:29` with a maintenance, a review and eight prewarms beside it,
/// and `main` had to be pushed by hand.
/// The two claim lists are what answers it: the round that answers the push's wake claims `PUSH_KINDS`
/// alone, so what the lane holds is not in its way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// void lane(common); await inside(1); const began = now(); await round(common); expect(remoteTip()).toBe(landed); expect(now() - began).toBeLessThan(PUSH_BOUND); expect(inside(1)).toBe(true)
/// ```
#[test]
fn a_push_goes_out_while_a_review_and_a_trash_are_held_claimed() {
    let staged: Stage = stage("side-by-side-push-beside-held-work");
    let git_dir: PathBuf = staged.repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    let trash: WorkItem = stand_in_record(&staged, WorkKind::Trash, "refs/cli-git/trash");
    let review: WorkItem = stand_in_record(&staged, WorkKind::Review, "refs/cli-git/review");
    let maintenance: WorkItem = stand_in_record(&staged, WorkKind::Maintenance, "refs/cli-git/upkeep");
    let push: WorkItem = stand_in_record(&staged, WorkKind::Push, BRANCH);
    for item in [&trash, &review, &maintenance, &push] {
        enqueue(work.as_path(), item).expect("the record is written");
    }
    let stand_ins: Arc<StandIns> = Arc::new(StandIns::new());
    let lane_handlers: Arc<StandIns> = Arc::clone(&stand_ins);
    let lane = async move { crate::service_work_slow_lane::drain_the_slow_work_awaiting(git_dir.as_path(), lane_handlers.as_ref()).await };
    let running: tokio::task::JoinHandle<()> = crate::lazy_runtime::spawn_on_runtime(lane).expect("the runtime starts the lane");
    assert!(wait_until(&|| { return stand_ins.inside.load(Ordering::SeqCst) > 0; }), "a slow item of the lane is running");
    // The claims the lane holds are its own, and the push's round takes only the pushes.
    assert!(wait_until(&|| { return claimed(&held_world(&staged, work.as_path()), WorkKind::Review) > 0; }), "the review waits claimed, as the measured service held it");
    let asked: Instant = Instant::now();
    let assignments: Vec<Assignment> = vec![Assignment::plain(staged.repo.join(".git"))];
    let _waits: Vec<(PathBuf, Duration)> =
        drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::clone(&stand_ins), now: &ClaimTime::read(), host: &LARGE_HOST }).waits;
    let waited: Duration = asked.elapsed();
    assert_eq!(remote_tip(&staged), staged.landed, "the push landed while the slow work was still held");
    assert!(waited < PUSH_BOUND, "the push waited {waited:?} behind slow work of the same repository");
    assert!(stand_ins.inside.load(Ordering::SeqCst) > 0, "the slow work was still running when the push landed");
    println!("measure push_beside_held_slow_work push_wait_ms={} hold_s={} slow_items_inside_when_it_landed={}", waited.as_millis(), HOLD.as_secs(), stand_ins.inside.load(Ordering::SeqCst));
    assert!(wait_until(&|| { return running.is_finished(); }), "the lane ends when its holds do");
    remove(staged.root.as_path());
}

/// How many records of `kind` are claimed in a work directory.
fn claimed(world: &World, kind: WorkKind) -> usize {
    let listing = state_directory(world.work.as_path(), crate::work_item::CLAIMED);
    return listing.read_dir().map_or(0, |entries| {
        return entries.flatten().filter(|entry| { return entry.file_name().to_string_lossy().starts_with(kind.name()); }).count();
    });
}
