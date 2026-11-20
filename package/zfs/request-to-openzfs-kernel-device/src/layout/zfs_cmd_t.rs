// Adapted from `src/ioctl.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The OpenZFS 2.4.4 `zfs_cmd_t`,
//! mirrored field for field as
//!  `#[repr(C)]` Rust,
//! including the union that lets `zc_inject_record`
//!  overlap `zc_defer_destroy`,
//! `zc_flags` and `zc_action_handle`.
//! Why:
//!  The kernel copies exactly its own `sizeof (zfs_cmd_t)` in and out of the
//!  caller's buffer,
//! so a field at a different offset would be read as another
//!  field:
//! `zc_defer_destroy` at 13648 is what marks a dataset or snapshot for
//!  deferred destruction,
//! and nothing in userspace can see a mismatch at
//!  runtime without a device.
//!  The size and offset assertions below are the device-free proof:
//! they pin
//!  the numbers the module's BTF reports for `zfs-2.4.4`
//!  (`sizeof 13744`,
//! `zc_inject_record` at 13296 with size 368,
//! and the three
//!  control fields at 13648,
//! 13652 and 13656).
//!  The union is why the struct cannot be fields alone:
//! OpenZFS grew
//!  `zinject_record_t` by 16 bytes without moving the fields after it,
//! by
//!  letting the inject record share their space.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface ZfsCmd { zcName: Uint8Array; zcNvlistSrc: bigint; /* ... */ }
//! ```

// The fixed field sizes and the struct size the design pins.
use super::abi_limits::{COMMAND_STRUCT_SIZE, MAX_VALUE_LEN, MAXNAMELEN, MAXPATHLEN};
// The C string reader the fixed fields need.
use std::ffi::CStr;
// The offset macro of the pinning assertions.
use std::mem::offset_of;

/// `zfs_share_t`:
/// the legacy share record,
/// unused by this crate's requests.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZfsShare {
    /// `z_exportdata`,
    /// really `(char *)`.
    pub z_exportdata: u64,
    /// `z_sharedata`,
    /// really `(char *)`.
    pub z_sharedata: u64,
    /// The share protocol number.
    pub z_sharetype: u64,
    /// `z_sharemax`,
    /// really `(char *)`.
    pub z_sharemax: u64,
}

/// `dmu_objset_stats_t`:
/// what `OBJSET_STATS` and both listing requests fill.
/// OpenZFS `zfs-2.4.4` added `dds_flags` as the last field
/// (`include/sys/dmu.h:1032-1043`);
/// it lands in the byte the earlier layout left as padding,
/// so the struct stays 288 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DmuObjsetStats {
    /// Number of clones of this dataset.
    pub dds_num_clones: u64,
    /// Transaction group that created the dataset.
    pub dds_creation_txg: u64,
    /// The dataset's unique identifier.
    pub dds_guid: u64,
    /// `dmu_objset_type_t`:
    /// filesystem or volume.
    pub dds_type: u32,
    /// Whether the object set is a snapshot.
    pub dds_is_snapshot: u8,
    /// Whether the object set is inconsistent.
    pub dds_inconsistent: u8,
    /// Whether the snapshot is redacted.
    pub dds_redacted: u8,
    /// The origin snapshot's name for a clone,
    /// empty otherwise.
    pub dds_origin: [u8; MAXNAMELEN],
    /// `dmu_objset_flag_t`:
    /// `DDS_FLAG_ENCRYPTED` and `DDS_FLAG_HAS_ENCRYPTED`,
    /// the field `zfs-2.4.4` added at offset 287.
    pub dds_flags: u8,
}

/// The methods of `DmuObjsetStats`.
impl DmuObjsetStats {
    /// The origin snapshot's name when this object is a clone,
    /// empty
    /// otherwise,
    /// which is what the kernel leaves in `dds_origin`.
    pub fn origin(&self) -> String {
        return text_field(&self.dds_origin);
    }
}

