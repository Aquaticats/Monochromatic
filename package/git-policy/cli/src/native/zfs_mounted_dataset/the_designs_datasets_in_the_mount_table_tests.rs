//! What:
//!  Controls of the lookups that find the design's datasets in a mount table:
//!  the one mounted checkout,
//!  the one mount of a dataset a file names,
//!  the dataset a path is on,
//!  the sibling container and the probe dataset under the pool.
//! Why:
//!  A name that matches none or several must refuse and list what the table
//!  held,
//!  because a root program that picked one of several would act on a name the
//!  account can arrange
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Where the checkout and the datasets are found").
//!  Each control scripts the text of a mount table,
//!  so the release parser and the release walk answer it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theOneMountedCheckout(table).dataset).toBe('pool/home/monochromatic-git-checkout');
//! ```

use super::{
    DESIGN_CONTAINER_DATASET_NAME,
    DESIGN_PROBE_DATASET_UNDER_THE_POOL,
    instances_of,
    the_checkout_dataset_at,
    the_designs_container_beside,
    the_designs_probe_dataset_under,
    the_one_mount_of,
    the_one_mounted_checkout,
};
use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
    the_designs_table,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountedDatasetInstance,
    dataset_mounts_of,
};
use crate::zfs_mounted_dataset::mounted_dataset_refusal::MountedDatasetRefusal;
use std::path::{
    Path,
    PathBuf,
};

/// The checkout dataset the design names.
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The container the design places beside it.
const CONTAINER: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// The probe dataset the start probe's design names in the same pool.
const PROBE_DATASET: &str = "zpcachyos/nosnap/git-worktree-probe";

/// The checkout the controls name.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// The dataset the account's `create` delegation on the container let it make in the guest of 2026-11-22.
const CLONE_UNDER_THE_CONTAINER: &str =
    "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/monochromatic-git-checkout";

/// The one mounted instance of the design's leaf name is the checkout dataset,
/// and its mount point is the checkout path.
#[test]
fn one_mounted_instance_is_the_checkout_dataset_and_its_path() {
    let table: ScriptedMountTable =
        ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET));
    let one: MountedDatasetInstance =
        the_one_mounted_checkout(&table, None).expect("the table holds the design's checkout once");
    assert_eq!(one.dataset, CHECKOUT_DATASET);
    assert_eq!(one.mount_point, PathBuf::from(CHECKOUT));
}

/// Two pools that hold the leaf name are several instances and are all listed.
#[test]
fn two_pools_holding_the_leaf_name_are_several_and_are_listed() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            50,
            1,
            "/",
            "/home/user/second",
            "zfs",
            "otherpool/ROOT/cos/home/monochromatic-git-checkout",
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_one_mounted_checkout(&table, None).expect_err("two pools are several instances");
    assert_eq!(refusal.word(), "several-mounted-checkouts");
    let said: String = refusal.message(Path::new(
        "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc",
    ));
    assert!(
        said.contains(CHECKOUT_DATASET),
        "the refusal lists the first: {said}"
    );
    assert!(
        said.contains("otherpool/ROOT/cos/home/monochromatic-git-checkout"),
        "the refusal lists the second: {said}"
    );
}

/// One dataset mounted at two paths is several instances,
/// and both paths are listed.
#[test]
fn one_dataset_mounted_at_two_paths_is_several() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/", "/home/user/again", "zfs", CHECKOUT_DATASET),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_one_mounted_checkout(&table, None).expect_err("one dataset twice is several instances");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("/home/user/again"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// A bind mount of the dataset's own root elsewhere is a second instance.
#[test]
fn a_bind_mount_of_the_datasets_own_root_elsewhere_is_several() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/", "/home/user/bound", "zfs", CHECKOUT_DATASET),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_one_mounted_checkout(&table, None).expect_err("the bind mount is a second row");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("/home/user/bound"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// A bind mount of a subtree is no instance,
/// so it neither counts nor is listed as one.
#[test]
fn a_bind_mount_of_a_subtree_is_no_instance() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            1,
            "/.claude",
            "/home/user/subtree",
            "zfs",
            CHECKOUT_DATASET,
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let one: MountedDatasetInstance =
        the_one_mounted_checkout(&table, None).expect("a subtree bind mount is no instance");
    assert_eq!(one.mount_point, PathBuf::from(CHECKOUT));
}

