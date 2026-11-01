//! Manifest and mode checks on real filesystem trees.

use super::*;
use crate::test_support::{fixture, remove};

/// A manifest entry shorthand for expected values.
fn entry(kind: EntryKind, mode: u32, relative_path: &str) -> WorktreeCopyEntry {
    return WorktreeCopyEntry {
        kind,
        mode,
        relative_path: String::from(relative_path),
    };
}

/// A tree with nested directories, a symbolic link and files, under one selected root.
fn tree(root: &Path) {
    std::fs::create_dir_all(root.join("cache/deep")).expect("deep");
    std::fs::write(root.join("cache/deep/file"), b"x").expect("file");
    std::fs::write(root.join("cache/a"), b"a").expect("a");
    std::fs::write(root.join("cache/B"), b"b").expect("B");
    #[cfg(unix)]
    std::os::unix::fs::symlink("../elsewhere", root.join("cache/link")).expect("symlink");
}

/// Parents come before children, names follow JavaScript order, and symbolic links are recorded without being followed.
#[cfg(unix)]
#[test]
fn manifest_is_parent_first_in_javascript_order_without_following_links() {
    let root: PathBuf = fixture("manifest-order");
    tree(root.as_path());
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(root.as_path(), &[String::from("cache")], &[]).expect("manifest");
    let paths: Vec<&str> = entries
        .iter()
        .map(|entry| return entry.relative_path.as_str())
        .collect();
    assert_eq!(
        paths,
        vec![
            "cache",
            "cache/B",
            "cache/a",
            "cache/deep",
            "cache/deep/file",
            "cache/link",
        ]
    );
    assert_eq!(entries[5].kind, EntryKind::Symlink);
    assert_eq!(entries[0].kind, EntryKind::Directory);
    remove(root.as_path());
}

/// Excluded roots are skipped with all their descendants, and an unselected path is never read.
#[test]
fn excluded_roots_are_omitted_with_descendants() {
    let root: PathBuf = fixture("manifest-excluded");
    tree(root.as_path());
    let excluded: Vec<PathBuf> = vec![root.join("cache/deep")];
    let entries: Vec<WorktreeCopyEntry> = collect_entry_manifest(
        root.as_path(),
        &[String::from("cache")],
        excluded.as_slice(),
    )
    .expect("manifest");
    assert!(
        entries
            .iter()
            .all(|entry| return !entry.relative_path.starts_with("cache/deep"))
    );
    assert!(
        entries
            .iter()
            .any(|entry| return entry.relative_path == "cache/a")
    );
    remove(root.as_path());
}

/// A selected file is one entry, and a missing selected path is a copy failure with its cause.
#[test]
fn single_file_and_missing_selection() {
    let root: PathBuf = fixture("manifest-single");
    std::fs::write(root.join("only"), b"x").expect("only");
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(root.as_path(), &[String::from("only")], &[]).expect("manifest");
    assert_eq!(
        entries,
        vec![entry(
            EntryKind::File,
            retained_mode_of(root.join("only").as_path()),
            "only"
        )]
    );
    let failure: WorktreeCopyError =
        collect_entry_manifest(root.as_path(), &[String::from("missing")], &[])
            .expect_err("missing");
    assert!(failure.0.contains("No such file or directory"));
    remove(root.as_path());
}

/// Sockets, FIFOs and device nodes stop the walk with the unsupported-type diagnostic and are never opened.
#[cfg(unix)]
#[test]
fn fifo_is_refused_without_being_opened() {
    let root: PathBuf = fixture("manifest-fifo");
    let status = std::process::Command::new("mkfifo")
        .arg(root.join("pipe"))
        .status()
        .expect("mkfifo");
    assert!(status.success());
    let failure: WorktreeCopyError =
        collect_entry_manifest(root.as_path(), &[String::from("pipe")], &[]).expect_err("fifo");
    assert!(failure.0.starts_with("cli-git: ignored path "));
    assert!(failure.0.ends_with(" has unsupported filesystem type."));
    remove(root.as_path());
}

/// The retained mode is the entry's own permission bits; file-type bits never appear in it.
#[cfg(unix)]
#[test]
fn retained_mode_keeps_only_permission_bits() {
    let root: PathBuf = fixture("manifest-mode");
    std::fs::write(root.join("script"), b"#!/bin/sh\n").expect("script");
    std::fs::set_permissions(
        root.join("script"),
        std::os::unix::fs::PermissionsExt::from_mode(0o751),
    )
    .expect("mode");
    assert_eq!(retained_mode_of(root.join("script").as_path()), 0o751);
    remove(root.as_path());
}

/// Applying modes sets every file and directory to its recorded mode, deepest directories last.
#[cfg(unix)]
#[test]
fn applied_modes_match_the_manifest() {
    let root: PathBuf = fixture("manifest-apply");
    std::fs::create_dir_all(root.join("dir/inner")).expect("dirs");
    std::fs::write(root.join("dir/inner/file"), b"x").expect("file");
    let entries: Vec<WorktreeCopyEntry> = vec![
        entry(EntryKind::Directory, 0o700, "dir"),
        entry(EntryKind::Directory, 0o500, "dir/inner"),
        entry(EntryKind::File, 0o640, "dir/inner/file"),
    ];
    apply_entry_modes(root.as_path(), entries.as_slice()).expect("apply");
    assert_eq!(
        retained_mode_of(root.join("dir/inner/file").as_path()),
        0o640
    );
    assert_eq!(retained_mode_of(root.join("dir/inner").as_path()), 0o500);
    std::fs::set_permissions(
        root.join("dir/inner"),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("restore");
    assert_eq!(retained_mode_of(root.join("dir").as_path()), 0o700);
    remove(root.as_path());
}

/// The mode of one path, as the manifest reads it.
fn retained_mode_of(path: &Path) -> u32 {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("metadata");
    return retained_mode(&metadata);
}
