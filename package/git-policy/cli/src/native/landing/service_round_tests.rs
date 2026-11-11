//! A round over several repositories:
//! the drains overlap in time,
//! the retry list keeps the order the directories were named,
//! the
//!  directories of one repository are drained one after another,
//! and the host budget bounds the overlap.

use super::*;
use std::sync::Arc;
use crate::parallel_test_support::{PATIENCE, Rendezvous, SERIAL_PATIENCE};
use crate::service_work::{Verdict, work_directory};
use crate::test_support::{fixture, remove};
use crate::work_item::{DONE, WorkItem, WorkKind, enqueue, state_directory};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many repositories the rounds of these controls name.
const REPOSITORIES: usize = 4;

/// A host large enough that neither the CPUs nor the memory bound a round of `REPOSITORIES` repositories.
const LARGE_HOST: HostResources = HostResources { parallelism: REPOSITORIES * 8, memory_available: None };

/// The branch every item names.
const BRANCH: &str = "refs/heads/main";

/// A repository directory standing in for a common Git directory,
/// with a worktree directory beside it and one pending push.
struct Repository {
    /// The fixture root.
    root: PathBuf,
    /// The common Git directory.
    common: PathBuf,
}

/// Make `count` repositories,
/// each with a pending push,
/// under one fixture.
fn repositories(name: &str, count: usize) -> (PathBuf, Vec<Repository>) {
    let root: PathBuf = fixture(name);
    let made: Vec<Repository> = (0..count)
        .map(|index: usize| {
            let own: PathBuf = root.join(format!("repository{index}"));
            let common: PathBuf = own.join("common");
            let worktree: PathBuf = own.join("worktree");
            std::fs::create_dir_all(&common).expect("common directory");
            std::fs::create_dir_all(&worktree).expect("worktree directory");
            let pending: WorkItem = WorkItem {
                kind: WorkKind::Push,
                repository: worktree.to_path_buf(),
                git_dir: common.to_path_buf(),
                branch_ref: BRANCH.to_string(),
                oid: format!("{index:040x}"),
                parent: String::new(),
                created: String::from("2026-01-01T00:00:00.000Z"),
                tries: 0,
            };
            enqueue(work_directory(common.as_path()).as_path(), &pending).expect("enqueue");
            return Repository { root: own, common };
        })
        .collect();
    return (root, made);
}

/// Handlers that run `on_push` for every push and answer done;
/// they hold only what several threads may share.
struct Handlers {
    /// Called with the Git directory of the pushed repository,
    /// shared so a job of the blocking pool may carry it.
    on_push: Arc<dyn Fn(&Path) + Sync + Send + 'static>,
    /// The Git directories pushed,
    /// in the order the pushes began,
    /// shared with the control that reads them after the round took the handlers.
    began: Arc<Mutex<Vec<PathBuf>>>,
    /// The point the awaited push waits at,
    /// when the control measures the lanes rather than the pool.
    /// Nothing means the waiting is a job of the blocking pool,
    /// as a handler that knows nothing of the runtime waits.
    gate: Option<Arc<tokio::sync::Barrier>>,
}

impl WorkHandlers for Handlers {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let directory: PathBuf = items.last().map(|item: &WorkItem| { return item.git_dir.clone(); }).unwrap_or_default();
        self.began.lock().expect("the order of the pushes").push(directory.clone());
        (self.on_push)(directory.as_path());
        return Verdict::Done;
    }

    /// What:
    ///  The same push,
    /// its wait on the runtime when the control gave it a point there,
    /// else on the blocking pool.
    /// Why:
    ///   A control's push may wait on a point several pushes must reach
    /// together,
    /// and where that wait sits decides what the control proves:
    /// a point on the
    ///  blocking pool is passed only as far as the process's room for blocking
    /// jobs reaches (two jobs in the gate's container,
    /// whatever the CPUs a
    /// control states for its own budget),
    /// while a point on the runtime is passed by every lane task,
    /// because an await frees the worker it waited on.
    /// The shipped handlers await their Git on the runtime,
    /// so a lane control measures the round with a point there,
    /// and a control of the pool's own bound keeps the pool.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async pushAwaiting(items) { began.push(dir); if (gate) await gate.wait(); else await offTheWorkers(() => onPush(dir)); return DONE; }
    /// ```
    fn push_awaiting<'a>(&'a self, items: &'a [WorkItem]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            let directory: PathBuf = items.last().map(|item: &WorkItem| { return item.git_dir.clone(); }).unwrap_or_default();
            self.began.lock().expect("the order of the pushes").push(directory.clone());
            if let Some(gate) = self.gate.as_ref() {
                // Every lane must arrive for the round to go on, so a round that runs its lanes in turn never finishes.
                gate.wait().await;
                return Verdict::Done;
            }
            let on_push: Arc<dyn Fn(&Path) + Sync + Send + 'static> = Arc::clone(&self.on_push);
            crate::blocking_job::run_blocking_job(move || {
                (on_push)(directory.as_path());
            })
            .await
            .expect("a thread of the pool runs the push of this control");
            return Verdict::Done;
        });
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// Drain `repositories` on `host` with `handlers` and return the retry list.
fn round(repositories: &[Repository], host: &HostResources, handlers: Arc<Handlers>) -> Vec<(PathBuf, Duration)> {
    let directories: Vec<Assignment> = repositories.iter().map(|repository: &Repository| { return Assignment::plain(repository.common.clone()); }).collect();
    return drain_directories(&RoundDrain { directories: directories.as_slice(), handlers, now: &ClaimTime::read(), host }).waits;
}

