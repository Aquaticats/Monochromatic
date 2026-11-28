//! What:
//!  The propagation set's controls:
//!  the kernel's own word for each of the five sets,
//!  the sets the kernel cannot make,
//!  the word each set is written as and read back from,
//!  and the three flags a table row's tags say.
//! Why:
//!  The step decides on this set and on nothing else,
//!  so a set read wrongly decides wrongly:
//!  a shared and slave mount treated as a slave alone would never be
//!  repaired,
//!  and an unbindable mount treated as shared would be changed where the
//!  project decided to leave it
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "What the repair does with a state that is neither private nor shared",
//!  and "A mount that is shared and slave at once").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a shared and slave set arrives as one set', () => expect(word(MountPropagation.fromKernel(MS_SHARED | MS_SLAVE))).toBe('shared+slave'));
//! ```

use super::*;
use crate::subvolume::fs_mount_options::MountFacts;
use std::path::PathBuf;

/// What:
///  One mount table row,
///  with only the tags the set's reading looks at.
/// Why:
///  The reading takes three fields and this control states exactly those,
///  so a change to the reading that took another field would not be hidden by
///  a row that happened to hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (optionalFields: string[], options: string[]): MountFacts => ({ mountId: 1, parentId: 0, ... });
/// ```
fn row(
    optional_fields: &[&str],
    mount_options: &[&str],
) -> MountFacts {
    return MountFacts {
        mount_id: 1,
        parent_id: 0,
        root: PathBuf::from("/"),
        optional_fields: optional_fields
            .iter()
            .map(|field| return String::from(*field))
            .collect(),
        mount_point: PathBuf::from("/home/user/Monochromatic"),
        filesystem_type: String::from("zfs"),
        source: String::from("pool/home/monochromatic-git-checkout"),
        mount_options: mount_options
            .iter()
            .map(|option| return String::from(*option))
            .collect(),
        super_options: Vec::new(),
    };
}

#[test]
fn a_private_set_is_the_kernels_private_flag_and_nothing_else() {
    let set: MountPropagation =
        MountPropagation::of_the_kernel_set(libc::MS_PRIVATE).expect("a private mount's set");
    assert_eq!(set.word(), PRIVATE_WORD);
    assert!(
        set.is_private(),
        "a private mount passes and receives nothing"
    );
    assert!(!set.carries_shared(), "a private mount passes nothing on");
}

#[test]
fn a_shared_set_carries_shared_and_is_not_private() {
    let set: MountPropagation =
        MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("a shared mount's set");
    assert_eq!(set.word(), SHARED_WORD);
    assert!(set.carries_shared(), "the step sets this set private");
    assert!(!set.is_private(), "a shared mount is not private");
}

#[test]
fn a_shared_and_slave_set_carries_shared_because_the_project_treats_it_so() {
    let set: MountPropagation =
        MountPropagation::of_the_kernel_set(libc::MS_SHARED | libc::MS_SLAVE)
            .expect("the set a machine inside a container has");
    assert_eq!(set.word(), SHARED_AND_SLAVE_WORD);
    assert!(
        set.carries_shared(),
        "the project chose to set a shared and slave checkout private like any shared one"
    );
    assert!(
        !set.is_private(),
        "a mount that receives another's mounts is not private"
    );
}

#[test]
fn a_slave_set_and_an_unbindable_set_are_left_alone() {
    let slave: MountPropagation =
        MountPropagation::of_the_kernel_set(libc::MS_SLAVE).expect("a slave mount's set");
    let unbindable: MountPropagation = MountPropagation::of_the_kernel_set(libc::MS_UNBINDABLE)
        .expect("an unbindable mount's set");
    assert_eq!(slave.word(), SLAVE_WORD);
    assert_eq!(unbindable.word(), UNBINDABLE_WORD);
    assert!(
        !slave.carries_shared(),
        "a slave mount passes nothing on to peers"
    );
    assert!(
        !unbindable.carries_shared(),
        "an unbindable mount has no peers to pass to"
    );
    assert!(
        !slave.is_private(),
        "a slave is not the step's already-private answer"
    );
    assert!(
        !unbindable.is_private(),
        "an unbindable mount is not private either"
    );
}

#[test]
fn a_set_the_kernel_cannot_make_is_no_answer_at_all() {
    // The kernel clears the peer group and the master when it makes a mount unbindable, so this pair is no set it reports.
    assert_eq!(
        MountPropagation::of_the_kernel_set(libc::MS_UNBINDABLE | libc::MS_SHARED),
        None
    );
    // A flag this build does not know is another kernel's answer rather than a private mount.
    assert_eq!(MountPropagation::of_the_kernel_set(1), None);
}

#[test]
fn every_word_reads_back_as_the_set_it_names() {
    let sets: [MountPropagation; 5] = [
        MountPropagation::of_the_kernel_set(libc::MS_PRIVATE).expect("private"),
        MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("shared"),
        MountPropagation::of_the_kernel_set(libc::MS_SLAVE).expect("slave"),
        MountPropagation::of_the_kernel_set(libc::MS_UNBINDABLE).expect("unbindable"),
        MountPropagation::of_the_kernel_set(libc::MS_SHARED | libc::MS_SLAVE)
            .expect("shared+slave"),
    ];
    for set in sets {
        assert_eq!(
            MountPropagation::from_word(set.word()),
            Some(set),
            "the word {} reads back as the set it names",
            set.word()
        );
    }
    assert_eq!(MountPropagation::from_word("receiving"), None);
}

#[test]
fn a_table_rows_tags_say_the_three_flags_the_kernel_answers_with() {
    let shared: MountPropagation =
        MountPropagation::of_a_table_row(&row(&["shared:247"], &["rw", "relatime"]));
    let both: MountPropagation =
        MountPropagation::of_a_table_row(&row(&["shared:247", "master:3"], &["rw"]));
    let slave: MountPropagation = MountPropagation::of_a_table_row(&row(&["master:3"], &["rw"]));
    let unbindable: MountPropagation =
        MountPropagation::of_a_table_row(&row(&[], &["rw", "unbindable"]));
    let private: MountPropagation = MountPropagation::of_a_table_row(&row(&[], &["rw"]));
    assert_eq!(shared.word(), SHARED_WORD);
    assert_eq!(both.word(), SHARED_AND_SLAVE_WORD);
    assert_eq!(slave.word(), SLAVE_WORD);
    assert_eq!(unbindable.word(), UNBINDABLE_WORD);
    assert_eq!(private.word(), PRIVATE_WORD);
}
