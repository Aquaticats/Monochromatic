//! The unit tests of the repository and workspace lookup.

use std::fs;

use crate::test_scratch::scratch;
use crate::workspace::{find_cargo_root, find_catalog_root};

#[test]
fn the_catalog_root_is_the_nearest_ancestor_with_the_marker() {
    let root = scratch();
    fs::write(root.path().join("rust-catalog.toml"), "").expect("marker");
    fs::create_dir_all(root.path().join("package/a/b")).expect("dirs");

    assert_eq!(find_catalog_root(&root.path().join("package/a/b")), Some(root.path().to_path_buf()));
    assert_eq!(find_catalog_root(root.path()), Some(root.path().to_path_buf()));
}

#[test]
fn no_marker_means_no_catalog_root() {
    let root = scratch();
    fs::create_dir_all(root.path().join("x")).expect("dirs");

    assert_eq!(find_catalog_root(&root.path().join("x")).filter(|found| { return found.starts_with(root.path()); }), None);
}

#[test]
fn a_member_belongs_to_the_nearest_ancestor_workspace() {
    let root = scratch();
    fs::create_dir_all(root.path().join("ws/member/src")).expect("dirs");
    fs::write(root.path().join("ws/Cargo.toml"), "[workspace]\nmembers = [\"member\"]\n").expect("root");
    fs::write(root.path().join("ws/member/Cargo.toml"), "[package]\nname = \"member\"\nversion = \"0.1.0\"\n").expect("member");

    assert_eq!(find_cargo_root(&root.path().join("ws/member/src")), Some(root.path().join("ws")));
}

#[test]
fn a_package_without_any_workspace_is_its_own_root() {
    let root = scratch();
    fs::create_dir_all(root.path().join("solo")).expect("dirs");
    fs::write(root.path().join("solo/Cargo.toml"), "[package]\nname = \"solo\"\nversion = \"0.1.0\"\n").expect("manifest");

    assert_eq!(find_cargo_root(&root.path().join("solo")), Some(root.path().join("solo")));
}

#[test]
fn a_directory_with_no_manifest_above_it_has_no_cargo_root() {
    let root = scratch();
    fs::create_dir_all(root.path().join("empty")).expect("dirs");

    assert_eq!(find_cargo_root(&root.path().join("empty")).filter(|found| { return found.starts_with(root.path()); }), None);
}