/// How many items a repository settled as done.
fn settled(repository: &Repository) -> usize {
    return std::fs::read_dir(state_directory(work_directory(repository.common.as_path()).as_path(), DONE)).map_or(0, |listing| { return listing.flatten().count(); });
}

/// What:
///  The pushes of all the repositories are under way at the same time,
/// so no repository waits for another,
/// and every item is settled.
/// Why:
///   The round hands each repository to a lane of tasks and takes the answers
/// back in order,
/// so a lane that waited for another would
///  lose the person's backup behind a slow push of another repository.
/// The point is on the runtime,
/// where the shipped
///  handlers wait for their Git:
/// a point on the blocking pool is passed only as far as the process's room
/// for blocking jobs
///  reaches,
/// which is two jobs in the gate's container whatever the CPUs a control states
/// for its own budget,
/// and a control that
///  measured that would prove the pool's bound rather than the round's.
/// A round that ran its lanes in turn never brings all of them to the point,
/// so it never finishes,
/// and the control fails by name
///  instead of passing on a patience.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = finishedWithin(PATIENCE, () => drainDirectories({ directories, handlers, now, host })); // every lane reached the barrier
/// ```
#[test]
fn repositories_are_drained_side_by_side() {
    let (root, made) = repositories("round-overlap", REPOSITORIES);
    let shared: Arc<Handlers> = Arc::new(Handlers {
        on_push: Arc::new(|_directory: &Path| {}) as Arc<dyn Fn(&Path) + Sync + Send>,
        began: Arc::new(Mutex::new(Vec::new())),
        gate: Some(Arc::new(tokio::sync::Barrier::new(REPOSITORIES))),
    });
    // The directories the round drains, so the control can still read what it settled after the closure took them.
    let directories: Vec<PathBuf> = made.iter().map(|repository: &Repository| { return repository.common.clone(); }).collect();
    let ran = crate::test_support::finished_within_at(PATIENCE, move || {
        let assignments: Vec<Assignment> = directories.iter().map(|directory: &PathBuf| { return Assignment::plain(directory.clone()); }).collect();
        return drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: shared, now: &ClaimTime::read(), host: &LARGE_HOST });
    });
    assert!(ran.is_some(), "the round brought every repository to the point within {PATIENCE:?}, so its lanes ran side by side");
    assert!(made.iter().all(|repository: &Repository| { return settled(repository) == 1; }), "every item was settled as done");
    remove(root.as_path());
}

/// The host's budget bounds the overlap:
/// on a host that may run few threads the overlap reaches that many and no more.
#[test]
fn the_budget_of_the_host_bounds_the_overlap() {
    let (root, made) = repositories("round-budget", REPOSITORIES * 2);
    let host: HostResources = HostResources { parallelism: 3, memory_available: None };
    let allowed: usize = worker_budget(&BudgetRequest { items: REPOSITORIES * 2, per_item_memory: REPOSITORY_MEMORY, ceiling: None, cap: None, host: &host });
    assert!(allowed > 1 && allowed < REPOSITORIES, "the control needs a bound between one and the repositories: {allowed}");
    let running: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let busiest: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    // The patience bounds only how long a run that never reaches the bound waits before it fails.
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(allowed, PATIENCE));
    let (counting, most, waiting) = (Arc::clone(&running), Arc::clone(&busiest), Arc::clone(&point));
    let handlers: Handlers = Handlers {
        on_push: Arc::new(move |_directory: &Path| {
            let now: usize = counting.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(now, Ordering::SeqCst);
            waiting.arrive_at_the_bound(allowed);
            counting.fetch_sub(1, Ordering::SeqCst);
        }) as Arc<dyn Fn(&Path) + Sync + Send>,
        began: Arc::new(Mutex::new(Vec::new())),
        gate: None,
    };
    round(made.as_slice(), &host, Arc::new(handlers));
    assert_eq!(busiest.load(Ordering::SeqCst), allowed);
    remove(root.as_path());
}

