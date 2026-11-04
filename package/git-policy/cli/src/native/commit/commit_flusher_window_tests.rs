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

/// Review 10a section 3A, healthy path: the helper reported ready; the launcher still holds its copy of the lock. A helper that did not
/// wait for the launcher to let go finished, released its description and exited, and client X, finding the lock held by the launcher's
/// copy, started nothing: no executor and no record. The helper now waits for the launcher's handover before it works, so X's receipt is
/// in the sweep it runs.
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
/// (its forcing of a FIFO receipt blocks until the test opens a writer), so the first flusher finds the lock held, returns `Handed`, and the
/// second then does its own work and records what it cannot force.
#[test]
fn a_second_executor_pinned_across_the_handover_decision_carries_the_receipt() {
    let lane: PathBuf = lane_with("flusher-pinned-handover", &["r1"]);
    let fifo: PathBuf = lane.join(RECEIPT_DIR).join("r2");
    let launch_lane: PathBuf = lane.clone();
    let mut launched: Option<i32> = None;
    let outcome: Flushed = flush_lane_with(
        &lane,
        &mut |_: &Path| { return Ok(()); },
        &mut || {
            {
                use std::os::unix::ffi::OsStrExt;
                let path = std::ffi::CString::new(fifo.as_os_str().as_bytes()).expect("a path without NUL");
                // SAFETY: `mkfifo` takes a NUL-terminated path and a mode.
                assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0, "mkfifo");
            }
            let lane_for_child: PathBuf = launch_lane.clone();
            launched = in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&lane_for_child)); });
        },
    )
    .expect("flush");
    assert_eq!(launched, Some(0), "the launch");
    assert_eq!(outcome, Flushed::Handed { forced: 1 });
    // The pinned executor's `open` of the FIFO returns once a writer exists; forcing a FIFO then fails and the failure is recorded.
    let writer = fs::OpenOptions::new().read(true).write(true).open(&fifo).expect("a writer releases the pinned executor");
    let deadline: Instant = Instant::now() + Duration::from_secs(10);
    while !recorded(&lane).iter().any(|name| { return name == "r2"; }) {
        assert!(Instant::now() < deadline, "the second executor recorded nothing: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(writer);
}

/// The fault point after the handover byte was sent: waits (bounded) for any helper that already works to have finished with the first
/// receipt, then lets client X create a receipt that cannot be forced and launch a flusher.
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

/// Review 11 section 3A, window B: the intermediate child (paused after its second fork) still held a copy of the lock when the handover
/// byte was sent. The executor finished and exited, client X found the lock held by the child's copy, and the launcher then killed the
/// child and returned success: X had no executor and no record. Every copy that is not the executor's is gone before the handover byte.
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
    // SAFETY: `_exit` takes an integer; the launcher dies here, before the handover, with no accounting of its own.
    unsafe { libc::_exit(0) };
}

/// Review 12 section 3A: the launcher dies after the helper reported ready and before the handover byte. The helper reads the end of the
/// handover pipe and used to exit without working, with nobody left to record X, who found the lock held by the dead launcher's
/// description. The helper takes the lock itself after the handover, whatever its outcome, and works when it gets it.
#[test]
fn a_launcher_that_dies_before_the_handover_leaves_no_receipt_without_an_executor() {
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
        // The launcher's own acquisition succeeds; the helper's, the next one, fails.
        git_policy_cli::commit_flusher::inject_lock_failure_after(Some(1));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(22), "a helper that could not take the lock was reported as a started launch");
    let names: Vec<String> = recorded(&lane);
    assert!(names.iter().any(|name| { return name == "r1"; }) && names.iter().any(|name| { return name == "r2"; }), "{:?}", names);
}

/// The fault point of review 13 section 1 A: the launcher is paused longer than the helper waits for the handover (a short wait is
/// injected), client X creates a receipt that cannot be forced in the pause, and the launcher dies before it accounts for anything.
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

/// Review 13 section 1 A: the helper's handover wait expires while the launcher is slow, the helper finds the launcher's lock taken and
/// ends, and the launcher dies before it accounts. Neither process was left responsible for X, who found the lock held. No process holds
/// the lock for the launch any more, so X finds it free (or held by the helper that works) and has an executor, and the receipts of the
/// lane are not left to a launcher that may not survive.
#[test]
fn a_helper_that_times_out_while_the_launcher_is_slow_and_then_loses_it_leaves_every_receipt_with_an_executor() {
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
    /// The lane of the probe-window controls (one per process of a control).
    static PROBE_LANE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    /// What the control's marks are written in.
    static PROBE_MARKS: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// The probe owner D of the controls of review 14: when it holds its probe it says so, waits for the release mark (bounded) and dies
/// before it forks anything.
fn probe_owner_dies_before_launch(name: &str) {
    if name != "flusher:probe-held" {
        return;
    }
    let Some(marks) = PROBE_MARKS.with(|cell| { return cell.borrow().clone(); }) else { return };
    mark(&marks, "d-holds");
    marked(&marks, "d-may-die", Duration::from_secs(3));
    // SAFETY: `_exit` takes an integer; the probe owner dies here, before it dropped the probe or forked a helper.
    unsafe { libc::_exit(0) };
}

/// A helper paused before its acquisition (it waits for D's announcement, bounded) and that says so when it lost the lock.
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

/// Runs `body` in a single-threaded process on a thread of the test, so that the test goes on while it runs.
fn spawn_process<F: FnOnce() -> i32 + Send + 'static>(body: F) -> std::thread::JoinHandle<Option<i32>> {
    return std::thread::spawn(move || { return in_single_threaded_process(body); });
}

