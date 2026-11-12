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
    ///   The pause is what an item of this kind really does:
    /// a trash is emptied and a push is sent,
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
/// named prewarms,
/// relinks and trashes of many worktrees of one repository
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// Two of them share nothing:
/// each trash is emptied below its own worktree's handle,
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
    runtime.block_on(crate::service_work_slow_lane::drain_the_slow_work_awaiting(
        world.common.as_path(),
        &both,
        crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
        crate::service_work::CLAIM_LEASE,
    ));
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
    runtime.block_on(crate::service_work_slow_lane::drain_the_slow_work_awaiting(
        world.common.as_path(),
        &both,
        crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
        crate::service_work::CLAIM_LEASE,
    ));
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
/// so eight worktrees of one repository keep that shape at a size a measurement can run twice on a loaded host.
const MEASURED_WORKTREES: usize = 8;

const MEASURED_ENTRIES: usize = 8_000;

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

/// The name of trash `index` of measurement round `round`,
/// which is what lets a round count only the trashes it planted.
fn trash_name_of_round(round: usize, index: usize) -> String {
    return format!("{round}{:0>39}", format!("{index:x}"));
}

/// What:
///  Measure what draining one repository's queue of slow work costs,
/// and what a person's push pays for it,
///  asking both widths of the same binary on the same host.
/// Why:
///   The owner asked "Shouldn't the service run anything that can be ran in parallel in parallel?"
/// and the answer's worth is a wall time:
/// the queue is eight worktrees' trashes of one repository,
/// each planted and proved the way a leave-out writes one,
/// drained by the service's own lane under the service's own caps,
/// once one item at a time and once four at a time,
/// alternating,
/// so the spread of one width from round to round is what the difference is read against.
/// A push is written behind a queue that is already claimed,
/// which is the state the coordinator measured on the installed `0.6.1-alpha.10`
/// (`2026-11-11T10:38:00-05:00`:
/// two reviews and four trashes claimed,
/// a push pending since 10:29),
/// and the service's own line reports how long the push itself took,
/// so a wait that is the push's own cost is told from one behind the queue.
/// This is a measurement for the record and not a control:
/// it asserts only what every arm must give,
/// that the queue drained and the push landed.
/// Run it alone,
/// as the service's own caps and its one process are what the figure is about:
/// `mise run //package/git-policy/cli:native:test:host -- --run-ignored side_by_side_measure --success-output=final`
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const width of [1, 4, 4, 1]) print(`width=${width} queue_ms=${await drain(width)}`)
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
        worktrees.push(staged.repo.join(name.as_str()));
    }
    let mut round: usize = 0;
    for width in MEASURED_WIDTHS {
        round += 1;
        for (index, worktree) in worktrees.iter().enumerate() {
            let name: String = trash_name_of_round(round, index);
            plant_a_big_trash_item(&staged, worktree, name.as_str(), MEASURED_ENTRIES, work.as_path());
        }
        crate::service_work_items_side_by_side::hold_the_width_to(width);
        let began: Instant = Instant::now();
        let lane_dir: PathBuf = git_dir.clone();
        let running: tokio::task::JoinHandle<Option<Duration>> = crate::lazy_runtime::spawn_on_runtime(async move {
            let handlers: GitHandlers = GitHandlers::new(git_context());
            return crate::service_work_slow_lane::drain_the_slow_work_awaiting(
                lane_dir.as_path(),
                &handlers,
                crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
                crate::service_work::CLAIM_LEASE,
            )
            .await;
        })
        .expect("the runtime starts the lane");
        assert!(emptied_by(work.as_path(), trash_name_of_round(round, 0).as_str(), MEASURED_WORKTREES, MEASURED_DRAIN_CEILING), "round {round} of width {width} emptied every trash of its queue");
        let drained: Duration = began.elapsed();
        assert!(wait_until(&|| { return running.is_finished(); }), "the lane ended");
        println!("measure round={round} width={width} queue_worktrees={MEASURED_WORKTREES} entries_each={MEASURED_ENTRIES} queue_ms={}", drained.as_millis());
    }
    crate::service_work_items_side_by_side::take_the_hosts_width();
    remove(staged.root.as_path());
}

