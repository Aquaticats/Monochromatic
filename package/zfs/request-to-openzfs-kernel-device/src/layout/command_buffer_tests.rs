//! What:
//!  The device-free proof that a command buffer starts zeroed,
//! is 8-byte
//!  aligned,
//! holds at least the struct with spare room past it,
//! refuses a size
//!  past the cap,
//! and hands out the address of its first byte as a fixed-width
//!  number.
//! Why:
//!  The kernel copies its whole struct in and out of this address;
//! a short or
//!  misaligned buffer would corrupt memory,
//! and a nonzero byte would be read as
//!  a request field the caller never set.

use super::*;
use crate::layout::{COMMAND_STRUCT_SIZE, ZfsCmd};

#[test]
fn a_new_buffer_is_the_default_size_zeroed_and_word_aligned() {
    let mut buffer: CommandBuffer = CommandBuffer::new();
    assert_eq!(buffer.len(), DEFAULT_COMMAND_BUFFER_SIZE);
    assert!(buffer.len() >= COMMAND_STRUCT_SIZE);
    assert_eq!(buffer.len() % size_of::<u64>(), 0);
    assert_eq!(buffer.as_mut_abi_address() % 8, 0);
    assert_eq!(buffer.command().name(), "");
    assert_eq!(buffer.command().zc_cookie, 0);
    assert!(buffer.command().zc_name.iter().all(|byte| {
        return *byte == 0;
    }));
}

#[test]
fn a_requested_size_never_shrinks_below_the_struct() {
    let small: CommandBuffer = CommandBuffer::with_size(0).expect("zero bytes is allowed");
    assert_eq!(small.len(), COMMAND_STRUCT_SIZE);
    let odd: CommandBuffer =
        CommandBuffer::with_size(COMMAND_STRUCT_SIZE + 1).expect("room for one more byte");
    assert_eq!(odd.len(), COMMAND_STRUCT_SIZE + 8);
}

#[test]
fn a_size_past_the_cap_is_refused_with_both_numbers() {
    let refused = CommandBuffer::with_size(MAXIMUM_COMMAND_BUFFER_SIZE + 1)
        .expect_err("past the cap must be refused");
    assert_eq!(refused.requested, MAXIMUM_COMMAND_BUFFER_SIZE + 1);
    assert_eq!(refused.maximum, MAXIMUM_COMMAND_BUFFER_SIZE);
    assert!(refused.to_string().contains("exceeds"));
    let error: &dyn std::error::Error = &refused;
    assert!(error.to_string().contains("byte maximum"));
}

#[test]
fn the_address_is_the_first_byte_of_the_owned_words() {
    let mut buffer: CommandBuffer = CommandBuffer::with_size(32).expect("32 bytes is allowed");
    let address: u64 = buffer.as_mut_abi_address();
    // SAFETY: the address points at `buffer.len()` initialized zero bytes, and
    // the read stays inside them.
    let first: u64 = unsafe { std::ptr::read_unaligned(address as *const u64) };
    assert_eq!(first, 0);
}

#[test]
fn a_command_written_through_the_mutable_view_reads_back() {
    let mut buffer: CommandBuffer = CommandBuffer::new();
    buffer.command_mut().zc_cookie = 42;
    assert_eq!(buffer.command().zc_cookie, 42);
    assert_eq!(buffer.command().zc_nvlist_dst, 0);
}

#[test]
fn a_command_buffer_is_never_empty() {
    let buffer: CommandBuffer = CommandBuffer::with_size(0).expect("zero bytes is allowed");
    assert!(!buffer.is_empty());
    assert_eq!(buffer.len(), size_of::<ZfsCmd>());
}