/// A table with no instance is refused with the leaf name looked for,
/// and the rows whose source carried it are listed as what was found.
#[test]
fn no_instance_names_the_leaf_it_looked_for_and_the_rows_that_carried_it() {
    let rows: Vec<String> = vec![row(
        41,
        1,
        "/sub",
        "/home/user/bound",
        "zfs",
        CHECKOUT_DATASET,
    )];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_one_mounted_checkout(&table, None).expect_err("a subtree bind mount is no instance");
    let said: String = refusal.message(Path::new("/etc/config"));
    assert_eq!(refusal.word(), "no-mounted-checkout");
    assert!(
        said.contains("`monochromatic-git-checkout`"),
        "the refusal names the leaf: {said}"
    );
    assert!(
        said.contains("/home/user/bound"),
        "the refusal lists the row it found: {said}"
    );
    assert!(
        said.contains("write `cloneMounts.checkout`"),
        "the refusal names what a person can write in this release: {said}"
    );
    assert!(
        !said.contains("plan step"),
        "the refusal sends no one to a step no release carries: {said}"
    );
    assert!(
        !said.contains("doctor"),
        "the refusal sends no one to a report this release lacks: {said}"
    );
}

/// A table this process cannot read is its own refusal,
/// not a machine with no checkout.
#[test]
fn an_unreadable_table_is_refused_as_unreadable() {
    let table: ScriptedMountTable = ScriptedMountTable::unreadable("permission denied");
    let refusal: MountedDatasetRefusal =
        the_one_mounted_checkout(&table, None).expect_err("an unread table names nothing");
    assert_eq!(refusal.word(), "mount-table-unreadable");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("permission denied"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// A dataset inside the clone container carries the design's leaf name too,
/// because the account's `create` delegation on the container lets it make one;
/// it is a clone the route attaches at a slot and never the checkout,
/// so the one mounted checkout is still the checkout.
#[test]
fn a_clone_inside_the_container_is_no_candidate_for_the_checkout() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            1,
            "/",
            "/home/user/Monochromatic/.claude/worktrees/clone",
            "zfs",
            CLONE_UNDER_THE_CONTAINER,
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let one: MountedDatasetInstance =
        the_one_mounted_checkout(&table, None).expect("a clone is no second instance");
    assert_eq!(one.dataset, CHECKOUT_DATASET);
    assert!(super::is_a_managed_clone(
        &MountedDatasetInstance {
            dataset: String::from(CLONE_UNDER_THE_CONTAINER),
            mount_point: PathBuf::from("/home/user/x"),
            root: PathBuf::from("/")
        },
        None,
    ));
    assert!(!super::is_a_managed_clone(
        &MountedDatasetInstance {
            dataset: String::from(CHECKOUT_DATASET),
            mount_point: PathBuf::from(CHECKOUT),
            root: PathBuf::from("/")
        },
        None,
    ));
}

/// The rule knows the container this run uses,
/// which root's file may name instead of the design's:
/// a clone row inside that container is no instance either.
#[test]
fn a_clone_row_in_the_container_the_file_names_is_ignored() {
    let named_container: &str = "zpcachyos/ROOT/cos/home/my-clones";
    let clone: String = format!("{named_container}/monochromatic-git-checkout");
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            1,
            "/",
            "/home/user/Monochromatic/.claude/worktrees/clone",
            "zfs",
            clone.as_str(),
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let one: MountedDatasetInstance = the_one_mounted_checkout(&table, Some(named_container))
        .expect("a clone in the named container is no second instance");
    assert_eq!(one.dataset, CHECKOUT_DATASET);
}

/// A snapshot's automounted directory is a row whose source carries `@name`,
/// so its last name component is never the checkout's leaf,
/// which is what keeps the route from taking 300 account-triggered rows for
/// checkouts.
#[test]
fn an_automounted_snapshot_row_is_no_instance_of_the_checkout() {
    let snapshot_source: String = format!("{CHECKOUT_DATASET}@account-origin");
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            1,
            "/",
            "/home/user/Monochromatic/.zfs/snapshot/account-origin",
            "zfs",
            snapshot_source.as_str(),
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let one: MountedDatasetInstance =
        the_one_mounted_checkout(&table, None).expect("a snapshot's directory is no checkout");
    assert_eq!(one.dataset, CHECKOUT_DATASET);
    let snapshot: MountedDatasetInstance = MountedDatasetInstance {
        dataset: snapshot_source,
        mount_point: PathBuf::from("/home/user/Monochromatic/.zfs/snapshot/account-origin"),
        root: PathBuf::from("/"),
    };
    assert_eq!(
        snapshot.last_name_component(),
        "monochromatic-git-checkout@account-origin"
    );
    assert_ne!(snapshot.last_name_component(), "monochromatic-git-checkout");
}

/// A row inside a container this run does not use still counts,
/// which is the cost the project accepted with the first answer.
#[test]
fn a_row_in_a_container_this_run_does_not_use_still_counts() {
    let named_container: &str = "zpcachyos/ROOT/cos/home/my-clones";
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            1,
            "/",
            "/home/user/clone",
            "zfs",
            CLONE_UNDER_THE_CONTAINER,
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal = the_one_mounted_checkout(&table, Some(named_container))
        .expect_err("a row in another container is a second instance");
    assert_eq!(refusal.word(), "several-mounted-checkouts");
}

/// A table whose only row under the leaf name is such a clone is refused,
/// and the refusal lists the clone and says why it is none.
#[test]
fn a_table_whose_only_leaf_row_is_a_clone_is_refused_and_the_clone_is_listed() {
    let rows: Vec<String> = vec![row(
        41,
        1,
        "/",
        "/home/user/Monochromatic/.claude/worktrees/clone",
        "zfs",
        CLONE_UNDER_THE_CONTAINER,
    )];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_one_mounted_checkout(&table, None).expect_err("a clone is no checkout");
    let said: String = refusal.message(Path::new("/etc/config"));
    assert!(
        said.contains(CLONE_UNDER_THE_CONTAINER),
        "the refusal lists the clone: {said}"
    );
    assert!(
        said.contains("managed clone"),
        "the refusal says why it is none: {said}"
    );
}

/// The one mount of a dataset a file names is that dataset's mount point.
#[test]
fn the_one_mount_of_a_named_dataset_is_its_path() {
    let table: ScriptedMountTable =
        ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET));
    assert_eq!(
        the_one_mount_of(CHECKOUT_DATASET, &table).expect("the dataset is mounted once"),
        PathBuf::from(CHECKOUT)
    );
}

