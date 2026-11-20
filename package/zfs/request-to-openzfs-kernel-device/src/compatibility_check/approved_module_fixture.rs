//! What:
//!  The layout the approved module declares,
//! written as data:
//! `zfs_cmd`,
//! its
//!  two records,
//! the anonymous union and the structure inside it,
//! and `enum zfs_ioc`,
//! with one
//!  field movable by a control.
//! Why:
//!  The deviceless controls compare this declaration with the table this crate
//! compiles,
//! and then move one field to prove the gate refuses the module that moved
//!  it.
//!  Every offset below is the byte offset the design's `pahole` read of
//!  `zfs-2.4.4` reports
//! (`doc/planning/cli-git-zfs-runtime-layout-checker-design.md`,
//!  "What the BTF carries"),
//! so a test that passes says the crate agrees with that read,
//! and a
//!  test that refuses says the gate sees the one field the test moved.
//!  The fixture is written by hand rather than from the expectation table,
//! so the
//!  two are independent statements of the same layout.

/// The writer of the bytes.
use super::blob_fixture::{Blob, WrittenMember};

/// `unsigned char`,
/// one byte.
const BYTE_BYTES: u32 = 1;
/// `unsigned int`,
/// four bytes.
const WORD_BYTES: u32 = 4;
/// `unsigned long`,
/// eight bytes.
const WIDE_BYTES: u32 = 8;
/// The bits one byte holds.
const BITS_PER_BYTE: u32 = 8;
/// `struct zfs_cmd`'s declared size,
/// `(MAXPATHLEN * 3) + MAXNAMELEN + 1200` at Linux's field widths.
pub const COMMAND_BYTES: u32 = 13_744;
/// `struct dmu_objset_stats`'s declared size.
pub const STATS_BYTES: u32 = 288;
/// `struct zinject_record`'s declared size at `zfs-2.4.4`.
pub const INJECT_BYTES: u32 = 368;
/// `struct zinject_record`'s size before `zfs-2.4.4` added its two counters.
pub const SHORT_INJECT_BYTES: u32 = 352;
/// The anonymous structure's declared size,
/// `sizeof (zinject_record_t) - 16` padded to eight bytes.
pub const DESTROY_BYTES: u32 = 368;
/// The anonymous union's declared size,
/// its largest member.
pub const OVERLAY_BYTES: u32 = 368;
/// `struct zfs_share`'s declared size.
const SHARE_BYTES: u32 = 32;
/// `struct drr_begin`'s declared size.
const BEGIN_BYTES: u32 = 304;
/// `struct zfs_stat`'s declared size.
const STAT_BYTES: u32 = 40;
/// The bytes `zc_name` holds.
const NAME_BYTES: u32 = 4_096;
/// The bytes `zc_value` holds.
const VALUE_BYTES: u32 = 8_192;
/// The bytes `zc_string`,
/// `dds_origin` and `zi_func` each hold.
const STRING_BYTES: u32 = 256;
/// The bytes `zc_pad` holds.
const PAD_BYTES: u32 = 3;
/// The bytes `zc_pad1` holds,
/// `sizeof (zinject_record_t) - 16`.
const DESTROY_PAD_BYTES: u32 = 352;
/// The bytes an empty base file's string section holds,
/// the empty name alone,
/// past which a module's names start.
pub const EMPTY_BASE_STRINGS: u32 = 1;

/// What:
///  The one field a control moves in the approved declaration.
/// Why:
///   A refusal must name the field the module changed,
/// so each test states the
///  change it makes and the fixture writes every other member as the approved
///  module declares it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = 'none' | 'no flags' | 'no union' | 'short inject record' | 'swapped members' | 'extra member' | 'renumbered clone' | 'missing channel program';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change {
    /// The approved declaration.
    None,
    /// `dmu_objset_stats` without `dds_flags`,
    /// the field `zfs-2.4.4` added in the byte that was padding.
    NoDdsFlags,
    /// `zfs_cmd` with the four control fields as direct members,
    /// the shape `zfs-2.2.2` declares,
    /// where the union stands.
    NoUnion,
    /// `zinject_record` at its former 352 bytes,
    /// without the two counters `zfs-2.4.4` added.
    ShortInject,
    /// `zinject_record` with `zi_duration` and `zi_iotype` swapped,
    /// so no member size and no total moves.
    SwappedMembers,
    /// `dmu_objset_stats` with one member this crate does not expect.
    ExtraMember,
    /// `ZFS_IOC_CLONE` renumbered,
    /// with every layout kept.
    RenumberedClone,
    /// `ZFS_IOC_CHANNEL_PROGRAM` absent,
    /// with every layout kept.
    MissingChannelProgram,
}

