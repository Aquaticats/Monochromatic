//! What:
//!  `dmu_objset_stats` as the loaded module must declare it:
//! every member's bit offset and byte size,
//! and the record's own size.
//! Why:
//!  A request reads the dataset's type,
//! identity and origin out of this record,
//! so a module whose record grew or moved a member would answer
//!  another field's bytes,
//! and the comparison has to see it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const dmuObjsetStats = { name: 'dmu_objset_stats', kind: 'struct', byteSize: 64, members: [...] };
//! ```

/// The container kind the record is declared with.
use super::declaration::ContainerKind;
/// What:
///  One member or container the module must declare,
/// and how a table states one.
/// Why:
///  The record below is written through these builders,
/// so its offsets and sizes come from the compiler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ExpectedContainer, ExpectedMember, array_member, member } from './expected_member_and_container';
/// ```
use super::expected_member_and_container::{
    ExpectedContainer,
    ExpectedMember,
    STRING_BYTES,
    array_member,
    member,
};
/// The crate's own mirror of the record.
use crate::layout::DmuObjsetStats;
/// The operators the table is built from.
use std::mem::{
    offset_of,
    size_of,
};

/// The members of `dmu_objset_stats`,
/// every one of them,
/// including the flag `zfs-2.4.4` added in the byte that was padding.
static DMU_OBJSET_STATS_MEMBERS: [ExpectedMember; 9] = [
    member(
        "dds_num_clones",
        offset_of!(DmuObjsetStats, dds_num_clones) * 8,
        size_of::<u64>(),
    ),
    member(
        "dds_creation_txg",
        offset_of!(DmuObjsetStats, dds_creation_txg) * 8,
        size_of::<u64>(),
    ),
    member(
        "dds_guid",
        offset_of!(DmuObjsetStats, dds_guid) * 8,
        size_of::<u64>(),
    ),
    member(
        "dds_type",
        offset_of!(DmuObjsetStats, dds_type) * 8,
        size_of::<u32>(),
    ),
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
    member(
        "dds_flags",
        offset_of!(DmuObjsetStats, dds_flags) * 8,
        size_of::<u8>(),
    ),
];

/// `dmu_objset_stats`,
/// compared member by member.
pub(super) static DMU_OBJSET_STATS: ExpectedContainer = ExpectedContainer {
    name: "dmu_objset_stats",
    kind: ContainerKind::Struct,
    byte_size: size_of::<DmuObjsetStats>() as u32,
    members: &DMU_OBJSET_STATS_MEMBERS,
};
