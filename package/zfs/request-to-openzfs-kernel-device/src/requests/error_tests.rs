//! What:
//!  The device-free proof of the failure type:
//! it reports the operation,
//! the
//!  target,
//! the kernel's error number and any per-name errors,
//! tells a
//!  listing's end from a failure,
//! and prints a hint for the common numbers.
//! Why:
//!  A removal or an add reports this text to a person;
//! a hint printed for the
//!  wrong operation,
//! or an end-of-listing taken for a failure,
//! would send them
//!  to the wrong remedy.

use super::*;
use crate::names::{NameError, NameField, NameProblem};
use crate::nvlist::NvError;
use crate::requests::element_error::ElementError;
use crate::requests::{ReplyError, ReplyRefusal, ReplyRefusalReason};

#[test]
fn a_call_failure_reports_operation_target_and_errno() {
    let error: Error = Error::Call {
        operation: Operation::ObjsetStats,
        target: "pool/dataset".to_string(),
        errno: libc::ENOENT,
    };
    assert_eq!(error.operation(), Operation::ObjsetStats);
    assert_eq!(error.target(), "pool/dataset");
    assert_eq!(error.errno(), Some(libc::ENOENT));
    assert!(error.elements().is_empty());
    assert!(error.to_string().contains("no such pool or dataset"));
}

#[test]
fn a_listing_end_is_not_a_failure_and_another_number_is() {
    let end: Error = Error::Call {
        operation: Operation::SnapshotListNext,
        target: "pool/dataset".to_string(),
        errno: libc::ESRCH,
    };
    assert!(end.is_end_of_list());
    assert!(end.to_string().contains("end of the listing"));
    let other: Error = Error::Call {
        operation: Operation::SnapshotListNext,
        target: "pool/dataset".to_string(),
        errno: libc::EIO,
    };
    assert!(!other.is_end_of_list());
}

#[test]
fn the_same_number_on_a_non_listing_is_a_failure() {
    let error: Error = Error::Call {
        operation: Operation::ObjsetStats,
        target: "pool/dataset".to_string(),
        errno: libc::ESRCH,
    };
    assert!(!error.is_end_of_list());
    assert!(!error.to_string().contains("end of the listing"));
}

#[test]
fn a_per_name_failure_keeps_every_element_and_prints_four() {
    let elements: Vec<ElementError> = (0..6)
        .map(|index| {
            return ElementError {
                name: format!("pool/ds@s{index}"),
                errno: libc::ENOENT,
            };
        })
        .collect();
    let error: Error = Error::Elements {
        operation: Operation::DestroySnaps,
        target: "pool".to_string(),
        errno: libc::ENOENT,
        elements,
    };
    assert_eq!(error.elements().len(), 6);
    let text: String = error.to_string();
    assert!(text.contains("pool/ds@s0"));
    assert!(text.contains("+2 more"));
}

#[test]
fn a_name_failure_carries_the_errno_the_kernel_would_have_given() {
    let error: Error = Error::Name {
        operation: Operation::Clone,
        target: "pool/clone".to_string(),
        source: NameError {
            field: NameField::CommandName,
            problem: NameProblem::Empty,
        },
    };
    assert_eq!(error.errno(), Some(libc::EINVAL));
    assert!(!error.to_string().contains("ObjsetStats"));
    assert!(error.to_string().contains("CLONE"));
}

#[test]
fn an_encode_failure_reports_the_operation_and_target() {
    let error: Error = Error::Encode {
        operation: Operation::Snapshot,
        target: "pool".to_string(),
        source: NvError::EmbeddedNul {
            name: "snaps".to_string(),
        },
    };
    assert_eq!(error.errno(), None);
    assert!(error.to_string().contains("encoding the request"));
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_some());
}

#[test]
fn a_reply_failure_reports_the_decode_error() {
    let error: Error = Error::Reply {
        operation: Operation::GetBookmarks,
        target: "pool/dataset".to_string(),
        source: ReplyError::LargerThanBuffer {
            reported: 10,
            capacity: 4,
        },
    };
    assert_eq!(error.errno(), None);
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_some());
}

#[test]
fn a_refusal_reports_the_operation_and_target() {
    let error: Error = Error::Refuse {
        operation: Operation::DestroySnaps,
        target: "pool".to_string(),
        source: ReplyRefusal {
            requested: 1,
            held: 0,
            limit: 1,
            reason: ReplyRefusalReason::NeverReplayAWrite,
        },
    };
    assert_eq!(error.operation(), Operation::DestroySnaps);
    assert_eq!(error.errno(), None);
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_some());
}

#[test]
fn the_common_error_numbers_get_their_hints() {
    let hints: [(i32, &str); 7] = [
        (libc::EPERM, "need root"),
        (libc::EACCES, "need root"),
        (libc::EEXIST, "already exists"),
        (libc::EBUSY, "busy"),
        (libc::ENAMETOOLONG, "name too long"),
        (libc::EINVAL, "invalid argument"),
        (libc::ENOSPC, "no space left"),
    ];
    for (errno, hint) in hints {
        let error: Error = Error::Call {
            operation: Operation::Destroy,
            target: "pool".to_string(),
            errno,
        };
        assert!(error.to_string().contains(hint), "errno {errno} hint");
    }
}

#[test]
fn the_memory_and_unknown_numbers_read_without_a_hint() {
    let error: Error = Error::Call {
        operation: Operation::Destroy,
        target: "pool".to_string(),
        errno: libc::ENOMEM,
    };
    assert!(error.to_string().contains("ran out of memory"));
    let unknown: Error = Error::Call {
        operation: Operation::Destroy,
        target: "pool".to_string(),
        errno: libc::EIO,
    };
    assert!(!unknown.to_string().contains("need root"));
    assert!(!unknown.to_string().contains("end of the listing"));
}

#[test]
fn a_checked_name_failure_can_be_built_from_the_packer_helper() {
    let long: String = "x".repeat(500);
    let error = crate::requests::checked_name(
        Operation::ChannelProgram,
        "pool",
        NameField::CommandString,
        &long,
    )
    .expect_err("500 bytes do not fit 256");
    assert_eq!(error.errno(), Some(libc::ENAMETOOLONG));
}
