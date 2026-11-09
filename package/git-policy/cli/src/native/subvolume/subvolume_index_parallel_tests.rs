//! The rewrite of every index of a checkout side by side:
//! the rewrites overlap,
//! the report keeps the order of the checkout's
//!  index first and then each linked worktree's,
//! one index that cannot be rewritten leaves the others rewritten,
//! and the host
//!  budget bounds the overlap.

use super::*;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::subvolume_index_refresh_of_every_worktree::{AllIndexes, RewriteOne, refresh_all_indexes_awaiting, refresh_all_indexes_with};
use crate::parallel_test_support::{PATIENCE, Rendezvous, hold_the_only_place, room_for};
use crate::subvolume_error::MigrateError;
use crate::subvolume_test_support::make_checkout;
use crate::test_support::{fixture, git, remove};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Duration;

/// How many linked worktrees the checkout of these controls has;
/// the checkout's own index makes one more.
const LINKED: usize = 3;

/// The files the fixture checkout commits:
/// the readme,
/// the program,
/// the leaf,
/// the empty file,
/// the script,
/// the link and the ignore list.
/// Every index of the checkout and of its linked worktrees records them.
const TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT: usize = 7;

/// The entries of the fixture checkout's own index that a refresh rewrites,
/// measured on this fixture:
/// of the tracked files the checkout commits,
/// one is not rewritten here.
/// A legitimate change to the fixture or to the rewrite breaks the control that names this on purpose.
const CHECKOUT_ENTRIES_REFRESHED: usize = TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT - 1;

/// The entries of a linked worktree's index that a refresh rewrites,
/// measured on this fixture:
/// `git worktree add` writes that index from the files it checks out,
/// so a refresh finds nothing to rewrite.
const LINKED_ENTRIES_REFRESHED: usize = 0;

/// A host large enough that neither the CPUs nor the memory bound a run over `LINKED + 1` indexes.
const LARGE_HOST: HostResources = HostResources { parallelism: (LINKED + 1) * 8, memory_available: None };

/// A checkout with `LINKED` linked worktrees `w0`,
/// `w1`,
/// ... inside it,
/// each with an index.
fn checkout_with_linked(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    for index in 0..LINKED {
        git(root.as_path(), &["worktree", "add", "--quiet", "--detach", format!("w{index}").as_str()]);
    }
    return (parent, root);
}

/// Rewrite every index of `root` with `rewrite` on `host` and return the count and the lines.
fn refresh_with(root: &Path, host: &HostResources, rewrite: &Arc<RewriteOne>) -> (usize, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let request: AllIndexes = AllIndexes { root: root.to_path_buf(), lock: IndexLock::Acquire, references: Vec::new(), host: *host, rewrite: Arc::clone(rewrite) };
    let count: usize = refresh_all_indexes_with(&request, &mut |text: &str| { lines.push(text.to_string()); }).expect("the indexes were refreshed");
    return (count, lines);
}

/// The rewrites of all the indexes start before any has finished:
/// a point that needs all of them at once is passed only by
///  parallel work.
#[test]
fn the_indexes_are_rewritten_side_by_side() {
    let _room: usize = room_for("the_indexes_are_rewritten_side_by_side", LINKED + 1);
    let (parent, root) = checkout_with_linked("index-parallel-overlap");
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::new(LINKED + 1));
    let met: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (at_the_point, met_it) = (Arc::clone(&point), Arc::clone(&met));
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        if at_the_point.arrive() {
            met_it.fetch_add(1, Ordering::SeqCst);
        }
        return refresh_index_file_with(index, tree, options);
    });
    let (count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &rewrite);
    assert_eq!(count, LINKED + 1);
    assert_eq!(met.load(Ordering::SeqCst), LINKED + 1, "{lines:#?}");
    remove(parent.as_path());
}

/// How long the whole refresh of the order control may take.
/// Why:
///  The checkout's job waits for the three worktrees' rewrites,
/// which cost what this fixture and this host make them cost;
/// a
///  clock over that work fails a busy host while the pipeline is fine,
/// so the wait takes no clock and this bound covers the
///  whole run,
/// and a job that never arrives is a named failure rather than a hang.
const ORDERING_RUN_ALLOWED: Duration = Duration::from_secs(150);

