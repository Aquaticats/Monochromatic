//! Entry creation checks on real filesystem trees.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// A manifest entry shorthand.
fn entry(kind: EntryKind, mode: u32, relative_path: &str) -> WorktreeCopyEntry {
    return WorktreeCopyEntry {
        kind,
        mode,
        relative_path: String::from(relative_path),
    };
}

/// The identity captured after creation names the object that exists at the path, and carries its selection flag.
#[test]
fn captured_identity_names_the_created_object() {
    let root: PathBuf = fixture("install-entry-identity");
    std::fs::write(root.join("file"), b"x").expect("file");
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(root.join("file")).expect("metadata");
    let captured: InstalledPath =
        capture_installed_path(root.join("file").as_path(), "file", true).expect("captured");
    assert_eq!(captured.device, metadata.dev());
    assert_eq!(captured.inode, metadata.ino());
    assert!(captured.selected);
    assert_eq!(captured.relative_path, "file");
    remove(root.as_path());
}

/// Missing parents are created shallow first and recorded as unselected; an existing directory parent is kept unrecorded.
#[test]
fn missing_parents_are_created_and_recorded() {
    let root: PathBuf = fixture("install-entry-parents");
    std::fs::create_dir(root.join("a")).expect("existing parent");
    let mut created: Vec<InstalledPath> = Vec::new();
    ensure_parents(
        root.as_path(),
        &entry(EntryKind::File, 0o644, "a/b/c/file"),
        &mut created,
    )
    .expect("parents");
    let paths: Vec<(&str, bool)> = created
        .iter()
        .map(|installed| return (installed.relative_path.as_str(), installed.selected))
        .collect();
    assert_eq!(paths, vec![("a/b", false), ("a/b/c", false)]);
    assert!(root.join("a/b/c").is_dir());
    remove(root.as_path());
}

/// A parent that exists as a file refuses the entry with the parent named.
#[test]
fn file_in_parent_position_is_refused() {
    let root: PathBuf = fixture("install-entry-file-parent");
    std::fs::write(root.join("a"), b"x").expect("file parent");
    let mut created: Vec<InstalledPath> = Vec::new();
    let failure: WorktreeCopyError = ensure_parents(
        root.as_path(),
        &entry(EntryKind::File, 0o644, "a/file"),
        &mut created,
    )
    .expect_err("file parent");
    assert_eq!(
        failure.0,
        format!(
            "cli-git: ignored-state parent is not a directory: \"a\" in {}.",
            quote(root.to_string_lossy().as_ref())
        )
    );
    assert!(created.is_empty());
    remove(root.as_path());
}

/// A selected directory is created private, a link keeps its exact target, and a file is a hard link to its stage copy.
#[cfg(unix)]
#[test]
fn selected_entries_are_created_by_kind() {
    let root: PathBuf = fixture("install-entry-kinds");
    let stage: PathBuf = root.join("stage");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(stage.join("dir")).expect("stage dir");
    std::fs::write(stage.join("file"), b"bytes").expect("stage file");
    std::os::unix::fs::symlink("../target text", stage.join("link")).expect("stage link");
    std::fs::create_dir(destination.as_path()).expect("destination");
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::Directory, 0o755, "dir"),
    )
    .expect("directory");
    assert_eq!(
        std::fs::metadata(destination.join("dir"))
            .expect("dir")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::File, 0o640, "file"),
    )
    .expect("file");
    let staged_inode: u64 = std::fs::metadata(stage.join("file")).expect("stage").ino();
    let installed_inode: u64 = std::fs::metadata(destination.join("file"))
        .expect("installed")
        .ino();
    assert_eq!(
        staged_inode, installed_inode,
        "a hard link shares the staged object"
    );
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::Symlink, 0o777, "link"),
    )
    .expect("link");
    assert_eq!(
        std::fs::read_link(destination.join("link")).expect("target"),
        PathBuf::from("../target text")
    );
    remove(root.as_path());
}

/// Only the link-refusal codes fall back to a copy; any other operating-system error is reported as it is.
#[cfg(unix)]
#[test]
fn only_link_refusals_fall_back_to_copy() {
    for code in [
        libc::EXDEV,
        libc::EPERM,
        libc::EMLINK,
        libc::ENOSYS,
        libc::EOPNOTSUPP,
    ] {
        assert!(
            is_link_unsupported(&std::io::Error::from_raw_os_error(code)),
            "code {code}"
        );
    }
    assert!(!is_link_unsupported(&std::io::Error::from_raw_os_error(
        libc::EIO
    )));
    assert!(!is_link_unsupported(&std::io::Error::other("no code")));
}

/// A file whose target already exists is never replaced by the link.
#[cfg(unix)]
#[test]
fn existing_destination_file_is_not_replaced() {
    let root: PathBuf = fixture("install-entry-exists");
    let stage: PathBuf = root.join("stage");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(stage.join("file"), b"staged").expect("staged");
    std::fs::write(destination.join("file"), b"existing").expect("existing");
    let failure: WorktreeCopyError = create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::File, 0o644, "file"),
    )
    .expect_err("exists");
    assert!(failure.0.contains("File exists"));
    assert_eq!(
        std::fs::read(destination.join("file")).expect("kept"),
        b"existing"
    );
    remove(root.as_path());
}
