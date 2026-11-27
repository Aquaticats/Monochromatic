//! What:
//!  `zinject_record` as the loaded module must declare it:
//! every member's bit offset and byte size,
//! and the record's own size.
//! Why:
//!  The record is the union's largest member,
//! so its size decides where the union ends,
//! and a module that changed it would put every field after the union at a
//!  different offset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const zinjectRecord = { name: 'zinject_record', kind: 'struct', byteSize: 368, members: [...] };
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
use crate::layout::ZinjectRecord;
/// The operators the table is built from.
use std::mem::{
    offset_of,
    size_of,
};

/// The members of `zinject_record`,
/// every one of them,
/// including the two counters `zfs-2.4.4` added.
static ZINJECT_MEMBERS: [ExpectedMember; 19] = [
    member(
        "zi_objset",
        offset_of!(ZinjectRecord, zi_objset) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_object",
        offset_of!(ZinjectRecord, zi_object) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_start",
        offset_of!(ZinjectRecord, zi_start) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_end",
        offset_of!(ZinjectRecord, zi_end) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_guid",
        offset_of!(ZinjectRecord, zi_guid) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_level",
        offset_of!(ZinjectRecord, zi_level) * 8,
        size_of::<u32>(),
    ),
    member(
        "zi_error",
        offset_of!(ZinjectRecord, zi_error) * 8,
        size_of::<u32>(),
    ),
    member(
        "zi_type",
        offset_of!(ZinjectRecord, zi_type) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_freq",
        offset_of!(ZinjectRecord, zi_freq) * 8,
        size_of::<u32>(),
    ),
    member(
        "zi_failfast",
        offset_of!(ZinjectRecord, zi_failfast) * 8,
        size_of::<u32>(),
    ),
    array_member(
        "zi_func",
        offset_of!(ZinjectRecord, zi_func) * 8,
        STRING_BYTES,
        size_of::<u8>(),
    ),
    member(
        "zi_iotype",
        offset_of!(ZinjectRecord, zi_iotype) * 8,
        size_of::<u32>(),
    ),
    member(
        "zi_duration",
        offset_of!(ZinjectRecord, zi_duration) * 8,
        size_of::<i32>(),
    ),
    member(
        "zi_timer",
        offset_of!(ZinjectRecord, zi_timer) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_nlanes",
        offset_of!(ZinjectRecord, zi_nlanes) * 8,
        size_of::<u64>(),
    ),
    member(
        "zi_cmd",
        offset_of!(ZinjectRecord, zi_cmd) * 8,
        size_of::<u32>(),
    ),
    member(
        "zi_dvas",
        offset_of!(ZinjectRecord, zi_dvas) * 8,
        size_of::<u32>(),
    ),
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
pub(super) static ZC_INJECT_RECORD: ExpectedContainer = ExpectedContainer {
    name: "zinject_record",
    kind: ContainerKind::Struct,
    byte_size: size_of::<ZinjectRecord>() as u32,
    members: &ZINJECT_MEMBERS,
};