/// `struct drr_begin`:
/// the leading record of a send stream,
/// unused here.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DrrBegin {
    /// `DMU_BACKUP_MAGIC`,
    /// in the sending host's byte order.
    pub drr_magic: u64,
    /// Stream format version flags.
    pub drr_versioninfo: u64,
    /// Send time,
    /// in seconds since the epoch.
    pub drr_creation_time: u64,
    /// The record type;
    /// `DRR_BEGIN` is zero.
    pub drr_type: u32,
    /// Stream flags such as `DRR_FLAG_CLONE`.
    pub drr_flags: u32,
    /// GUID of the snapshot the stream creates.
    pub drr_toguid: u64,
    /// GUID of the incremental base,
    /// zero for a full stream.
    pub drr_fromguid: u64,
    /// The sender-side `pool/dataset@snapshot` name.
    pub drr_toname: [u8; MAXNAMELEN],
}

/// `zinject_record_t` on OpenZFS `zfs-2.4.4`:
/// 368 bytes,
/// the union's largest
/// member and the reason the control fields share its tail.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZinjectRecord {
    /// Object set number.
    pub zi_objset: u64,
    /// Object number.
    pub zi_object: u64,
    /// First byte offset.
    pub zi_start: u64,
    /// Last byte offset.
    pub zi_end: u64,
    /// Dataset GUID.
    pub zi_guid: u64,
    /// Block level.
    pub zi_level: u32,
    /// Error to inject.
    pub zi_error: u32,
    /// Injection type.
    pub zi_type: u64,
    /// Injection frequency denominator.
    pub zi_freq: u32,
    /// `zi_failfast` flag.
    pub zi_failfast: u32,
    /// Function name,
    /// `MAXNAMELEN` bytes.
    pub zi_func: [u8; MAXNAMELEN],
    /// I/O type.
    pub zi_iotype: u32,
    /// Injection duration.
    pub zi_duration: i32,
    /// Timer value.
    pub zi_timer: u64,
    /// Lane count.
    pub zi_nlanes: u64,
    /// Injection command.
    pub zi_cmd: u32,
    /// DVA count.
    pub zi_dvas: u32,
    /// Times the rule matched,
    /// added in `zfs-2.4.4`.
    pub zi_match_count: u64,
    /// Times a fault was injected,
    /// added in `zfs-2.4.4`.
    pub zi_inject_count: u64,
}

/// The overlay member of the union:
/// 352 bytes of padding,
/// then the three
/// fields OpenZFS kept in place when the inject record grew.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZcDestroyControl {
    /// `sizeof (zinject_record_t) - 16` bytes of unused space.
    pub zc_pad1: [u8; 352],
    /// Nonzero marks the target for deferred destruction.
    pub zc_defer_destroy: u32,
    /// Request flags.
    pub zc_flags: u32,
    /// Handle of the action to run.
    pub zc_action_handle: u64,
}

/// The anonymous union OpenZFS declares before `zc_cleanup_fd`.
#[repr(C)]
pub union ZcInjectOverlay {
    /// The inject ioctl's record,
    /// the union's largest member.
    pub inject: ZinjectRecord,
    /// The three control fields every other request uses.
    pub control: ZcDestroyControl,
}

/// `zfs_stat_t`:
/// the legacy stat record,
/// unused by this crate's requests.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZfsStat {
    /// Generation number.
    pub zs_gen: u64,
    /// Mode bits.
    pub zs_mode: u64,
    /// Link count.
    pub zs_links: u64,
    /// Creation and modification times.
    pub zs_ctime: [u64; 2],
}

