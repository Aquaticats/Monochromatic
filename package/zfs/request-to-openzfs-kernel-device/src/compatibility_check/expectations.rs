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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const table = { zfsCmd: { size: 13744, members: [...] }, requests: [...] };
//! ```

/// The command struct and the records it holds.
use crate::layout::{
    DrrBegin, DmuObjsetStats, MAXNAMELEN, MAX_VALUE_LEN, MAXPATHLEN, ZcDestroyControl,
    ZcInjectOverlay, ZfsCmd, ZfsShare, ZfsStat, ZinjectRecord,
};
/// The two record kinds the loader numbers.
use super::btf_entry::{KIND_STRUCT, KIND_UNION};
/// The offsets the table is built from.
use std::mem::{offset_of, size_of};

/// The bytes the struct's first field holds,
/// kept as one number so the table below reads as one list.
const NAME_BYTES: usize = size_of::<[u8; MAXPATHLEN]>();
/// The bytes the struct's secondary name holds.
const VALUE_BYTES: usize = size_of::<[u8; MAX_VALUE_LEN]>();
/// The bytes the struct's short string holds.
const STRING_BYTES: usize = size_of::<[u8; MAXNAMELEN]>();
/// The bytes the destroyed-name padding holds,
/// `sizeof (zinject_record_t) - 16` in `include/sys/zfs_ioctl.h`,
/// which keeps the three control fields where they sit.
const DESTROY_PAD_BYTES: usize = size_of::<ZinjectRecord>() - 16;

/// What:
///  Which of the two record kinds an expected container is.
/// Why:
///   The format gives each kind its own number,
/// and an anonymous container is matched
///  by kind and position,
/// so the kind is part of the expectation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ContainerKind = 'struct' | 'union';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerKind {
    /// A structure.
    Struct,
    /// A union,
    /// whose members share their first bit.
    Union,
}

/// The format's kind number for one container kind.
impl ContainerKind {
    /// The kind number the loader reports for this kind.
    pub fn btf_kind(self) -> u8 {
        return match self {
            ContainerKind::Struct => KIND_STRUCT,
            ContainerKind::Union => KIND_UNION,
        };
    }
}

/// What:
///  One member the module must declare,
/// with where it sits and the type size it holds.
/// Why:
///   A named member is matched by name;
/// an anonymous container is matched by its kind
///  and bit offset,
/// because the format writes it no name.
///  The byte size is the member type's own size,
/// and `element_bytes` is the size of
///  an array member's element,
/// so a module that keeps a total size while changing the
///  element cannot pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedMember = { name?: string; container?: ContainerKind; bitOffset: number; byteSize: number; elementBytes?: number; interior?: ExpectedContainer };
/// ```
pub struct ExpectedMember {
    /// The member's name,
    /// and nothing for an anonymous container.
    pub name: Option<&'static str>,
    /// The container kind,
    /// for an anonymous container only.
    pub container: Option<ContainerKind>,
    /// Bits from the container's first bit.
    pub bit_offset: u32,
    /// The member type's size in bytes.
    pub byte_size: u32,
    /// An array member's element size in bytes.
    pub element_bytes: Option<u32>,
    /// The members of a container compared member by member.
    pub interior: Option<&'static ExpectedContainer>,
}

/// What:
///  One container the module must declare,
/// and the members it must hold.
/// Why:
///   The container's own size is compared before any member,
/// because a container that
///  grew is the one fact a moved member alone would not show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedContainer = { name: string; kind: ContainerKind; byteSize: number; members: ExpectedMember[] };
/// ```
pub struct ExpectedContainer {
    /// The name the module declares it with.
    pub name: &'static str,
    /// Which of the two record kinds it is.
    pub kind: ContainerKind,
    /// Its size in bytes.
    pub byte_size: u32,
    /// Its members.
    pub members: &'static [ExpectedMember],
}

/// One whole member at the offset `offset_of!` gives its field.
const fn member(name: &'static str, bit_offset: usize, byte_size: usize) -> ExpectedMember {
    return ExpectedMember {
        name: Some(name),
        container: None,
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: None,
        interior: None,
    };
}

/// One array member,
/// with the element size its total hides.
const fn array_member(
    name: &'static str,
    bit_offset: usize,
    byte_size: usize,
    element_bytes: usize,
) -> ExpectedMember {
    return ExpectedMember {
        name: Some(name),
        container: None,
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: Some(element_bytes as u32),
        interior: None,
    };
}