/// The retry list names the directories in the order they were named,
/// whichever drain finishes first.
/// A claim another instance
///  holds gives a wait,
/// so the first repository holds one while the others finish ahead of it.
#[test]
fn the_retry_list_keeps_the_order_the_directories_were_named() {
    let (root, made) = repositories("round-order", REPOSITORIES);
    for repository in &made {
        // A claim whose lease has a long time to run: the round is told to look again.
        let foreign: WorkItem = WorkItem {
            kind: WorkKind::Review,
            repository: repository.root.join("worktree").to_path_buf(),
            git_dir: repository.common.to_path_buf(),
            branch_ref: BRANCH.to_string(),
            oid: format!("{:040x}", 0xfeed),
            parent: String::new(),
            created: String::from("2026-01-01T00:00:00.000Z"),
            tries: 0,
        };
        enqueue(work_directory(repository.common.as_path()).as_path(), &foreign).expect("enqueue");
        crate::work_item::claim_next_of(work_directory(repository.common.as_path()).as_path(), &ClaimTime::read(), &[WorkKind::Review]).expect("claimed").expect("a claim");
    }
    let first_done: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(REPOSITORIES - 1, PATIENCE));
    let first_common: PathBuf = made[0].common.clone();
    let waiting: Arc<Rendezvous> = Arc::clone(&first_done);
    let handlers: Handlers = Handlers {
        on_push: Arc::new(move |directory: &Path| {
            if directory == first_common.as_path() {
                assert!(waiting.wait_for_target(), "the other repositories were drained while the first waited");
            } else {
                waiting.arrive_without_waiting();
            }
        }) as Arc<dyn Fn(&Path) + Sync + Send>,
        began: Arc::new(Mutex::new(Vec::new())),
        gate: None,
    };
    let retries = round(made.as_slice(), &LARGE_HOST, Arc::new(handlers));
    let named: Vec<&PathBuf> = retries.iter().map(|(directory, _wait)| { return directory; }).collect();
    let expected: Vec<&PathBuf> = made.iter().map(|repository: &Repository| { return &repository.common; }).collect();
    assert_eq!(named, expected);
    remove(root.as_path());
}

/// Two directories of one repository (the common directory and a linked worktree's) are drained one after another,
/// never at once.
///  The first drain keeps its place inside the work until a second drain of the same repository arrives,
/// so a drain that begins while another is inside is an event the arriving drain reports,
///  and no fixture sleep decides whether the overlap is long enough to see.
#[test]
fn the_directories_of_one_repository_are_drained_one_after_another() {
    let (root, made) = repositories("round-lane", 1);
    let linked: PathBuf = made[0].root.join("linked");
    std::fs::create_dir_all(&linked).expect("linked directory");
    std::fs::write(linked.join("commondir"), "../common\n").expect("commondir");
    let pending: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: made[0].root.join("worktree").to_path_buf(),
        git_dir: linked.to_path_buf(),
        branch_ref: BRANCH.to_string(),
        oid: format!("{:040x}", 0xbeef),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(linked.as_path()).as_path(), &pending).expect("enqueue");
    let running: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let busiest: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let overlapped: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    // A drain waits here for a second drain of the same repository: a lane rule that gives each directory a lane of its
    // own brings one, and a lane rule that holds them one after another cannot.
    let together: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(2, SERIAL_PATIENCE));
    let (counting, most, seen, meeting) = (Arc::clone(&running), Arc::clone(&busiest), Arc::clone(&overlapped), Arc::clone(&together));
    let handlers: Handlers = Handlers {
        on_push: Arc::new(move |_directory: &Path| {
            most.fetch_max(counting.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
            if meeting.arrive_and_give_back() {
                seen.fetch_add(1, Ordering::SeqCst);
            }
            counting.fetch_sub(1, Ordering::SeqCst);
        }) as Arc<dyn Fn(&Path) + Sync + Send>,
        began: Arc::new(Mutex::new(Vec::new())),
        gate: None,
    };
    let directories: Vec<Assignment> = vec![Assignment::plain(made[0].common.clone()), Assignment::plain(linked.clone())];
    let began: Arc<Mutex<Vec<PathBuf>>> = Arc::clone(&handlers.began);
    drain_directories(&RoundDrain { directories: directories.as_slice(), handlers: Arc::new(handlers), now: &ClaimTime::read(), host: &LARGE_HOST });
    assert_eq!(began.lock().expect("pushes").len(), 2, "both directories were drained");
    assert_eq!(overlapped.load(Ordering::SeqCst), 0, "a drain of one repository began while another was inside its push");
    assert_eq!(busiest.load(Ordering::SeqCst), 1);
    remove(root.as_path());
}

