//! Rollback checks on real filesystem trees: only proven paths go, everything else is named and kept.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_install_entry::capture_installed_path;
use std::path::PathBuf;

/// A stage holding one file, plus the entries and the destination the rollback works against.
fn staged_file(root: &Path) -> (PathBuf, PathBuf, Vec<WorktreeCopyEntry>) {
    let stage: PathBuf = root.join("stage");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(stage.join("cache")).expect("stage");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(stage.join("cache/file"), b"bytes").expect("staged file");
    let entries: Vec<WorktreeCopyEntry> = vec![
        WorktreeCopyEntry {
            kind: EntryKind::Directory,
            mode: 0o755,
            relative_path: String::from("cache"),
        },
        WorktreeCopyEntry {
            kind: EntryKind::File,
            mode: 0o644,
            relative_path: String::from("cache/file"),
        },
    ];
    return (stage, destination, entries);
}

/// A selected file installed as a hard link is removed, and so is its selected parent directory once empty.
#[cfg(unix)]
#[test]
fn proven_paths_are_removed_child_first() {
    let root: PathBuf = fixture("rollback-removes");
    let (stage, destination, entries) = staged_file(root.as_path());
    std::fs::create_dir(destination.join("cache")).expect("dir");
    std::fs::hard_link(stage.join("cache/file"), destination.join("cache/file")).expect("link");
    let created: Vec<InstalledPath> = vec![
        capture_installed_path(destination.join("cache").as_path(), "cache", true).expect("dir id"),
        capture_installed_path(destination.join("cache/file").as_path(), "cache/file", true)
            .expect("file id"),
    ];
    let retained: Vec<String> = rollback_created(
        stage.as_path(),
        destination.as_path(),
        entries.as_slice(),
        created.as_slice(),
    )
    .expect("rollback");
    assert!(retained.is_empty(), "{retained:?}");
    assert!(!destination.join("cache").exists());
    remove(root.as_path());
}

/// A path that is no longer there has nothing to remove and is not retained.
#[test]
fn absent_paths_are_neither_removed_nor_retained() {
    let root: PathBuf = fixture("rollback-absent");
    let (stage, destination, entries) = staged_file(root.as_path());
    std::fs::create_dir(destination.join("cache")).expect("dir");
    let created: Vec<InstalledPath> = vec![
        capture_installed_path(destination.join("cache").as_path(), "cache", true).expect("dir id"),
    ];
    std::fs::remove_dir(destination.join("cache")).expect("gone");
    let retained: Vec<String> = rollback_created(
        stage.as_path(),
        destination.as_path(),
        entries.as_slice(),
        created.as_slice(),
    )
    .expect("rollback");
    assert!(retained.is_empty());
    remove(root.as_path());
}

/// A path replaced by another object is named as changed and left in place.
#[cfg(unix)]
#[test]
fn replaced_path_is_retained_as_changed() {
    let root: PathBuf = fixture("rollback-changed");
    let (stage, destination, entries) = staged_file(root.as_path());
    std::fs::create_dir(destination.join("cache")).expect("dir");
    let created: Vec<InstalledPath> = vec![
        capture_installed_path(destination.join("cache").as_path(), "cache", true).expect("dir id"),
    ];
    // Renaming keeps the original object alive, so the replacement cannot reuse its inode.
    std::fs::rename(destination.join("cache"), destination.join("moved")).expect("move aside");
    std::fs::create_dir(destination.join("cache")).expect("replacement");
    let retained: Vec<String> = rollback_created(
        stage.as_path(),
        destination.as_path(),
        entries.as_slice(),
        created.as_slice(),
    )
    .expect("rollback");
    assert_eq!(
        retained,
        vec![String::from("cache: filesystem identity changed")]
    );
    assert!(destination.join("cache").is_dir());
    remove(root.as_path());
}

/// A file whose bytes changed in place keeps its identity but no longer equals the stage, so it is kept by its path name.
#[cfg(unix)]
#[test]
fn edited_file_is_retained_by_path() {
    let root: PathBuf = fixture("rollback-edited");
    let (stage, destination, entries) = staged_file(root.as_path());
    std::fs::create_dir(destination.join("cache")).expect("dir");
    std::fs::write(destination.join("cache/file"), b"bytes").expect("file");
    let created: Vec<InstalledPath> = vec![
        capture_installed_path(destination.join("cache/file").as_path(), "cache/file", true)
            .expect("id"),
    ];
    std::fs::write(destination.join("cache/file"), b"edited!").expect("edit in place");
    let retained: Vec<String> = rollback_created(
        stage.as_path(),
        destination.as_path(),
        entries.as_slice(),
        created.as_slice(),
    )
    .expect("rollback");
    assert_eq!(retained, vec![String::from("cache/file")]);
    assert_eq!(
        std::fs::read(destination.join("cache/file")).expect("kept"),
        b"edited!"
    );
    remove(root.as_path());
}

/// A scaffolding directory that gained a user file is kept, and the diagnostic carries the operating-system reason.
#[cfg(unix)]
#[test]
fn non_empty_scaffold_is_retained_with_reason() {
    let root: PathBuf = fixture("rollback-scaffold");
    let (stage, destination, entries) = staged_file(root.as_path());
    std::fs::create_dir(destination.join("scaffold")).expect("scaffold");
    let created: Vec<InstalledPath> = vec![
        capture_installed_path(destination.join("scaffold").as_path(), "scaffold", false)
            .expect("id"),
    ];
    std::fs::write(destination.join("scaffold/user"), b"mine").expect("user file");
    let retained: Vec<String> = rollback_created(
        stage.as_path(),
        destination.as_path(),
        entries.as_slice(),
        created.as_slice(),
    )
    .expect("rollback");
    assert_eq!(retained.len(), 1, "{retained:?}");
    assert!(retained[0].starts_with("scaffold: "), "{retained:?}");
    assert!(retained[0].contains("Directory not empty"), "{retained:?}");
    assert!(destination.join("scaffold/user").exists());
    remove(root.as_path());
}
