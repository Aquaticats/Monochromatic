//! What:
//!  The record's one text:
//!  the lines it holds,
//!  the `none` word a refusal leaves,
//!  and the four path lines that follow the count.
//! Why:
//!  The file is what a person and `doctor` read,
//!  and the writer proves the parser reads this text back before it writes a
//!  byte,
//!  so the text is one subject and the grammar that reads it another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(applied().text()).toContain('\nbeneath 1\nbeneath-path ');
//! ```

use super::*;
use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};

/// What:
///  One answer a step reached,
///  with every field the writer fills.
/// Why:
///  A control that changes one field must not change another,
///  so the whole record is stated once and each malformed shape takes its
///  text and breaks one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const applied = (): PropagationRecord => ({ boot: 'b0ot', mountId: 9001, ... });
/// ```
fn applied() -> PropagationRecord {
    return PropagationRecord {
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
        mount_id: Some(9001),
        mount_id_old: Some(41),
        mount_namespace_id: Some(4_022_345_678),
        checkout: Some(String::from("/home/user/Monochromatic")),
        source: Some(String::from("pool/home/monochromatic-git-checkout")),
        before: Some(MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("shared")),
        after: Some(MountPropagation::of_the_kernel_set(libc::MS_PRIVATE).expect("private")),
        master: None,
        beneath: SharedMountsBeneath::Counted {
            count: 1,
            paths: vec![String::from(
                "/home/user/Monochromatic/package/vm-builder/cli/output",
            )],
        },
        outcome: PropagationOutcome::Applied,
        step: PropagationStep::NoStep,
        errno: None,
        time: String::from("2026-11-27T21:00:00.000-05:00"),
    };
}

/// What:
///  One refusal a run filed before it read the mount.
/// Why:
///  The refusal's record is the other shape the writer makes,
///  and every field it never reached must read `none`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = (): PropagationRecord => ({ boot: 'b0ot', mountId: undefined, ..., beneath: { kind: 'unread' }, outcome: 'refused', step: 'open', errno: 2 });
/// ```
fn refused() -> PropagationRecord {
    return PropagationRecord {
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
        mount_id: None,
        mount_id_old: None,
        mount_namespace_id: None,
        checkout: None,
        source: None,
        before: None,
        after: None,
        master: None,
        beneath: SharedMountsBeneath::Unread,
        outcome: PropagationOutcome::Refused,
        step: PropagationStep::Open,
        errno: Some(libc::ENOENT),
        time: String::from("2026-11-27T21:00:00.000-05:00"),
    };
}

#[test]
fn a_full_answer_is_written_as_its_own_lines_in_the_records_order() {
    let text: String = applied().text();
    let first: &str = text.lines().next().expect("the version line");
    assert_eq!(first, "v1");
    assert!(text.contains("\nkind checkout-propagation\n"));
    assert!(text.contains("\nmount-id 9001\n"));
    assert!(text.contains("\nmount-id-old 41\n"));
    assert!(text.contains("\nbefore shared\n"));
    assert!(text.contains("\nafter private\n"));
    assert!(text.contains("\noutcome applied\n"));
    assert!(text.contains("\nstep none\n"));
    assert!(text.ends_with('\n'), "the file ends with one newline");
}

#[test]
fn a_refusals_text_carries_none_where_it_never_looked_and_unread_for_the_mounts_beneath() {
    let text: String = refused().text();
    assert!(text.contains("\nmount-id none\n"), "the text: {text}");
    assert!(text.contains("\nbeneath unread\n"), "the text: {text}");
    assert!(text.contains("\nstep open\n"));
    assert!(text.contains("\nerrno 2\n"));
    assert!(text.contains("\noutcome refused\n"));
}

#[test]
fn the_paths_follow_the_count_one_line_each() {
    let record: PropagationRecord = PropagationRecord {
        beneath: SharedMountsBeneath::Counted {
            count: 5,
            paths: vec![
                String::from("/home/user/Monochromatic/a"),
                String::from("/home/user/Monochromatic/b"),
                String::from("/home/user/Monochromatic/c"),
                String::from("/home/user/Monochromatic/d"),
            ],
        },
        ..applied()
    };
    let text: String = record.text();
    assert!(text.contains("\nbeneath 5\n"));
    assert_eq!(
        text.matches("\nbeneath-path ").count(),
        4,
        "the record names four paths and counts five: {text}"
    );
}