/// The estimate is at least the measured peak of one repository's service,
/// so a budget never admits more drains than the memory
///  they were measured to hold.
#[test]
fn the_repository_estimate_covers_the_measured_peak() {
    const { assert!(REPOSITORY_MEMORY >= MEASURED_REPOSITORY_PEAK, "the repository estimate must cover the measured peak") };
}

/// The estimate is the measurement rounded up to a power of two and not more,
/// so it stays tied to the figure it cites.
#[test]
fn the_repository_estimate_is_the_measurement_rounded_up_to_a_power_of_two() {
    assert_eq!(REPOSITORY_MEMORY, MEASURED_REPOSITORY_PEAK.next_power_of_two());
}

/// How long before the first new try the assignments of these controls plan,
/// so a push that could pass leaves a plan behind as well as a wait.
const FIRST_PAUSE: Duration = Duration::from_secs(3);

/// How far two readings of the same lease may differ:
/// each world reads the clock for itself,
/// and a round takes time.
const LEASE_READING_SLACK: Duration = Duration::from_secs(5);

/// Handlers whose pushes could pass,
/// so the round plans a try for them,
/// and which record the Git directory of every push in the order they began.
struct CouldPass {
    /// The Git directories pushed,
    /// in the order the pushes began,
    /// shared with the control that reads them after the round took the handlers.
    began: Arc<Mutex<Vec<PathBuf>>>,
}

