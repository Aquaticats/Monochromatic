//! Controls for what stands at one slot:
//!  an empty slot with this add's pointer and registration,
//!  a slot holding a file besides `.git`,
//!  a pointer naming another registration,
//!  a registration of another identity,
//!  a registration the recipe already removed,
//!  a mount at the slot,
//!  and a mount table that cannot be read.

use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
};
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_record_settling::settling_fixtures::{
    TheFixture,
    the_fixture,
    write_the_pointer,
};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_slot::{
    ThePointer,
    TheRegistration,
    TheSlot,
    read_the_slot,
};

/// The fixture's registration identity,
/// which the slot's record would hold.
fn the_identity(made: &TheFixture) -> FolderIdentity {
    return FolderIdentity::of_folder(made.registration.as_path())
        .expect("the registration's identity");
}

/// An unmounted slot with this add's pointer and registration has nothing beside `.git`,
/// so no checkout has begun.
#[test]
fn an_empty_slot_is_the_abandoned_registration() {
    let made: TheFixture = the_fixture("settling-slot-empty");
    write_the_pointer(&made);
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(
        40,
        1,
        "/",
        made.root.to_str().expect("utf8 root"),
        "zfs",
        "tank/checkout",
    )]);
    let slot: TheSlot = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        the_identity(&made),
        &table,
    )
    .expect("the slot reads");
    assert_eq!(slot.pointer, ThePointer::NamesThis);
    assert!(
        matches!(slot.registration, TheRegistration::Stands { .. }),
        "{:?}",
        slot.registration
    );
    assert!(!slot.a_checkout_has_begun());
    assert!(
        slot.registration_entries.contains(&String::from("HEAD")),
        "{:?}",
        slot.registration_entries
    );
}

/// One file besides `.git` is a checkout begun,
/// which is the design's one test.
#[test]
fn a_file_beside_the_pointer_is_a_checkout_begun() {
    let made: TheFixture = the_fixture("settling-slot-checkout");
    write_the_pointer(&made);
    std::fs::write(made.slot_path.join("README.md"), b"# the person's file\n")
        .expect("the file writes");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[]);
    let slot: TheSlot = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        the_identity(&made),
        &table,
    )
    .expect("the slot reads");
    assert_eq!(
        slot.entries_beside_the_pointer,
        vec![String::from("README.md")]
    );
    assert!(slot.a_checkout_has_begun());
}

/// A pointer naming another registration is another worktree's entry.
#[test]
fn a_pointer_naming_another_registration_is_no_file_of_this_adds() {
    let made: TheFixture = the_fixture("settling-slot-other-pointer");
    std::fs::write(
        made.slot_path.join(".git"),
        b"gitdir: /elsewhere/.git/worktrees/other\n",
    )
    .expect("the pointer writes");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[]);
    let slot: TheSlot = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        the_identity(&made),
        &table,
    )
    .expect("the slot reads");
    assert_eq!(
        slot.pointer,
        ThePointer::NamesAnother {
            found: String::from("gitdir: /elsewhere/.git/worktrees/other")
        }
    );
}

/// A registration standing with another identity is another folder's.
#[test]
fn a_registration_of_another_identity_is_another_folders() {
    let made: TheFixture = the_fixture("settling-slot-other-registration");
    write_the_pointer(&made);
    let recorded: FolderIdentity =
        FolderIdentity::decode("131074:1763750400:123456789").expect("the fixture identity reads");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[]);
    let slot: TheSlot = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        recorded,
        &table,
    )
    .expect("the slot reads");
    assert!(
        matches!(slot.registration, TheRegistration::Differs { .. }),
        "{:?}",
        slot.registration
    );
}

/// A registration the recipe already removed is that step done,
/// so a rerun after a kill goes on.
#[test]
fn a_gone_registration_is_that_step_done() {
    let made: TheFixture = the_fixture("settling-slot-gone-registration");
    let recorded: FolderIdentity = the_identity(&made);
    std::fs::remove_dir_all(made.registration.as_path()).expect("the registration goes");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[]);
    let slot: TheSlot = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        recorded,
        &table,
    )
    .expect("the slot reads");
    assert_eq!(slot.registration, TheRegistration::Gone);
    assert!(slot.registration_entries.is_empty());
}

/// A mount at the slot refuses outright,
/// because a mounted slot's entries are the mount's and not a checkout's.
#[test]
fn a_mount_at_the_slot_refuses() {
    let made: TheFixture = the_fixture("settling-slot-mounted");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(
        41,
        1,
        "/",
        made.slot_path.to_str().expect("utf8 slot"),
        "zfs",
        "tank/worktree-clones/topic",
    )]);
    let refused = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        the_identity(&made),
        &table,
    )
    .expect_err("a mount refuses");
    let SettlingStop::AMountAtTheSlot {
        described,
    } = &refused
    else {
        panic!("the mount is its own arm, and this answered {refused:?}")
    };
    assert!(
        described.contains("tank/worktree-clones/topic"),
        "{described}"
    );
}

/// A mount table that cannot be read proves nothing about the slot.
#[test]
fn an_unreadable_mount_table_refuses() {
    let made: TheFixture = the_fixture("settling-slot-unreadable-table");
    let table: ScriptedMountTable = ScriptedMountTable::unreadable("the table could not be read");
    let refused = read_the_slot(
        made.slot_path.as_path(),
        made.registration.as_path(),
        the_identity(&made),
        &table,
    )
    .expect_err("the table refuses");
    assert_eq!(
        refused,
        SettlingStop::TheMountTableCouldNotBeRead {
            said: String::from("the table could not be read")
        }
    );
}
