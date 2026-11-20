//! What:
//!  The field tables of the records `zfs_cmd` holds:
//! the share record,
//! the legacy
//!  stat record,
//! the send-stream header and the inject record,
//! each field at the
//!  offset the Rust mirror compiles to.
//! Why:
//!  The layout test compares every one of them against the running module,
//! and a
//!  table written twice would drift;
//! keeping them beside `zfs_cmd`'s table
//!  would pass the folder's line limit.

/// The mirrored records and the name-field size.
use crate::layout::{DrrBegin, MAXNAMELEN, ZfsShare, ZfsStat, ZinjectRecord};
/// The offset and size operators the tables are built from.
use std::mem::{offset_of, size_of};

/// The bytes `zfs_share` must report,
/// and its fields.
pub(crate) const ZFS_SHARE_BYTES: usize = 32;

/// Every member of `zfs_share`.
pub(crate) fn zfs_share_fields() -> Vec<(&'static str, usize, usize)> {
    return vec![
        (
            "z_exportdata",
            offset_of!(ZfsShare, z_exportdata),
            size_of::<u64>(),
        ),
        (
            "z_sharedata",
            offset_of!(ZfsShare, z_sharedata),
            size_of::<u64>(),
        ),
        (
            "z_sharetype",
            offset_of!(ZfsShare, z_sharetype),
            size_of::<u64>(),
        ),
        (
            "z_sharemax",
            offset_of!(ZfsShare, z_sharemax),
            size_of::<u64>(),
        ),
    ];
}

/// The bytes `zfs_stat` must report,
/// and its fields.
pub(crate) const ZFS_STAT_BYTES: usize = 40;

/// Every member of `zfs_stat`.
pub(crate) fn zfs_stat_fields() -> Vec<(&'static str, usize, usize)> {
    return vec![
        ("zs_gen", offset_of!(ZfsStat, zs_gen), size_of::<u64>()),
        ("zs_mode", offset_of!(ZfsStat, zs_mode), size_of::<u64>()),
        ("zs_links", offset_of!(ZfsStat, zs_links), size_of::<u64>()),
        (
            "zs_ctime",
            offset_of!(ZfsStat, zs_ctime),
            size_of::<[u64; 2]>(),
        ),
    ];
}

/// The bytes `drr_begin` must report,
/// and its fields.
pub(crate) const DRR_BEGIN_BYTES: usize = 304;

/// Every member of `drr_begin`.
pub(crate) fn drr_begin_fields() -> Vec<(&'static str, usize, usize)> {
    return vec![
        (
            "drr_magic",
            offset_of!(DrrBegin, drr_magic),
            size_of::<u64>(),
        ),
        (
            "drr_versioninfo",
            offset_of!(DrrBegin, drr_versioninfo),
            size_of::<u64>(),
        ),
        (
            "drr_creation_time",
            offset_of!(DrrBegin, drr_creation_time),
            size_of::<u64>(),
        ),
        ("drr_type", offset_of!(DrrBegin, drr_type), size_of::<u32>()),
        (
            "drr_flags",
            offset_of!(DrrBegin, drr_flags),
            size_of::<u32>(),
        ),
        (
            "drr_toguid",
            offset_of!(DrrBegin, drr_toguid),
            size_of::<u64>(),
        ),
        (
            "drr_fromguid",
            offset_of!(DrrBegin, drr_fromguid),
            size_of::<u64>(),
        ),
        (
            "drr_toname",
            offset_of!(DrrBegin, drr_toname),
            size_of::<[u8; MAXNAMELEN]>(),
        ),
    ];
}

/// The bytes `zinject_record` must report,
/// and its fields.
pub(crate) const ZINJECT_RECORD_BYTES: usize = 368;

/// Every member of `zinject_record`.
pub(crate) fn zinject_record_fields() -> Vec<(&'static str, usize, usize)> {
    return vec![
        (
            "zi_objset",
            offset_of!(ZinjectRecord, zi_objset),
            size_of::<u64>(),
        ),
        (
            "zi_object",
            offset_of!(ZinjectRecord, zi_object),
            size_of::<u64>(),
        ),
        (
            "zi_start",
            offset_of!(ZinjectRecord, zi_start),
            size_of::<u64>(),
        ),
        (
            "zi_end",
            offset_of!(ZinjectRecord, zi_end),
            size_of::<u64>(),
        ),
        (
            "zi_guid",
            offset_of!(ZinjectRecord, zi_guid),
            size_of::<u64>(),
        ),
        (
            "zi_level",
            offset_of!(ZinjectRecord, zi_level),
            size_of::<u32>(),
        ),
        (
            "zi_error",
            offset_of!(ZinjectRecord, zi_error),
            size_of::<u32>(),
        ),
        (
            "zi_type",
            offset_of!(ZinjectRecord, zi_type),
            size_of::<u64>(),
        ),
        (
            "zi_freq",
            offset_of!(ZinjectRecord, zi_freq),
            size_of::<u32>(),
        ),
        (
            "zi_failfast",
            offset_of!(ZinjectRecord, zi_failfast),
            size_of::<u32>(),
        ),
        (
            "zi_func",
            offset_of!(ZinjectRecord, zi_func),
            size_of::<[u8; MAXNAMELEN]>(),
        ),
        (
            "zi_iotype",
            offset_of!(ZinjectRecord, zi_iotype),
            size_of::<u32>(),
        ),
        (
            "zi_duration",
            offset_of!(ZinjectRecord, zi_duration),
            size_of::<i32>(),
        ),
        (
            "zi_timer",
            offset_of!(ZinjectRecord, zi_timer),
            size_of::<u64>(),
        ),
        (
            "zi_nlanes",
            offset_of!(ZinjectRecord, zi_nlanes),
            size_of::<u64>(),
        ),
        (
            "zi_cmd",
            offset_of!(ZinjectRecord, zi_cmd),
            size_of::<u32>(),
        ),
        (
            "zi_dvas",
            offset_of!(ZinjectRecord, zi_dvas),
            size_of::<u32>(),
        ),
        (
            "zi_match_count",
            offset_of!(ZinjectRecord, zi_match_count),
            size_of::<u64>(),
        ),
        (
            "zi_inject_count",
            offset_of!(ZinjectRecord, zi_inject_count),
            size_of::<u64>(),
        ),
    ];
}
