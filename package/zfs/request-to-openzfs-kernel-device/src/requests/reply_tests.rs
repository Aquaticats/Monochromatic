//! What:
//!  The device-free proof of the reply rule:
//! a read grows to the kernel's
//!  requested size up to the cap,
//! a write never grows,
//! a reported size past the
//!  cap or past the buffer is refused,
//! and an unfilled reply is no list rather
//!  than a failure.
//! Why:
//!  Growing a write's buffer would replay a snapshot or a destroy;
//! refusing to
//!  grow a read would turn a large property list into a failed read.

use super::*;
use crate::nvlist::NvList;

#[test]
fn a_read_starts_at_the_initial_size_and_a_write_doubles_its_input() {
    assert_eq!(initial_size(ReplyUse::Read, 0), INITIAL_REPLY_SIZE);
    assert_eq!(initial_size(ReplyUse::Read, 1_000_000), INITIAL_REPLY_SIZE);
    assert_eq!(initial_size(ReplyUse::Mutation, 0), INITIAL_REPLY_SIZE);
    assert_eq!(initial_size(ReplyUse::Mutation, 300_000), 600_000);
    assert_eq!(
        initial_size(ReplyUse::Mutation, MAXIMUM_REPLY_SIZE),
        MAXIMUM_REPLY_SIZE
    );
}

#[test]
fn a_new_buffer_reports_its_size_and_address() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    assert_eq!(buffer.len(), INITIAL_REPLY_SIZE);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.use_of_reply(), ReplyUse::Read);
    assert_eq!(buffer.as_abi_size(), INITIAL_REPLY_SIZE as u64);
    assert_eq!(buffer.as_mut_abi_address() % 8, 0);
    assert!(buffer.as_mut_bytes().iter().all(|byte| {
        return *byte == 0;
    }));
}

#[test]
fn a_read_grows_to_the_next_power_of_two() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    let grown: usize = buffer
        .grow_to_requested(300_000)
        .expect("300 KB is under the cap");
    assert_eq!(grown, 524_288);
    assert_eq!(buffer.len(), 524_288);
    assert_eq!(buffer.as_abi_size(), 524_288);
}

#[test]
fn a_request_no_larger_than_the_buffer_does_not_shrink_it() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    let size: usize = buffer
        .grow_to_requested(1)
        .expect("a smaller size is no request to grow");
    assert_eq!(size, INITIAL_REPLY_SIZE);
}

#[test]
fn a_read_past_the_cap_is_refused() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    let refusal: ReplyRefusal = buffer
        .grow_to_requested(MAXIMUM_REPLY_SIZE as u64 + 1)
        .expect_err("past the cap");
    assert_eq!(refusal.reason, ReplyRefusalReason::PastTheCap);
    assert_eq!(refusal.held, INITIAL_REPLY_SIZE);
    assert_eq!(refusal.limit, MAXIMUM_REPLY_SIZE);
    assert!(refusal.to_string().contains("allocates at most"));
}

#[test]
fn a_write_never_grows() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Mutation, 0);
    let refusal: ReplyRefusal = buffer
        .grow_to_requested(INITIAL_REPLY_SIZE as u64 + 1)
        .expect_err("a write is never replayed");
    assert_eq!(refusal.reason, ReplyRefusalReason::NeverReplayAWrite);
    assert!(refusal.to_string().contains("changes state"));
    assert_eq!(buffer.len(), INITIAL_REPLY_SIZE);
}

#[test]
fn a_write_that_already_fits_is_not_refused() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Mutation, 0);
    assert_eq!(
        buffer.grow_to_requested(10).expect("it fits"),
        INITIAL_REPLY_SIZE
    );
}

#[test]
fn an_unfilled_reply_is_no_list() {
    let buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    assert_eq!(
        buffer.decode(false, 0).expect("unfilled is not a failure"),
        None
    );
}

#[test]
fn a_filled_reply_decodes_the_bytes_the_kernel_left() {
    let mut list: NvList = NvList::new();
    list.add_u64("guid", 7);
    let packed: Vec<u8> = list.pack().expect("a simple list packs");
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    buffer.as_mut_bytes()[..packed.len()].copy_from_slice(&packed);
    let decoded: Option<NvList> = buffer
        .decode(true, packed.len() as u64)
        .expect("our own packing decodes");
    assert_eq!(decoded, Some(list));
}

#[test]
fn a_reported_size_past_the_buffer_is_refused() {
    let buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    let error: ReplyError = buffer
        .decode(true, INITIAL_REPLY_SIZE as u64 + 1)
        .expect_err("the kernel cannot have written past the buffer");
    assert!(matches!(
        error,
        ReplyError::LargerThanBuffer { capacity: _, .. }
    ));
    assert!(error.to_string().contains("holds"));
}

#[test]
fn a_filled_reply_that_is_not_a_list_is_refused() {
    let mut buffer: ReplyBuffer = ReplyBuffer::new(ReplyUse::Read, 0);
    buffer.as_mut_bytes()[..4].copy_from_slice(&[9, 9, 9, 9]);
    let error: ReplyError = buffer.decode(true, 4).expect_err("not a native list");
    assert!(matches!(error, ReplyError::Decode { value: _ }));
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_some());
}

#[test]
fn a_larger_than_buffer_error_has_no_source() {
    let error: ReplyError = ReplyError::LargerThanBuffer {
        reported: 10,
        capacity: 4,
    };
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_none());
}
