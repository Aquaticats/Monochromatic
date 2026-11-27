//! What:
//!  The layout the loaded module must declare,
//! compiled from this crate's own
//!  `#[repr(C)]` type:
//! the struct's size,
//! every member's bit offset and byte size,
//! the
//!  anonymous union and the anonymous structure inside it,
//! every member of
//!  `dmu_objset_stats` and `zinject_record`,
//! and every request number this crate can send with the
//!  range's two endpoints.
//! Why:
//!  The two sides must move together:
//! a field added to `ZfsCmd` gains a path here in
//!  the same commit,
//! and a test proves the table covers every byte of the struct,
//!  so a module cannot pass a field nothing compared.
//!  The offsets come from `offset_of!` and the sizes from `size_of!`,
//! so the
//!  compiler answers what the module must declare,
//! and only the two members the
//!  language cannot name from this side carry a literal with a test that
//! recomputes
//!  them.
//!  The request numbers come from the crate's own operation list,
//! so a request
//!  added to it cannot go uncompared.
//!  The table is split across siblings because this file is over the line limit:
//!  this one holds the `zfs_cmd` row the walk starts at and the structures it descends into,
//!  `expected_member_and_container.rs` holds the two record types,
//!  the builders a row is written with and the byte width of each fixed-size name field,
//!  `expected_dmu_objset_stats.rs` holds the `dmu_objset_stats` record,
//!  `expected_zinject_record.rs` holds the `zinject_record` record,
//!  and `expected_zc_inject_overlay.rs` holds the anonymous union the struct overlays with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const table = { zfsCmd: { size: 13744, members: [...] }, requests: [...] };
//! ```

/// The container kind the `zfs_cmd` table declares its union with.
use super::declaration::ContainerKind;
/// The `dmu_objset_stats` record the struct holds.
use super::expected_dmu_objset_stats::DMU_OBJSET_STATS;
/// One member or container the module must declare,
/// re-exported because the comparison and its controls name the two types.
pub use super::expected_member_and_container::{
    ExpectedContainer,
    ExpectedMember,
};
/// How one row of the table below is written.
use super::expected_member_and_container::{
    NAME_BYTES,
    STRING_BYTES,
    VALUE_BYTES,
    array_member,
    descended,
    member,
};
/// The destroy control inside the union,
/// which only the structures the controls walk name outside the table.
#[cfg(test)]
use super::expected_zc_inject_overlay::DESTROY_CONTROL;
/// The anonymous union the struct holds,
/// and the builder that states it.
use super::expected_zc_inject_overlay::anonymous;
/// The `zinject_record` the union overlaps with its destroy control,
/// which only the structures the controls walk name outside the table.
#[cfg(test)]
use super::expected_zinject_record::ZC_INJECT_RECORD;
/// The command struct and the records it holds.
use crate::layout::{
    DmuObjsetStats,
    DrrBegin,
    ZcInjectOverlay,
    ZfsCmd,
    ZfsShare,
    ZfsStat,
};
/// The operators the table is built from.
use std::mem::{
    offset_of,
    size_of,
};