/// The member types one declaration reads,
/// created in the order the format hands out ids.
pub struct MemberTypes {
    /// `unsigned char`.
    byte: u32,
    /// `unsigned int`,
    /// which carries the enum members this fixture does not model.
    word: u32,
    /// `unsigned long`.
    wide: u32,
    /// A pointer.
    pointer: u32,
    /// A 4,096-byte character array.
    name: u32,
    /// An 8,192-byte character array.
    value: u32,
    /// A 256-byte character array.
    string: u32,
    /// A three-byte character array.
    pad: u32,
    /// `struct zinject_record`.
    inject: u32,
    /// `struct zfs_share`.
    share: u32,
    /// `struct dmu_objset_stats`.
    stats: u32,
    /// `struct drr_begin`.
    begin: u32,
    /// The anonymous union.
    overlay: u32,
    /// `struct zfs_stat`.
    stat: u32,
}

/// The approved declaration with one change,
/// written as a module over an empty base file whose string section is
/// `EMPTY_BASE_STRINGS` bytes.
pub fn module_with(change: Change) -> Blob {
    let mut blob: Blob = Blob::over(EMPTY_BASE_STRINGS);
    let types: MemberTypes = member_types(&mut blob, change);
    blob.structure("zfs_cmd", COMMAND_BYTES, &command_members(&types, change));
    blob.enumeration("zfs_ioc", 4, &request_values(change));
    return blob;
}

/// One member's bit offset from the byte offset `pahole` reports.
const fn bits(bytes: u32) -> u32 {
    return bytes * BITS_PER_BYTE;
}

/// One member at one byte offset.
const fn at(name: &'static str, type_id: u32, bytes: u32) -> WrittenMember {
    return WrittenMember {
        name,
        type_id,
        bit_offset: bits(bytes),
    };
}

/// The member types one declaration is built from.
fn member_types(blob: &mut Blob, change: Change) -> MemberTypes {
    let byte: u32 = blob.int("unsigned char", BYTE_BYTES);
    let word: u32 = blob.int("unsigned int", WORD_BYTES);
    let wide: u32 = blob.int("unsigned long", WIDE_BYTES);
    let pointer: u32 = blob.pointer("pointer");
    let name: u32 = blob.array(byte, NAME_BYTES);
    let value: u32 = blob.array(byte, VALUE_BYTES);
    let string: u32 = blob.array(byte, STRING_BYTES);
    let pad: u32 = blob.array(byte, PAD_BYTES);
    let destroy_pad: u32 = blob.array(byte, DESTROY_PAD_BYTES);
    let stats: u32 = blob.structure(
        "dmu_objset_stats",
        STATS_BYTES,
        &stats_members(wide, word, byte, string, change),
    );
    let inject: u32 = blob.structure(
        "zinject_record",
        inject_bytes(change),
        &inject_members(wide, word, string, change),
    );
    let control: u32 = blob.structure(
        "",
        DESTROY_BYTES,
        &[
            at("zc_pad1", destroy_pad, 0),
            at("zc_defer_destroy", word, 352),
            at("zc_flags", word, 356),
            at("zc_action_handle", wide, 360),
        ],
    );
    let overlay: u32 = blob.union(
        "",
        OVERLAY_BYTES,
        &[
            at("zc_inject_record", inject, 0),
            WrittenMember {
                name: "",
                type_id: control,
                bit_offset: bits(0),
            },
        ],
    );
    let share: u32 = blob.structure("zfs_share", SHARE_BYTES, &[]);
    let begin: u32 = blob.structure("drr_begin", BEGIN_BYTES, &[]);
    let stat: u32 = blob.structure("zfs_stat", STAT_BYTES, &[]);
    return MemberTypes {
        byte,
        word,
        wide,
        pointer,
        name,
        value,
        string,
        pad,
        inject,
        share,
        stats,
        begin,
        overlay,
        stat,
    };
}

/// The inject record's declared size under one change.
fn inject_bytes(change: Change) -> u32 {
    return match change {
        Change::ShortInject => SHORT_INJECT_BYTES,
        _ => INJECT_BYTES,
    };
}