impl WorkHandlers for CouldPass {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let directory: PathBuf = items.last().map(|item: &WorkItem| { return item.git_dir.clone(); }).unwrap_or_default();
        self.began.lock().expect("the order of the pushes").push(directory);
        return Verdict::Retry(String::from("Could not resolve host"));
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// What:
///  A repository whose round has one push that could pass and one claim another instance holds,
/// so the answer carries a plan and a wait.
/// Why:
///   Both halves of the answer are what the step must keep,
/// and a round of pushes alone would pin neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = plannedWorld('round-planned', 4); // [{ directory, retrying }, ...]
/// ```
fn planned_world(name: &str, count: usize) -> (PathBuf, Vec<Assignment>) {
    let (root, made): (PathBuf, Vec<Repository>) = repositories(name, count);
    let mut assignments: Vec<Assignment> = Vec::new();
    for repository in &made {
        // A claim another instance holds for a while: the round is told when to look again.
        let foreign: WorkItem = WorkItem {
            kind: WorkKind::Review,
            repository: repository.root.join("worktree").to_path_buf(),
            git_dir: repository.common.to_path_buf(),
            branch_ref: BRANCH.to_string(),
            oid: format!("{:040x}", 0xfeed),
            parent: String::new(),
            created: String::from("2026-01-01T00:00:00.000Z"),
            tries: 0,
        };
        enqueue(work_directory(repository.common.as_path()).as_path(), &foreign).expect("enqueue");
        crate::work_item::claim_next_of(work_directory(repository.common.as_path()).as_path(), &ClaimTime::read(), &[WorkKind::Review]).expect("claimed").expect("a claim");
        assignments.push(Assignment { directory: repository.common.clone(), retrying: Retrying { first: Some(FIRST_PAUSE), ..Retrying::default() } });
    }
    return (root, assignments);
}

/// What:
///  What an answer says,
///  as the index of each directory and the plan it asks for,
/// so two rounds over two fixtures can be compared.
/// Why:
///   The two worlds are separate fixtures,
/// so their paths differ and their blocks cannot;
/// the order and the content of the answer are what is compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const compared = rendered(answer, directories); // ['wait 0', 'plan 0 refs/heads/main 1', ...]
/// ```
fn rendered(answer: &RoundAnswer, directories: &[Assignment]) -> Vec<String> {
    let index_of = |directory: &Path| { return directories.iter().position(|assignment: &Assignment| { return assignment.directory == directory; }).expect("a directory of the round"); };
    let mut said: Vec<String> = Vec::new();
    for (directory, _wait) in &answer.waits {
        said.push(format!("wait {}", index_of(directory.as_path())));
    }
    for change in &answer.changes {
        said.push(match change {
            RetryChange::Plan { git_dir, item, attempt, .. } => format!("plan {} {} {attempt}", index_of(git_dir.as_path()), item.branch_ref),
            RetryChange::Forget { git_dir, branch_ref, .. } => format!("forget {} {branch_ref}", index_of(git_dir.as_path())),
        });
    }
    return said;
}

/// What:
///  Drains the directories one after another on the calling thread,
///  and gives the answer a drain of one directory at a time gives.
/// Why:
///   This is the oracle the round is compared against:
/// the same item work,
/// with no runner between them,
/// so a difference is the runner's
///  ordering or assembly and never the drain's own behaviour.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serial = drainOneAfterAnother(directories, handlers, now);
/// ```
fn drain_one_after_another(directories: &[Assignment], handlers: &CouldPass, now: &ClaimTime) -> RoundAnswer {
    let mut answer: RoundAnswer = RoundAnswer::default();
    for assignment in directories.iter() {
        let (wait, changes): (Option<Duration>, Vec<RetryChange>) = crate::merge_second_pass::tests::await_it(drain_one(handlers, now, assignment));
        if let Some(lease_wait) = wait {
            answer.waits.push((assignment.directory.clone(), lease_wait));
        }
        answer.changes.extend(changes);
    }
    return answer;
}

/// What:
///  The round's answer is the one a drain of one directory after another gives:
/// the same directories with a wait,
/// in the
///  order they were named,
/// and the same plan for the same branch.
/// Why:
///   The service loop applies those changes and wakes on those waits,
/// so a round that reordered or
///  dropped one would plan a different set of tries than the reader of the answer expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(rendered(round(dirs))).toEqual(rendered(drainOneAfterAnother(dirs)))
/// ```
#[test]
fn the_answer_of_a_round_is_the_one_a_drain_of_one_directory_after_another_gives() {
    let (round_root, assignments) = planned_world("round-answer-parallel", REPOSITORIES);
    let (serial_root, serial_assignments) = planned_world("round-answer-serial", REPOSITORIES);
    let round_handlers: CouldPass = CouldPass { began: Arc::new(Mutex::new(Vec::new())) };
    let serial_handlers: CouldPass = CouldPass { began: Arc::new(Mutex::new(Vec::new())) };
    let now: ClaimTime = ClaimTime::read();
    let parallel: RoundAnswer = drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::new(round_handlers), now: &now, host: &LARGE_HOST });
    let serial: RoundAnswer = drain_one_after_another(serial_assignments.as_slice(), &serial_handlers, &now);
    assert_eq!(rendered(&parallel, assignments.as_slice()), rendered(&serial, serial_assignments.as_slice()));
    assert!(!parallel.waits.is_empty(), "the control needs a claim another instance holds, so the answer carries a wait");
    assert!(!parallel.changes.is_empty(), "the control needs a push that could pass, so the answer carries a plan");
    for ((directory, waited), (_same, serial_waited)) in parallel.waits.iter().zip(serial.waits.iter()) {
        assert!(
            waited.abs_diff(*serial_waited) <= LEASE_READING_SLACK,
            "the wait of {} is the one a drain of one directory after another gives: {waited:?} against {serial_waited:?}",
            directory.display()
        );
    }
    remove(round_root.as_path());
    remove(serial_root.as_path());
}

