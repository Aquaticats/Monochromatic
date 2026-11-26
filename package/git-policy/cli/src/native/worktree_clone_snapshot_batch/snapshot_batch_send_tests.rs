//! What:
//!  Controls for one chunk's send:
//! the empty set makes no call,
//! the chunk reaches the
//!  device exactly as the plan gave it,
//! and the answer comes back as per-name
//!  outcomes.
//! Why:
//!  The empty-set skip is the route's rule,
//! and its control is red without it:
//!  the recorder would see a call with no names.
//!  The device is a seam of the piece's own,
//! so a host run proves the call's
//!  shape and the answer's reading without a pool.
//!  The `defer` guard's red run cannot be
//!  built here,
//! because it reads the binding's constructor:
//! the round's keep
//!  folder records that run,
//! where a key added to the constructor made the
//!  guard refuse before the device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorder = new Recorder();
//! expect(sendRequest(recorder, pool, [])).toEqual({ kind: 'skipped' });
//! expect(recorder.calls).toHaveLength(0);
//! ```

use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::request_refusal::BatchRefusal;
use crate::worktree_clone_snapshot_batch::snapshot_batch_plan::plan_requests;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::{
    RequestResult,
    SnapshotDestroyDevice,
    send_request,
};
use crate::worktree_clone_snapshot_batch::snapshot_name_outcome::NameOutcome;
use monochromatic_zfs_request_to_openzfs_kernel_device::requests::ElementError;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    Error,
    NvList,
    Operation,
};
use std::cell::RefCell;

/// The pool every send names.
const POOL: &str = "zfsstep16";

/// The full snapshot names one chunk holds.
fn chunk() -> Vec<String> {
    return ["one", "two"]
        .iter()
        .map(|suffix| {
            return format!("{POOL}/clones/removal@{suffix}");
        })
        .collect();
}

/// What one call of the fake device saw.
type Call = (String, Vec<String>);

/// A device that records every call and answers with the given closure.
struct Recorder {
    /// The calls the piece made.
    calls: RefCell<Vec<Call>>,
    /// The answer the call returns.
    answer: Box<dyn Fn() -> Result<Option<NvList>, DeviceError>>,
}

/// The methods of `Recorder`.
impl Recorder {
    /// A device that answers with `answer` and records what it was asked.
    fn answering(answer: impl Fn() -> Result<Option<NvList>, DeviceError> + 'static) -> Recorder {
        return Recorder {
            calls: RefCell::new(Vec::new()),
            answer: Box::new(answer),
        };
    }
}

/// The one call the send makes.
impl SnapshotDestroyDevice for Recorder {
    /// Records the pool and names,
    /// then answers.
    fn destroy_snaps(
        &self,
        pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        self.calls.borrow_mut().push((
            pool.to_string(),
            names
                .iter()
                .map(|name| {
                    return name.to_string();
                })
                .collect(),
        ));
        return (self.answer)();
    }
}

/// An empty set is skipped and the device is never called.
#[test]
fn an_empty_set_is_skipped_without_calling_the_device() {
    let device: Recorder = Recorder::answering(|| {
        return Ok(None);
    });
    let result: RequestResult =
        send_request(&device, POOL, &[]).expect("an empty set refuses nothing");
    assert_eq!(result, RequestResult::Skipped);
    assert!(
        device.calls.borrow().is_empty(),
        "the skip must stand between the caller and the kernel"
    );
}

/// The chunk reaches the device exactly as the plan gave it.
#[test]
fn the_chunk_is_submitted_exactly_as_planned() {
    let set: Vec<String> = chunk();
    let budget: RequestBudget = RequestBudget::of_input_bytes(4096);
    let plan = plan_requests(POOL, set.as_slice(), budget).expect("the set fits");
    let device: Recorder = Recorder::answering(|| {
        return Ok(None);
    });
    let result: RequestResult =
        send_request(&device, POOL, plan.chunks()[0].as_slice()).expect("the chunk sends");
    assert_eq!(
        device.calls.borrow().as_slice(),
        [(POOL.to_string(), set.clone())]
    );
    let RequestResult::Answered(outcome) = result else {
        panic!("a chunk that fits is sent, not skipped");
    };
    assert_eq!(outcome.names().len(), set.len());
    assert!(outcome.names().iter().all(|named| {
        return named.outcome == NameOutcome::Destroyed;
    }));
}

/// A refused answer is read per name through the seam.
#[test]
fn a_refusal_is_answered_per_name() {
    let set: Vec<String> = chunk();
    let refused: String = set[1].clone();
    let device: Recorder = Recorder::answering(move || {
        return Err(DeviceError::Request {
            source: Error::Elements {
                operation: Operation::DestroySnaps,
                target: POOL.to_string(),
                errno: libc::EBUSY,
                elements: vec![ElementError {
                    name: refused.clone(),
                    errno: libc::EBUSY,
                }],
            },
        });
    });
    let result: RequestResult =
        send_request(&device, POOL, set.as_slice()).expect("the chunk sends");
    let RequestResult::Answered(outcome) = result else {
        panic!("a non-empty chunk is sent");
    };
    assert_eq!(
        outcome.outcome_of(set[1].as_str()),
        Some(&NameOutcome::Refused {
            errno: libc::EBUSY
        })
    );
    assert_eq!(
        outcome.outcome_of(set[0].as_str()),
        Some(&NameOutcome::Intact)
    );
}

/// A failure the answer cannot attribute is kept as unknown,
/// never as success.
#[test]
fn an_unattributable_failure_is_unknown() {
    let set: Vec<String> = chunk();
    let device: Recorder = Recorder::answering(|| {
        return Err(DeviceError::Request {
            source: Error::Call {
                operation: Operation::DestroySnaps,
                target: POOL.to_string(),
                errno: libc::EPERM,
            },
        });
    });
    let result: RequestResult =
        send_request(&device, POOL, set.as_slice()).expect("the chunk sends");
    let RequestResult::Answered(outcome) = result else {
        panic!("a non-empty chunk is sent");
    };
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}

/// A name the binding refuses stops the send before the device sees anything.
#[test]
fn a_name_the_binding_refuses_reaches_no_device() {
    let device: Recorder = Recorder::answering(|| {
        return Ok(None);
    });
    let refusal: BatchRefusal = send_request(&device, POOL, &[String::from("zfsstep16/x@a\0b")])
        .expect_err("a NUL cannot be packed");
    assert!(matches!(refusal, BatchRefusal::Unbuildable { .. }));
    assert!(device.calls.borrow().is_empty());
}
