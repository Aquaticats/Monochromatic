//! What:
//!  The comparison's decisions:
//! the approved declaration passes,
//! and each change a
//!  release made or a patch makes refuses at the step and the field the design
//!  says comes first.
//! Why:
//!  A gate that refused everything would pass every arm here that expects a
//! refusal,
//!  so the first test is the control:
//! the declaration the module publishes passes the
//!  same call.

use super::approved_module_fixture::{Change, module_with};
use super::{Compatible, Refusal};
use super::blob_fixture::{Blob, check_pair, check_raw};
use super::refusal::Step;

/// The gate's verdict for one changed declaration.
fn verdict(change: Change, name: &str) -> Result<Compatible, Refusal> {
    return check_pair(name, &Blob::new(), &module_with(change));
}

/// The refusal one changed declaration must get.
fn refused(change: Change, name: &str) -> Refusal {
    let answer: Result<Compatible, Refusal> = verdict(change, name);
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("the {change:?} declaration passed the gate");
    };
    return refusal;
}

#[test]
fn the_approved_declaration_and_its_requests_pass() {
    let answer: Result<Compatible, Refusal> = verdict(Change::None, "comparison-approved");
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the approved declaration refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.base_header_bytes, 32);
    assert_eq!(compatible.module_header_bytes, 32);
    assert_eq!(compatible.base_bytes, 33);
    assert_eq!(
        compatible.module_bytes,
        module_with(Change::None).bytes().len()
    );
}

#[test]
fn a_module_with_the_oldest_header_passes() {
    let module: Vec<u8> = module_with(Change::None).bytes_with_header(24);
    let base: Vec<u8> = Blob::new().bytes_with_header(24);
    let answer: Result<Compatible, Refusal> = check_raw("comparison-oldest-header", &base, &module);
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the 24-byte header refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.module_header_bytes, 24);
}

#[test]
fn a_record_that_lost_its_last_flag_refuses_at_that_member() {
    let refusal: Refusal = refused(Change::NoDdsFlags, "comparison-no-dds-flags");
    assert_eq!(refusal.step, Step::Member);
    assert!(refusal.detail.contains("dds_flags"), "{}", refusal.detail);
    assert!(refusal.detail.contains("2296"), "{}", refusal.detail);
}

#[test]
fn a_union_that_is_neither_anonymous_nor_the_union_refuses_at_the_type() {
    let refusal: Refusal = refused(Change::NoUnion, "comparison-no-union");
    assert_eq!(refusal.step, Step::Type);
    assert!(refusal.detail.contains("zc_inject_record"), "{}", refusal.detail);
    assert!(refusal.detail.contains("106368"), "{}", refusal.detail);
}

#[test]
fn an_inject_record_that_lost_its_counters_refuses_with_both_sizes() {
    let refusal: Refusal = refused(Change::ShortInject, "comparison-short-inject");
    assert_eq!(refusal.step, Step::Member);
    assert!(refusal.detail.contains("zc_inject_record"), "{}", refusal.detail);
    assert!(refusal.detail.contains("352"), "{}", refusal.detail);
    assert!(refusal.detail.contains("368"), "{}", refusal.detail);
}

#[test]
fn two_members_that_swapped_offsets_refuse_with_both_offsets() {
    let refusal: Refusal = refused(Change::SwappedMembers, "comparison-swapped");
    assert_eq!(refusal.step, Step::Member);
    assert!(refusal.detail.contains("zi_iotype"), "{}", refusal.detail);
    assert!(refusal.detail.contains("2592"), "{}", refusal.detail);
    assert!(refusal.detail.contains("2560"), "{}", refusal.detail);
}

#[test]
fn a_member_no_member_expects_refuses_at_the_count() {
    let refusal: Refusal = refused(Change::ExtraMember, "comparison-extra");
    assert_eq!(refusal.step, Step::Member);
    assert!(refusal.detail.contains("10 members where 9"), "{}", refusal.detail);
    assert!(refusal.detail.contains("dds_extra"), "{}", refusal.detail);
}

#[test]
fn a_renumbered_request_refuses_with_both_values() {
    let refusal: Refusal = refused(Change::RenumberedClone, "comparison-renumbered");
    assert_eq!(refusal.step, Step::Enum);
    assert!(refusal.detail.contains("ZFS_IOC_CLONE"), "{}", refusal.detail);
    assert!(refusal.detail.contains("23107"), "{}", refusal.detail);
    assert!(refusal.detail.contains("23106"), "{}", refusal.detail);
}

#[test]
fn a_missing_request_refuses_naming_it() {
    let refusal: Refusal = refused(Change::MissingChannelProgram, "comparison-missing-request");
    assert_eq!(refusal.step, Step::Enum);
    assert!(refusal.detail.contains("ZFS_IOC_CHANNEL_PROGRAM"), "{}", refusal.detail);
}

#[test]
fn a_module_without_the_command_struct_refuses_at_the_type() {
    let answer: Result<Compatible, Refusal> = check_pair(
        "comparison-no-command",
        &Blob::new(),
        &Blob::new(),
    );
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("a module without zfs_cmd passed the gate");
    };
    assert_eq!(refusal.step, Step::Type);
    assert!(refusal.detail.contains("zfs_cmd"), "{}", refusal.detail);
}

#[test]
fn a_big_endian_module_refuses_at_the_header() {
    let module: Vec<u8> = module_with(Change::None).bytes();
    let swapped: Vec<u8> = Blob::with_byte(module, 0, 0xeb);
    let total: Vec<u8> = Blob::with_byte(swapped, 1, 0x9f);
    let answer: Result<Compatible, Refusal> = check_raw(
        "comparison-big-endian",
        &Blob::new().bytes(),
        &total,
    );
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("a big-endian file passed the gate");
    };
    assert_eq!(refusal.step, Step::Header);
    assert!(refusal.detail.contains("big-endian"), "{}", refusal.detail);
}