/// One anonymous container,
/// which the format writes no name for.
const fn anonymous(bit_offset: usize, byte_size: usize, kind: ContainerKind) -> ExpectedMember {
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

/// One named member whose record is compared member by member.
const fn descended(
    name: &'static str,
    bit_offset: usize,
    byte_size: usize,
    interior: &'static ExpectedContainer,
) -> ExpectedMember {
    return ExpectedMember {
        name: Some(name),
        container: None,
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: None,
        interior: Some(interior),
    };
}

/// The members of `dmu_objset_stats`,
/// every one of them,
/// including the flag `zfs-2.4.4` added in the byte that was padding.
static DMU_OBJSET_STATS_MEMBERS: [ExpectedMember; 9] = [
    member("dds_num_clones", offset_of!(DmuObjsetStats, dds_num_clones) * 8, size_of::<u64>()),
    member(
        "dds_creation_txg",
        offset_of!(DmuObjsetStats, dds_creation_txg) * 8,
        size_of::<u64>(),
    ),
    member("dds_guid", offset_of!(DmuObjsetStats, dds_guid) * 8, size_of::<u64>()),
    member("dds_type", offset_of!(DmuObjsetStats, dds_type) * 8, size_of::<u32>()),
    member(
        "dds_is_snapshot",
        offset_of!(DmuObjsetStats, dds_is_snapshot) * 8,
        size_of::<u8>(),
    ),
    member(
        "dds_inconsistent",
        offset_of!(DmuObjsetStats, dds_inconsistent) * 8,
        size_of::<u8>(),
    ),
    member(
        "dds_redacted",
        offset_of!(DmuObjsetStats, dds_redacted) * 8,
        size_of::<u8>(),
    ),
    array_member(
        "dds_origin",
        offset_of!(DmuObjsetStats, dds_origin) * 8,
        STRING_BYTES,
        size_of::<u8>(),
    ),
    member("dds_flags", offset_of!(DmuObjsetStats, dds_flags) * 8, size_of::<u8>()),
];

/// `dmu_objset_stats`,
/// compared member by member.
static DMU_OBJSET_STATS: ExpectedContainer = ExpectedContainer {
    name: "dmu_objset_stats",
    kind: ContainerKind::Struct,
    byte_size: size_of::<DmuObjsetStats>() as u32,
    members: &DMU_OBJSET_STATS_MEMBERS,
};

/// The members of `zinject_record`,
/// every one of them,
/// including the two counters `zfs-2.4.4` added.
static ZINJECT_MEMBERS: [ExpectedMember; 19] = [
    member("zi_objset", offset_of!(ZinjectRecord, zi_objset) * 8, size_of::<u64>()),
    member("zi_object", offset_of!(ZinjectRecord, zi_object) * 8, size_of::<u64>()),
    member("zi_start", offset_of!(ZinjectRecord, zi_start) * 8, size_of::<u64>()),
    member("zi_end", offset_of!(ZinjectRecord, zi_end) * 8, size_of::<u64>()),
    member("zi_guid", offset_of!(ZinjectRecord, zi_guid) * 8, size_of::<u64>()),
    member("zi_level", offset_of!(ZinjectRecord, zi_level) * 8, size_of::<u32>()),
    member("zi_error", offset_of!(ZinjectRecord, zi_error) * 8, size_of::<u32>()),
    member("zi_type", offset_of!(ZinjectRecord, zi_type) * 8, size_of::<u64>()),
    member("zi_freq", offset_of!(ZinjectRecord, zi_freq) * 8, size_of::<u32>()),
    member("zi_failfast", offset_of!(ZinjectRecord, zi_failfast) * 8, size_of::<u32>()),
    array_member(
        "zi_func",
        offset_of!(ZinjectRecord, zi_func) * 8,
        STRING_BYTES,
        size_of::<u8>(),
    ),
    member("zi_iotype", offset_of!(ZinjectRecord, zi_iotype) * 8, size_of::<u32>()),
    member("zi_duration", offset_of!(ZinjectRecord, zi_duration) * 8, size_of::<i32>()),
    member("zi_timer", offset_of!(ZinjectRecord, zi_timer) * 8, size_of::<u64>()),
    member("zi_nlanes", offset_of!(ZinjectRecord, zi_nlanes) * 8, size_of::<u64>()),
    member("zi_cmd", offset_of!(ZinjectRecord, zi_cmd) * 8, size_of::<u32>()),
    member("zi_dvas", offset_of!(ZinjectRecord, zi_dvas) * 8, size_of::<u32>()),
    member(
        "zi_match_count",
        offset_of!(ZinjectRecord, zi_match_count) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_inject_count",
        offset_of!(ZinjectRecord, zi_inject_count) * 8,
        size_of::<u64>(),
    ),
];

/// `zinject_record`,
/// compared member by member,
/// and the union's largest member.
static ZC_INJECT_RECORD: ExpectedContainer = ExpectedContainer {
    name: "zinject_record",
    kind: ContainerKind::Struct,
    byte_size: size_of::<ZinjectRecord>() as u32,
    members: &ZINJECT_MEMBERS,
};

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
    member("zc_flags", offset_of!(ZcDestroyControl, zc_flags) * 8, size_of::<u32>()),
    member(
        "zc_action_handle",
        offset_of!(ZcDestroyControl, zc_action_handle) * 8,
        size_of::<u64>(),
    ),
];