/// `zfs_cmd`'s 32 members,
/// in the order the module declares them.
fn command_members(types: &MemberTypes, change: Change) -> Vec<WrittenMember> {
    let mut members: Vec<WrittenMember> = vec![
        at("zc_name", types.name, 0),
        at("zc_nvlist_src", types.wide, 4_096),
        at("zc_nvlist_src_size", types.wide, 4_104),
        at("zc_nvlist_dst", types.wide, 4_112),
        at("zc_nvlist_dst_size", types.wide, 4_120),
        at("zc_nvlist_dst_filled", types.word, 4_128),
        at("zc_pad2", types.word, 4_132),
        at("zc_history", types.pointer, 4_136),
        at("zc_value", types.value, 4_144),
        at("zc_string", types.string, 12_336),
        at("zc_guid", types.wide, 12_592),
        at("zc_nvlist_conf", types.wide, 12_600),
        at("zc_nvlist_conf_size", types.wide, 12_608),
        at("zc_cookie", types.wide, 12_616),
        at("zc_objset_type", types.wide, 12_624),
        at("zc_perm_action", types.wide, 12_632),
        at("zc_history_len", types.wide, 12_640),
        at("zc_history_offset", types.wide, 12_648),
        at("zc_obj", types.wide, 12_656),
        at("zc_iflags", types.wide, 12_664),
        at("zc_share", types.share, 12_672),
        at("zc_objset_stats", types.stats, 12_704),
        at("zc_begin_record", types.begin, 12_992),
    ];
    match change {
        Change::NoUnion => {
            members.push(at("zc_inject_record", types.inject, 13_296));
            members.push(at("zc_defer_destroy", types.word, 13_648));
            members.push(at("zc_flags", types.word, 13_652));
            members.push(at("zc_action_handle", types.wide, 13_656));
        }
        _ => members.push(at("", types.overlay, 13_296)),
    }
    members.push(at("zc_cleanup_fd", types.word, 13_664));
    members.push(at("zc_simple", types.byte, 13_668));
    members.push(at("zc_pad", types.pad, 13_669));
    members.push(at("zc_sendobj", types.wide, 13_672));
    members.push(at("zc_fromobj", types.wide, 13_680));
    members.push(at("zc_createtxg", types.wide, 13_688));
    members.push(at("zc_stat", types.stat, 13_696));
    members.push(at("zc_zoneid", types.wide, 13_736));
    return members;
}

/// `dmu_objset_stats`'s nine members,
/// the flag `zfs-2.4.4` added last.
fn stats_members(wide: u32, word: u32, byte: u32, origin: u32, change: Change) -> Vec<WrittenMember> {
    let mut members: Vec<WrittenMember> = vec![
        at("dds_num_clones", wide, 0),
        at("dds_creation_txg", wide, 8),
        at("dds_guid", wide, 16),
        at("dds_type", word, 24),
        at("dds_is_snapshot", byte, 28),
        at("dds_inconsistent", byte, 29),
        at("dds_redacted", byte, 30),
        at("dds_origin", origin, 31),
    ];
    if change != Change::NoDdsFlags {
        members.push(at("dds_flags", byte, 287));
    }
    if change == Change::ExtraMember {
        members.push(at("dds_extra", byte, 287));
    }
    return members;
}

/// `zinject_record`'s 19 members,
/// the two counters `zfs-2.4.4` added last.
fn inject_members(wide: u32, word: u32, function: u32, change: Change) -> Vec<WrittenMember> {
    let mut members: Vec<WrittenMember> = vec![
        at("zi_objset", wide, 0),
        at("zi_object", wide, 8),
        at("zi_start", wide, 16),
        at("zi_end", wide, 24),
        at("zi_guid", wide, 32),
        at("zi_level", word, 40),
        at("zi_error", word, 44),
        at("zi_type", wide, 48),
        at("zi_freq", word, 56),
        at("zi_failfast", word, 60),
        at("zi_func", function, 64),
    ];
    if change == Change::SwappedMembers {
        members.push(at("zi_duration", word, 320));
        members.push(at("zi_iotype", word, 324));
    } else {
        members.push(at("zi_iotype", word, 320));
        members.push(at("zi_duration", word, 324));
    }
    members.push(at("zi_timer", wide, 328));
    members.push(at("zi_nlanes", wide, 336));
    members.push(at("zi_cmd", word, 344));
    members.push(at("zi_dvas", word, 348));
    if change == Change::ShortInject {
        return members;
    }
    members.push(at("zi_match_count", wide, 352));
    members.push(at("zi_inject_count", wide, 360));
    return members;
}

/// `enum zfs_ioc`'s numbers this crate sends,
/// with the range's two endpoints.
fn request_values(change: Change) -> Vec<(&'static str, i32)> {
    let mut values: Vec<(&'static str, i32)> = vec![
        ("ZFS_IOC_FIRST", 0x5a00),
        ("ZFS_IOC_OBJSET_STATS", 0x5a12),
        ("ZFS_IOC_DATASET_LIST_NEXT", 0x5a14),
        ("ZFS_IOC_SNAPSHOT_LIST_NEXT", 0x5a15),
        ("ZFS_IOC_SET_PROP", 0x5a16),
        ("ZFS_IOC_DESTROY", 0x5a18),
        ("ZFS_IOC_PROMOTE", 0x5a22),
        ("ZFS_IOC_SNAPSHOT", 0x5a23),
        ("ZFS_IOC_GET_FSACL", 0x5a29),
        ("ZFS_IOC_DESTROY_SNAPS", 0x5a3b),
        ("ZFS_IOC_GET_BOOKMARKS", 0x5a44),
    ];
    if change != Change::RenumberedClone {
        values.push(("ZFS_IOC_CLONE", 0x5a42));
    } else {
        values.push(("ZFS_IOC_CLONE", 0x5a43));
    }
    if change != Change::MissingChannelProgram {
        values.push(("ZFS_IOC_CHANNEL_PROGRAM", 0x5a48));
    }
    values.push(("ZFS_IOC_LAST", 0x5a89));
    return values;
}
