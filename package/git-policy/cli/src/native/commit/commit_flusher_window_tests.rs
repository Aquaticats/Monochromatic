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
    let joined = launch_flusher(&lane).expect("client X's launch");
    assert_eq!(joined, git_policy_cli::commit_flusher::Launched::LockHeld, "X found the lock held by the launcher's description");
}

/// Review 10a section 3A, healthy path: the helper reported ready; the launcher still holds its copy of the lock. A helper that did not
/// wait for the launcher to let go finished, released its description and exited, and client X, finding the lock held by the launcher's
/// copy, started nothing: no executor and no record. The helper now waits for the launcher's handover before it works, so X's receipt is
/// in the sweep it runs.
#[test]
fn a_client_that_joins_while_the_launcher_still_holds_its_copy_has_an_executor() {
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
    let joined = launch_flusher(&lane).expect("client X's launch");
    assert_eq!(joined, git_policy_cli::commit_flusher::Launched::LockHeld);
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

/// The fault point of the handover-timeout control: the launcher is paused after the helper reported ready, longer than the helper waits
/// for the handover (a short wait is injected). A helper that then ran its entry would finish, release and exit while the launcher's copy
/// still held the lock; client X, created after that, finds the lock held.
fn launcher_paused_past_the_helpers_wait(name: &str) {
    if name != "launch:after-report" {
        return;
    }
    let Some(lane) = LAUNCH_WINDOW.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    std::thread::sleep(Duration::from_millis(400));
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("x")).expect("client X's receipt that cannot be forced");
    let joined = launch_flusher(&lane).expect("client X's launch");
    assert_eq!(joined, git_policy_cli::commit_flusher::Launched::LockHeld, "X found the lock held by the launcher's description");
}

/// Review 11 section 3A, window A: an expired wait for the handover must not run the entry as though ownership were transferred. The
/// helper gives up without working, the launch reports that the handover was not acknowledged, and the launch-failure accounting records
/// every receipt of the lane, X's included.
#[test]
fn a_helper_whose_handover_wait_expires_runs_nothing_and_the_launch_accounts_for_every_receipt() {
    let lane: PathBuf = lane_with("flusher-handover-timeout", &[]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("r1")).expect("the launching client's receipt that cannot be forced");
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        git_policy_cli::commit_helper_launch::inject_launch_fault(Some(git_policy_cli::commit_helper_launch::LaunchFault::ShortHandoverWait));
        set_fault_point(Some(launcher_paused_past_the_helpers_wait));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(19), "the launch must report the unacknowledged handover");
    let names: Vec<String> = recorded(&lane);
    assert!(names.iter().any(|name| { return name == "x"; }), "X has no record: {:?}", names);
    assert!(names.iter().any(|name| { return name == "r1"; }), "the launching client's receipt has no record: {:?}", names);
}

/// The fault point after the handover byte was sent: waits (bounded) for any helper that already works to have finished with the first
/// receipt, then lets client X create a receipt that cannot be forced and launch a flusher.
fn client_joins_after_the_handover(name: &str) {
    if name != "launch:after-handover" {
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
fn no_copy_of_the_lock_but_the_executors_exists_when_the_handover_is_sent() {
    let lane: PathBuf = lane_with("flusher-intermediate-copy", &[]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("r1")).expect("the launching client's receipt that cannot be forced");
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        LAUNCH_WINDOW.with(|cell| { *cell.borrow_mut() = Some(launch_lane.clone()); });
        git_policy_cli::commit_helper_launch::inject_launch_fault(Some(git_policy_cli::commit_helper_launch::LaunchFault::FirstChildHangs));
        set_fault_point(Some(client_joins_after_the_handover));
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    assert_eq!(launched, Some(0));
    let deadline: Instant = Instant::now() + Duration::from_secs(5);
    while !recorded(&lane).iter().any(|name| { return name == "x"; }) {
        assert!(Instant::now() < deadline, "client X has no executor and no record: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}
