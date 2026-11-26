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
//!  What a member is belongs beside the member tables,
//! `approved_module_member_fixture.rs`,
//! and the pair that types one member
//!  from a base file belongs to `base_file_fixture.rs`.

/// The one change a control makes,
/// re-exported so a test names it beside the builder.
pub use super::approved_module_member_fixture::Change;
/// The member tables,
/// their bit offsets and the types they name.
use super::approved_module_member_fixture::{
    MemberTypes,
    bits,
    command_members,
    member_types,
};
/// The writer of the bytes.
use super::blob_fixture::{
    Blob,
    WrittenMember,
};

/// `struct zfs_cmd`'s declared size,
/// `(MAXPATHLEN * 3) + MAXNAMELEN + 1200` at Linux's field widths.
pub const COMMAND_BYTES: u32 = 13_744;
/// The number one control gives `ZFS_IOC_OBJSET_STATS`,
/// whose 32 bits as a signed integer are `-2` and as an unsigned one are not.
const NEGATIVE_REQUEST: i32 = -2;

/// The approved declaration with one change,
/// written as a module over an empty base file.
pub fn module_with(change: Change) -> Blob {
    return module_over(&Blob::new(), change);
}

/// What:
///  The approved declaration with one change over the base file given.
/// Why:
///   A module's ids and string offsets start past its base file's,
/// and this file's
///  writer takes both from the base it is handed,
/// so a control can move one member's
///  type and its name into the base file without shifting another number.
pub fn module_over(
    base: &Blob,
    change: Change,
) -> Blob {
    let mut blob: Blob = Blob::over_base(base);
    let types: MemberTypes = member_types(&mut blob, change);
    blob.structure("zfs_cmd", COMMAND_BYTES, &command_members(&types, change));
    blob.enumeration("zfs_ioc", 4, &request_values(change));
    return blob;
}

/// The approved declaration with one extra member in `dmu_objset_stats`
/// whose name offset points past the file's string section,
/// so no reader can read that member.
pub fn module_with_an_unreadable_extra_member() -> Blob {
    let mut blob: Blob = Blob::over_base(&Blob::new());
    let types: MemberTypes = member_types(&mut blob, Change::None);
    blob.structure(
        "zfs_cmd",
        COMMAND_BYTES,
        &command_members(&types, Change::None),
    );
    blob.enumeration("zfs_ioc", 4, &request_values(Change::None));
    blob.member_with_unreadable_name(types.stats, types.byte, bits(287));
    return blob;
}

/// What:
///  The approved declaration over the base file given,
/// whose one member the caller
///  types and names instead of this file's own entry.
/// Why:
///   A split module may type and name a member from the file it extends,
/// and the comparison
///  reads the module through both files,
/// so a control hands the base file's own id
///  and name here and nothing else moves.
pub(super) fn module_over_a_member(
    base: &Blob,
    name: &'static str,
    type_id: u32,
    name_offset: u32,
) -> Blob {
    let mut blob: Blob = Blob::over_base(base);
    let types: MemberTypes = member_types(&mut blob, Change::None);
    let members: Vec<WrittenMember> = command_members(&types, Change::None)
        .into_iter()
        .map(|member: WrittenMember| {
            if member.name != name {
                return member;
            }
            return WrittenMember {
                name: member.name,
                type_id,
                bit_offset: member.bit_offset,
                name_offset: Some(name_offset),
            };
        })
        .collect();
    blob.structure("zfs_cmd", COMMAND_BYTES, &members);
    blob.enumeration("zfs_ioc", 4, &request_values(Change::None));
    return blob;
}

/// The approved declaration with the kind flag set on every record,
/// and every member's offset and width exactly as approved,
/// which is the shape a module's own tools write when the flag is on.
pub fn module_with_the_kind_flag_set() -> Blob {
    return module_with(Change::None).with_kind_flag_on_records();
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
    if change == Change::AppendedRequest {
        values.push(("ZFS_IOC_LAST", 0x5a8a));
    } else {
        values.push(("ZFS_IOC_LAST", 0x5a89));
    }
    if change == Change::SignedEnumerator {
        let stats: &mut (&'static str, i32) = values
            .iter_mut()
            .find(|(name, _)| return *name == "ZFS_IOC_OBJSET_STATS")
            .expect("the approved table names the stats request");
        stats.1 = NEGATIVE_REQUEST;
    }
    return values;
}
