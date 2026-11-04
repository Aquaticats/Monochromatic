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

use crate::commit_helper_launch_tests::launch_code;
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

/// Review 9a section 4: the release window with a real second flusher. A receipt that cannot be forced (a dangling link) is created in
/// the window and a launched flusher takes the free lock; the first flusher hands over, and the record of the failure is the second
/// flusher's.
#[test]
fn a_flusher_launched_in_the_release_window_carries_the_receipt_the_first_hands_over() {
    let lane: PathBuf = lane_with("flusher-real-handover", &["r1"]);
    let dangling: PathBuf = lane.join(RECEIPT_DIR).join("r2");
    let launch_lane: PathBuf = lane.clone();
    let mut launched: Option<i32> = None;
    let outcome: Flushed = flush_lane_with(
        &lane,
        &mut |_: &Path| { return Ok(()); },
        &mut || {
            std::os::unix::fs::symlink(launch_lane.join("nowhere"), &dangling).expect("a receipt that cannot be forced");
            let lane_for_child: PathBuf = launch_lane.clone();
            launched = in_single_threaded_process(move || { return launch_code(launch_flusher(&lane_for_child)); });
        },
    )
    .expect("flush");
    assert_eq!(launched, Some(0), "the launch");
    assert_eq!(outcome, Flushed::Handed { forced: 1 });
    let deadline: Instant = Instant::now() + Duration::from_secs(10);
    while !recorded(&lane).iter().any(|name| { return name == "r2"; }) {
        assert!(Instant::now() < deadline, "the second flusher recorded nothing: {:?}", recorded(&lane));
        std::thread::sleep(Duration::from_millis(5));
    }
}
