//! What:
//!  The field tables the layout test compares against the running module:
//! each
//!  field's name,
//! its offset in the Rust mirror,
//! and the bytes it holds.
//! Why:
//!  A table written twice would drift from the struct it mirrors.
//! Every offset here
//!  comes from `offset_of!` on the Rust field and every size from
//!  `size_of` of its type,
//! so the numbers are the compiler's,
//! and a changed
//!  struct changes the table.

// The mirrored structs and the field bounds.
use crate::layout::{
    DmuObjsetStats, DrrBegin, MAX_VALUE_LEN, MAXNAMELEN, MAXPATHLEN, ZcDestroyControl, ZfsCmd,
    ZfsShare, ZfsStat, ZinjectRecord,
};
// The offset and size operators the table is built from.
use std::mem::{offset_of, size_of};

/// The size of the command struct the module's type information must report.
pub(crate) const COMMAND_STRUCT_BYTES: usize = 13_744;

/// Every named member of `zfs_cmd`,
/// in declaration order,
/// as the C header spells
/// them.
pub(crate) fn zfs_cmd_fields() -> Vec<(&'static str, usize, usize)> {
    let union_base: usize = offset_of!(ZfsCmd, zc_inject_overlay);
    let control = |name: &'static str, offset: usize, size: usize| {
        return (name, union_base + offset, size);
    };
    let mut fields: Vec<(&'static str, usize, usize)> = vec![
        (
            "zc_name",
            offset_of!(ZfsCmd, zc_name),
            size_of::<[u8; MAXPATHLEN]>(),
        ),
        (
            "zc_nvlist_src",
            offset_of!(ZfsCmd, zc_nvlist_src),
            size_of::<u64>(),
        ),
        (
            "zc_nvlist_src_size",
            offset_of!(ZfsCmd, zc_nvlist_src_size),
            size_of::<u64>(),
        ),
        (
            "zc_nvlist_dst",
            offset_of!(ZfsCmd, zc_nvlist_dst),
            size_of::<u64>(),
        ),
        (
            "zc_nvlist_dst_size",
            offset_of!(ZfsCmd, zc_nvlist_dst_size),
            size_of::<u64>(),
        ),
        (
            "zc_nvlist_dst_filled",
            offset_of!(ZfsCmd, zc_nvlist_dst_filled),
            size_of::<i32>(),
        ),
        ("zc_pad2", offset_of!(ZfsCmd, zc_pad2), size_of::<i32>()),
        (
            "zc_history",
            offset_of!(ZfsCmd, zc_history),
            size_of::<u64>(),
        ),
        (
            "zc_value",
            offset_of!(ZfsCmd, zc_value),
            size_of::<[u8; MAX_VALUE_LEN]>(),
        ),
        (
            "zc_string",
            offset_of!(ZfsCmd, zc_string),
            size_of::<[u8; MAXNAMELEN]>(),
        ),
        ("zc_guid", offset_of!(ZfsCmd, zc_guid), size_of::<u64>()),
        (
            "zc_nvlist_conf",
            offset_of!(ZfsCmd, zc_nvlist_conf),
            size_of::<u64>(),
        ),
        (
            "zc_nvlist_conf_size",
            offset_of!(ZfsCmd, zc_nvlist_conf_size),
            size_of::<u64>(),
        ),
        ("zc_cookie", offset_of!(ZfsCmd, zc_cookie), size_of::<u64>()),
        (
            "zc_objset_type",
            offset_of!(ZfsCmd, zc_objset_type),
            size_of::<u64>(),
        ),
        (
            "zc_perm_action",
            offset_of!(ZfsCmd, zc_perm_action),
            size_of::<u64>(),
        ),
        (
            "zc_history_len",
            offset_of!(ZfsCmd, zc_history_len),
            size_of::<u64>(),
        ),
        (
            "zc_history_offset",
            offset_of!(ZfsCmd, zc_history_offset),
            size_of::<u64>(),
        ),
        ("zc_obj", offset_of!(ZfsCmd, zc_obj), size_of::<u64>()),
        ("zc_iflags", offset_of!(ZfsCmd, zc_iflags), size_of::<u64>()),
        (
            "zc_share",
            offset_of!(ZfsCmd, zc_share),
            size_of::<ZfsShare>(),
        ),
        (
            "zc_objset_stats",
            offset_of!(ZfsCmd, zc_objset_stats),
            size_of::<DmuObjsetStats>(),
        ),
        (
            "zc_begin_record",
            offset_of!(ZfsCmd, zc_begin_record),
            size_of::<DrrBegin>(),
        ),
        ("zc_inject_record", union_base, size_of::<ZinjectRecord>()),
        control(
            "zc_pad1",
            offset_of!(ZcDestroyControl, zc_pad1),
            size_of::<[u8; 352]>(),
        ),
        control(
            "zc_defer_destroy",
            offset_of!(ZcDestroyControl, zc_defer_destroy),
            size_of::<u32>(),
        ),
        control(
            "zc_flags",
            offset_of!(ZcDestroyControl, zc_flags),
            size_of::<u32>(),
        ),
        control(
            "zc_action_handle",
            offset_of!(ZcDestroyControl, zc_action_handle),
            size_of::<u64>(),
        ),
        (
            "zc_cleanup_fd",
            offset_of!(ZfsCmd, zc_cleanup_fd),
            size_of::<i32>(),
        ),
        ("zc_simple", offset_of!(ZfsCmd, zc_simple), size_of::<u8>()),
        ("zc_pad", offset_of!(ZfsCmd, zc_pad), size_of::<[u8; 3]>()),
        (
            "zc_sendobj",
            offset_of!(ZfsCmd, zc_sendobj),
            size_of::<u64>(),
        ),
        (
            "zc_fromobj",
            offset_of!(ZfsCmd, zc_fromobj),
            size_of::<u64>(),
        ),
        (
            "zc_createtxg",
            offset_of!(ZfsCmd, zc_createtxg),
            size_of::<u64>(),
        ),
        ("zc_stat", offset_of!(ZfsCmd, zc_stat), size_of::<ZfsStat>()),
        ("zc_zoneid", offset_of!(ZfsCmd, zc_zoneid), size_of::<u64>()),
    ];
    fields.sort_by_key(|field| {
        return (field.1, field.0);
    });
    return fields;
}

