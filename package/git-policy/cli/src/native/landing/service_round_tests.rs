//! A round over several repositories:
//! the drains overlap in time,
//! the retry list keeps the order the directories were named,
//! the
//!  directories of one repository are drained one after another,
//! and the host budget bounds the overlap.

use super::*;
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
struct Handlers<'a> {
    /// Called with the Git directory of the pushed repository.
    on_push: Box<dyn Fn(&Path) + Sync + 'a>,
    /// The Git directories pushed,
    /// in the order the pushes began.
    began: Mutex<Vec<PathBuf>>,
}

impl WorkHandlers for Handlers<'_> {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let directory: PathBuf = items.last().map(|item: &WorkItem| { return item.git_dir.clone(); }).unwrap_or_default();
        self.began.lock().expect("the order of the pushes").push(directory.clone());
        (self.on_push)(directory.as_path());
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// Drain `repositories` on `host` with `handlers` and return the retry list.
fn round(repositories: &[Repository], host: &HostResources, handlers: &Handlers<'_>) -> Vec<(PathBuf, Duration)> {
    let directories: Vec<Assignment> = repositories.iter().map(|repository: &Repository| { return Assignment::plain(repository.common.clone()); }).collect();
    return drain_directories(&RoundDrain { directories: directories.as_slice(), handlers, now: &ClaimTime::read(), host }).waits;
}

/// How many items a repository settled as done.
fn settled(repository: &Repository) -> usize {
    return std::fs::read_dir(state_directory(work_directory(repository.common.as_path()).as_path(), DONE)).map_or(0, |listing| { return listing.flatten().count(); });
}

/// The pushes of all the repositories start before any has finished:
/// a point that needs all of them at once is passed only by
///  parallel work,
/// and a serial round waits there until its patience ends.
#[test]
fn repositories_are_drained_side_by_side() {
    let (root, made) = repositories("round-overlap", REPOSITORIES);
    let point: Rendezvous = Rendezvous::new(REPOSITORIES);
    let met: AtomicUsize = AtomicUsize::new(0);
    let handlers: Handlers<'_> = Handlers {
        on_push: Box::new(|_directory: &Path| {
            if point.arrive() {
                met.fetch_add(1, Ordering::SeqCst);
            }
        }),
        began: Mutex::new(Vec::new()),
    };
    round(made.as_slice(), &LARGE_HOST, &handlers);
    assert_eq!(met.load(Ordering::SeqCst), REPOSITORIES, "every push met the others while it ran");
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
    let running: AtomicUsize = AtomicUsize::new(0);
    let busiest: AtomicUsize = AtomicUsize::new(0);
    // The patience bounds only how long a run that never reaches the bound waits before it fails.
    let point: Rendezvous = Rendezvous::impatient(allowed, PATIENCE);
    let handlers: Handlers<'_> = Handlers {
        on_push: Box::new(|_directory: &Path| {
            let now: usize = running.fetch_add(1, Ordering::SeqCst) + 1;
            busiest.fetch_max(now, Ordering::SeqCst);
            point.arrive_at_the_bound(allowed);
            running.fetch_sub(1, Ordering::SeqCst);
        }),
        began: Mutex::new(Vec::new()),
    };
    round(made.as_slice(), &host, &handlers);
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
    let first_done: Rendezvous = Rendezvous::impatient(REPOSITORIES - 1, PATIENCE);
    let handlers: Handlers<'_> = Handlers {
        on_push: Box::new(|directory: &Path| {
            if directory == made[0].common.as_path() {
                assert!(first_done.wait_for_target(), "the other repositories were drained while the first waited");
            } else {
                first_done.arrive_without_waiting();
            }
        }),
        began: Mutex::new(Vec::new()),
    };
    let retries = round(made.as_slice(), &LARGE_HOST, &handlers);
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
    let running: AtomicUsize = AtomicUsize::new(0);
    let busiest: AtomicUsize = AtomicUsize::new(0);
    let overlapped: AtomicUsize = AtomicUsize::new(0);
    // A drain waits here for a second drain of the same repository: a lane rule that gives each directory a lane of its
    // own brings one, and a lane rule that holds them one after another cannot.
    let together: Rendezvous = Rendezvous::impatient(2, SERIAL_PATIENCE);
    let handlers: Handlers<'_> = Handlers {
        on_push: Box::new(|_directory: &Path| {
            busiest.fetch_max(running.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
            if together.arrive_and_give_back() {
                overlapped.fetch_add(1, Ordering::SeqCst);
            }
            running.fetch_sub(1, Ordering::SeqCst);
        }),
        began: Mutex::new(Vec::new()),
    };
    let directories: Vec<Assignment> = vec![Assignment::plain(made[0].common.clone()), Assignment::plain(linked.clone())];
    drain_directories(&RoundDrain { directories: directories.as_slice(), handlers: &handlers, now: &ClaimTime::read(), host: &LARGE_HOST });
    assert_eq!(handlers.began.lock().expect("pushes").len(), 2, "both directories were drained");
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
    /// in the order the pushes began.
    began: Mutex<Vec<PathBuf>>,
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
            RetryChange::Forget { git_dir, branch_ref } => format!("forget {} {branch_ref}", index_of(git_dir.as_path())),
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
    let work: DrainEachDirectory<'_, CouldPass> = DrainEachDirectory { handlers, now };
    let mut answer: RoundAnswer = RoundAnswer::default();
    let mut lines: Vec<String> = Vec::new();
    for (index, assignment) in directories.iter().enumerate() {
        let (wait, changes): (Option<Duration>, Vec<RetryChange>) = work.run(index, assignment, &mut lines);
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
    let round_handlers: CouldPass = CouldPass { began: Mutex::new(Vec::new()) };
    let serial_handlers: CouldPass = CouldPass { began: Mutex::new(Vec::new()) };
    let now: ClaimTime = ClaimTime::read();
    let parallel: RoundAnswer = drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: &round_handlers, now: &now, host: &LARGE_HOST });
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
    let handlers: CouldPass = CouldPass { began: Mutex::new(Vec::new()) };
    let directories: Vec<Assignment> = vec![
        Assignment { directory: made[0].common.clone(), retrying: Retrying { first: Some(FIRST_PAUSE), ..Retrying::default() } },
        Assignment { directory: linked.clone(), retrying: Retrying { first: Some(FIRST_PAUSE), ..Retrying::default() } },
    ];
    drain_directories(&RoundDrain { directories: directories.as_slice(), handlers: &handlers, now: &ClaimTime::read(), host: &LARGE_HOST });
    let began: Vec<PathBuf> = handlers.began.lock().expect("the order of the pushes").clone();
    assert_eq!(began, vec![made[0].common.clone(), linked.clone()], "the lane drained its directories in the order they were named");
    remove(root.as_path());
}

