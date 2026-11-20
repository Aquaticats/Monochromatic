//! What:
//!  The device-free proof that a request owns its command,
//! input and reply
//!  buffers and hands out the addresses of those owned bytes as fixed-width
//!  numbers,
//! that the input list survives independently of the list it was
//!  packed from,
//! and that growing a reply rewrites the command's address.
//! Why:
//!  A temporary list or buffer would leave the kernel reading freed memory;
//! the
//!  addresses are the only thing the kernel sees,
//! so they are what the tests
//!  compare.

use super::*;
use crate::names::NameField;
use crate::nvlist::{NvData, NvList};
use crate::requests::{Error, INITIAL_REPLY_SIZE};

#[test]
fn a_new_request_names_its_target_and_clears_both_addresses() {
    let checked = checked_name(
        Operation::ObjsetStats,
        "pool/dataset",
        NameField::CommandName,
        "pool/dataset",
    )
    .expect("the name fits");
    let request: Request = Request::new(Operation::ObjsetStats, checked);
    assert_eq!(request.operation(), Operation::ObjsetStats);
    assert_eq!(request.target(), "pool/dataset");
    assert_eq!(request.command().name(), "pool/dataset");
    assert_eq!(request.command().zc_nvlist_src, 0);
    assert_eq!(request.command().zc_nvlist_src_size, 0);
    assert!(request.input_bytes().is_empty());
    assert_eq!(request.reply().len(), INITIAL_REPLY_SIZE);
}

#[test]
fn the_command_names_the_owned_reply_buffer() {
    let checked = checked_name(Operation::GetFsacl, "pool", NameField::CommandName, "pool")
        .expect("the name fits");
    let mut request: Request = Request::new(Operation::GetFsacl, checked);
    let address: u64 = request.reply_mut().as_mut_abi_address();
    let size: u64 = request.reply().as_abi_size();
    assert_eq!(request.command().zc_nvlist_dst, address);
    assert_eq!(request.command().zc_nvlist_dst_size, size);
    assert_eq!(request.command().zc_nvlist_dst_filled, 0);
}

#[test]
fn an_attached_input_is_owned_and_named_by_the_command() {
    let checked = checked_name(Operation::SetProp, "pool", NameField::CommandName, "pool")
        .expect("the name fits");
    let mut request: Request = Request::new(Operation::SetProp, checked);
    let mut properties: NvList = NvList::new();
    properties.add_str("monochromatic:desired-state", "pruned");
    request.attach_input(&properties).expect("the list packs");
    let packed: Vec<u8> = properties.pack().expect("the same list packs again");
    assert_eq!(request.input_bytes(), packed);
    assert_eq!(
        request.command().zc_nvlist_src,
        request.input_bytes().as_ptr() as u64
    );
    assert_eq!(request.command().zc_nvlist_src_size, packed.len() as u64);
}

#[test]
fn a_write_sizes_its_reply_for_its_own_input() {
    let checked = checked_name(
        Operation::DestroySnaps,
        "pool",
        NameField::CommandName,
        "pool",
    )
    .expect("the name fits");
    let mut request: Request = Request::new(Operation::DestroySnaps, checked);
    let mut input: NvList = NvList::new();
    input.add_str("padding", "x".repeat(200_000));
    request.attach_input(&input).expect("the list packs");
    let expected: usize = INITIAL_REPLY_SIZE.max(request.input_bytes().len() * 2);
    assert_eq!(request.reply().len(), expected);
}

#[test]
fn growing_a_read_reply_rewrites_the_command_address_and_size() {
    let checked = checked_name(Operation::GetFsacl, "pool", NameField::CommandName, "pool")
        .expect("the name fits");
    let mut request: Request = Request::new(Operation::GetFsacl, checked);
    let grown: usize = request.grow_reply(300_000).expect("a read may grow");
    assert_eq!(grown, 524_288);
    let reply_address: u64 = request.reply_mut().as_mut_abi_address();
    let command_address: u64 = request.command().zc_nvlist_dst;
    assert_eq!(command_address, reply_address);
    assert_eq!(request.command().zc_nvlist_dst_size, 524_288);
}

#[test]
fn growing_a_write_reply_is_refused_with_the_operation_and_target() {
    let checked = checked_name(Operation::Snapshot, "pool", NameField::CommandName, "pool")
        .expect("the name fits");
    let mut request: Request = Request::new(Operation::Snapshot, checked);
    let error: Error = request
        .grow_reply(INITIAL_REPLY_SIZE as u64 + 1)
        .expect_err("a write is never replayed");
    assert_eq!(error.operation(), Operation::Snapshot);
    assert_eq!(error.target(), "pool");
    assert!(error.to_string().contains("not replayed"));
}

#[test]
fn an_unfilled_reply_decodes_to_no_list() {
    let checked = checked_name(Operation::GetFsacl, "pool", NameField::CommandName, "pool")
        .expect("the name fits");
    let request: Request = Request::new(Operation::GetFsacl, checked);
    assert_eq!(
        request.take_reply().expect("unfilled is not a failure"),
        None
    );
}

#[test]
fn a_filled_reply_decodes_and_a_bad_one_names_the_request() {
    let checked = checked_name(
        Operation::GetBookmarks,
        "pool/dataset",
        NameField::CommandName,
        "pool/dataset",
    )
    .expect("the name fits");
    let mut request: Request = Request::new(Operation::GetBookmarks, checked);
    let mut reply: NvList = NvList::new();
    reply.add_str("pool/dataset#bookmark", "pool/dataset@snapshot");
    let packed: Vec<u8> = reply.pack().expect("a simple list packs");
    request.reply_mut().as_mut_bytes()[..packed.len()].copy_from_slice(&packed);
    request.command_mut().zc_nvlist_dst_filled = 1;
    request.command_mut().zc_nvlist_dst_size = packed.len() as u64;
    assert_eq!(
        request.take_reply().expect("our own packing decodes"),
        Some(reply)
    );
    request.reply_mut().as_mut_bytes()[..4].copy_from_slice(&[9, 9, 9, 9]);
    let error: Error = request.take_reply().expect_err("not a native list");
    assert_eq!(error.operation(), Operation::GetBookmarks);
    assert!(error.to_string().contains("pool/dataset"));
}

#[test]
fn a_checked_name_refusal_names_the_request() {
    let error: Error = checked_name(
        Operation::Clone,
        "pool/target",
        NameField::CommandString,
        &"x".repeat(300),
    )
    .expect_err("300 bytes do not fit a 256-byte field");
    assert_eq!(error.operation(), Operation::Clone);
    assert_eq!(error.target(), "pool/target");
    assert_eq!(error.errno(), Some(libc::ENAMETOOLONG));
}

#[test]
fn a_flag_list_carries_one_flag_per_name() {
    let list: NvList = flag_list(&["pool/ds@s1", "pool/ds@s2"]);
    assert_eq!(list.get("pool/ds@s1"), Some(&NvData::BooleanFlag));
    assert_eq!(list.get("pool/ds@s2"), Some(&NvData::BooleanFlag));
    assert_eq!(list.pairs.len(), 2);
}
