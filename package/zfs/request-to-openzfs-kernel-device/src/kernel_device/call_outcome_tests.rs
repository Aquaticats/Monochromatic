//! What:
//!  The device-free proof of the call-outcome rules:
//! a listing's end is not a
//!  failure,
//! a read that names a bigger size is asked again,
//! a write that names
//!  one is refused rather than replayed,
//! per-name failures travel with the error
//!  number,
//! a failed write whose reply does not decode keeps the error number,
//! and a missing reply is a disagreement rather than an empty answer.
//! Why:
//!  These rules decide whether a mutating request is ever sent twice.
//! A test over
//!  the facts alone pins them without a pool,
//! and the guest fixture then proves the
//!  same rules against the kernel.

use super::call_outcome::{
    AnswerFacts, ReadFailure, interpret_read_failure, interpret_write_failure, require_reply,
    write_failure, write_failure_elements,
};
use super::device_error::DeviceError;
use crate::names::NameField;
use crate::nvlist::NvList;
use crate::requests::{
    ElementError, Error, INITIAL_REPLY_SIZE, Operation, ReplyError, ReplyRefusalReason, Request,
    checked_name,
};

#[test]
fn a_listing_that_reached_its_end_is_not_a_failure() {
    let failure = interpret_read_failure(facts(Operation::SnapshotListNext, libc::ESRCH, 0));
    assert!(matches!(failure, ReadFailure::End));
}

#[test]
fn esrch_on_a_request_that_does_not_list_is_a_failure() {
    let failure = interpret_read_failure(facts(Operation::ObjsetStats, libc::ESRCH, 0));
    let ReadFailure::Failed { source: error } = failure else {
        panic!("only a listing's end is not a failure");
    };
    assert_eq!(error.errno(), Some(libc::ESRCH));
}

#[test]
fn a_read_with_a_reply_past_the_buffer_asks_for_a_bigger_one() {
    let failure = interpret_read_failure(facts(
        Operation::GetBookmarks,
        libc::ENOMEM,
        INITIAL_REPLY_SIZE as u64 + 1,
    ));
    let ReadFailure::NeedsBiggerReply { requested } = failure else {
        panic!("a named size past the buffer is a request to grow it");
    };
    assert_eq!(requested, INITIAL_REPLY_SIZE as u64 + 1);
}

#[test]
fn a_read_with_enomem_and_no_bigger_size_is_a_failure() {
    let failure = interpret_read_failure(facts(
        Operation::GetBookmarks,
        libc::ENOMEM,
        INITIAL_REPLY_SIZE as u64,
    ));
    let ReadFailure::Failed { source: error } = failure else {
        panic!("a size the buffer already holds is not a reason to grow it");
    };
    assert_eq!(error.errno(), Some(libc::ENOMEM));
}

#[test]
fn a_write_with_a_reply_past_the_buffer_is_refused_not_replayed() {
    let error = interpret_write_failure(
        facts(
            Operation::DestroySnaps,
            libc::ENOMEM,
            INITIAL_REPLY_SIZE as u64 + 1,
        ),
        Vec::new(),
    );
    let Some(request) = error.request() else {
        panic!("a write failure is a request failure");
    };
    let crate::requests::Error::Refuse { source, .. } = request else {
        panic!("a write whose reply was too small is refused");
    };
    assert_eq!(source.reason, ReplyRefusalReason::NeverReplayAWrite);
    assert_eq!(source.requested, INITIAL_REPLY_SIZE as u64 + 1);
    assert_eq!(error.errno(), None);
}

#[test]
fn per_name_failures_travel_with_the_error_number() {
    let elements: Vec<ElementError> = vec![ElementError {
        name: String::from("pool/clone@origin"),
        errno: libc::EEXIST,
    }];
    let error = interpret_write_failure(
        facts(Operation::DestroySnaps, libc::EEXIST, 0),
        elements.clone(),
    );
    let Some(request) = error.request() else {
        panic!("a write failure is a request failure");
    };
    assert_eq!(request.errno(), Some(libc::EEXIST));
    assert_eq!(request.elements(), elements.as_slice());
    assert!(!request.is_end_of_list());
}

