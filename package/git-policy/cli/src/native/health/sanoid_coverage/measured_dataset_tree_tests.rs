//! What:
//!  Controls of the measured dataset tree:
//!  ancestry on `/`,
//!  which mount points cover a folder,
//!  and the dataset that holds a path.
//! Why:
//!  Recursive coverage follows dataset ancestry and never a name prefix,
//!  so `tank/homework` must not count as under `tank/home`,
//!  and a store held by the wrong dataset is the drift the coverage check
//!  reports,
//!  which only a tree that answers both correctly can show.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isUnder('tank/home/x', 'tank/home')).toBe(true);
//! expect(tree.holdingDataset('/home/someone/.npm')).toBe('tank/nosnap/npm');
//! ```

use super::{
    MeasuredDataset,
    MeasuredTree,
    has_path_mountpoint,
    is_under,
};

/// A tree of four datasets with the mount points a machine reports.
fn a_tree() -> MeasuredTree {
    return MeasuredTree {
        datasets: vec![
            MeasuredDataset {
                name: String::from("tank/home"),
                mountpoint: Some(String::from("/fixture/home")),
                auto_snapshot: None,
            },
            MeasuredDataset {
                name: String::from("tank/home/worktrees"),
                mountpoint: Some(String::from("/fixture/home/.claude/worktrees")),
                auto_snapshot: None,
            },
            MeasuredDataset {
                name: String::from("tank/homework"),
                mountpoint: Some(String::from("/fixture/homework")),
                auto_snapshot: None,
            },
            MeasuredDataset {
                name: String::from("tank/nosnap/npm"),
                mountpoint: Some(String::from("/fixture/home/.npm")),
                auto_snapshot: Some(String::from("false")),
            },
            MeasuredDataset {
                name: String::from("tank/nosnap"),
                mountpoint: Some(String::from("none")),
                auto_snapshot: Some(String::from("false")),
            },
        ],
    };
}

/// A name on `/` is under another and one that only begins with it is not.
#[test]
fn a_name_on_a_separator_is_under_and_a_prefix_alone_is_not() {
    assert!(is_under("tank/home/worktrees", "tank/home"));
    assert!(
        is_under("tank/home", "tank"),
        "a child of the pool is under it"
    );
    assert!(
        !is_under("tank/homework", "tank/home"),
        "a name that merely begins with the ancestor is not under it"
    );
    assert!(!is_under("tank/hom", "tank/home"));
    assert!(
        !is_under("tank/home", "tank/home"),
        "a dataset is not under itself"
    );
}

/// Only a value that begins with `/` is a path mount point.
#[test]
fn only_a_value_that_begins_with_a_slash_is_a_path() {
    assert!(has_path_mountpoint(Some("/fixture/home")));
    assert!(!has_path_mountpoint(Some("none")));
    assert!(!has_path_mountpoint(Some("legacy")));
    assert!(!has_path_mountpoint(Some("-")));
    assert!(!has_path_mountpoint(None));
}

/// The datasets under an ancestor are the descendants and not the ancestor,
/// in name order.
#[test]
fn the_datasets_under_an_ancestor_are_its_descendants_in_name_order() {
    let tree: MeasuredTree = a_tree();
    let under: Vec<&str> = tree
        .datasets_under("tank/home")
        .iter()
        .map(|dataset| return dataset.name.as_str())
        .collect();
    assert_eq!(
        under,
        vec!["tank/home/worktrees"],
        "tank/homework only begins with the ancestor's name"
    );
    assert_eq!(tree.datasets_under("tank/homework").len(), 0);
    let under_pool: Vec<&str> = tree
        .datasets_under("tank")
        .iter()
        .map(|dataset| return dataset.name.as_str())
        .collect();
    assert_eq!(
        under_pool,
        vec![
            "tank/home",
            "tank/home/worktrees",
            "tank/homework",
            "tank/nosnap",
            "tank/nosnap/npm"
        ]
    );
}

/// The measured ancestors of one dataset come nearest first.
#[test]
fn the_measured_ancestors_come_nearest_first() {
    let tree: MeasuredTree = a_tree();
    let ancestors: Vec<&str> = tree
        .ancestors("tank/home/worktrees")
        .iter()
        .map(|dataset| return dataset.name.as_str())
        .collect();
    assert_eq!(ancestors, vec!["tank/home"]);
    let npm_ancestors: Vec<&str> = tree
        .ancestors("tank/nosnap/npm")
        .iter()
        .map(|dataset| return dataset.name.as_str())
        .collect();
    assert_eq!(npm_ancestors, vec!["tank/nosnap"]);
}

/// The dataset that holds a path is the one with the longest mount point the
/// path is under.
#[test]
fn the_holding_dataset_is_the_longest_mount_point_the_path_is_under() {
    let tree: MeasuredTree = a_tree();
    assert_eq!(
        tree.holding_dataset("/fixture/home/.npm/packages"),
        Some("tank/nosnap/npm")
    );
    assert_eq!(
        tree.holding_dataset("/fixture/home/.local/share/pnpm/store"),
        Some("tank/home")
    );
    assert_eq!(tree.holding_dataset("/fixture/home"), Some("tank/home"));
}

/// The dataset that holds a path is the one whose mount point is deepest,
/// not the one whose name is longest:
/// a container's name says nothing about where its children mount.
#[test]
fn the_holding_dataset_is_the_deepest_mount_point_not_the_longest_name() {
    let tree: MeasuredTree = MeasuredTree {
        datasets: vec![
            MeasuredDataset {
                name: String::from("tank/nosnap/npm"),
                mountpoint: Some(String::from("/fixture/home")),
                auto_snapshot: Some(String::from("false")),
            },
            MeasuredDataset {
                name: String::from("tank/home"),
                mountpoint: Some(String::from("/fixture/home/.npm")),
                auto_snapshot: None,
            },
        ],
    };
    assert_eq!(
        tree.holding_dataset("/fixture/home/.npm/packages"),
        Some("tank/home")
    );
    assert_eq!(
        tree.holding_dataset("/fixture/home/other"),
        Some("tank/nosnap/npm")
    );
}

/// A path next to a mount point but not under it is held by nothing there,
/// and a dataset with no path mount point holds nothing.
#[test]
fn a_path_beside_a_mount_point_is_held_by_nothing_there() {
    let tree: MeasuredTree = a_tree();
    assert_eq!(
        tree.holding_dataset("/fixture/homework/x"),
        Some("tank/homework")
    );
    assert_eq!(tree.holding_dataset("/fixture/other"), None);
    let only_unmounted: MeasuredTree = MeasuredTree {
        datasets: vec![MeasuredDataset {
            name: String::from("tank/nosnap"),
            mountpoint: Some(String::from("none")),
            auto_snapshot: None,
        }],
    };
    assert_eq!(only_unmounted.holding_dataset("/anywhere"), None);
}