/// What:
///  A lane drains its directories in the order they were named.
/// Why:
///   The directories of one repository share a work directory and its upkeep,
/// so they run one after another,
///  and a reader of the
///  answer expects them in the order the wakes named them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(began).toEqual(['common', 'linked'])
/// ```
#[test]
fn a_lane_drains_its_directories_in_the_order_they_were_named() {
    let (root, made) = repositories("round-lane-order", 1);
    let linked: PathBuf = made[0].root.join("linked");
    std::fs::create_dir_all(&linked).expect("linked directory");
    std::fs::write(linked.join("commondir"), "../common\n").expect("commondir");
    let pending: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: made[0].root.join("worktree").to_path_buf(),
        git_dir: linked.clone(),
        branch_ref: BRANCH.to_string(),
        oid: format!("{:040x}", 0xbeef),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(linked.as_path()).as_path(), &pending).expect("enqueue");
    let handlers: CouldPass = CouldPass { began: Arc::new(Mutex::new(Vec::new())) };
    let directories: Vec<Assignment> = vec![
        Assignment { directory: made[0].common.clone(), retrying: Retrying { first: Some(FIRST_PAUSE), ..Retrying::default() } },
        Assignment { directory: linked.clone(), retrying: Retrying { first: Some(FIRST_PAUSE), ..Retrying::default() } },
    ];
    let began_log: Arc<Mutex<Vec<PathBuf>>> = Arc::clone(&handlers.began);
    drain_directories(&RoundDrain { directories: directories.as_slice(), handlers: Arc::new(handlers), now: &ClaimTime::read(), host: &LARGE_HOST });
    let began: Vec<PathBuf> = began_log.lock().expect("the order of the pushes").clone();
    assert_eq!(began, vec![made[0].common.clone(), linked.clone()], "the lane drained its directories in the order they were named");
    remove(root.as_path());
}

/// What:
///  A lane that stops partway through a round says so,
/// runs no item after the one it stopped in,
/// and loses nothing:
/// the record it was inside keeps its claim,
/// the record behind it stays pending,
/// and a repository of another lane is drained.
/// Why:
///   The round's items are tasks,
/// so a lane stops where a task panics or the pool refuses a thread,
/// and the
///  caller that is told nothing reads the silence as a refresh that covered every directory.
/// The plan says what the round must answer
///  instead of a full answer:
/// `OrderedRunError`,
/// with the claims of the stopped lane left to their lease (`service_work::CLAIM_LEASE`),
///  which is the next round of the work loop's to find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [answer, stopped] = await drainDirectoriesAwaiting({ dirs }); // stopped is Panicked, the record behind stays pending
/// ```
#[test]
fn a_lane_that_panics_stops_its_lane_and_the_round_says_so() {
    let (root, made) = repositories("round-lane-panics", 2);
    let linked: PathBuf = made[0].root.join("linked");
    std::fs::create_dir_all(linked.as_path()).expect("linked directory");
    std::fs::write(linked.join("commondir"), "../common\n").expect("commondir");
    let behind: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: made[0].root.join("worktree").to_path_buf(),
        git_dir: linked.clone(),
        branch_ref: BRANCH.to_string(),
        oid: format!("{:040x}", 0xbeef),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(linked.as_path()).as_path(), &behind).expect("enqueue");
    let handlers: PanicsOn = PanicsOn { when: made[0].common.clone() };
    let directories: Vec<Assignment> = vec![Assignment::plain(made[0].common.clone()), Assignment::plain(linked.clone()), Assignment::plain(made[1].common.clone())];
    let (answered, stopped) = crate::merge_second_pass::tests::await_it(drain_directories_awaiting(&RoundDrain {
        directories: directories.as_slice(),
        handlers: Arc::new(handlers),
        now: &ClaimTime::read(),
        host: &LARGE_HOST,
    }));
    let why: crate::parallel_runner_on_the_runtime::OrderedRunError = stopped.expect("the round says it did not finish");
    assert!(why.to_string().contains("panicked"), "the round names a step that panicked: {why}");
    assert_eq!(answered.changes.len(), 0, "the lane that stopped planned nothing: {:?}", answered.changes);
    assert_eq!(settled(&made[1]), 1, "the repository of another lane was drained");
    let claimed: usize = std::fs::read_dir(state_directory(work_directory(made[0].common.as_path()).as_path(), crate::work_item::CLAIMED)).map_or(0, |listing| { return listing.flatten().count(); });
    assert_eq!(claimed, 1, "the record the lane was inside keeps its claim, which its lease ends");
    let pending: usize = std::fs::read_dir(state_directory(work_directory(linked.as_path()).as_path(), crate::work_item::PENDING)).map_or(0, |listing| { return listing.flatten().count(); });
    assert_eq!(pending, 1, "the record behind the one that stopped was not run and stays pending");
    remove(root.as_path());
}

