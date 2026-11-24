//! What:
//!  Controls for reading one request's answer as an outcome per name:
//! a success,
//!  a refusal whose per-name list names the failures,
//! a failure with no list at all,
//!  a shortened list,
//! and replies that contradict the request.
//! Why:
//!  The kernel's no-destruction-on-failure rule is what lets an unnamed name be
//!  `intact`,
//! and every answer that does not carry a complete list must leave the
//!  names it cannot speak for `unknown` rather than guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(outcomeOfAnswer(pool, names, refusal(EBUSY, [second])).names()[0].outcome).toEqual({ kind: 'intact' });
//! ```

use crate::worktree_clone_snapshot_batch::snapshot_name_outcome::{
    BatchOutcome, MORE_ERRORS_KEY, NameOutcome, outcome_of_answer,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::requests::{
    ElementError, MAXIMUM_ELEMENTS_KEPT, MAXIMUM_REPLY_SIZE, ReplyRefusal, ReplyRefusalReason,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError, Error, NvData, NvList, Operation,
};

/// The pool every answer names.
const POOL: &str = "zfsstep16";

/// One verified set of three names.
fn submitted() -> Vec<String> {
    return ["one", "two", "three"]
        .iter()
        .map(|suffix| {
            return format!("{POOL}/clones/removal@{suffix}");
        })
        .collect();
}

/// The first name of the set,
/// which the refusal controls fail.
fn second(set: &[String]) -> String {
    return set[1].clone();
}

/// The per-name failure of one snapshot.
fn failure(name: &str, errno: i32) -> ElementError {
    return ElementError {
        name: name.to_string(),
        errno,
    };
}

/// A success with no failures destroys every name.
#[test]
fn a_success_with_no_failures_destroys_every_name() {
    let set: Vec<String> = submitted();
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), Ok(Some(NvList::new())));
    assert_eq!(outcome.pool(), POOL);
    assert!(outcome.failures().is_empty());
    assert!(outcome.failures_complete());
    assert_eq!(outcome.errno(), None);
    assert!(outcome.names().iter().all(|named| {
        return named.outcome == NameOutcome::Destroyed;
    }));
    let absent: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), Ok(None));
    assert!(absent.names().iter().all(|named| {
        return named.outcome == NameOutcome::Destroyed;
    }));
}