/// What:
///  A thread the system refuses partway through a round loses no directory:
/// every one is drained,
/// on the
///  calling thread,
/// and the answer is the one a round with every thread gives.
/// Why:
///   The owner's rule is that a thread is only a way to do the work sooner
/// (`foundation/scoped_start.rs`),
/// and a refusal that lost a directory would lose the person's push with it.
/// This is
///  the seam a refusal can be measured through:
/// `RefusedStarts` makes every scoped start from this thread fail,
/// as a process at its
///  limit does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = withRefusedStarts(() => round(dirs)); // every lane ran here
/// ```
#[test]
fn a_refused_thread_partway_through_a_round_leaves_no_directory_undrained() {
    let (refused_root, refused_assignments) = planned_world("round-refused-thread", REPOSITORIES);
    let (threaded_root, threaded_assignments) = planned_world("round-threaded", REPOSITORIES);
    let refused_handlers: CouldPass = CouldPass { began: Mutex::new(Vec::new()) };
    let threaded_handlers: CouldPass = CouldPass { began: Mutex::new(Vec::new()) };
    let now: ClaimTime = ClaimTime::read();
    let threaded: RoundAnswer = drain_directories(&RoundDrain {
        directories: threaded_assignments.as_slice(),
        handlers: &threaded_handlers,
        now: &now,
        host: &LARGE_HOST,
    });
    let refused: RoundAnswer = {
        let _refused: crate::scoped_start::RefusedStarts = crate::scoped_start::RefusedStarts::here();
        drain_directories(&RoundDrain { directories: refused_assignments.as_slice(), handlers: &refused_handlers, now: &now, host: &LARGE_HOST })
    };
    assert_eq!(rendered(&refused, refused_assignments.as_slice()), rendered(&threaded, threaded_assignments.as_slice()), "the answer a refused thread gives is the one a round with every thread gives");
    let began: Vec<PathBuf> = refused_handlers.began.lock().expect("the order of the pushes").clone();
    assert_eq!(
        began,
        refused_assignments.iter().map(|assignment: &Assignment| { return assignment.directory.clone(); }).collect::<Vec<PathBuf>>(),
        "every directory was drained in the order it was named, on the thread that could not start the others"
    );
    remove(refused_root.as_path());
    remove(threaded_root.as_path());
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