/// The checkout's index is reported first and then each worktree's by path,
/// though the checkout's index finishes last.
#[test]
fn the_report_keeps_the_order_of_the_checkout_then_its_worktrees() {
    // The checkout's own rewrite holds one place and waits for the others, so one more must be able to run; with room for
    //  all of them the wait below cannot depend on the CPUs this machine has.
    // The checkout's own rewrite waits inside its job for the others, so it holds a place while it waits: the room is one
    //  place more than the indexes, and a run whose waiting item took the last place its peers need cannot happen.
    let _room: usize = room_for("the_report_keeps_the_order_of_the_checkout_then_its_worktrees", LINKED + 2);
    let (parent, root) = checkout_with_linked("index-parallel-order");
    let others_done: Arc<Rendezvous> = Arc::new(Rendezvous::new(LINKED));
    let main_index: PathBuf = root.join(".git").join("index");
    let (mine, others) = (main_index.clone(), Arc::clone(&others_done));
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        if index == mine.as_path() {
            // The peers arrive only after their own rewrites, so this waits for their work; a clock here would fail a host
            // whose work is slower than the clock, and `ORDERING_RUN_ALLOWED` bounds the run instead.
            others.wait_for_target_without_a_clock();
        }
        let line: String = refresh_index_file_with(index, tree, options);
        if index != mine.as_path() {
            others.arrive_without_waiting();
        }
        return line;
    });
    let refreshed = {
        let compared_root: PathBuf = root.clone();
        let held_rewrite: Arc<RewriteOne> = Arc::clone(&rewrite);
        crate::parallel_test_support::finished_within_at(ORDERING_RUN_ALLOWED, move || { return refresh_with(compared_root.as_path(), &LARGE_HOST, &held_rewrite); })
    };
    let Some((_count, lines)) = refreshed else {
        panic!("the checkout's index kept waiting for the worktrees' rewrites for longer than {ORDERING_RUN_ALLOWED:?}, so the report never finished; nothing here says the ordering is wrong, only that the run did not end")
    };
    assert_eq!(lines.len(), LINKED + 1, "{lines:#?}");
    assert!(lines[0].starts_with(format!("index {}:", main_index.display()).as_str()), "{lines:#?}");
    for index in 0..LINKED {
        assert!(lines[index + 1].starts_with(format!("index {}:", root.join(format!(".git/worktrees/w{index}/index")).display()).as_str()), "{lines:#?}");
    }
    remove(parent.as_path());
}

/// An index whose lock a Git command holds is skipped and said so;
/// every other index is rewritten.
#[test]
fn an_index_that_cannot_be_rewritten_leaves_the_others_rewritten() {
    let (parent, root) = checkout_with_linked("index-parallel-skip");
    std::fs::write(root.join(".git/worktrees/w0/index.lock"), b"another git").expect("lock");
    let (count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &(Arc::new(refresh_index_file_with) as Arc<RewriteOne>));
    assert_eq!(count, LINKED + 1);
    assert_eq!(lines.iter().filter(|line: &&String| { return line.contains("skipped (index.lock exists"); }).count(), 1, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.ends_with("entries refreshed"); }).count(), LINKED, "{lines:#?}");
    remove(parent.as_path());
}

/// The host's budget bounds the overlap:
/// on a host that may run few threads the overlap reaches that many and no more.
#[test]
fn the_budget_of_the_host_bounds_the_overlap() {
    let (parent, root) = checkout_with_linked("index-parallel-budget");
    let host: HostResources = HostResources { parallelism: 3, memory_available: None };
    let allowed: usize = worker_budget(&BudgetRequest { items: LINKED + 1, per_item_memory: 1, ceiling: None, cap: None, host: &host });
    assert!(allowed > 1 && allowed < LINKED + 1, "the control needs a bound between one and the indexes: {allowed}");
    // The rewrites reach that bound only when the process itself allows as many at once.
    let _room: usize = room_for("the_budget_of_the_host_bounds_the_overlap", allowed);
    let running: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let busiest: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    // The patience bounds only how long a run that never reaches the bound waits before it fails.
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(allowed, PATIENCE));
    let (live, most, at_the_point) = (Arc::clone(&running), Arc::clone(&busiest), Arc::clone(&point));
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        let now: usize = live.fetch_add(1, Ordering::SeqCst) + 1;
        most.fetch_max(now, Ordering::SeqCst);
        at_the_point.arrive_at_the_bound(allowed);
        let line: String = refresh_index_file_with(index, tree, options);
        live.fetch_sub(1, Ordering::SeqCst);
        return line;
    });
    let (_count, lines) = refresh_with(root.as_path(), &host, &rewrite);
    assert_eq!(busiest.load(Ordering::SeqCst), allowed, "{lines:#?}");
    remove(parent.as_path());
}

