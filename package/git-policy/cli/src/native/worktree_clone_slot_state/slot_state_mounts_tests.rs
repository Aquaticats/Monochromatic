//! Controls for the mount readings:
//!  a clone at its slot,
//!  a clone nowhere,
//!  a clone somewhere else,
//!  something else at the slot,
//!  a bind mount of a subtree at the slot,
//!  an unreadable table,
//!  and a slot that does not exist.

use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
};
use crate::worktree_clone_slot_state::slot_state_mounts::{
    CloneMount,
    MountRows,
    SlotMount,
    clone_mount_of,
    read_mount_rows,
    slot_mount_of,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::path::{
    Path,
    PathBuf,
};

/// The fixture's slot path.
fn slot() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// The fixture's clone dataset.
fn clone_dataset() -> &'static str {
    return "tank/monochromatic-git-worktree-clone-datasets/topic";
}

/// One ZFS row of a table,
/// read through the release parser.
fn rows(entries: &[String]) -> MountRows {
    let table: ScriptedMountTable = ScriptedMountTable::holding(entries);
    return read_mount_rows(&table);
}

/// The checkout row every table holds below the slot.
fn checkout_row() -> String {
    return row(
        40,
        1,
        "/",
        "/repo",
        "zfs",
        "tank/monochromatic-git-checkout",
    );
}

/// The clone's own root wins over every other answer.
#[test]
fn a_clone_mounted_at_its_slot_is_at_its_slot() {
    let table: MountRows = rows(&[
        checkout_row(),
        row(
            41,
            40,
            "/",
            slot().to_str().expect("utf8 slot"),
            "zfs",
            clone_dataset(),
        ),
    ]);
    let mount: CloneMount = clone_mount_of(
        &table,
        &SlotMount::Nothing,
        clone_dataset(),
        slot().as_path(),
    );
    assert_eq!(mount, CloneMount::AtItsSlot, "{mount:?}");
}

/// No row of the clone dataset is the known unmounted clone's reading.
#[test]
fn a_clone_with_no_row_is_nowhere() {
    let table: MountRows = rows(&[checkout_row()]);
    let mount: CloneMount = clone_mount_of(
        &table,
        &SlotMount::Nothing,
        clone_dataset(),
        slot().as_path(),
    );
    assert_eq!(mount, CloneMount::Nowhere, "{mount:?}");
}

/// A clone mounted at another path is not at its slot.
#[test]
fn a_clone_mounted_elsewhere_is_reported_where_it_is() {
    let other: &str = "/repo/.claude/worktrees/other";
    let table: MountRows = rows(&[
        checkout_row(),
        row(41, 40, "/", other, "zfs", clone_dataset()),
    ]);
    let mount: CloneMount = clone_mount_of(
        &table,
        &SlotMount::Nothing,
        clone_dataset(),
        slot().as_path(),
    );
    match &mount {
        CloneMount::Elsewhere {
            described,
        } => assert!(
            described.contains(other) && described.contains(clone_dataset()),
            "{described}"
        ),
        found => panic!("expected the clone elsewhere, got {found:?}"),
    }
}

/// A bind mount of a subtree of the clone is not the clone's own root at the slot.
#[test]
fn a_bind_mount_of_a_subtree_is_something_else_at_the_slot() {
    let table: MountRows = rows(&[
        checkout_row(),
        row(
            41,
            40,
            "/sub",
            slot().to_str().expect("utf8 slot"),
            "zfs",
            clone_dataset(),
        ),
    ]);
    let mount: CloneMount = clone_mount_of(
        &table,
        &SlotMount::Nothing,
        clone_dataset(),
        slot().as_path(),
    );
    match &mount {
        CloneMount::SomethingElseAtTheSlot {
            described,
        } => assert!(described.contains("/sub"), "{described}"),
        found => panic!("expected something else at the slot, got {found:?}"),
    }
}

/// A mount at the slot that is not the clone's is reported with what it is.
#[test]
fn another_mount_at_the_slot_is_something_else() {
    let table: MountRows = rows(&[checkout_row()]);
    let slot_mount: SlotMount = SlotMount::Something {
        described: String::from("`tank/other` at the slot, mounted as `zfs`"),
    };
    let mount: CloneMount = clone_mount_of(&table, &slot_mount, clone_dataset(), slot().as_path());
    assert_eq!(
        mount,
        CloneMount::SomethingElseAtTheSlot {
            described: String::from("`tank/other` at the slot, mounted as `zfs`")
        }
    );
}

/// A table that could not be read proves nothing.
#[test]
fn an_unreadable_table_names_why_it_was_not_read() {
    // The slot path exists here, so the only refusal in play is the table's.
    let root: PathBuf = crate::testing::test_support::fixture("slot-state-unreadable-table");
    let table: ScriptedMountTable = ScriptedMountTable::unreadable("permission denied");
    let slot_mount: SlotMount = slot_mount_of(&table, root.as_path());
    assert_eq!(
        slot_mount,
        SlotMount::Unreadable {
            reason: String::from("permission denied")
        }
    );
    let rows: MountRows = read_mount_rows(&table);
    let clone: CloneMount = clone_mount_of(&rows, &slot_mount, clone_dataset(), root.as_path());
    assert_eq!(
        clone,
        CloneMount::Unreadable {
            reason: String::from("permission denied")
        }
    );
}

/// A slot whose own path is on no mount of its own shows nothing there.
#[test]
fn a_slot_the_table_does_not_mount_shows_nothing() {
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[checkout_row()]);
    assert_eq!(slot_mount_of(&table, slot().as_path()), SlotMount::Nothing);
}

/// A slot path that does not exist holds no mount,
/// which is the empty arm rather than an unreadable one.
#[test]
fn a_slot_path_that_does_not_exist_shows_nothing() {
    let missing: &Path = Path::new("/repo/.claude/worktrees/never-made");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[checkout_row()]);
    assert!(
        !missing.exists(),
        "the fixture path must not exist on this machine"
    );
    assert_eq!(slot_mount_of(&table, missing), SlotMount::Nothing);
}

/// A path the table mounts shows the row it is on,
/// which for a dataset mount names the dataset.
#[test]
fn a_mounted_slot_shows_the_dataset_that_holds_it() {
    let holding: String = row(
        41,
        40,
        "/",
        slot().to_str().expect("utf8 slot"),
        "zfs",
        clone_dataset(),
    );
    let table: ScriptedMountTable =
        ScriptedMountTable::text(&format!("{}\n{holding}", checkout_row()));
    let mount: SlotMount = slot_mount_of(&table, slot().as_path());
    let described: String = format!("`{}` at the slot, mounted as `zfs`", clone_dataset());
    assert_eq!(
        mount,
        SlotMount::Something {
            described
        }
    );
}

/// One row of a table for a dataset the fixture does not mount anywhere.
#[test]
fn a_row_of_another_dataset_is_ignored() {
    let instance: MountedDatasetInstance = MountedDatasetInstance {
        dataset: String::from("tank/other"),
        mount_point: PathBuf::from("/repo/other"),
        root: PathBuf::from("/"),
    };
    assert_ne!(instance.dataset, clone_dataset());
    let table: MountRows = rows(&[
        checkout_row(),
        row(41, 40, "/", "/repo/other", "zfs", "tank/other"),
    ]);
    assert_eq!(
        clone_mount_of(
            &table,
            &SlotMount::Nothing,
            clone_dataset(),
            slot().as_path()
        ),
        CloneMount::Nowhere
    );
}