/// A lane with the receipt `c` that cannot be forced, and a directory for the marks.
fn probe_lane(label: &str) -> (PathBuf, PathBuf) {
    let lane: PathBuf = lane_with(label, &[]);
    let marks: PathBuf = scratch(&format!("{}-marks", label));
    return (lane, marks);
}

/// Waits until every name in `wanted` is recorded in the lane's error record.
fn wait_recorded(lane: &Path, wanted: &[&str]) {
    let deadline: Instant = Instant::now() + Duration::from_secs(6);
    while !wanted.iter().all(|name| { return recorded(lane).iter().any(|recorded_name| { return recorded_name == name; }); }) {
        assert!(Instant::now() < deadline, "a surviving client's receipt has no executor and no record: wanted {:?}, recorded {:?}", wanted, recorded(lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Review 14 section 2, the smallest schedule: D holds its successful probe, E creates a receipt, sees the lock held and starts nothing,
/// and D dies before it launches. A probe is not an executor, so no client treats a held lock as proof of one: E's receipt is forced or
/// recorded.
#[test]
fn a_client_that_sees_a_probe_hold_the_lock_still_has_an_executor_when_the_probe_owner_dies() {
    let (lane, marks) = probe_lane("flusher-probe-smallest");
    let (d_lane, d_marks) = (lane.clone(), marks.clone());
    let d = spawn_process(move || {
        PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(d_marks.clone()); });
        set_fault_point(Some(probe_owner_dies_before_launch));
        return flusher_launch_code(launch_flusher(&d_lane));
    });
    marked(&marks, "d-holds", Duration::from_secs(2));
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("e")).expect("E's receipt that cannot be forced");
    let e_lane: PathBuf = lane.clone();
    let e: Option<i32> = in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&e_lane)); });
    mark(&marks, "d-may-die");
    let _ = d.join();
    assert_eq!(e, Some(0), "E's launch");
    wait_recorded(&lane, &["e"]);
}

/// Review 14 section 1: a helper launched for C is paused before its acquisition; D acquires its probe; the helper's acquisition fails
/// against the probe and it ends; E, arriving meanwhile, sees the lock held; D dies. Neither c nor e may be left to a probe.
#[test]
fn a_helper_that_loses_the_lock_to_a_probe_and_a_client_that_sees_it_do_not_leave_receipts_to_the_probe_owner() {
    let (lane, marks) = probe_lane("flusher-probe-helper");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("c")).expect("C's receipt that cannot be forced");
    let (c_lane, c_marks) = (lane.clone(), marks.clone());
    let c = spawn_process(move || {
        PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(c_marks.clone()); });
        set_fault_point(Some(helper_waits_for_the_probe));
        return flusher_launch_code(launch_flusher(&c_lane));
    });
    std::thread::sleep(Duration::from_millis(100));
    let (d_lane, d_marks) = (lane.clone(), marks.clone());
    let d = spawn_process(move || {
        PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(d_marks.clone()); });
        set_fault_point(Some(probe_owner_dies_before_launch));
        return flusher_launch_code(launch_flusher(&d_lane));
    });
    marked(&marks, "hc-done", Duration::from_millis(1500));
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("e")).expect("E's receipt that cannot be forced");
    let e_lane: PathBuf = lane.clone();
    let e: Option<i32> = in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&e_lane)); });
    mark(&marks, "d-may-die");
    let _ = d.join();
    let _ = c.join();
    assert_eq!(e, Some(0), "E's launch");
    wait_recorded(&lane, &["c", "e"]);
}

/// Review 14 section 2, the release and recheck takeover window: the flusher releases the lock after its last empty sweep, a client's
/// receipt appears, a client D holds a probe when the flusher rechecks and tries to take the lock again, and D dies. The failed
/// reacquisition is not a handover to a probe: the receipt is forced or recorded.
#[test]
fn a_recheck_that_loses_the_lock_to_a_probe_does_not_leave_the_receipt_to_the_probe_owner() {
    let (lane, marks) = probe_lane("flusher-probe-recheck");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt that can be forced");
    let (hook_lane, hook_marks) = (lane.clone(), marks.clone());
    let mut d: Option<std::thread::JoinHandle<Option<i32>>> = None;
    let _ = flush_lane_with(
        &lane,
        &mut |path: &Path| { return fs::File::open(path).and_then(|file| { return file.sync_all(); }); },
        &mut || {
            std::os::unix::fs::symlink(hook_lane.join("nowhere"), hook_lane.join(RECEIPT_DIR).join("z")).expect("a receipt that cannot be forced");
            let (d_lane, d_marks) = (hook_lane.clone(), hook_marks.clone());
            d = Some(spawn_process(move || {
                PROBE_MARKS.with(|cell| { *cell.borrow_mut() = Some(d_marks.clone()); });
                set_fault_point(Some(probe_owner_dies_before_launch));
                return flusher_launch_code(launch_flusher(&d_lane));
            }));
            marked(&hook_marks, "d-holds", Duration::from_millis(800));
        },
    );
    mark(&marks, "d-may-die");
    if let Some(handle) = d {
        let _ = handle.join();
    }
    wait_recorded(&lane, &["z"]);
}