/// What a person reads is the report the code gave before this step,
/// line for line and in the order of the checkout's index first and then each worktree's by path.
#[test]
fn the_lines_of_a_run_are_the_ones_recorded_before_the_step() {
    let (parent, root) = checkout_with_linked("index-parallel-lines");
    let (count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &(Arc::new(refresh_index_file_with) as Arc<RewriteOne>));
    assert_eq!(count, LINKED + 1);
    let of_the_checkout: String = format!("{CHECKOUT_ENTRIES_REFRESHED} of {TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT} entries refreshed");
    let of_a_worktree: String = format!("{LINKED_ENTRIES_REFRESHED} of {TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT} entries refreshed");
    let mut expected: Vec<String> = vec![format!("index {}: {of_the_checkout}", root.join(".git").join("index").display())];
    for index in 0..LINKED {
        expected.push(format!("index {}: {of_a_worktree}", root.join(format!(".git/worktrees/w{index}")).join("index").display()));
    }
    assert_eq!(lines, expected, "the report of a run is no longer the one recorded before this step");
    remove(parent.as_path());
}

/// What:
///  Every rewrite runs inside the process's runtime,
/// as a job of its blocking pool,
/// and not on a thread of its own.
/// Why:
///   This is the whole of the conversion:
/// the rewrites used to run on scoped threads,
/// where the process's runtime is not entered at all,
///  and the runtime is what the owner asked every wait to sit on.
#[test]
fn the_rewrites_run_on_the_processs_runtime() {
    let (parent, root) = checkout_with_linked("index-parallel-runtime");
    let probe: Arc<std::sync::Mutex<Vec<bool>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let inside = Arc::clone(&probe);
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        inside.lock().expect("lock").push(tokio::runtime::Handle::try_current().is_ok());
        return refresh_index_file_with(index, tree, options);
    });
    let (_count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &rewrite);
    let seen: Vec<bool> = probe.lock().expect("lock").clone();
    assert_eq!(seen, vec![true; LINKED + 1], "a rewrite ran outside the process's runtime: {lines:#?}");
    remove(parent.as_path());
}

/// A rewrite waits for the one place the process's blocking pool has:
/// only a job of the pool takes one,
/// and work that ran on a worker of the runtime instead would start at once.
#[test]
fn the_rewrites_take_places_of_the_processs_bound_on_blocking_jobs() {
    let (parent, root) = checkout_with_linked("index-parallel-a-place-of-the-bound");
    let (given_back, finished) = hold_the_only_place();
    let started_after: Arc<std::sync::Mutex<Vec<bool>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let (began, given) = (Arc::clone(&started_after), Arc::clone(&given_back));
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        began.lock().expect("lock").push(given.load(Ordering::SeqCst));
        return refresh_index_file_with(index, tree, options);
    });
    let (count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &rewrite);
    assert_eq!(count, LINKED + 1, "{lines:#?}");
    let seen: Vec<bool> = started_after.lock().expect("lock").clone();
    assert_eq!(seen, vec![true; LINKED + 1], "a rewrite started while another job held the only place of the process's bound, so it did not take one: {lines:#?}");
    // The holder gives its place back when a job of this work asks for one, so this is the proof that the work
    //  reached the process's bound instead of starting at once.
    assert!(
        finished.recv_timeout(PATIENCE).expect("the job that holds the place never returned"),
        "no job of the work under test asked for a place of the process's bound, so it did not take one"
    );
    remove(parent.as_path());
}

/// What:
///  An index whose rewrite installed the file and then ended without a result is not reported as one nobody
///  touched.
/// Why:
///  "The indexes stay as they were" is a promise about the person's data;
///  a step that ended without a result after it installed its index breaks it,
///  so the report must keep that uncertainty.
#[test]
fn an_index_whose_rewrite_ended_without_a_result_is_not_promised_to_be_as_it_was() {
    let (parent, root) = checkout_with_linked("index-ends-without-a-result");
    let main_index: PathBuf = root.join(".git").join("index");
    let installed: Arc<std::sync::atomic::AtomicBool> = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (panicking, did_install) = (main_index.clone(), Arc::clone(&installed));
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        let report: RefreshReport = refresh_index_file_report(index, tree, options);
        if index == panicking.as_path() {
            // The index is installed and then the step ends without a result, as a defect inside it would.
            assert_eq!(report.status, RefreshStatus::Refreshed, "the control needs an index this run installs before it ends");
            did_install.store(true, Ordering::SeqCst);
            panic!("the rewrite installed its index and then panicked");
        }
        return report.line;
    });
    let mut lines: Vec<String> = Vec::new();
    let request: AllIndexes = AllIndexes { root: root.clone(), lock: IndexLock::Acquire, references: Vec::new(), host: LARGE_HOST, rewrite };
    let outcome: Result<usize, MigrateError> = refresh_all_indexes_with(&request, &mut |text: &str| { lines.push(text.to_string()); });
    assert!(installed.load(Ordering::SeqCst), "the rewrite of the checkout's index installed it before it ended, so a claim that it stays as it was is false");
    let refused: MigrateError = outcome.expect_err("a rewrite that ended without a result fails the call");
    let text: String = refused.to_string();
    assert!(!text.contains("stay as they were"), "the report promised an unchanged index although the rewrite had installed one: {text}");
    assert!(text.contains("not known"), "the report must keep the uncertainty about an index it cannot vouch for: {text}");
    assert!(lines.iter().any(|line: &String| { return line.contains("without a known result"); }), "the line of that index says its rewrite ended without a known result: {lines:#?}");
    remove(parent.as_path());
}

