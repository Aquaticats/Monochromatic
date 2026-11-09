//! A round over several repositories:
//! the drains overlap in time,
//! the retry list keeps the order the directories were named,
//! the
//!  directories of one repository are drained one after another,
//! and the host budget bounds the overlap.

use super::*;
use crate::parallel_test_support::{PATIENCE, Rendezvous};
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
                repository: worktree.display().to_string(),
                git_dir: common.display().to_string(),
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
    on_push: Box<dyn Fn(&str) + Sync + 'a>,
    /// The Git directories pushed,
    /// in the order the pushes began.
    began: Mutex<Vec<String>>,
}

impl WorkHandlers for Handlers<'_> {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let directory: String = items.last().map(|item: &WorkItem| { return item.git_dir.clone(); }).unwrap_or_default();
        self.began.lock().expect("the order of the pushes").push(directory.clone());
        (self.on_push)(directory.as_str());
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
        on_push: Box::new(|_directory: &str| {
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
        on_push: Box::new(|_directory: &str| {
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
            repository: repository.root.join("worktree").display().to_string(),
            git_dir: repository.common.display().to_string(),
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
        on_push: Box::new(|directory: &str| {
            if directory == made[0].common.display().to_string() {
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
#[test]
fn the_directories_of_one_repository_are_drained_one_after_another() {
    let (root, made) = repositories("round-lane", 1);
    let linked: PathBuf = made[0].root.join("linked");
    std::fs::create_dir_all(&linked).expect("linked directory");
    std::fs::write(linked.join("commondir"), "../common\n").expect("commondir");
    let pending: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: made[0].root.join("worktree").display().to_string(),
        git_dir: linked.display().to_string(),
        branch_ref: BRANCH.to_string(),
        oid: format!("{:040x}", 0xbeef),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(linked.as_path()).as_path(), &pending).expect("enqueue");
    let running: AtomicUsize = AtomicUsize::new(0);
    let busiest: AtomicUsize = AtomicUsize::new(0);
    let handlers: Handlers<'_> = Handlers {
        on_push: Box::new(|_directory: &str| {
            busiest.fetch_max(running.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(150));
            running.fetch_sub(1, Ordering::SeqCst);
        }),
        began: Mutex::new(Vec::new()),
    };
    let directories: Vec<Assignment> = vec![Assignment::plain(made[0].common.clone()), Assignment::plain(linked.clone())];
    drain_directories(&RoundDrain { directories: directories.as_slice(), handlers: &handlers, now: &ClaimTime::read(), host: &LARGE_HOST });
    assert_eq!(handlers.began.lock().expect("pushes").len(), 2, "both directories were drained");
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