#[test]
fn a_write_failure_without_per_name_errors_names_its_number() {
    let error = interpret_write_failure(facts(Operation::Destroy, libc::EBUSY, 0), Vec::new());
    let Some(request) = error.request() else {
        panic!("a write failure is a request failure");
    };
    assert_eq!(request.errno(), Some(libc::EBUSY));
    assert!(request.elements().is_empty());
}

#[test]
fn a_write_failure_whose_reply_does_not_decode_keeps_the_kernel_error_number() {
    let refusal: Error = Error::Reply {
        operation: Operation::Clone,
        target: String::from("pool/clone"),
        source: ReplyError::Decode {
            value: NvList::unpack(&[u8::MAX; 4])
                .expect_err("four 0xff bytes are not a native list"),
        },
    };
    let elements: Vec<ElementError> = write_failure_elements(Err(refusal));
    assert!(
        elements.is_empty(),
        "a reply that did not decode names no failures"
    );
    let error = interpret_write_failure(facts(Operation::Clone, libc::E2BIG, 0), elements);
    assert_eq!(
        error.errno(),
        Some(libc::E2BIG),
        "the kernel's error number survives a reply that did not decode"
    );
}

#[test]
fn a_write_failure_keeps_the_failures_a_decodable_reply_names() {
    let mut reply: NvList = NvList::new();
    reply.add_i32("pool/clone@origin", libc::EEXIST);
    let elements: Vec<ElementError> = write_failure_elements(Ok(Some(reply)));
    assert_eq!(
        elements,
        vec![ElementError {
            name: String::from("pool/clone@origin"),
            errno: libc::EEXIST,
        }]
    );
}

#[test]
fn a_write_failure_without_a_reply_names_no_failures() {
    assert!(write_failure_elements(Ok(None)).is_empty());
}

#[test]
fn a_failed_write_whose_reply_does_not_decode_keeps_the_kernel_error_number() {
    let checked = checked_name(
        Operation::Clone,
        "pool/clone",
        NameField::CommandName,
        "pool/clone",
    )
    .expect("the name fits");
    let mut request: Request = Request::new(Operation::Clone, checked);
    // The kernel says it wrote four bytes, and they are not a native list.
    request.reply_mut().as_mut_bytes()[..4].copy_from_slice(&[u8::MAX; 4]);
    request.command_mut().zc_nvlist_dst_filled = 1;
    request.command_mut().zc_nvlist_dst_size = 4;
    let reported: u64 = request.command().zc_nvlist_dst_size;
    let held: usize = request.reply().len();
    let error: DeviceError = write_failure(&request, libc::E2BIG, reported, held);
    assert_eq!(
        error.errno(),
        Some(libc::E2BIG),
        "the kernel's error number survives a reply that did not decode"
    );
    let Some(request_error) = error.request() else {
        panic!("a write failure is a request failure");
    };
    assert!(
        request_error.elements().is_empty(),
        "a reply that did not decode names no failures"
    );
}

#[test]
fn a_read_without_a_list_is_a_disagreement() {
    let error = require_reply(Operation::ObjsetStats, "pool/dataset", None)
        .expect_err("a read must put a list in the buffer");
    assert!(matches!(error, DeviceError::MissingReply { .. }));
    assert!(error.to_string().contains("OBJSET_STATS"));
}

#[test]
fn a_read_with_a_list_passes_it_through() {
    let list: NvList = NvList::new();
    let read: NvList = require_reply(Operation::GetFsacl, "pool/dataset", Some(list))
        .expect("a list is what the read returns");
    assert!(read.iter().next().is_none());
}

/// The facts one answer supplies.
fn facts(operation: Operation, errno: i32, reported: u64) -> AnswerFacts<'static> {
    return AnswerFacts {
        operation,
        target: "pool/dataset",
        errno,
        reported_reply_size: reported,
        held: INITIAL_REPLY_SIZE,
    };
}