/// What:
///  Measure what a person's push pays while a queue of slow work of the same repository is already claimed and running.
/// Why:
///   This is the state the coordinator measured on the installed `0.6.1-alpha.10` at `2026-11-11T10:38:00-05:00`:
/// two reviews and four trashes claimed since 10:24,
/// and a fresh commit's push still pending from 10:29,
/// so `main` was pushed by hand.
/// The push is written only once half the queue is claimed,
/// so an arm cannot land it before any slow work began,
/// and the service's own line reports how much of that wait was the push itself.
/// A measurement for the record,
/// not a control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wake(queue); await claimed(4); const asked = now(); enqueue(push); wake(); await remoteTip(); print(now() - asked)
/// ```
#[ignore = "a measurement for the record, not a control"]
#[test]
fn side_by_side_measure_the_push_behind_a_queue_that_is_already_running() {
    crate::lazy_runtime::cap_workers(crate::service_main::SERVICE_WORKER_CAP);
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(crate::service_main::SERVICE_WORKER_CAP);
    let staged: Stage = stage("side-by-side-measure-push");
    let git_dir: PathBuf = staged.repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    for index in 0..MEASURED_WORKTREES {
        let name: String = format!("measured-{index}");
        git(staged.repo.as_path(), &["worktree", "add", "--quiet", "--detach", name.as_str()]);
        plant_a_trash_item(&staged, staged.repo.join(name.as_str()).as_path(), trash_name_of(index).as_str(), MEASURED_PUSH_ENTRIES, work.as_path());
    }
    let (wake, running): (tokio::sync::mpsc::UnboundedSender<Wake>, tokio::task::JoinHandle<()>) = start_the_loop(&staged);
    let began: Instant = Instant::now();
    wake.send((git_dir.clone(), None)).expect("the queue's wake");
    assert!(
        wait_until(&|| { return claimed(&held_world(&staged, work.as_path()), WorkKind::Trash) >= MEASURED_PUSH_CLAIMED; }),
        "{MEASURED_PUSH_CLAIMED} trashes of the queue are claimed and running"
    );
    let asked: Instant = Instant::now();
    let push: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: staged.repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from(BRANCH),
        oid: staged.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-11T10:29:00.000Z"),
        tries: 0,
    };
    enqueue(work.as_path(), &push).expect("the push is written");
    wake.send((git_dir.clone(), None)).expect("the push's wake");
    assert!(wait_until(&|| { return remote_tip(&staged) == staged.landed; }), "the push landed");
    let push_wait: Duration = asked.elapsed();
    let claimed_then: usize = claimed(&held_world(&staged, work.as_path()), WorkKind::Trash);
    let queue_then: Duration = began.elapsed();
    println!(
        "measure push_worktrees={MEASURED_WORKTREES} entries_each={MEASURED_PUSH_ENTRIES} push_wait_ms={} queue_running_ms_when_it_landed={} slow_records_claimed_when_it_landed={claimed_then}",
        push_wait.as_millis(),
        queue_then.as_millis()
    );
    assert!(emptied_by(work.as_path(), trash_name_of(0).as_str(), MEASURED_WORKTREES, MEASURED_DRAIN_CEILING), "every trash of the queue was emptied");
    // The same push with no queue beside it, which is what separates the queue's cost from what this host costs a push
    //  at the load it has.
    std::fs::write(staged.repo.join("b.txt"), b"after the queue\n").expect("file");
    git(staged.repo.as_path(), &["add", "--", "b.txt"]);
    let committed = crate::test_support::git_output(staged.repo.as_path(), &["commit", "--quiet", "--no-verify", "--message=after the queue"]);
    assert!(committed.status.success(), "git commit after the queue failed: {:?} {}", committed.status.code(), String::from_utf8_lossy(&committed.stderr));
    let later: String = git_text(staged.repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let alone: WorkItem = WorkItem { oid: later, ..push };
    let alone_asked: Instant = Instant::now();
    enqueue(work.as_path(), &alone).expect("the second push is written");
    wake.send((git_dir, None)).expect("the second push's wake");
    assert!(wait_until(&|| { return remote_tip(&staged) == alone.oid; }), "the push with no queue landed");
    println!("measure push_alone_ms={}", alone_asked.elapsed().as_millis());
    drop(wake);
    assert!(wait_until(&|| { return running.is_finished(); }), "the after-commit work ends when its queue is closed");
    remove(staged.root.as_path());
}