/// A dataset a file names that the table mounts nowhere is refused,
/// and the entry that settles it is named.
#[test]
fn a_named_dataset_with_no_mount_is_refused() {
    let table: ScriptedMountTable =
        ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET));
    let refusal: MountedDatasetRefusal =
        the_one_mount_of("zpcachyos/ROOT/cos/home/unmounted", &table)
            .expect_err("nothing mounts that dataset");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("`cloneMounts.checkoutDataset`"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// A dataset mounted twice says nothing about which path a walk starts at,
/// so it is refused and both paths are listed.
#[test]
fn a_named_dataset_mounted_twice_is_refused_and_both_paths_listed() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/", "/home/user/again", "zfs", CHECKOUT_DATASET),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_one_mount_of(CHECKOUT_DATASET, &table).expect_err("two mounts name no single path");
    let said: String = refusal.message(Path::new("/etc/config"));
    assert!(
        said.contains(CHECKOUT),
        "the refusal lists the first path: {said}"
    );
    assert!(
        said.contains("/home/user/again"),
        "the refusal lists the second: {said}"
    );
}

/// The dataset a checkout path is on is the one the walk reaches,
/// and a dataset of another leaf name there is another checkout's.
#[test]
fn the_checkout_dataset_at_a_path_of_another_leaf_name_is_refused() {
    let rows: Vec<String> = vec![row(
        40,
        1,
        "/",
        CHECKOUT,
        "zfs",
        "zpcachyos/ROOT/cos/home/another-checkout",
    )];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal = the_checkout_dataset_at(Path::new(CHECKOUT), &table)
        .expect_err("another leaf name is another checkout");
    assert_eq!(refusal.word(), "no-mounted-checkout");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("another-checkout"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
    assert_eq!(
        the_checkout_dataset_at(
            Path::new(CHECKOUT),
            &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))
        )
        .expect("the design's dataset"),
        CHECKOUT_DATASET
    );
}

