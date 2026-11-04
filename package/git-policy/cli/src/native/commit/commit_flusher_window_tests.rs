//! What:
//!  Controls of the one-shot receipt flusher at a named decision point of its error path: a receipt that joins the lock holder after the
//!  holder has listed what is unforced, and the real second flusher that takes over in the release window.
//! Why:
//!  Review 9a sections 3D and 4: the holder of the flusher lock was the executor of every receipt that found the lock held, so a receipt
//!  created after the error handler's last listing had no executor and no record. The decision point is a fault-point callback, which is a
//!  plain function, so the lane it acts on is passed through a thread-local cell.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // faultPoint('flusher:after-error-snapshot', () => createReceipt('joiner'));
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_helper_launch_tests::flusher_launch_code;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_flusher::{FlushError, Flushed, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane_with, launch_flusher};
use git_policy_cli::commit_identity::set_fault_point;
use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

thread_local! {
    /// The receipt the fault point creates, when it is reached.
    static JOINER: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// The fault point: creates the pending joiner receipt at the named decision point.
fn create_joiner(name: &str) {
    if name == "flusher:after-error-snapshot"
        && let Some(path) = JOINER.with(|cell| { return cell.borrow_mut().take(); })
    {
        fs::write(path, b"receipt").expect("the joiner's receipt");
    }
}

/// The receipt names recorded in the lane's error record, in order.
fn recorded(lane: &Path) -> Vec<String> {
    let record: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).unwrap_or_default();
    return record.lines().map(|line| { return line.split('\t').next().expect("a name").to_string(); }).collect();
}

/// A lane with a receipt directory holding `names`.
fn lane_with(label: &str, names: &[&str]) -> PathBuf {
    let lane: PathBuf = scratch(label);
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    for name in names {
        fs::write(lane.join(RECEIPT_DIR).join(name), b"receipt").expect("receipt");
    }
    return lane;
}

/// Review 9a section 3D: forcing `b` fails; the holder has listed its unforced receipts; a client creates `x`, finds the lock held and
/// starts nothing; the holder records its listing and exits. The error path lists once more after it releases the lock, so `x` is recorded
/// (a client later than that finds the lock free and starts its own flusher).
#[test]
fn a_receipt_that_joins_after_the_error_listing_is_recorded() {
    let lane: PathBuf = lane_with("flusher-join-after-error", &["a", "b"]);
    JOINER.with(|cell| { *cell.borrow_mut() = Some(lane.join(RECEIPT_DIR).join("x")); });
    set_fault_point(Some(create_joiner));
    let result = flush_lane_with(
        &lane,
        &mut |path: &Path| {
            if path.file_name().is_some_and(|leaf| { return leaf == "b"; }) {
                return Err(io::Error::other("the disk refused"));
            }
            return Ok(());
        },
        &mut || {},
    );
    set_fault_point(None);
    assert!(matches!(result, Err(FlushError::Forcing(_))), "{:?}", result);
    let names: Vec<String> = recorded(&lane);
    assert!(names.iter().any(|name| { return name == "x"; }), "the joiner has a record: {:?}", names);
    assert_eq!(names.iter().filter(|name| { return name.as_str() == "x"; }).count(), 1, "and one record: {:?}", names);
}