/// Handlers that panic on the push of one repository and answer done for every other,
/// with the Git directory of the push that stops them.
struct PanicsOn {
    /// The Git directory whose push panics.
    when: PathBuf,
}

impl WorkHandlers for PanicsOn {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let directory: PathBuf = items.last().map(|item: &WorkItem| { return item.git_dir.clone(); }).unwrap_or_default();
        assert!(directory != self.when, "the push of this repository panics, which is what the control measures");
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// What:
///  A Git read asked from inside a blocking job of the pool is refused,
/// which is why a round cannot put its items there.
/// Why:
///   Every item of this round runs Git,
/// and the steps it runs are ended at a ceiling,
/// so they read their
///  child through pipes on the runtime
/// (`process/step_output_read_whole_through_pipes.rs`,
/// `read_until_the_child_ends`).
///  A job of the blocking pool is inside the runtime,
/// so that read is refused there
/// (`lazy_runtime::RuntimeError::Nested`),
///  and a round that ran its items as jobs would fail every push.
/// The measured fact decides the shape the
///  conversion must take:
/// the lanes become tasks of the runtime and their Git is awaited
/// (`transaction/transaction_git_awaiting.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = await inABlockingJob(() => git(['rev-parse', '--show-toplevel'])); // Err(Nested)
/// ```
#[test]
fn a_git_read_asked_inside_a_blocking_job_of_the_pool_is_refused() {
    let root: PathBuf = fixture("round-git-inside-a-job");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let context = crate::test_support::git_context();
    let request: crate::transaction_git::GitRequest = crate::transaction_git::GitRequest::new(repo.as_path(), &["rev-parse", "--show-toplevel"]);
    let asked: Result<std::io::Result<crate::transaction_git::GitOutput>, crate::blocking_job::BlockingJobError> = crate::lazy_runtime::run_on_runtime(async move {
        return crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            return crate::transaction_git::run_git(&context, &request);
        })
        .await;
    })
    .expect("the runtime");
    let refused: std::io::Error = asked.expect("the job ran and gave Git's answer as a result").expect_err("a step read through pipes cannot be waited for inside the runtime");
    assert!(
        refused.to_string().contains("inside itself"),
        "the refusal names the runtime that was asked for from inside itself: {refused}"
    );
    crate::test_support::remove(root.as_path());
}

/// How long the control of a lane key gives the runtime's one worker to pick up its next task.
const THE_TICK: Duration = Duration::from_millis(50);