/// What:
///  A report that says an index rewrite could not be started leaves no job that can rewrite that index later.
///  With the preparation itself a job of the blocking pool (`prepare_the_run`),
/// a process with no thread to start now fails at the preparation,
///  so the rewrites are never even queued:
/// the control pins that no rewrite runs after the report either way,
///  and its message keeps the uncertainty.
/// Why:
///  The pool queues a job before it asks for a thread (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`),
///  so "no thread" means the job has not run *yet*:
/// it stays queued and the pool's next thread runs it.
/// A report may say an index stays as it was only when nothing that can write it is left queued,
/// which is what `TheClaimOnAJobsWork` decides.
/// The control runs in a child whose process limit is one,
/// because only a refused thread start puts a job in that queue.
#[test]
fn a_report_that_says_an_index_could_not_be_started_leaves_no_job_that_can_rewrite_it() {
    if !crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_report_that_says_an_index_could_not_be_started_leaves_no_job_that_can_rewrite_it") {
        return;
    }
    // A runtime with workers of its own, and a fixture made by a Git that is allowed to start.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    crate::lazy_runtime::run_on_runtime(async {}).expect("the runtime");
    let (parent, root) = checkout_with_linked("index-could-not-be-started");
    // Every thread this process starts is refused now, so each rewrite is queued and none can start.
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    let ran: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let counter: Arc<AtomicUsize> = Arc::clone(&ran);
    let rewrite: Arc<RewriteOne> = Arc::new(move |index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        counter.fetch_add(1, Ordering::SeqCst);
        return refresh_index_file_with(index, tree, options);
    });
    let mut lines: Vec<String> = Vec::new();
    let request: AllIndexes = AllIndexes { root: root.clone(), lock: IndexLock::Acquire, references: Vec::new(), host: LARGE_HOST, rewrite };
    let refused: MigrateError = refresh_all_indexes_with(&request, &mut |text: &str| { lines.push(text.to_string()); }).expect_err("a thread is refused for every rewrite");
    assert_eq!(ran.load(Ordering::SeqCst), 0, "no rewrite ran before a thread was free");
    let said: String = refused.to_string();
    // A thread may start again: the pool's first new thread runs what it queued before anything else.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises again");
    let drained = crate::lazy_runtime::run_on_runtime(async { return crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(|| ()).await; });
    assert!(matches!(drained, Ok(Ok(()))), "the job that starts the pool's first thread ran: {drained:?}");
    let ran_afterwards: usize = ran.load(Ordering::SeqCst);
    let promised_unchanged: bool = said.contains("given up on before it could start");
    assert!(
        !promised_unchanged || ran_afterwards == 0,
        "the report said the work of each index was given up on before it could start, and {ran_afterwards} of them ran afterwards, so the indexes are not as they were: {said}"
    );
    assert!(
        promised_unchanged || said.contains("not known") || said.contains("will not run"),
        "the report must say either that no rewrite could start or that the work it lost had not started: {said}"
    );
    remove(parent.as_path());
}

/// How long the control of the preparation waits for the refresh to give up on its own.
/// The refresh's own timeout is `REFRESH_STOP_BY`,
/// far below this,
/// so only a runtime blocked before the first
/// await can make this wait run out.
const PREPARATION_STOP_BY: Duration = Duration::from_secs(5);

/// The timeout the control puts around the refresh itself:
/// a preparation read that hangs must not outlast it.
const REFRESH_STOP_BY: Duration = Duration::from_millis(200);

