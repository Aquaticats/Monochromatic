//! What:
//!  The expectation table's own checks:
//! every container's members tile its size
//! with no gap and no overlap,
//! the two members the language cannot name from this side are recomputed
//!  from the Rust type,
//! and the request list covers every operation this crate can send plus the
//!  range's two endpoints.
//! Why:
//!  A field added to `ZfsCmd` with no path here would leave one member
//! uncompared,
//!  and the tiling is what fails then;
//! the counts alone would still agree.

use super::expectations::{
    ContainerKind, ExpectedContainer, ExpectedMember, ZFS_CMD, compared_structures,
};
use super::expected_requests::{ExpectedEnumerator, expected_enumerators};
use crate::layout::{ZcDestroyControl, ZcInjectOverlay, ZfsCmd};
use crate::requests::Operation;
use std::mem::{offset_of, size_of};

/// Every member's bit range,
/// sorted by offset.
fn ranges(container: &ExpectedContainer) -> Vec<(u32, u32)> {
    let mut sorted: Vec<(u32, u32)> = container
        .members
        .iter()
        .map(|member: &ExpectedMember| {
            return (member.bit_offset, member.byte_size * 8);
        })
        .collect();
    sorted.sort_unstable();
    return sorted;
}

#[test]
fn every_structures_members_tile_its_size() {
    for container in compared_structures() {
        let sorted: Vec<(u32, u32)> = ranges(container);
        let mut expected_start: u32 = 0;
        for (offset, length) in sorted {
            assert_eq!(offset, expected_start, "{}", container.name);
            expected_start = expected_start.saturating_add(length);
        }
        assert_eq!(expected_start, container.byte_size * 8, "{}", container.name);
    }
}

#[test]
fn the_command_struct_is_the_size_this_crate_compiles() {
    assert_eq!(ZFS_CMD.byte_size, size_of::<ZfsCmd>() as u32);
    assert_eq!(ZFS_CMD.kind, ContainerKind::Struct);
    assert_eq!(ZFS_CMD.name, "zfs_cmd");
}

#[test]
fn the_anonymous_union_sits_where_the_rust_type_puts_it() {
    let overlay: &ExpectedMember = ZFS_CMD
        .members
        .iter()
        .find(|member: &&ExpectedMember| {
            return member.name.is_none();
        })
        .expect("the union has no name");
    assert_eq!(overlay.container, Some(ContainerKind::Union));
    assert_eq!(overlay.bit_offset, (offset_of!(ZfsCmd, zc_inject_overlay) * 8) as u32);
    assert_eq!(overlay.byte_size, size_of::<ZcInjectOverlay>() as u32);
    let interior: &ExpectedContainer = overlay.interior.expect("the union is compared");
    assert_eq!(interior.byte_size, size_of::<ZcInjectOverlay>() as u32);
    let inject: &ExpectedMember = interior
        .members
        .iter()
        .find(|member: &&ExpectedMember| {
            return member.name == Some("zc_inject_record");
        })
        .expect("the inject record is the union's first member");
    assert_eq!(inject.bit_offset, (offset_of!(ZcInjectOverlay, inject) * 8) as u32);
    let control: &ExpectedMember = interior
        .members
        .iter()
        .find(|member: &&ExpectedMember| {
            return member.name.is_none();
        })
        .expect("the destroy control is the union's anonymous member");
    assert_eq!(control.bit_offset, (offset_of!(ZcInjectOverlay, control) * 8) as u32);
    assert_eq!(control.byte_size, size_of::<ZcDestroyControl>() as u32);
    let pad: &ExpectedMember = control
        .interior
        .expect("the control is compared")
        .members
        .first()
        .expect("the padding is the first member");
    assert_eq!(pad.byte_size, size_of::<ZcDestroyControl>() as u32 - 16);
}

#[test]
fn every_request_this_crate_sends_has_one_expectation_and_two_endpoints() {
    let wanted: Vec<ExpectedEnumerator> = expected_enumerators();
    assert_eq!(wanted.len(), Operation::ALL.len() + 2);
    for operation in Operation::ALL {
        let name: String = format!("ZFS_IOC_{}", operation.as_str());
        let found: Option<&ExpectedEnumerator> = wanted.iter().find(|entry: &&ExpectedEnumerator| {
            return entry.name == name;
        });
        let entry: &ExpectedEnumerator = found.expect("every operation is expected");
        assert_eq!(entry.value, operation.code() as i64, "{name}");
    }
    let first: &ExpectedEnumerator = wanted.last().expect("the endpoints are last");
    assert_eq!(first.name, "ZFS_IOC_LAST");
    assert_eq!(first.value, 0x5a89);
    let second_last: &ExpectedEnumerator = wanted.get(wanted.len() - 2).expect("two endpoints");
    assert_eq!(second_last.name, "ZFS_IOC_FIRST");
    assert_eq!(second_last.value, 0x5a00);
}
