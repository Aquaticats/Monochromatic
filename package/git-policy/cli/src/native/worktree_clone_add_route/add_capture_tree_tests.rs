//! Controls for the shared walk of tracked paths:
//!  a file, a link, an absent path and a folder,
//!  each read below one held root,
//!  and the refusal a folder on the way earns.

use super::{
    FoundFacts,
    FoundKind,
    walk_paths,
};
use crate::test_support::{
    fixture,
    remove,
};
use std::path::{
    Path,
    PathBuf,
};

/// One fixture tree: a file, a link, a folder and no `gone.txt`.
fn a_tree(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    std::fs::write(root.join("a.txt"), b"a\n").expect("the file");
    std::os::unix::fs::symlink("a.txt", root.join("link")).expect("the link");
    std::fs::create_dir(root.join("folder")).expect("the folder");
    return root;
}

#[test]
fn every_kind_the_walk_meets_is_read() {
    let root: PathBuf = a_tree("walk-kinds");
    let paths: Vec<&[u8]> = vec![b"a.txt", b"link", b"gone.txt"];
    let mut seen: Vec<(String, FoundFacts)> = Vec::new();
    walk_paths(
        root.as_path(),
        paths.iter(),
        |path| return path,
        |_item, path, found, _parent, _name| {
            seen.push((path.display().to_string(), found.clone()));
            return Ok(());
        },
    )
    .expect("the walk reads the tree");
    assert_eq!(seen.len(), 3);
    assert_eq!(seen[0].1.kind, FoundKind::Regular);
    assert_eq!(seen[0].1.size, 2);
    assert_eq!(seen[1].1.kind, FoundKind::Link);
    assert_eq!(seen[1].1.link_target.as_deref(), Some(b"a.txt".as_slice()));
    assert_eq!(
        seen[2].1.kind,
        FoundKind::Absent,
        "nothing at the path is a fact the proof compares"
    );
    remove(root.as_path());
}

#[test]
fn a_folder_and_a_climbing_path_are_refused() {
    let root: PathBuf = a_tree("walk-refusals");
    let folders: Vec<&[u8]> = vec![b"folder"];
    let refusal: String = walk_paths(
        root.as_path(),
        folders.iter(),
        |path| return path,
        |_item, _path, _found, _parent, _name| return Ok(()),
    )
    .expect_err("a folder is no file the route proves");
    assert!(refusal.contains("is a folder"), "{refusal}");
    let climbing: Vec<&[u8]> = vec![b"../outside"];
    let climbs: String = walk_paths(
        root.as_path(),
        climbing.iter(),
        |path| return path,
        |_item, _path, _found, _parent, _name| return Ok(()),
    )
    .expect_err("a path that climbs out is no tracked path");
    assert!(climbs.contains("not a plain relative path"), "{climbs}");
    remove(root.as_path());
}

#[test]
fn a_visit_that_refuses_stops_the_walk() {
    let root: PathBuf = a_tree("walk-visit");
    let paths: Vec<&[u8]> = vec![b"a.txt", b"link"];
    let mut visited: usize = 0;
    let refusal: String = walk_paths(
        root.as_path(),
        paths.iter(),
        |path| return path,
        |_item, _path, _found, _parent, _name| {
            visited += 1;
            return Err(String::from("the control refused"));
        },
    )
    .expect_err("the visit refused");
    assert_eq!(refusal, "the control refused");
    assert_eq!(visited, 1, "the walk stopped at the first refusal");
    remove(root.as_path());
}

#[test]
fn the_shared_path_is_one_plain_name_at_a_time() {
    // A path whose parent is a link is refused rather than followed: the walk
    // opens each folder with `openat2`'s no-symlink rule.
    let root: PathBuf = a_tree("walk-link-parent");
    std::fs::create_dir(root.join("real")).expect("the real folder");
    std::fs::write(root.join("real").join("f.txt"), b"f\n").expect("the file");
    std::os::unix::fs::symlink("real", root.join("linked")).expect("the folder link");
    let through_link: Vec<&[u8]> = vec![b"linked/f.txt"];
    let refusal: String = walk_paths(
        root.as_path(),
        through_link.iter(),
        |path| return path,
        |_item, _path, _found, _parent, _name| return Ok(()),
    )
    .expect_err("a folder reached through a link is refused");
    assert!(
        refusal.contains("cannot be opened without a link"),
        "{refusal}"
    );
    let _kept: &Path = root.as_path();
    remove(root.as_path());
}