/// What:
///  A refresh whose preparation read hangs gives up in bounded time,
///  because that read is a job of the blocking pool and never runs on the thread that polls the runtime.
/// Why:
///  The first thing `refresh_all_indexes_awaiting` reads is the repository's configuration,
/// with a synchronous
/// `std::fs::read` (`subvolume_index_files::repository_hash_length`).
/// Opening a named pipe with no writer blocks until one comes,
///  so on the runtime's own thread the future never reaches an await,
/// no timeout can expire,
/// and every other task
/// of that runtime stops with it ("nothing should ever wait forever").
#[test]
fn a_preparation_read_that_hangs_leaves_the_runtime_free_and_its_timeout_expires() {
    let (parent, root) = checkout_with_linked("index-preparation-hangs");
    let config: PathBuf = root.join(".git").join("config");
    let kept: PathBuf = root.join(".git").join("config.kept");
    std::fs::rename(&config, &kept).expect("the configuration is kept");
    let named = std::ffi::CString::new(config.to_str().expect("UTF-8")).expect("no NUL");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode, and touches no memory.
    assert_eq!(unsafe { libc::mkfifo(named.as_ptr(), 0o600) }, 0, "the named pipe at the configuration");
    let request: AllIndexes =
        AllIndexes { root: root.clone(), lock: IndexLock::Acquire, references: Vec::new(), host: LARGE_HOST, rewrite: Arc::new(refresh_index_file_with) };
    // The process's only place is held by another job, so the preparation this run sends to the pool never starts and no
    //  reader ever opens the pipe: the state a reviewer of this control found the cleanup hanging in.
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(crate::parallel_test_support::ONE_PLACE);
    let (holding, held) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let busy = crate::lazy_runtime::spawn_on_runtime(async move {
        let _outcome = crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = holding.send(());
            let _ = waiting.recv();
        })
        .await;
    });
    assert!(busy.is_some(), "the runtime has a worker, so a place of the blocking pool can be held");
    assert!(held.recv_timeout(PATIENCE).is_ok(), "the job that holds the only place never started");
    let (told, heard) = std::sync::mpsc::channel::<&'static str>();
    let refresh = std::thread::spawn(move || {
        // A runtime of its own, so a thread this control hangs cannot take the suite with it.
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
        let ended = runtime.block_on(async move {
            let mut sink: Vec<String> = Vec::new();
            let outcome = tokio::time::timeout(REFRESH_STOP_BY, refresh_all_indexes_awaiting(&request, &mut sink)).await;
            return match outcome {
                Ok(Ok(_count)) => "the refresh finished",
                Ok(Err(_refused)) => "the refresh failed",
                Err(_elapsed) => "the refresh gave up",
            };
        });
        let _told = told.send(ended);
        // The runtime keeps its blocking pool until it is dropped, and one job may still be opening the pipe.
        std::mem::forget(runtime);
    });
    let answered: Result<&'static str, std::sync::mpsc::RecvTimeoutError> = heard.recv_timeout(PREPARATION_STOP_BY);
    // The place goes back, so the job the pool queued is not left waiting in this process.
    let _let_go = release.send(());
    // The cleanup runs on a thread and is waited for with a bound, because it must not hang where no reader can come.
    let (cleaned, cleaning) = std::sync::mpsc::channel::<bool>();
    let pipe: PathBuf = config.clone();
    let tidy = std::thread::spawn(move || {
        // A writer, if a reader is blocked, gives it an empty file and closes; with no reader the open fails and this
        //  control says so instead of waiting for a writer's partner that cannot come.
        let opened = std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&pipe);
        let met_a_reader: bool = opened.is_ok();
        drop(opened);
        let joined: bool = refresh.join().is_ok();
        let _told = cleaned.send(met_a_reader && joined);
    });
    let cleanup_ended = cleaning.recv_timeout(PREPARATION_STOP_BY);
    // The thread is joined only once it has answered, because a thread blocked in the open cannot be joined at all.
    assert!(
        cleanup_ended.is_ok(),
        "the control's cleanup did not end within {} ms: opening the named pipe for a reader that can never come waits for ever",
        PREPARATION_STOP_BY.as_millis()
    );
    let met_a_reader: bool = cleanup_ended.expect("the cleanup answered");
    let _ended = tidy.join();
    assert!(!met_a_reader, "nothing opened the named pipe, so the cleanup's writer found no reader");
    std::fs::remove_file(&config).expect("the named pipe goes");
    std::fs::rename(&kept, &config).expect("the configuration comes back");
    remove(parent.as_path());
    let said: &'static str = answered.expect("the refresh answered within the control's wait");
    assert_eq!(said, "the refresh gave up", "a preparation read that hangs must be a job of the blocking pool, so the runtime stays free and the refresh's own timeout expires");
}
