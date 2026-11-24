//! Controls of the mount table and number readers:
//!  a mount the table names at the slot is found,
//!  a table that names it at another mount point is not,
//!  and a number written into a stack array reads back as itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(namesMount(TABLE, 1201, '/run/x/slot')).toBe(true);
//! expect(namesMount(TABLE, 1201, '/run/x/other')).toBe(false);
//! ```

use super::{names_mount, names_mount_id, parse_number, write_number};

/// One mount-table line the read-back sees,
/// in the kernel's own field order.
const TABLE: &[u8] = b"1200 31 0:26 / / rw,nosuid - tmpfs tmpfs rw,size=64k\n1201 31 0:27 / /run/x/slot rw,nosuid - tmpfs tmpfs rw,size=64k,mode=0700\n";

/// What:
///  The mount the attach made is found by its ID at its mount point,
///  and the same ID at another mount point is not.
#[test]
fn a_mount_is_found_by_its_id_at_its_own_mount_point_only() {
    assert!(names_mount(TABLE, 1201, b"/run/x/slot"));
    assert!(!names_mount(TABLE, 1201, b"/run/x/other"), "another mount point of the same table is another mount");
    assert!(!names_mount(TABLE, 1202, b"/run/x/slot"), "another ID at that mount point is another mount");
    assert!(names_mount_id(TABLE, 1201));
    assert!(!names_mount_id(TABLE, 1202));
}

/// What:
///  An ID that shares a prefix with another is not that other,
///  which is why the number is compared with the space after it.
#[test]
fn an_id_that_is_a_prefix_of_another_is_not_found() {
    assert!(names_mount_id(TABLE, 1200));
    assert!(!names_mount_id(TABLE, 120));
    assert!(!names_mount_id(TABLE, 12_010));
}

/// What:
///  A number written into a stack array reads back as itself,
///  from nothing to the largest ID a table carries.
#[test]
fn a_number_written_into_a_stack_array_reads_back_as_itself() {
    let mut bytes: [u8; 24] = [0; 24];
    for number in [0_u64, 1, 9, 10, 99, 4096, 1_234_567, u64::from(u32::MAX)] {
        let written: usize = write_number(number, bytes.as_mut_slice());
        assert_eq!(parse_number(&bytes[..written]), Some(number), "{number} written as {}", String::from_utf8_lossy(&bytes[..written]));
    }
    assert_eq!(parse_number(b"12x"), None, "a byte that is no digit names no number");
    assert_eq!(parse_number(b""), None, "no digits name no number");
}