/// The most the control of a lane key waits for that task,
/// far beyond what a free worker needs,
/// so only a held worker reaches it.
const THE_PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  Stand a pipe with no writer at `path`,
/// so a read of it waits in the kernel until a writer opens it.
/// Why:
///  This is the one way to make a read of a file hold its thread with no end,
/// which is what a mount that stopped answering does
///  (`work_awaited_within_a_ceiling.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
fn stall_a_file(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  fixture takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where the file would be: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends the read a control left behind.
/// Why:
///  The read of the pipe is abandoned,
/// never cancelled,
/// so the thread that holds it ends only when a writer appears;
/// the open does
///  not wait for a reader,
///  and the control asks again until `THE_PATIENCE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { close(openSync(path, 'w')); return } catch { await sleep(10) } }
/// ```
fn release_the_pipe(path: &Path) {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < THE_PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            Err(_not_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// What:
///  A `commondir` that gives no answer does not hold the round's worker:
/// a task beside the reading runs while the read waits.
/// Why:
///  Finding 5 of the review of `b3d00b3be4`:
/// the round read each directory's `commondir` on the task that runs it,
/// so a linked worktree on a mount
///  that stopped answering held a worker of the runtime for good,
/// and the after-commit queue made no further progress.
/// The read is a job
///  of the blocking pool here,
/// so the worker polls what is beside it.
/// One worker makes the difference visible:
/// a held worker runs
///  nothing else.
#[test]
fn a_common_directory_that_gives_no_answer_does_not_hold_the_rounds_worker() {
    use std::sync::atomic::AtomicBool;
    let root: PathBuf = fixture("round-lane-key-stalls");
    let git_dir: PathBuf = root.join("worktree/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("a linked worktree's Git directory");
    let stalled: PathBuf = git_dir.join("commondir");
    stall_a_file(stalled.as_path());
    let directories: Vec<Assignment> = vec![Assignment::plain(git_dir.clone())];
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let reading = runtime.spawn(async move {
        return lane_keys_awaiting(directories.as_slice()).await;
    });
    let began: std::time::Instant = std::time::Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    release_the_pipe(stalled.as_path());
    let answered: TheAnswerOfAJobAwaited<Vec<PathBuf>> = runtime.block_on(async { return reading.await.expect("the reading task ends"); });
    assert!(ran_beside, "the task beside a common directory that gives no answer still runs, so no worker is held by that read");
    assert!(matches!(answered, TheAnswerOfAJobAwaited::Ran(_)), "the reading gave the lane of the directory once the pipe was let go");
    crate::test_support::remove(root.as_path());
}

/// How long the control of a drain's step lets the whole measurement run before it calls the step lost.
/// Why:
///  The control has to fail where the defect is, and the defect is a wait with no end:
/// a control that waited on that one would be ended by the runner's per-test limit instead of saying what it found.
/// The work itself reaches its own answers within a second on any host,
/// so this is the control's deadline,
/// not a clock the claim rests on.
const THE_STEPS_DEADLINE: Duration = Duration::from_secs(30);

/// The ceiling the control gives the lane read and the drain's own step,
/// well inside `THE_STEPS_DEADLINE`,
/// so the step's own answer is what the deadline waits for rather than the read's ceiling.
const THE_STEPS_CEILING: Duration = Duration::from_millis(200);

/// What:
///  A `commondir` read that outlasted its ceiling holds the only place of the blocking bound,
///  and the step the drain runs next still answers instead of waiting for that place for ever.
/// Why:
///  A place is held by the job that took it until that job ends,
/// and a job whose read outlasted its ceiling is left behind still holding it
///  (`blocking_jobs_within_the_budget.rs`).
/// The drain's storage steps awaited a place without an end,
/// so one stalled read with the process held to one place left the round waiting for ever:
/// no later push, no upkeep round,
/// and the idle exit never ran
///  (finding 4 of the review of `5686c7bb29`).
/// The place the control's read holds is the process's only one,
/// so the step's answer can only come from the bound the step itself keeps.
#[test]
fn a_lane_read_that_outlasted_its_ceiling_does_not_hold_the_drains_next_storage_step() {
    use crate::parallel_test_support::ONE_PLACE;
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(ONE_PLACE);
    let root: PathBuf = fixture("round-lane-read-holds-the-only-place");
    let git_dir: PathBuf = root.join("worktree/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("a linked worktree's Git directory");
    let stalled: PathBuf = git_dir.join("commondir");
    stall_a_file(stalled.as_path());
    let work_dir: PathBuf = root.join("work");
    let measured: PathBuf = root.clone();
    let let_go: PathBuf = stalled.clone();
    let answered = crate::parallel_test_support::finished_within_at(THE_STEPS_DEADLINE, move || {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
        let (read, step) = runtime.block_on(async move {
            // The round's lane read, which the round awaits no longer than this ceiling.
            let read = run_a_job_awaited_no_longer_than(THE_STEPS_CEILING, vec![git_dir], lane_keys_in_a_job).await;
            // The drain's first storage step, given the same ceiling here so the control reaches its bound in a moment.
            let step = crate::service_work_storage_awaited::claim_all_within_awaiting(THE_STEPS_CEILING, work_dir.as_path()).await;
            return (read, step);
        });
        // The place the read holds cannot go back while the read waits, and a runtime's own drop waits for the job that
        // reads it: the pipe is let go here, after the step has answered against the place the read still held.
        release_the_pipe(let_go.as_path());
        drop(runtime);
        return (read, step);
    });
    if answered.is_none() {
        // A run that reached the deadline left its read behind; the writer ends it so the fixture goes.
        release_the_pipe(stalled.as_path());
    }
    let (read, step) = answered.expect(
        "the drain's first storage step never answered within the control's deadline, so it waited for the place the abandoned lane read holds and the round never ends",
    );
    assert!(
        matches!(read, TheAnswerOfAJobAwaited::NoAnswer),
        "the lane read reached its ceiling, so the job that read the common directory is still holding the only place of the bound"
    );
    let said: String = step.expect_err("the drain's first storage step answered, so no round waits for a place nothing gives back");
    assert!(said.contains("did not run"), "the step says it did not run, so nothing it would have claimed was claimed: {said}");
    assert!(said.contains("no place"), "the step names the place it waited for: {said}");
    crate::test_support::remove(measured.as_path());
}
