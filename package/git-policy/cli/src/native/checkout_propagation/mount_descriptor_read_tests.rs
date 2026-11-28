//! What:
//!  The descriptor read's controls:
//!  an answer with every field filled,
//!  an answer from a kernel that reports no namespace,
//!  an answer for another mount than the descriptor's own,
//!  a propagation set this build does not know,
//!  and an answer that never reported the mount's own numbers.
//! Why:
//!  The step proves which mount it read from this answer and from nothing
//!  else,
//!  so a field read at the wrong offset or a set guessed at would set a
//!  different instance private than the one it verified
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "What it verifies before").
//!  The answer's layout is the kernel's `struct statmount`,
//!  so the control builds one at the offsets the uapi prints rather than
//!  calling a kernel no test on this machine may call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the facts of a full answer', () => expect(factsOf(answer({ mountId: 9001, ... }), 9001).mountId).toBe(9001));
//! ```

use super::*;
use std::path::PathBuf;

/// What:
///  The fields one control writes into a synthetic answer.
/// Why:
///  Every field the read takes is stated once,
///  so a control that leaves one out says which answer a kernel would give
///  instead,
///  and the writer below puts each at the uapi's own offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnAnswer = { mask?: number; mountId?: number; mountIdOld?: number; propagation?: number; peerGroup?: number; master?: number; namespaceId?: number; point?: string; fsType?: string; source?: string };
/// ```
#[derive(Clone, Copy)]
struct AnAnswer {
    /// The mask of the fields the kernel filled,
    /// absent for the full answer every field appears in.
    mask: Option<u64>,
    /// The mount's unique number.
    mount_id: u64,
    /// The number the mount table prints.
    mount_id_old: u64,
    /// The propagation set.
    propagation: u64,
    /// The peer group's number,
    /// zero when the mount is not shared.
    peer_group: u64,
    /// The master's number,
    /// zero when the mount receives nothing.
    master: u64,
    /// The namespace's number.
    namespace_id: u64,
    /// The mount point.
    point: &'static str,
    /// The filesystem type.
    fs_type: &'static str,
    /// The source,
    /// absent for a kernel older than Linux 6.13.
    source: Option<&'static str>,
}

impl AnAnswer {
    /// The answer a current kernel gives for one checkout mount.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const full = (): AnAnswer => ({ mask: undefined, mountId: 9001, ... });
    /// ```
    fn the_full_answer() -> Self {
        return AnAnswer {
            mask: None,
            mount_id: 9001,
            mount_id_old: 41,
            propagation: libc::MS_PRIVATE,
            peer_group: 0,
            master: 0,
            namespace_id: 4_022_345_678,
            point: "/home/user/Monochromatic",
            fs_type: "zfs",
            source: Some("pool/home/monochromatic-git-checkout"),
        };
    }
}

/// What:
///  One synthetic `struct statmount` answer,
///  laid out at the offsets the uapi prints.
/// Why:
///  The read is a little-endian reader at fixed offsets,
///  so the control writes each field there and the strings after the header,
///  which is the one way to prove the offsets without a kernel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = (a: AnAnswer): Uint8Array => { writeU64(buf, 40, a.mountId); ... };
/// ```
fn answer(one: AnAnswer) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0_u8; STATMOUNT_HEADER_BYTES + STATMOUNT_STRING_BYTES];
    let mask: u64 = one.mask.unwrap_or(
        STATMOUNT_MNT_BASIC
            | STATMOUNT_MNT_POINT
            | STATMOUNT_FS_TYPE
            | STATMOUNT_MNT_NS_ID
            | STATMOUNT_SB_SOURCE,
    );
    write_number(&mut bytes, MASK_AT, mask);
    write_number(&mut bytes, MOUNT_ID_AT, one.mount_id);
    write_field(&mut bytes, MOUNT_ID_OLD_AT, one.mount_id_old);
    write_number(&mut bytes, PROPAGATION_AT, one.propagation);
    write_number(&mut bytes, PEER_GROUP_AT, one.peer_group);
    write_number(&mut bytes, MASTER_AT, one.master);
    write_number(&mut bytes, NAMESPACE_ID_AT, one.namespace_id);
    // The three strings take one place each, so a control reads the same offsets whatever a string's length is.
    write_string(&mut bytes, FS_TYPE_AT_PLACE, one.fs_type);
    write_field(&mut bytes, FS_TYPE_AT, u64::from(FS_TYPE_AT_PLACE));
    write_string(&mut bytes, MOUNT_POINT_AT_PLACE, one.point);
    write_field(&mut bytes, MOUNT_POINT_AT, u64::from(MOUNT_POINT_AT_PLACE));
    let source: &'static str = one.source.unwrap_or("");
    write_string(&mut bytes, SOURCE_AT_PLACE, source);
    write_field(&mut bytes, SOURCE_AT, u64::from(SOURCE_AT_PLACE));
    return bytes;
}

/// Where the control writes the filesystem type's string,
/// which is the answer's first string.
const FS_TYPE_AT_PLACE: u32 = 0;

/// Where the control writes the mount point's string.
const MOUNT_POINT_AT_PLACE: u32 = 64;

/// Where the control writes the source's string.
const SOURCE_AT_PLACE: u32 = 256;

/// One little-endian number written at `offset`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeNumber = (buf: Uint8Array, offset: number, value: number): void => { ... };
/// ```
fn write_number(
    bytes: &mut [u8],
    offset: usize,
    value: u64,
) {
    for index in 0..8 {
        bytes[offset + index] = (value >> (8 * index)) as u8;
    }
}