/// How many trashes the push measurement queues,
/// how many entries each holds,
/// and how many of them are claimed before the push is written.
/// Why:
///  The queue is in flight when the push arrives,
/// which is what makes the wait a wait behind slow work;
/// the entries are few because what the push's step competes for is a place of the pool,
/// not the entries' bytes.
const MEASURED_PUSH_ENTRIES: usize = 500;

const MEASURED_PUSH_CLAIMED: usize = 2;

/// The widths a measurement asks its own binary for,
/// alternating so the spread is visible.
const MEASURED_WIDTHS: [usize; 8] = [1, 2, crate::service_work_items_side_by_side::SLOW_ITEMS_CEILING, 8, 8, crate::service_work_items_side_by_side::SLOW_ITEMS_CEILING, 2, 1];

/// How long the whole queue may take before the measurement says it did not drain.
/// Why:
///  `wait_until` gives up only when the run stops moving,
/// and the after-commit work keeps moving,
/// so an arm whose trashes are refused would spin until the harness kills it:
/// this ceiling makes that a named failure instead.
const MEASURED_DRAIN_CEILING: Duration = Duration::from_secs(120);

/// What:
///  Whether `trashes` trash records of the queue are settled,
///  waited for no longer than `ceiling`.
/// Why:
///  A record of another kind in `done` must not stand in for a trash:
/// the push a control writes settles there too,
/// so only the trash records are counted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await withDeadline(ceiling, waitUntil(() => doneTrashes() === worktrees))
/// ```
fn emptied_by(work: &Path, of_round: &str, trashes: usize, ceiling: Duration) -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < ceiling {
        if done_trashes(work, of_round) >= trashes {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return done_trashes(work, of_round) >= trashes;
}

/// How many trash records of the work directory `work` whose object begins with `of_round` are settled as done.
fn done_trashes(work: &Path, of_round: &str) -> usize {
    let header: String = format!("{}-{}", WorkKind::Trash.name(), &of_round[..1.min(of_round.len())]);
    return state_directory(work, DONE)
        .read_dir()
        .map_or(0, |listing| { return listing.flatten().filter(|entry| { return entry.file_name().to_string_lossy().starts_with(header.as_str()); }).count(); });
}

/// What:
///  Write one trash whose single entry holds `files` files below `worktree`,
///  and the record that has the service empty it.
/// Why:
///   One entry is one job of the removal,
/// so a trash of one big entry is the case where an item's work is serial
/// and the wave's width is what a queue of items costs:
/// with many small entries each item already runs its own entries side by side,
/// and two widths drain such a queue in the same time
/// (measured on 2026-11-11:
/// 424,
/// 483,
/// 422 and 424 ms for eight trashes of a thousand one-file entries at the widths 1,
/// 4,
/// 4 and 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(work, trashItem(plantOneBigEntry(worktree, name, files)))
/// ```
fn plant_a_big_trash_item(staged: &Stage, worktree: &Path, name: &str, files: usize, work: &Path) {
    let inside: PathBuf = worktree.join(crate::worktree_trash::TRASH_PARENT).join(name).join(crate::worktree_trash_manifest::entry_name(0));
    std::fs::create_dir_all(&inside).expect("the entry directory");
    for index in 0..files {
        std::fs::write(inside.join(format!("file-{index}")), b"left out\n").expect("file");
    }
    let root: std::fs::File = crate::fs_beneath::open_root(worktree).expect("the worktree");
    let names: Vec<std::ffi::OsString> = crate::fs_beneath::plain_components(Path::new(crate::worktree_trash::TRASH_PARENT)).expect("plain parent");
    let parent: std::fs::File = crate::fs_beneath::open_directory_beneath(&root, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = crate::fs_beneath::open_directory_at(&parent, std::ffi::OsStr::new(name)).expect("the trash");
    let manifest: crate::worktree_trash_manifest::TrashManifest = crate::worktree_trash_manifest::TrashManifest {
        name: String::from(name),
        trash: crate::fs_beneath::identity_of(&trash).expect("trash identity"),
        entries: vec![crate::fs_beneath::identity_at(&trash, std::ffi::OsStr::new(crate::worktree_trash_manifest::entry_name(0).as_str())).expect("entry identity")],
    };
    let item: WorkItem = WorkItem {
        kind: WorkKind::Trash,
        repository: worktree.to_path_buf(),
        git_dir: staged.repo.join(".git"),
        branch_ref: String::from(crate::worktree_trash_dispose::TRASH_REFERENCE),
        oid: manifest.name.clone(),
        parent: manifest.encode(),
        created: String::from("2026-11-11T08:50:59.000Z"),
        tries: 0,
    };
    enqueue(work, &item).expect("the trash is written");
}

/// What:
///  Write one trash of `entries` entries below `worktree` and the record that has the service empty it.
/// Why:
///   The handler proves the trash is the directory the leave-out made and each entry the object it moved,
/// so a trash planted any other way measures a refusal and not an emptying.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(work, trashItem(plantTrash(worktree, name, entries)))
/// ```
fn plant_a_trash_item(staged: &Stage, worktree: &Path, name: &str, entries: usize, work: &Path) {
    let manifest: crate::worktree_trash_manifest::TrashManifest = plant_trash(worktree, name, entries);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Trash,
        repository: worktree.to_path_buf(),
        git_dir: staged.repo.join(".git"),
        branch_ref: String::from(crate::worktree_trash_dispose::TRASH_REFERENCE),
        oid: manifest.name.clone(),
        parent: manifest.encode(),
        created: String::from("2026-11-11T08:50:59.000Z"),
        tries: 0,
    };
    enqueue(work, &item).expect("the trash is written");
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
/// and the control asks for the order of magnitude,
/// not for a latency budget:
/// ten seconds is two orders of magnitude under the hold and far above what the push itself takes.
const PUSH_BOUND: Duration = Duration::from_secs(10);

/// What:
///  Handlers whose review and trash work is held open for `HOLD`,
/// and whose push is the real one.
/// Why:
///   The service the coordinator measured held a push behind a review and a trash that had nothing to do with it,
/// and no fault hook of this release can hold a review open
/// (`worktree_trash_dispose_awaiting.rs` is the one that can,
/// and it holds a trash).
/// The slow kinds answer the same four verdicts whatever runs them,
/// so a stand-in that holds them open says what a slow review and a slow trash say about the push beside them,
/// and the push itself is real Git against a real bare remote.
struct StandIns {
    /// The real handlers,
    /// which run the push.
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

/// The record of `kind` for `staged`,
/// naming the commit,
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
/// `push-a52a4adaa8…` sat pending from `10:29` with a maintenance,
/// a review and eight prewarms beside it,
/// and `main` had to be pushed by hand.
/// The two claim lists are what answers it:
/// the round that answers the push's wake claims `PUSH_KINDS`
/// alone,
/// so what the lane holds is not in its way.
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
    let lane = async move {
        return crate::service_work_slow_lane::drain_the_slow_work_awaiting(
            git_dir.as_path(),
            lane_handlers.as_ref(),
            crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
            crate::service_work::CLAIM_LEASE,
        )
        .await;
    };
    let running: tokio::task::JoinHandle<Option<Duration>> = crate::lazy_runtime::spawn_on_runtime(lane).expect("the runtime starts the lane");
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