/// A refused request names its failures and leaves the rest intact.
#[test]
fn a_refusal_names_its_failures_and_leaves_the_rest_intact() {
    let set: Vec<String> = submitted();
    let refused: String = second(set.as_slice());
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Request {
        source: Error::Elements {
            operation: Operation::DestroySnaps,
            target: POOL.to_string(),
            errno: libc::EBUSY,
            elements: vec![failure(refused.as_str(), libc::EBUSY)],
        },
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert_eq!(outcome.errno(), Some(libc::EBUSY));
    assert_eq!(outcome.failures(), [failure(refused.as_str(), libc::EBUSY)]);
    assert!(outcome.failures_complete());
    assert_eq!(
        outcome.outcome_of(refused.as_str()),
        Some(&NameOutcome::Refused { errno: libc::EBUSY })
    );
    assert_eq!(outcome.outcome_of(set[0].as_str()), Some(&NameOutcome::Intact));
    assert_eq!(outcome.outcome_of(set[2].as_str()), Some(&NameOutcome::Intact));
}

/// A failure without a per-name list is unknown for every name.
#[test]
fn a_failure_without_a_per_name_list_is_unknown() {
    let set: Vec<String> = submitted();
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Request {
        source: Error::Call {
            operation: Operation::DestroySnaps,
            target: POOL.to_string(),
            errno: libc::EPERM,
        },
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert_eq!(outcome.errno(), Some(libc::EPERM));
    assert!(outcome.failures().is_empty());
    assert!(!outcome.failures_complete());
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}

/// A reply too small to read is never replayed,
/// so its outcome is unknown.
#[test]
fn a_reply_too_small_for_a_write_is_unknown() {
    let set: Vec<String> = submitted();
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Request {
        source: Error::Refuse {
            operation: Operation::DestroySnaps,
            target: POOL.to_string(),
            source: ReplyRefusal {
                requested: 2048,
                held: 1024,
                limit: MAXIMUM_REPLY_SIZE,
                reason: ReplyRefusalReason::NeverReplayAWrite,
            },
        },
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}

/// A device that could not be opened made no request at all.
#[test]
fn an_open_failure_is_unknown() {
    let set: Vec<String> = submitted();
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Open {
        path: std::path::PathBuf::from("/dev/zfs"),
        source: std::io::Error::from_raw_os_error(libc::ENOENT),
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert_eq!(outcome.errno(), None);
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}

/// A failure naming a snapshot this request did not submit settles nothing.
#[test]
fn a_failure_naming_an_unsubmitted_snapshot_is_unknown() {
    let set: Vec<String> = submitted();
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Request {
        source: Error::Elements {
            operation: Operation::DestroySnaps,
            target: POOL.to_string(),
            errno: libc::EEXIST,
            elements: vec![failure("zfsstep16/clones/another@one", libc::EEXIST)],
        },
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}

/// A shortened list names its failures and leaves the other names unknown,
/// not intact.
#[test]
fn a_shortened_per_name_list_leaves_unnamed_names_unknown() {
    let set: Vec<String> = submitted();
    let refused: String = second(set.as_slice());
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Request {
        source: Error::Elements {
            operation: Operation::DestroySnaps,
            target: POOL.to_string(),
            errno: libc::EBUSY,
            elements: vec![
                failure(refused.as_str(), libc::EBUSY),
                failure(MORE_ERRORS_KEY, 5),
            ],
        },
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert!(!outcome.failures_complete());
    assert_eq!(outcome.failures(), [failure(refused.as_str(), libc::EBUSY)]);
    assert_eq!(
        outcome.outcome_of(refused.as_str()),
        Some(&NameOutcome::Refused { errno: libc::EBUSY })
    );
    assert!(matches!(
        outcome.outcome_of(set[0].as_str()),
        Some(NameOutcome::Unknown { .. })
    ));
}

/// A list at the binding's reader cap is not called complete.
#[test]
fn a_list_at_the_readers_cap_is_not_complete() {
    let set: Vec<String> = (0..(MAXIMUM_ELEMENTS_KEPT + 2))
        .map(|index| {
            return format!("{POOL}/clones/removal@{index}");
        })
        .collect::<Vec<String>>();
    let elements: Vec<ElementError> = set
        .iter()
        .take(MAXIMUM_ELEMENTS_KEPT)
        .map(|name| {
            return failure(name.as_str(), libc::EBUSY);
        })
        .collect();
    let answer: Result<Option<NvList>, DeviceError> = Err(DeviceError::Request {
        source: Error::Elements {
            operation: Operation::DestroySnaps,
            target: POOL.to_string(),
            errno: libc::EBUSY,
            elements,
        },
    });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), answer);
    assert!(!outcome.failures_complete());
    assert!(matches!(
        outcome.outcome_of(set[MAXIMUM_ELEMENTS_KEPT + 1].as_str()),
        Some(NameOutcome::Unknown { .. })
    ));
    assert!(matches!(
        outcome.outcome_of(set[0].as_str()),
        Some(NameOutcome::Refused { .. })
    ));
}

/// A success that names a failure contradicts the kernel and settles nothing.
#[test]
fn a_success_naming_a_failure_is_unknown() {
    let set: Vec<String> = submitted();
    let mut reply: NvList = NvList::new();
    reply.push(second(set.as_slice()), NvData::Int32 { value: libc::EBUSY });
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), Ok(Some(reply)));
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}

/// A success whose reply holds another shape is not the per-name list,
/// and settles
///  nothing.
#[test]
fn a_success_reply_of_another_shape_is_unknown() {
    let set: Vec<String> = submitted();
    let mut reply: NvList = NvList::new();
    reply.push(second(set.as_slice()), NvData::BooleanFlag);
    let outcome: BatchOutcome = outcome_of_answer(POOL, set.as_slice(), Ok(Some(reply)));
    assert!(outcome.names().iter().all(|named| {
        return matches!(named.outcome, NameOutcome::Unknown { .. });
    }));
}