thread_local! {
    /// The lane and the joiner of the launch-window controls, for their fault-point callback.
    static LAUNCH_WINDOW: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// The fault point after the launcher read the helper's report and before it lets go of its copy of the lock: waits (bounded) for any
/// helper that does not wait for the launcher to have recorded the first receipt's failure, then lets a client X create a receipt that
/// cannot be forced and launch a flusher (it finds the lock held).
fn client_joins_in_the_launch_window(name: &str) {
    if name != "launch:after-report" {
        return;
    }
    let Some(lane) = LAUNCH_WINDOW.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    let deadline: Instant = Instant::now() + Duration::from_millis(300);
    while !recorded(&lane).iter().any(|recorded_name| { return recorded_name == "r1"; }) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    launch_flusher(&lane).expect("client X's launch");
}

/// Review 10a section 3A, healthy path, kept as a regression after the launcher stopped holding the lock: a client that joins while a
/// launch is under way has an executor (it forks its own helper, or finds the lock held by the flusher that works, which lists after it
/// took the lock), so its receipt is forced or recorded.
#[test]
fn a_client_that_joins_during_the_launch_has_an_executor() {
    let lane: PathBuf = lane_with("flusher-launch-window", &[]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("r1")).expect("the launching client's receipt that cannot be forced");
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        set_fault_point(Some(client_joins_in_the_launch_window));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(0));
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !recorded(&lane).iter().any(|name| { return name == "x"; }) {
        assert!(Instant::now() < deadline, "client X has no executor and no record: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Review 10a section 3A, failed setup: clients that joined while the launcher held the lock for a helper whose setup then failed started
/// nothing. The launcher drops the lock and records every receipt it finds, joiners included, as unforced.
#[test]
fn a_client_that_joined_a_launch_whose_setup_failed_is_recorded() {
    let lane: PathBuf = lane_with("flusher-launch-failed", &["r1"]);
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        git_policy_cli::commit_helper_launch::inject_launch_fault(Some(git_policy_cli::commit_helper_launch::LaunchFault::Streams));
        set_fault_point(Some(client_joins_in_the_failed_launch));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(14), "the launch failed in the helper's setup");
    let names: Vec<String> = recorded(&lane);
    assert!(names.iter().any(|name| { return name == "x"; }), "the joiner has no record: {:?}", names);
    assert!(names.iter().any(|name| { return name == "r1"; }), "the launching client's own receipt has no record: {:?}", names);
}

/// The joiner of the failed launch: a receipt created and a launch that finds the lock held.
fn client_joins_in_the_failed_launch(name: &str) {
    if name != "launch:after-report" {
        return;
    }
    let Some(lane) = LAUNCH_WINDOW.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    fs::write(lane.join(RECEIPT_DIR).join("x"), b"receipt").expect("client X's receipt");
    // The injected setup failure applies to this nested launch too, which fails and records the lane's receipts as well.
    let _ = launch_flusher(&lane);
}

/// Review 10a section 4 and 9a section 4: the real handover, deterministic. The second executor is pinned across the handover decision
/// (it is held before its first forcing until the test releases it), so the first flusher finds the lock held, returns `Handed`, and the
/// second then does its own work and records what it cannot force.
#[test]
fn a_second_executor_pinned_across_the_recheck_decision_carries_the_receipt() {
    let lane: PathBuf = lane_with("flusher-pinned-handover", &["r1"]);
    let fifo: PathBuf = lane.join(RECEIPT_DIR).join("r2");
    let launch_lane: PathBuf = lane.clone();
    let mut launched: Option<i32> = None;
    let outcome: Flushed = flush_lane_with(
        &lane,
        &mut |_: &Path| { return Ok(()); },
        &mut || {
            std::os::unix::fs::symlink(launch_lane.join("nowhere"), &fifo).expect("a receipt that cannot be forced");
            let lane_for_child: PathBuf = launch_lane.clone();
            launched = in_single_threaded_process(move || {
                crate::commit_flusher_tests::hold_helpers_in(&lane_for_child);
                return flusher_launch_code(launch_flusher(&lane_for_child));
            });
        },
    )
    .expect("flush");
    assert_eq!(launched, Some(0), "the launch");
    assert_eq!(outcome, Flushed::Handed { forced: 1 });
    // The pinned executor forces its receipts once released; the one that cannot be forced is recorded.
    fs::write(lane.join("release"), b"").expect("the release");
    let deadline: Instant = Instant::now() + Duration::from_secs(10);
    while !recorded(&lane).iter().any(|name| { return name == "r2"; }) {
        assert!(Instant::now() < deadline, "the second executor recorded nothing: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The fault point after the launcher read the helper's report: waits (bounded) for any helper that already works to have finished with the
/// first receipt, then lets client X create a receipt that cannot be forced and launch a flusher.
fn client_joins_while_the_intermediate_child_is_stalled(name: &str) {
    if name != "launch:after-report" {
        return;
    }
    let Some(lane) = LAUNCH_WINDOW.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    let deadline: Instant = Instant::now() + Duration::from_millis(400);
    while !recorded(&lane).iter().any(|recorded_name| { return recorded_name == "r1"; }) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    launch_flusher(&lane).expect("client X's launch");
}

/// Review 11 section 3A, window B, kept as a regression: the intermediate child (paused after its second fork) must not make a client that
/// joins meanwhile lose its receipt. It carries no lock now, so X has an executor whatever the child does.
#[test]
fn a_client_that_joins_while_the_intermediate_child_is_stalled_has_an_executor() {
    let lane: PathBuf = lane_with("flusher-intermediate-copy", &[]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("r1")).expect("the launching client's receipt that cannot be forced");
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        git_policy_cli::commit_helper_launch::inject_launch_fault(Some(git_policy_cli::commit_helper_launch::LaunchFault::FirstChildHangs));
        set_fault_point(Some(client_joins_while_the_intermediate_child_is_stalled));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(0));
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !recorded(&lane).iter().any(|name| { return name == "x"; }) {
        assert!(Instant::now() < deadline, "client X has no executor and no record: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The fault point of the launcher-death control: client X joins while the launcher holds the lock, then the launcher dies before it sends
/// the handover byte.
fn client_joins_then_the_launcher_dies(name: &str) {
    if name != "launch:after-report" {
        return;
    }
    let Some(lane) = LAUNCH_WINDOW.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    launch_flusher(&lane).expect("client X's launch");
    // SAFETY: `_exit` takes an integer; the launcher dies here, with no accounting of its own.
    unsafe { libc::_exit(0) };
}

/// Review 12 section 3A, kept as a regression: the launcher dies after the helper reported ready, with nothing for the helper to wait for
/// or to be handed. Client X's receipt is forced or recorded by the helper, which takes the lock itself and lists after taking it.
#[test]
fn a_launcher_that_dies_right_after_the_helper_is_ready_leaves_no_receipt_without_an_executor() {
    let lane: PathBuf = lane_with("flusher-launcher-dies", &[]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("r1")).expect("the launching client's receipt that cannot be forced");
    let launch_lane: PathBuf = lane.clone();
    let _ = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        set_fault_point(Some(client_joins_then_the_launcher_dies));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !(recorded(&lane).iter().any(|name| { return name == "x"; }) && recorded(&lane).iter().any(|name| { return name == "r1"; })) {
        assert!(Instant::now() < deadline, "the receipts have no executor and no record: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Review 12 section 3B, the failure tail: the helper's setup failed, it wrote the failure byte and lingers before it exits, still holding
/// a copy of the lock. The launcher returned the error and recorded the receipts then visible; client X, arriving after that, found the lock
/// held by the lingering helper, which exits without any accounting. No process but the launcher holds the lock now, so X finds it free and
/// starts its own flusher.
#[test]
fn a_client_that_arrives_after_a_failed_launch_finds_the_lock_free_and_has_an_executor() {
    let lane: PathBuf = lane_with("flusher-failure-tail", &["r1"]);
    let launch_lane: PathBuf = lane.clone();
    let failed: Option<i32> = in_single_threaded_process(move || {
        git_policy_cli::commit_helper_launch::inject_launch_fault(Some(git_policy_cli::commit_helper_launch::LaunchFault::Streams));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(failed, Some(14), "the launch failed in the helper's setup");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    let second_lane: PathBuf = lane.clone();
    let second: Option<i32> = in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&second_lane)); });
    assert_eq!(second, Some(0));
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !recorded(&lane).iter().any(|name| { return name == "x"; }) {
        assert!(Instant::now() < deadline, "X found the lock held by an unaccounted holder: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Review 13 section 4: the helper acknowledged after `prepare()` whatever it returned, so a helper whose lock acquisition failed (the lock
/// file could not be opened) read as a started launch and then exited through the failure code with no receipt accounting. The acquisition
/// result is part of the acknowledgment: the launch fails and every receipt of the lane is recorded.
#[test]
fn a_helper_that_cannot_take_the_lock_is_a_failed_launch_with_every_receipt_recorded() {
    let lane: PathBuf = lane_with("flusher-helper-acquisition", &["r1", "r2"]);
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        // The client takes no lock; the helper's acquisition, the first one, fails.
        git_policy_cli::commit_flusher::inject_lock_failure_after(Some(0));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(22), "a helper that could not take the lock was reported as a started launch");
    let names: Vec<String> = recorded(&lane);
    assert!(names.iter().any(|name| { return name == "r1"; }) && names.iter().any(|name| { return name == "r2"; }), "{:?}", names);
}

/// The fault point of review 13 section 1 A: the launcher is paused after the helper reported ready, client X creates a receipt that
/// cannot be forced in the pause, and the launcher dies before it accounts for anything.
fn launcher_slow_then_dead(name: &str) {
    if name != "launch:after-report" {
        return;
    }
    let Some(lane) = LAUNCH_WINDOW.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    std::thread::sleep(Duration::from_millis(400));
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    let _ = launch_flusher(&lane);
    // SAFETY: `_exit` takes an integer; the launcher dies here, before it holds anything to account for.
    unsafe { libc::_exit(0) };
}

/// Review 13 section 1 A, kept as a regression: a slow launcher that then dies leaves nothing to account for, because it holds nothing.
/// Every receipt of the lane has an executor: the helper that works lists after it took the lock.
#[test]
fn a_slow_launcher_that_then_dies_leaves_every_receipt_with_an_executor() {
    let lane: PathBuf = lane_with("flusher-timeout-then-death", &[]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("r1")).expect("the launching client's receipt that cannot be forced");
    let launch_lane: PathBuf = lane.clone();
    let _ = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        set_fault_point(Some(launcher_slow_then_dead));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !(recorded(&lane).iter().any(|name| { return name == "x"; }) && recorded(&lane).iter().any(|name| { return name == "r1"; })) {
        assert!(Instant::now() < deadline, "a receipt has no executor and no record: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Review 13 section 1 B: the intermediate child inherits the lock when the launcher forks and can still hold that copy when the launcher
/// gives up on it (the kill is refused here, standing for a child that is not gone within the bound), drops its own copy and records.
/// Client X, arriving after that, found the lock held by the child, who never accounts. Nothing but the helper holds the lock now, so X
/// finds it free and starts its own flusher.
#[test]
fn a_client_that_arrives_after_a_launch_that_could_not_reap_its_intermediate_child_has_an_executor() {
    let lane: PathBuf = lane_with("flusher-unreaped-child", &["r1"]);
    let launch_lane: PathBuf = lane.clone();
    let failed: Option<i32> = in_single_threaded_process(move || {
        git_policy_cli::commit_helper_launch::inject_launch_fault(Some(git_policy_cli::commit_helper_launch::LaunchFault::KillRefused));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(failed, Some(12), "the launch failed because the intermediate child could not be killed");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    let second_lane: PathBuf = lane.clone();
    let second: Option<i32> = in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&second_lane)); });
    assert_eq!(second, Some(0));
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !recorded(&lane).iter().any(|name| { return name == "x"; }) {
        assert!(Instant::now() < deadline, "X found the lock held by the unreaped intermediate child: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Marks `name` in `dir` (a file that a process of a probe-window control writes to tell another one something).
fn mark(dir: &Path, name: &str) {
    fs::write(dir.join(name), b"x").expect("a mark");
}

/// Waits up to `limit` for the mark `name` in `dir`; whether it appeared.
fn marked(dir: &Path, name: &str, limit: Duration) -> bool {
    let deadline: Instant = Instant::now() + limit;
    while Instant::now() < deadline {
        if dir.join(name).exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    return dir.join(name).exists();
}

thread_local! {
    /// What the marks of a control are written in (one per process of a control).
    static PROBE_MARKS: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// The probe owner D of the negative controls: when it holds its probe it says so, waits for the release mark and dies before it forks
/// anything. Only the `legacy` probe seam emits `flusher:probe-held`; the fixed implementation never does.
fn probe_owner_dies_before_launch(name: &str) {
    if name != "flusher:probe-held" {
        return;
    }
    let Some(marks) = PROBE_MARKS.with(|cell| { return cell.borrow().clone(); }) else { return };
    mark(&marks, "d-holds");
    assert!(marked(&marks, "d-may-die", Duration::from_secs(5)), "the control never told D to die");
    // SAFETY: `_exit` takes an integer; the probe owner dies here, before it dropped the probe or forked a helper.
    unsafe { libc::_exit(0) };
}

/// A helper that waits (bounded) for D's announcement before its acquisition, and says so when it lost the lock.
fn helper_waits_for_the_probe(name: &str) {
    let Some(marks) = PROBE_MARKS.with(|cell| { return cell.borrow().clone(); }) else { return };
    match name {
        "flusher:helper-before-acquire" => {
            marked(&marks, "d-holds", Duration::from_millis(600));
        }
        "flusher:helper-lost-the-lock" => mark(&marks, "hc-done"),
        _ => {}
    }
}

/// A helper that only says so when it lost the lock.
fn helper_says_when_it_loses(name: &str) {
    if name != "flusher:helper-lost-the-lock" {
        return;
    }
    let Some(marks) = PROBE_MARKS.with(|cell| { return cell.borrow().clone(); }) else { return };
    mark(&marks, "hc-done");
}

/// Runs `body` in a single-threaded process on a thread of the test, so that the test goes on while it runs.
fn spawn_process<F: FnOnce() -> i32 + Send + 'static>(body: F) -> std::thread::JoinHandle<Option<i32>> {
    return std::thread::spawn(move || { return in_single_threaded_process(body); });
}

/// A lane without receipts and a directory for the marks.
fn probe_lane(label: &str) -> (PathBuf, PathBuf) {
    let lane: PathBuf = lane_with(label, &[]);
    let marks: PathBuf = scratch(&format!("{}-marks", label));
    return (lane, marks);
}

/// A receipt that cannot be forced (a dangling link): any executor that handles it records it.
fn unforceable(lane: &Path, name: &str) {
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join(name)).expect("a receipt that cannot be forced");
}

/// Waits until every name in `wanted` is recorded in the lane's error record.
fn wait_recorded(lane: &Path, wanted: &[&str]) {
    let deadline: Instant = Instant::now() + Duration::from_secs(6);
    while !wanted.iter().all(|name| { return recorded(lane).iter().any(|recorded_name| { return recorded_name == name; }); }) {
        assert!(Instant::now() < deadline, "a surviving client's receipt has no executor and no record: wanted {:?}, recorded {:?}", wanted, recorded(lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Asserts that none of `absent` is recorded after the time a live executor would have needed (an executor acts within milliseconds).
fn assert_never_recorded(lane: &Path, absent: &[&str]) {
    std::thread::sleep(Duration::from_millis(1500));
    for name in absent {
        assert!(!recorded(lane).iter().any(|recorded_name| { return recorded_name == name; }), "{} was recorded: {:?}", name, recorded(lane));
    }
}

/// Starts D, a client that launches a flusher, in a process of its own; `legacy` makes it probe the lock first, as the removed
/// implementation did.
fn start_d(lane: &Path, marks: &Path, legacy: bool) -> std::thread::JoinHandle<Option<i32>> {
    let (d_lane, d_marks) = (lane.to_path_buf(), marks.to_path_buf());
    return spawn_process(move || {
        PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(d_marks.clone()); });
        git_policy_cli::commit_flusher::inject_legacy_probe(legacy);
        set_fault_point(Some(probe_owner_dies_before_launch));
        return flusher_launch_code(launch_flusher(&d_lane));
    });
}

/// Runs E, a client with the receipt `e` already created, to its end in a process of its own.
fn run_e(lane: &Path, legacy: bool) -> Option<i32> {
    let e_lane: PathBuf = lane.to_path_buf();
    return in_single_threaded_process(move || {
        git_policy_cli::commit_flusher::inject_legacy_probe(legacy);
        return flusher_launch_code(launch_flusher(&e_lane));
    });
}

/// Review 14 section 2, the smallest schedule: D holds its successful probe, E creates a receipt, sees the lock held and starts nothing,
/// and D dies before it launches. Negative control, with the removed probe reproduced by the `legacy` seam: every coordination step is
/// asserted (D holds the probe, E's launch ends, D dies) and E's receipt is then never forced or recorded.
#[test]
fn the_removed_probe_loses_a_receipt_when_its_owner_dies_before_launching() {
    let (lane, marks) = probe_lane("flusher-probe-smallest-legacy");
    let d = start_d(&lane, &marks, true);
    assert!(marked(&marks, "d-holds", Duration::from_secs(3)), "D never held its probe: the schedule was not established");
    unforceable(&lane, "e");
    assert_eq!(run_e(&lane, true), Some(0), "E's launch");
    mark(&marks, "d-may-die");
    assert_eq!(d.join().expect("D's thread"), Some(0), "D died at its probe");
    assert_never_recorded(&lane, &["e"]);
}

/// The same schedule against the fixed implementation: D launches normally (it holds no probe, and says nothing), E's receipt is
/// recorded by an executor.
#[test]
fn the_fixed_launch_gives_a_client_an_executor_whatever_another_client_does() {
    let (lane, marks) = probe_lane("flusher-probe-smallest-current");
    let d = start_d(&lane, &marks, false);
    unforceable(&lane, "e");
    assert_eq!(run_e(&lane, false), Some(0), "E's launch");
    assert_eq!(d.join().expect("D's thread"), Some(0), "D's launch");
    assert!(!marks.join("d-holds").exists(), "the fixed implementation holds no probe");
    wait_recorded(&lane, &["e"]);
}

/// Review 14 section 1 against the fenced lane (plan step 3, owner decision 23): a helper launched for C is paused before its acquisition;
/// D, an old client, holds its probe on the old lock name; the helper's acquisition is of the new lock name and succeeds; E, a new client,
/// launches its own helper; D dies. Both receipts are recorded: an old probe holder cannot make a new helper lose, which is what the
/// fence is for. (Before the fence this schedule lost both receipts; the old clients' own loss, probe against probe, is the next test.)
#[test]
fn an_old_probe_cannot_make_a_new_helper_lose_the_lock_so_neither_receipt_is_lost() {
    let (lane, marks) = probe_lane("flusher-probe-helper-legacy");
    unforceable(&lane, "c");
    let (c_lane, c_marks) = (lane.clone(), marks.clone());
    let c = spawn_process(move || {
        PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(c_marks.clone()); });
        set_fault_point(Some(helper_waits_for_the_probe));
        return flusher_launch_code(launch_flusher(&c_lane));
    });
    std::thread::sleep(Duration::from_millis(100));
    let d = start_d(&lane, &marks, true);
    assert!(marked(&marks, "d-holds", Duration::from_secs(3)), "D never held its probe");
    unforceable(&lane, "e");
    assert_eq!(run_e(&lane, false), Some(0), "E's launch");
    mark(&marks, "d-may-die");
    assert_eq!(d.join().expect("D's thread"), Some(0), "D died at its probe");
    assert_eq!(c.join().expect("C's thread"), Some(0), "C's launch");
    assert!(!marks.join("hc-done").exists(), "C's helper lost the lock to an old holder: the fence does not hold");
    wait_recorded(&lane, &["c", "e"]);
}

thread_local! {
    /// The lane and the marks of the late-arrival control, for its fault-point callback.
    static LATE_ARGS: RefCell<Option<(PathBuf, PathBuf)>> = const { RefCell::new(None) };
    /// What C's launch returned in the late-arrival control.
    static LATE_CODE: RefCell<Option<Option<i32>>> = const { RefCell::new(None) };
}

/// At F's last empty sweep (F holds the lock): C exposes receipt R and launches a flusher whose helper says when it loses the lock.
fn client_exposes_a_receipt_at_the_last_empty_sweep(name: &str) {
    if name != "flusher:after-last-empty-sweep" {
        return;
    }
    let Some((lane, marks)) = LATE_ARGS.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    fs::write(lane.join(RECEIPT_DIR).join("r"), b"receipt").expect("receipt R");
    let code = in_single_threaded_process(move || {
        PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(marks.clone()); });
        set_fault_point(Some(helper_says_when_it_loses));
        return flusher_launch_code(launch_flusher(&lane));
    });
    LATE_CODE.with(|slot| { *slot.borrow_mut() = Some(code); });
}

/// Review 15 section 1, the real invariant, against the fixed implementation. F acquires the lock, sweeps an empty lane and is at its last
/// empty sweep (it holds the lock; the receipt does not exist yet). C exposes receipt R and launches a flusher; C's helper loses the
/// lock to F and ends. F then releases, rechecks, finds R and forces it. F acquired before R existed: R is covered by the release and
/// recheck path, not by a listing after F's acquisition. Every coordination step is asserted.
#[test]
fn a_receipt_exposed_after_the_last_empty_sweep_is_covered_by_the_holders_release_and_recheck() {
    let (lane, marks) = probe_lane("flusher-late-arrival");
    LATE_ARGS.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), marks.clone())); });
    set_fault_point(Some(client_exposes_a_receipt_at_the_last_empty_sweep));
    let forced: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let outcome = flush_lane_with(&lane, &mut |path: &Path| { forced.borrow_mut().push(path.file_name().expect("a leaf").to_string_lossy().into_owned()); return Ok(()); }, &mut || {});
    set_fault_point(None);
    let c_code: Option<Option<i32>> = LATE_CODE.with(|slot| { return slot.borrow_mut().take(); });
    assert_eq!(c_code, Some(Some(0)), "C's launch ran inside F's last empty sweep and was acknowledged");
    assert!(marks.join("hc-done").exists(), "C's helper never lost the lock to F: the schedule was not established");
    assert!(outcome.is_ok(), "{:?}", outcome);
    assert!(forced.borrow().iter().any(|name| { return name == "r"; }), "R was not forced by F's recheck: {:?}", forced.borrow());
}

/// Review 14 section 2 against the fenced lane: the flusher releases, a client's receipt appears, an old client D holds a probe on the old
/// lock name when the flusher rechecks; the recheck takes the new lock, which D cannot hold, so there is no handover to a holder that never
/// lists: the flusher forces and accounts for the receipt itself, and D's death changes nothing.
#[test]
fn an_old_probe_cannot_make_a_new_recheck_hand_a_receipt_over_to_a_holder_that_never_lists() {
    let (lane, marks) = probe_lane("flusher-probe-recheck-legacy");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt that can be forced");
    let (hook_lane, hook_marks) = (lane.clone(), marks.clone());
    let mut d: Option<std::thread::JoinHandle<Option<i32>>> = None;
    let outcome = flush_lane_with(
        &lane,
        &mut |path: &Path| { return fs::File::open(path).and_then(|file| { return file.sync_all(); }); },
        &mut || {
            if d.is_some() {
                // The hook runs at every release; the window of this control is the first.
                return;
            }
            unforceable(&hook_lane, "z");
            d = Some(start_d(&hook_lane, &hook_marks, true));
            assert!(marked(&hook_marks, "d-holds", Duration::from_secs(3)), "D never held its probe");
        },
    );
    assert!(!matches!(outcome, Ok(Flushed::Handed { .. })), "the recheck handed over to an old probe holder: {:?}", outcome);
    mark(&marks, "d-may-die");
    assert_eq!(d.expect("D").join().expect("D's thread"), Some(0), "D died at its probe");
    wait_recorded(&lane, &["z"]);
}

/// The same window against the fixed implementation: D launches a helper (no probe), the flusher's recheck either takes the lock again or
/// finds a helper that lists after it took it, and the receipt is recorded.
#[test]
fn the_fixed_launch_leaves_no_receipt_to_a_recheck_that_loses_the_lock() {
    let (lane, marks) = probe_lane("flusher-probe-recheck-current");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt that can be forced");
    let (hook_lane, hook_marks) = (lane.clone(), marks.clone());
    let mut d: Option<std::thread::JoinHandle<Option<i32>>> = None;
    let outcome = flush_lane_with(
        &lane,
        &mut |path: &Path| { return fs::File::open(path).and_then(|file| { return file.sync_all(); }); },
        &mut || {
            if d.is_some() {
                return;
            }
            unforceable(&hook_lane, "z");
            d = Some(start_d(&hook_lane, &hook_marks, false));
            std::thread::sleep(Duration::from_millis(100));
        },
    );
    assert!(outcome.is_ok() || matches!(outcome, Err(FlushError::Forcing(_))), "{:?}", outcome);
    mark(&marks, "d-may-die");
    assert_eq!(d.expect("D").join().expect("D's thread"), Some(0), "D's launch");
    wait_recorded(&lane, &["z"]);
}