/// OpenZFS `zfs_cmd_t` for `zfs-2.4.4`,
/// in declaration order.
#[repr(C)]
pub struct ZfsCmd {
    /// `zc_name`:
    /// the primary pool or dataset name.
    pub zc_name: [u8; MAXPATHLEN],
    /// `zc_nvlist_src`:
    /// address of the packed input list,
    /// fixed-width.
    pub zc_nvlist_src: u64,
    /// `zc_nvlist_src_size`:
    /// bytes of the input list.
    pub zc_nvlist_src_size: u64,
    /// `zc_nvlist_dst`:
    /// address of the reply buffer,
    /// fixed-width.
    pub zc_nvlist_dst: u64,
    /// `zc_nvlist_dst_size`:
    /// bytes available for the reply.
    pub zc_nvlist_dst_size: u64,
    /// Nonzero when the kernel put a list in `zc_nvlist_dst`.
    pub zc_nvlist_dst_filled: i32,
    /// Padding that keeps the next field 8-byte aligned.
    pub zc_pad2: i32,
    /// `zc_history`,
    /// really `(char *)`.
    pub zc_history: u64,
    /// `zc_value`:
    /// a secondary name,
    /// at most `MAX_VALUE_LEN` bytes.
    pub zc_value: [u8; MAX_VALUE_LEN],
    /// `zc_string`:
    /// a secondary short string.
    pub zc_string: [u8; MAXNAMELEN],
    /// A GUID or a flag,
    /// depending on the request.
    pub zc_guid: u64,
    /// `zc_nvlist_conf`,
    /// really `(char *)`.
    pub zc_nvlist_conf: u64,
    /// Bytes of the configuration list.
    pub zc_nvlist_conf_size: u64,
    /// The kernel's listing cursor.
    pub zc_cookie: u64,
    /// Object set type.
    pub zc_objset_type: u64,
    /// Permission action.
    pub zc_perm_action: u64,
    /// Entries of a history record.
    pub zc_history_len: u64,
    /// Offset into a history record.
    pub zc_history_offset: u64,
    /// A per-request object number.
    pub zc_obj: u64,
    /// In-kernel input flags.
    pub zc_iflags: u64,
    /// The legacy share record.
    pub zc_share: ZfsShare,
    /// The dataset statistics the kernel fills.
    pub zc_objset_stats: DmuObjsetStats,
    /// The send-stream header.
    pub zc_begin_record: DrrBegin,
    /// The inject record or the three control fields,
    /// whichever the request uses.
    pub zc_inject_overlay: ZcInjectOverlay,
    /// An fd the kernel closes or uses for the request.
    pub zc_cleanup_fd: i32,
    /// Nonzero asks a listing for fast stats without properties.
    pub zc_simple: u8,
    /// Padding for the next 8-byte field.
    pub zc_pad: [u8; 3],
    /// Send-side object number.
    pub zc_sendobj: u64,
    /// Receive-side origin object number.
    pub zc_fromobj: u64,
    /// Creation transaction group filter.
    pub zc_createtxg: u64,
    /// The legacy stat record.
    pub zc_stat: ZfsStat,
    /// The zone the request belongs to.
    pub zc_zoneid: u64,
}

/// The methods of `ZfsCmd`.
impl ZfsCmd {
    /// The `zc_name` field as text,
    /// stopped at its terminating NUL.
    pub fn name(&self) -> String {
        return text_field(&self.zc_name);
    }

    /// The `zc_value` field as text,
    /// stopped at its terminating NUL.
    pub fn value(&self) -> String {
        return text_field(&self.zc_value);
    }

    /// The `zc_string` field as text,
    /// stopped at its terminating NUL.
    pub fn string(&self) -> String {
        return text_field(&self.zc_string);
    }

    /// Read `zc_defer_destroy` from the union's control member.
    pub fn defer_destroy(&self) -> u32 {
        // SAFETY: the union's control member covers the whole union, and its
        // first 352 bytes are plain padding, so any bit pattern is valid.
        return unsafe { self.zc_inject_overlay.control.zc_defer_destroy };
    }

    /// Set `zc_defer_destroy` in the union's control member.
    pub fn set_defer_destroy(&mut self, defer: bool) {
        // Writing a union field is safe; only reading one needs `unsafe`.
        self.zc_inject_overlay.control.zc_defer_destroy = u32::from(defer);
    }