/// The members of `zfs_cmd`,
/// every field of this crate's mirror in declaration order.
static ZFS_CMD_MEMBERS: [ExpectedMember; 32] = [
    array_member(
        "zc_name",
        offset_of!(ZfsCmd, zc_name) * 8,
        NAME_BYTES,
        size_of::<u8>(),
    ),
    member(
        "zc_nvlist_src",
        offset_of!(ZfsCmd, zc_nvlist_src) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_nvlist_src_size",
        offset_of!(ZfsCmd, zc_nvlist_src_size) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_nvlist_dst",
        offset_of!(ZfsCmd, zc_nvlist_dst) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_nvlist_dst_size",
        offset_of!(ZfsCmd, zc_nvlist_dst_size) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_nvlist_dst_filled",
        offset_of!(ZfsCmd, zc_nvlist_dst_filled) * 8,
        size_of::<i32>(),
    ),
    member("zc_pad2", offset_of!(ZfsCmd, zc_pad2) * 8, size_of::<i32>()),
    member(
        "zc_history",
        offset_of!(ZfsCmd, zc_history) * 8,
        size_of::<u64>(),
    ),
    array_member(
        "zc_value",
        offset_of!(ZfsCmd, zc_value) * 8,
        VALUE_BYTES,
        size_of::<u8>(),
    ),
    array_member(
        "zc_string",
        offset_of!(ZfsCmd, zc_string) * 8,
        STRING_BYTES,
        size_of::<u8>(),
    ),
    member("zc_guid", offset_of!(ZfsCmd, zc_guid) * 8, size_of::<u64>()),
    member(
        "zc_nvlist_conf",
        offset_of!(ZfsCmd, zc_nvlist_conf) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_nvlist_conf_size",
        offset_of!(ZfsCmd, zc_nvlist_conf_size) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_cookie",
        offset_of!(ZfsCmd, zc_cookie) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_objset_type",
        offset_of!(ZfsCmd, zc_objset_type) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_perm_action",
        offset_of!(ZfsCmd, zc_perm_action) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_history_len",
        offset_of!(ZfsCmd, zc_history_len) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_history_offset",
        offset_of!(ZfsCmd, zc_history_offset) * 8,
        size_of::<u64>(),
    ),
    member("zc_obj", offset_of!(ZfsCmd, zc_obj) * 8, size_of::<u64>()),
    member(
        "zc_iflags",
        offset_of!(ZfsCmd, zc_iflags) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_share",
        offset_of!(ZfsCmd, zc_share) * 8,
        size_of::<ZfsShare>(),
    ),
    descended(
        "zc_objset_stats",
        offset_of!(ZfsCmd, zc_objset_stats) * 8,
        size_of::<DmuObjsetStats>(),
        &DMU_OBJSET_STATS,
    ),
    member(
        "zc_begin_record",
        offset_of!(ZfsCmd, zc_begin_record) * 8,
        size_of::<DrrBegin>(),
    ),
    anonymous(
        offset_of!(ZfsCmd, zc_inject_overlay) * 8,
        size_of::<ZcInjectOverlay>(),
        ContainerKind::Union,
    ),
    member(
        "zc_cleanup_fd",
        offset_of!(ZfsCmd, zc_cleanup_fd) * 8,
        size_of::<i32>(),
    ),
    member(
        "zc_simple",
        offset_of!(ZfsCmd, zc_simple) * 8,
        size_of::<u8>(),
    ),
    array_member(
        "zc_pad",
        offset_of!(ZfsCmd, zc_pad) * 8,
        size_of::<[u8; 3]>(),
        size_of::<u8>(),
    ),
    member(
        "zc_sendobj",
        offset_of!(ZfsCmd, zc_sendobj) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_fromobj",
        offset_of!(ZfsCmd, zc_fromobj) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_createtxg",
        offset_of!(ZfsCmd, zc_createtxg) * 8,
        size_of::<u64>(),
    ),
    member(
        "zc_stat",
        offset_of!(ZfsCmd, zc_stat) * 8,
        size_of::<ZfsStat>(),
    ),
    member(
        "zc_zoneid",
        offset_of!(ZfsCmd, zc_zoneid) * 8,
        size_of::<u64>(),
    ),
];

/// `zfs_cmd`,
/// the struct every request carries.
pub static ZFS_CMD: ExpectedContainer = ExpectedContainer {
    name: "zfs_cmd",
    kind: ContainerKind::Struct,
    byte_size: size_of::<ZfsCmd>() as u32,
    members: &ZFS_CMD_MEMBERS,
};

/// Every structure compared member by member,
/// in the order the walk descends into them,
/// compiled for the controls only.
#[cfg(test)]
pub(super) fn compared_structures() -> Vec<&'static ExpectedContainer> {
    return vec![
        &ZFS_CMD,
        &DMU_OBJSET_STATS,
        &ZC_INJECT_RECORD,
        &DESTROY_CONTROL,
    ];
}