/// The bytes `dmu_objset_stats` must report,
/// and its fields.
pub(crate) const DMU_OBJSET_STATS_BYTES: usize = 288;

/// Every member of `dmu_objset_stats`,
/// in declaration order.
pub(crate) fn dmu_objset_stats_fields() -> Vec<(&'static str, usize, usize)> {
    return vec![
        (
            "dds_num_clones",
            offset_of!(DmuObjsetStats, dds_num_clones),
            size_of::<u64>(),
        ),
        (
            "dds_creation_txg",
            offset_of!(DmuObjsetStats, dds_creation_txg),
            size_of::<u64>(),
        ),
        (
            "dds_guid",
            offset_of!(DmuObjsetStats, dds_guid),
            size_of::<u64>(),
        ),
        (
            "dds_type",
            offset_of!(DmuObjsetStats, dds_type),
            size_of::<u32>(),
        ),
        (
            "dds_is_snapshot",
            offset_of!(DmuObjsetStats, dds_is_snapshot),
            size_of::<u8>(),
        ),
        (
            "dds_inconsistent",
            offset_of!(DmuObjsetStats, dds_inconsistent),
            size_of::<u8>(),
        ),
        (
            "dds_redacted",
            offset_of!(DmuObjsetStats, dds_redacted),
            size_of::<u8>(),
        ),
        (
            "dds_origin",
            offset_of!(DmuObjsetStats, dds_origin),
            size_of::<[u8; MAXNAMELEN]>(),
        ),
        (
            "dds_flags",
            offset_of!(DmuObjsetStats, dds_flags),
            size_of::<u8>(),
        ),
    ];
}
