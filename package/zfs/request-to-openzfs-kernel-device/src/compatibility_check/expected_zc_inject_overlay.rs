//! What:
//!  The anonymous union `zfs_cmd` holds before `zc_cleanup_fd`:
//! the inject record,
//! the anonymous destroy control that overlaps it,
//! and the union itself.
//! Why:
//!  The format writes no name for either anonymous container,
//! so the comparison matches them by kind and bit offset,
//! and the padding the destroy control keeps is the one size no field of
//!  this crate can name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const overlay = { name: '', kind: 'union', byteSize: 368, members: [ { name: 'zc_inject_record', interior: zinjectRecord }, { container: 'struct', bitOffset: 0, interior: destroyControl } ] };
//! ```

/// The container kind an anonymous member is declared with.
use super::declaration::ContainerKind;
/// What:
///  One member or container the module must declare,
/// and how a table states one.
/// Why:
///  The records below are written through these builders,
/// so their offsets and sizes come from the compiler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ExpectedContainer, ExpectedMember, array_member, descended, member } from './expected_member_and_container';
/// ```
use super::expected_member_and_container::{
    ExpectedContainer,
    ExpectedMember,
    array_member,
    descended,
    member,
};
/// The inject record the union overlaps with the destroy control.
use super::expected_zinject_record::ZC_INJECT_RECORD;
/// The crate's own mirrors of the two records and of the union.
use crate::layout::{
    ZcDestroyControl,
    ZcInjectOverlay,
    ZinjectRecord,
};
/// The operators the tables are built from.
use std::mem::{
    offset_of,
    size_of,
};

/// The bytes the destroyed-name padding holds,
/// `sizeof (zinject_record_t) - 16` in `include/sys/zfs_ioctl.h`,
/// which keeps the three control fields where they sit.
const DESTROY_PAD_BYTES: usize = size_of::<ZinjectRecord>() - 16;

/// The members of the anonymous structure inside the union,
/// the three control fields and the padding that holds them in place.
static DESTROY_CONTROL_MEMBERS: [ExpectedMember; 4] = [
    array_member(
        "zc_pad1",
        offset_of!(ZcDestroyControl, zc_pad1) * 8,
        DESTROY_PAD_BYTES,
        size_of::<u8>(),
    ),
    member(
        "zc_defer_destroy",
        offset_of!(ZcDestroyControl, zc_defer_destroy) * 8,
        size_of::<u32>(),
    ),
    member(
        "zc_flags",
        offset_of!(ZcDestroyControl, zc_flags) * 8,
        size_of::<u32>(),
    ),
    member(
        "zc_action_handle",
        offset_of!(ZcDestroyControl, zc_action_handle) * 8,
        size_of::<u64>(),
    ),
];

/// The anonymous structure inside the union.
pub(super) static DESTROY_CONTROL: ExpectedContainer = ExpectedContainer {
    name: "",
    kind: ContainerKind::Struct,
    byte_size: size_of::<ZcDestroyControl>() as u32,
    members: &DESTROY_CONTROL_MEMBERS,
};

/// One anonymous container,
/// which the format writes no name for.
pub(super) const fn anonymous(
    bit_offset: usize,
    byte_size: usize,
    kind: ContainerKind,
) -> ExpectedMember {
    return ExpectedMember {
        name: None,
        container: Some(kind),
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: None,
        interior: match kind {
            ContainerKind::Struct => Some(&DESTROY_CONTROL),
            ContainerKind::Union => Some(&OVERLAY_UNION),
        },
    };
}

/// The members of the anonymous union,
/// the inject record and the anonymous structure that overlaps it.
static OVERLAY_MEMBERS: [ExpectedMember; 2] = [
    descended(
        "zc_inject_record",
        offset_of!(ZcInjectOverlay, inject) * 8,
        size_of::<ZinjectRecord>(),
        &ZC_INJECT_RECORD,
    ),
    anonymous(
        offset_of!(ZcInjectOverlay, control) * 8,
        size_of::<ZcDestroyControl>(),
        ContainerKind::Struct,
    ),
];

/// The anonymous union the struct holds before `zc_cleanup_fd`.
static OVERLAY_UNION: ExpectedContainer = ExpectedContainer {
    name: "",
    kind: ContainerKind::Union,
    byte_size: size_of::<ZcInjectOverlay>() as u32,
    members: &OVERLAY_MEMBERS,
};