/// The anonymous structure inside the union.
static DESTROY_CONTROL: ExpectedContainer = ExpectedContainer {
    name: "",
    kind: ContainerKind::Struct,
    byte_size: size_of::<ZcDestroyControl>() as u32,
    members: &DESTROY_CONTROL_MEMBERS,
};

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

/// The members of `zfs_cmd`,
/// every field of this crate's mirror in declaration order.
static ZFS_CMD_MEMBERS: [ExpectedMember; 32] = [
    array_member("zc_name", offset_of!(ZfsCmd, zc_name) * 8, NAME_BYTES, size_of::<u8>()),
    member("zc_nvlist_src", offset_of!(ZfsCmd, zc_nvlist_src) * 8, size_of::<u64>()),
    member(
        "zc_nvlist_src_size",
        offset_of!(ZfsCmd, zc_nvlist_src_size) * 8,
        size_of::<u64>(),
    ),
    member("zc_nvlist_dst", offset_of!(ZfsCmd, zc_nvlist_dst) * 8, size_of::<u64>()),
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
    member("zc_history", offset_of!(ZfsCmd, zc_history) * 8, size_of::<u64>()),
    array_member("zc_value", offset_of!(ZfsCmd, zc_value) * 8, VALUE_BYTES, size_of::<u8>()),
    array_member("zc_string", offset_of!(ZfsCmd, zc_string) * 8, STRING_BYTES, size_of::<u8>()),
    member("zc_guid", offset_of!(ZfsCmd, zc_guid) * 8, size_of::<u64>()),
    member("zc_nvlist_conf", offset_of!(ZfsCmd, zc_nvlist_conf) * 8, size_of::<u64>()),
    member(
        "zc_nvlist_conf_size",
        offset_of!(ZfsCmd, zc_nvlist_conf_size) * 8,
        size_of::<u64>(),
    ),
    member("zc_cookie", offset_of!(ZfsCmd, zc_cookie) * 8, size_of::<u64>()),
    member("zc_objset_type", offset_of!(ZfsCmd, zc_objset_type) * 8, size_of::<u64>()),
    member("zc_perm_action", offset_of!(ZfsCmd, zc_perm_action) * 8, size_of::<u64>()),
    member("zc_history_len", offset_of!(ZfsCmd, zc_history_len) * 8, size_of::<u64>()),
    member(
        "zc_history_offset",
        offset_of!(ZfsCmd, zc_history_offset) * 8,
        size_of::<u64>(),
    ),
    member("zc_obj", offset_of!(ZfsCmd, zc_obj) * 8, size_of::<u64>()),
    member("zc_iflags", offset_of!(ZfsCmd, zc_iflags) * 8, size_of::<u64>()),
    member("zc_share", offset_of!(ZfsCmd, zc_share) * 8, size_of::<ZfsShare>()),
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
    member("zc_cleanup_fd", offset_of!(ZfsCmd, zc_cleanup_fd) * 8, size_of::<i32>()),
    member("zc_simple", offset_of!(ZfsCmd, zc_simple) * 8, size_of::<u8>()),
    array_member("zc_pad", offset_of!(ZfsCmd, zc_pad) * 8, size_of::<[u8; 3]>(), size_of::<u8>()),
    member("zc_sendobj", offset_of!(ZfsCmd, zc_sendobj) * 8, size_of::<u64>()),
    member("zc_fromobj", offset_of!(ZfsCmd, zc_fromobj) * 8, size_of::<u64>()),
    member("zc_createtxg", offset_of!(ZfsCmd, zc_createtxg) * 8, size_of::<u64>()),
    member("zc_stat", offset_of!(ZfsCmd, zc_stat) * 8, size_of::<ZfsStat>()),
    member("zc_zoneid", offset_of!(ZfsCmd, zc_zoneid) * 8, size_of::<u64>()),
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
    return vec![&ZFS_CMD, &DMU_OBJSET_STATS, &ZC_INJECT_RECORD, &DESTROY_CONTROL];
}