/// One little-endian 32-bit field written at `offset`,
/// which is what `struct statmount` gives the filesystem type's offset,
/// the reused mount number and the two string offsets this read fills.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeField = (buf: Uint8Array, offset: number, value: number): void => { new DataView(buf.buffer).setUint32(offset, value, true); };
/// ```
fn write_field(
    bytes: &mut [u8],
    offset: usize,
    value: u64,
) {
    for index in 0..4 {
        bytes[offset + index] = (value >> (8 * index)) as u8;
    }
}

/// One NUL-terminated string written at `at`,
/// which the answer's own field names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeString = (buf: Uint8Array, at: number, text: string): void => { ... };
/// ```
fn write_string(
    bytes: &mut [u8],
    at: u32,
    text: &str,
) {
    let start: usize = STATMOUNT_HEADER_BYTES + usize::try_from(at).unwrap_or(0);
    for (index, byte) in text.as_bytes().iter().enumerate() {
        bytes[start + index] = *byte;
    }
}

#[test]
fn a_full_answer_reports_both_numbers_the_set_the_group_the_master_and_the_names() {
    let facts: DescriptorMount =
        facts_of(answer(AnAnswer::the_full_answer()).as_slice(), 9001).expect("the full answer");
    assert_eq!(
        facts.mount_id, 9001,
        "the unique number decides the instance"
    );
    assert_eq!(
        facts.mount_id_old, 41,
        "the reused number finds the table's row"
    );
    assert_eq!(facts.namespace_id, Some(4_022_345_678));
    assert!(facts.reports_namespace_id, "this kernel reports the field");
    assert_eq!(facts.propagation.word(), "private");
    assert_eq!(facts.peer_group, None, "a private mount has no peer group");
    assert_eq!(facts.master, None, "a private mount receives nothing");
    assert_eq!(facts.mount_point, PathBuf::from("/home/user/Monochromatic"));
    assert_eq!(facts.filesystem_type, "zfs");
    assert_eq!(
        facts.source.as_deref(),
        Some("pool/home/monochromatic-git-checkout")
    );
}

#[test]
fn a_shared_and_slave_answer_reports_the_group_and_the_master_it_detaches() {
    let mut one: AnAnswer = AnAnswer::the_full_answer();
    one.propagation = libc::MS_SHARED | libc::MS_SLAVE;
    one.peer_group = 247;
    one.master = 3;
    let facts: DescriptorMount = facts_of(answer(one).as_slice(), 9001).expect("the shared answer");
    assert_eq!(facts.propagation.word(), "shared+slave");
    assert_eq!(facts.peer_group, Some(247));
    assert_eq!(
        facts.master,
        Some(3),
        "the master the set detached is what the record says the cost was"
    );
}

#[test]
fn an_answer_without_the_namespace_field_is_read_as_no_namespace_rather_than_zero() {
    let mut one: AnAnswer = AnAnswer::the_full_answer();
    // The answer a kernel before Linux 6.9 gives: the bits it knows, and the namespace word left spare.
    one.mask = Some(STATMOUNT_MNT_BASIC | STATMOUNT_MNT_POINT | STATMOUNT_FS_TYPE);
    let facts: DescriptorMount = facts_of(answer(one).as_slice(), 9001).expect("the older answer");
    assert_eq!(facts.namespace_id, None);
    assert!(
        !facts.reports_namespace_id,
        "a kernel that never reports the field is not a mount in no namespace"
    );
}

#[test]
fn an_answer_for_another_mount_is_no_answer() {
    let failure: MountReadFailure = facts_of(answer(AnAnswer::the_full_answer()).as_slice(), 7)
        .expect_err("another mount's answer");
    assert!(
        failure.message().contains("9001") && failure.message().contains("is 7"),
        "the refusal names both the mount the answer was about and the descriptor's own number: {}",
        failure.message()
    );
}

#[test]
fn a_propagation_set_this_build_does_not_know_is_no_answer() {
    let mut one: AnAnswer = AnAnswer::the_full_answer();
    // The kernel never makes an unbindable mount shared, so this answer is another layout's.
    one.propagation = libc::MS_UNBINDABLE | libc::MS_SHARED;
    let failure: MountReadFailure =
        facts_of(answer(one).as_slice(), 9001).expect_err("a set this build does not know");
    assert!(
        failure.message().contains("propagation"),
        "the refusal says the set is what it could not read: {}",
        failure.message()
    );
}

#[test]
fn an_answer_without_the_mounts_own_numbers_is_no_answer() {
    let mut one: AnAnswer = AnAnswer::the_full_answer();
    one.mask = Some(STATMOUNT_MNT_POINT | STATMOUNT_FS_TYPE);
    let failure: MountReadFailure =
        facts_of(answer(one).as_slice(), 9001).expect_err("an answer without the basic fields");
    assert!(
        failure.message().contains("numbers"),
        "the refusal names what the kernel did not report: {}",
        failure.message()
    );
}

#[test]
fn a_refused_call_carries_the_number_it_answered_and_a_kernel_that_cannot_answer_carries_none() {
    let refused: MountReadFailure = MountReadFailure::Refused {
        call: "statmount",
        errno: libc::ENOENT,
    };
    let unsupported: MountReadFailure = MountReadFailure::Unsupported {
        why: String::from("this kernel reports no source"),
    };
    assert_eq!(refused.errno(), Some(libc::ENOENT));
    assert_eq!(unsupported.errno(), None);
    assert!(refused.message().contains("statmount"));
    assert!(unsupported.message().contains("no source"));
}
