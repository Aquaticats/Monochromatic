//! Directory operations read disposable project trees and reject escapes without mutating them.

/// The same project boundary will serve tree and search navigation.
use ide_app::workspace::{DirectoryEntry, Workspace};
/// Native names and Unix symlinks exercise paths without UI-string normalization.
use std::{ffi::OsString, fs, os::unix::fs::symlink, path::Path};

/// The native names of a listing,
///  in listing order.
fn names(entries: &[DirectoryEntry]) -> Vec<OsString> {
    let mut found: Vec<OsString> = Vec::new();
    for entry in entries {
        found.push(entry.name.clone());
    }
    return found;
}

/// Whether the listing holds an entry of this name,
///  and if it does,
///  whether that entry is a directory.
fn listed_as_directory(entries: &[DirectoryEntry], name: &str) -> Option<bool> {
    for entry in entries {
        if entry.name == name {
            return Some(entry.is_directory);
        }
    }
    return None;
}

/// Hidden entries and native names arrive in display order with accurate directory kinds.
#[test]
fn listing_matches_dirents_and_refreshes_from_disk() {
    let fixture = tempfile::tempdir().expect("disposable project");
    fs::create_dir(fixture.path().join("src")).expect("source directory");
    fs::write(fixture.path().join(".hidden"), "hidden").expect("hidden fixture");
    fs::write(fixture.path().join("猫 'quoted'.ts"), "const 猫 = 1;").expect("Unicode fixture");
    let project = Workspace::new(fixture.path()).expect("open project");
    let expected = vec![
        OsString::from(".hidden"),
        OsString::from("src"),
        OsString::from("猫 'quoted'.ts"),
    ];
    let first = project.list(Path::new(".")).expect("initial listing");
    assert_eq!(names(&first), expected);
    assert_eq!(listed_as_directory(&first, "src"), Some(true));
    assert!(listed_as_directory(&first, ".hidden").is_some());
    fs::write(fixture.path().join("new.rs"), "fn main() {}").expect("external new file");
    let second = project.list(Path::new(".")).expect("fresh listing");
    assert_eq!(second.len(), first.len() + 1);
    assert!(listed_as_directory(&second, "new.rs").is_some());
    assert_eq!(
        names(&second),
        vec![
            OsString::from(".hidden"),
            OsString::from("new.rs"),
            OsString::from("src"),
            OsString::from("猫 'quoted'.ts"),
        ],
        "a new file takes its place in the display order"
    );
}

/// Relative paths use the project root,
///  with valid parent segments allowed inside it.
#[test]
fn contained_relative_and_absolute_paths_resolve() {
    let fixture = tempfile::tempdir().expect("disposable project");
    fs::create_dir(fixture.path().join("src")).expect("source directory");
    fs::write(fixture.path().join("src/main.rs"), "fn main() {}").expect("source file");
    let project = Workspace::new(fixture.path()).expect("open project");
    let path = project
        .resolve(Path::new("src/../src/main.rs"))
        .expect("contained relative path");
    assert!(path.starts_with(project.root()));
    assert_eq!(
        project.resolve(&path).expect("contained absolute path"),
        path
    );
}

/// Component containment rejects sibling-prefix paths and parent traversal.
#[test]
fn sibling_prefix_and_parent_escape_are_rejected() {
    let fixture = tempfile::tempdir().expect("disposable parent");
    let root = fixture.path().join("project");
    let sibling = fixture.path().join("project-other");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&sibling).expect("sibling directory");
    let project = Workspace::new(&root).expect("open project");
    assert!(project.resolve(&sibling).is_err());
    assert!(project.list(Path::new("../project-other")).is_err());
}

/// Tree operations reject outside symlink targets while retaining dirent link semantics.
#[test]
fn symbolic_links_do_not_bypass_project_boundary() {
    let fixture = tempfile::tempdir().expect("disposable parent");
    let root = fixture.path().join("project");
    let outside = fixture.path().join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    fs::create_dir(root.join("inside")).expect("inside directory");
    symlink(&outside, root.join("escape")).expect("outside link fixture");
    symlink(root.join("inside"), root.join("alias")).expect("inside link fixture");
    let project = Workspace::new(&root).expect("open project");
    assert!(project.list(Path::new("escape")).is_err());
    assert!(
        project
            .list(Path::new("alias"))
            .expect("contained explicit alias")
            .is_empty()
    );
    let entries = project.list(Path::new(".")).expect("root entries");
    assert_eq!(
        listed_as_directory(&entries, "alias"),
        Some(false),
        "a symbolic link is listed, and is not a directory"
    );
}

/// Missing paths,
///  file roots,
///  and listing a file report failures rather than empty directories.
#[test]
fn invalid_directory_inputs_are_errors() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let file = fixture.path().join("file.txt");
    fs::write(&file, "source").expect("file fixture");
    assert!(Workspace::new(&file).is_err());
    assert!(Workspace::new(&fixture.path().join("missing")).is_err());
    let project = Workspace::new(fixture.path()).expect("open project");
    assert!(project.list(&file).is_err());
    assert!(project.resolve(Path::new("missing")).is_err());
}