/// A checkout path on a mount that is no dataset is refused with the
/// filesystem the table showed.
#[test]
fn the_checkout_dataset_at_a_path_on_no_dataset_names_the_filesystem() {
    let rows: Vec<String> = vec![row(40, 1, "/", CHECKOUT, "tmpfs", "tmpfs")];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: MountedDatasetRefusal =
        the_checkout_dataset_at(Path::new(CHECKOUT), &table).expect_err("a tmpfs names no dataset");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("`tmpfs` mount"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// The container is the sibling of the checkout dataset in the same parent.
#[test]
fn the_container_is_the_sibling_of_the_checkout_dataset() {
    assert_eq!(
        the_designs_container_beside(CHECKOUT_DATASET).expect("the dataset has a parent"),
        CONTAINER
    );
    let refusal: MountedDatasetRefusal = the_designs_container_beside("monochromatic-git-checkout")
        .expect_err("a pool-name dataset has no sibling");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("has no parent component"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// The probe dataset is the design's own path under the pool that holds the
/// checkout,
/// which is the checkout dataset's first component.
#[test]
fn the_probe_dataset_is_under_the_pools_nosnap_container() {
    assert_eq!(
        the_designs_probe_dataset_under(CHECKOUT_DATASET).expect("the dataset names a pool"),
        PROBE_DATASET
    );
    assert!(
        PROBE_DATASET.ends_with(DESIGN_PROBE_DATASET_UNDER_THE_POOL),
        "{PROBE_DATASET}"
    );
    let refusal: MountedDatasetRefusal =
        the_designs_probe_dataset_under("monochromatic-git-checkout")
            .expect_err("a pool-name dataset names no pool");
    assert!(
        refusal
            .message(Path::new("/etc/config"))
            .contains("`startProbe.dataset`"),
        "{}",
        refusal.message(Path::new("/etc/config"))
    );
}

/// The instances of one dataset are the rows of its own root and never a bind
/// mount of a subtree.
#[test]
fn the_instances_of_one_dataset_are_the_rows_of_its_own_root() {
    let text: String = [
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/sub", "/home/user/subtree", "zfs", CHECKOUT_DATASET),
    ]
    .join("\n");
    let rows: Vec<MountedDatasetInstance> = dataset_mounts_of(text.as_bytes());
    let instances: Vec<MountedDatasetInstance> = instances_of(CHECKOUT_DATASET, rows.as_slice());
    assert_eq!(instances.len(), 1);
    assert_eq!(
        instances.first().map(|one| return one.mount_point.clone()),
        Some(PathBuf::from(CHECKOUT))
    );
}

/// The container's own leaf name is the design's,
/// and it is not the checkout's.
#[test]
fn the_container_leaf_name_is_its_own() {
    assert_ne!(DESIGN_CONTAINER_DATASET_NAME, "monochromatic-git-checkout");
    assert!(
        CONTAINER.ends_with(DESIGN_CONTAINER_DATASET_NAME),
        "{CONTAINER}"
    );
}
