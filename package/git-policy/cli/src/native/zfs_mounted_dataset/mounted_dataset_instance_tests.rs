//! What:
//!  Controls of the rows the mount table gives:
//!  which rows are dataset mounts,
//!  what a row says about its dataset,
//!  its path and its root,
//!  and which mount a path is on.
//! Why:
//!  Every finding the configuration makes counts these rows
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Where the checkout and the datasets are found"),
//!  and a row of another filesystem type must never be counted as a dataset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(datasetMountsOf(table).map((row) => row.dataset)).toEqual(['pool/a', 'pool/b']);
//! ```

use super::{
    MountedDatasetInstance,
    dataset_mounts_of,
    holding_of,
};
use crate::fs_mount_options::{
    mount_holding,
    parse_mount_table,
};
use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
    the_designs_table,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::path::{
    Path,
    PathBuf,
};

/// The checkout dataset the design names.
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The checkout the controls name.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// Only a `zfs` row is a dataset mount,
/// and the tmpfs and root rows beside it are left out.
#[test]
fn only_zfs_rows_are_dataset_mounts() {
    let rows: Vec<MountedDatasetInstance> =
        dataset_mounts_of(the_designs_table(CHECKOUT, CHECKOUT_DATASET).as_bytes());
    let datasets: Vec<&str> = rows.iter().map(|one| return one.dataset.as_str()).collect();
    assert_eq!(datasets, vec!["zpcachyos/ROOT/cos/root", CHECKOUT_DATASET]);
}

/// A row carries its dataset,
/// its mount point and its root,
/// which are the three fields every finding reads.
#[test]
fn a_row_carries_its_dataset_its_path_and_its_root() {
    let text: String = row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET);
    let rows: Vec<MountedDatasetInstance> = dataset_mounts_of(text.as_bytes());
    assert_eq!(rows.len(), 1);
    let one: &MountedDatasetInstance = rows.first().expect("the table holds one row");
    assert_eq!(one.dataset, CHECKOUT_DATASET);
    assert_eq!(one.mount_point, PathBuf::from(CHECKOUT));
    assert_eq!(one.root, PathBuf::from("/"));
    assert!(one.shows_the_datasets_own_root());
}

/// A bind mount of a subtree shows a deeper root,
/// so it does not show the dataset's own root.
#[test]
fn a_bind_mount_of_a_subtree_shows_a_deeper_root() {
    let text: String = row(
        41,
        1,
        "/.claude",
        "/home/user/Monochromatic-subtree",
        "zfs",
        CHECKOUT_DATASET,
    );
    let rows: Vec<MountedDatasetInstance> = dataset_mounts_of(text.as_bytes());
    let one: &MountedDatasetInstance = rows.first().expect("the row is there");
    assert!(!one.shows_the_datasets_own_root());
    assert_eq!(one.root, PathBuf::from("/.claude"));
}

/// The leaf name of a row's dataset is its last component,
/// which is what the design's own name matches.
#[test]
fn the_leaf_name_is_the_last_component() {
    let text: String = row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET);
    let one: MountedDatasetInstance = dataset_mounts_of(text.as_bytes())
        .into_iter()
        .next()
        .expect("the row is there");
    assert_eq!(one.last_name_component(), "monochromatic-git-checkout");
    let pool_named: MountedDatasetInstance = MountedDatasetInstance {
        dataset: String::from("monochromatic-git-checkout"),
        mount_point: PathBuf::from(CHECKOUT),
        root: PathBuf::from("/"),
    };
    assert_eq!(
        pool_named.last_name_component(),
        "monochromatic-git-checkout"
    );
}

/// A row in words names its dataset and its path,
/// and a subtree bind mount names its root as well.
#[test]
fn a_row_in_words_names_what_a_person_goes_to_look_at() {
    let text: String = row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET);
    let one: MountedDatasetInstance = dataset_mounts_of(text.as_bytes())
        .into_iter()
        .next()
        .expect("the row is there");
    assert_eq!(
        one.described(),
        format!("`{CHECKOUT_DATASET}` at {CHECKOUT}")
    );
    let subtree: MountedDatasetInstance = MountedDatasetInstance {
        dataset: String::from(CHECKOUT_DATASET),
        mount_point: PathBuf::from("/home/user/bound"),
        root: PathBuf::from("/sub"),
    };
    assert!(
        subtree.described().contains("with root /sub"),
        "{}",
        subtree.described()
    );
}

/// The mount one path is on is the row that path reaches,
/// and a path inside the dataset's own root is on the dataset.
#[test]
fn the_mount_one_path_is_on_is_the_row_it_reaches() {
    let table: ScriptedMountTable =
        ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET));
    let inside: PathBuf = Path::new(CHECKOUT).join(".claude/worktrees");
    let mount = table
        .mount_holding(inside.as_path())
        .expect("the scripted table is readable")
        .expect("the path is on the checkout's mount");
    assert_eq!(mount.dataset.as_deref(), Some(CHECKOUT_DATASET));
    assert_eq!(mount.filesystem_type, "zfs");
    assert_eq!(mount.mount_point, PathBuf::from(CHECKOUT));
}

/// A path on a mount that is no dataset names its filesystem instead,
/// which is what a refusal shows a person.
#[test]
fn a_path_on_no_dataset_names_the_filesystem_it_found() {
    let table: ScriptedMountTable =
        ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET));
    let mount = table
        .mount_holding(Path::new("/run/foo"))
        .expect("the scripted table is readable")
        .expect("the path is on the tmpfs");
    assert_eq!(mount.dataset, None);
    assert_eq!(mount.filesystem_type, "tmpfs");
}

/// The walk that places a path takes the topmost mount made there,
/// so a mount stacked over one at the same path is the one a path sees.
#[test]
fn a_mount_stacked_at_one_path_is_the_one_a_path_sees() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            40,
            "/",
            CHECKOUT,
            "zfs",
            "zpcachyos/ROOT/cos/home/second",
        ),
    ];
    let facts = parse_mount_table(rows.join("\n").as_bytes());
    let found =
        mount_holding(facts.as_slice(), Path::new(CHECKOUT)).expect("the path is on a mount");
    assert_eq!(
        holding_of(found).dataset.as_deref(),
        Some("zpcachyos/ROOT/cos/home/second")
    );
}

/// Whether the release reader answers at all is the machine's business,
/// but its answer must be a list of rows rather than a refusal on this host.
#[test]
fn the_release_reader_answers_a_list_of_rows() {
    let rows: Vec<MountedDatasetInstance> = super::LiveMountedDatasets
        .mounted_datasets()
        .expect("this process can read its own mount table");
    for one in &rows {
        assert!(
            !one.dataset.is_empty(),
            "a row names its dataset: {}",
            one.described()
        );
    }
}