    /// Read `zc_flags` from the union's control member.
    pub fn flags(&self) -> u32 {
        // SAFETY: as in `defer_destroy`.
        return unsafe { self.zc_inject_overlay.control.zc_flags };
    }

    /// Read `zc_action_handle` from the union's control member.
    pub fn action_handle(&self) -> u64 {
        // SAFETY: as in `defer_destroy`.
        return unsafe { self.zc_inject_overlay.control.zc_action_handle };
    }
}

/// Decode one fixed C string field,
/// stopping at its NUL and replacing invalid
/// UTF-8 rather than failing:
/// the value is a kernel-written name,
/// not input.
fn text_field(field: &[u8]) -> String {
    let Ok(text) = CStr::from_bytes_until_nul(field) else {
        return String::new();
    };
    return text.to_string_lossy().into_owned();
}

/// What:
///  The pinning assertions:
/// a failure here means this mirror and the module the
///  repository tested against disagree,
/// so a request would land in the wrong
///  bytes.
/// Why:
///  One constant holds every assertion,
/// so the proof is one item,
/// and the build fails with the first line that moved.
const _: () = {
    assert!(size_of::<ZfsShare>() == 32);
    assert!(size_of::<DmuObjsetStats>() == 288);
    // `dds_flags`, the field `zfs-2.4.4` added (`include/sys/dmu.h:1041`),
    // sits in the padding byte after `dds_origin`.
    assert!(offset_of!(DmuObjsetStats, dds_flags) == 287);
    assert!(size_of::<DrrBegin>() == 304);
    assert!(size_of::<ZinjectRecord>() == 368);
    assert!(size_of::<ZcDestroyControl>() == 368);
    assert!(size_of::<ZcInjectOverlay>() == 368);
    assert!(size_of::<ZfsStat>() == 40);
    assert!(align_of::<ZfsCmd>() == 8);
    assert!(size_of::<ZfsCmd>() == COMMAND_STRUCT_SIZE);
    // The four `u64` address and size fields follow `zc_name`.
    assert!(offset_of!(ZfsCmd, zc_nvlist_src) == MAXPATHLEN);
    assert!(offset_of!(ZfsCmd, zc_nvlist_dst_filled) == MAXPATHLEN + 4 * size_of::<u64>());
    // Pinned ABI offsets from the module's BTF for `zfs-2.4.4`.
    assert!(offset_of!(ZfsCmd, zc_objset_stats) == 12_704);
    assert!(offset_of!(ZfsCmd, zc_begin_record) == 12_992);
    assert!(offset_of!(ZfsCmd, zc_inject_overlay) == 13_296);
    assert!(offset_of!(ZfsCmd, zc_cleanup_fd) == 13_664);
    assert!(offset_of!(ZfsCmd, zc_zoneid) == 13_736);
    // Every field a request writes, pinned to the module's BTF too, because a
    // swap among them would otherwise leave every device-free test green:
    // `zc_cookie` is the cursor both list requests move.
    assert!(offset_of!(ZfsCmd, zc_name) == 0);
    assert!(offset_of!(ZfsCmd, zc_nvlist_src_size) == 4_104);
    assert!(offset_of!(ZfsCmd, zc_nvlist_dst) == 4_112);
    assert!(offset_of!(ZfsCmd, zc_nvlist_dst_size) == 4_120);
    assert!(offset_of!(ZfsCmd, zc_value) == 4_144);
    assert!(offset_of!(ZfsCmd, zc_string) == 12_336);
    assert!(offset_of!(ZfsCmd, zc_cookie) == 12_616);
    assert!(offset_of!(ZfsCmd, zc_objset_type) == 12_624);
    assert!(offset_of!(ZcDestroyControl, zc_defer_destroy) == 352);
    assert!(offset_of!(ZcDestroyControl, zc_flags) == 356);
    assert!(offset_of!(ZcDestroyControl, zc_action_handle) == 360);
};

/// The device-free tests of this module.
#[cfg(test)]
#[path = "zfs_cmd_t_tests.rs"]
mod tests;
