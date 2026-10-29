//! Malformed or detached directory snapshots cannot partially mutate the visible tree.

/// Tests exercise the same model entries supplied by the production workspace reader.
use ide_app::{
    file_tree::FileTree,
    workspace::{DirectoryEntry, Workspace},
};
/// Native filenames include byte sequences that cannot be represented by a Rust UTF-8 String.
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path};

/// Create an owned root child without interpreting its display text as a filesystem path.
fn child(name: &str) -> DirectoryEntry {
    // into creates an owned OsString; joining retains the native filename without any shell parsing.
    return DirectoryEntry {
        name: name.into(),
        path: Path::new("/project").join(name),
        is_directory: false,
    };
}

/// Invalid names,
///  duplicate keys,
///  and mismatched paths reject the entire replacement snapshot.
#[test]
fn invalid_snapshots_leave_existing_rows_unchanged() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    tree.apply_listing(root, vec![child("original.rs")])
        .expect("valid snapshot");
    let original = tree.rows();
    for name in [
        "",
        ".",
        "..",
        "../escape",
        "nested/file",
        "nested/.",
        "/outside",
    ] {
        assert!(
            tree.apply_listing(root, vec![child("new.rs"), child(name)])
                .is_err(),
            "accepted invalid name {name:?}"
        );
        assert_eq!(tree.rows(), original);
    }
    let mut mismatched = child("safe.rs");
    mismatched.path = root.join("different.rs");
    assert!(tree.apply_listing(root, vec![mismatched]).is_err());
    assert_eq!(tree.rows(), original);
    assert!(
        tree.apply_listing(root, vec![child("duplicate.rs"), child("duplicate.rs")])
            .is_err()
    );
    assert_eq!(tree.rows(), original);
}

/// Replies for directories removed by an ancestor refresh cannot resurrect cached rows.
#[test]
fn detached_directory_replies_are_rejected() {
    let root = Path::new("/project");
    let mut directory = child("src");
    directory.is_directory = true;
    // Clone the path before transferring directory metadata into the model.
    let path = directory.path.clone();
    let mut tree = FileTree::new(root);
    tree.apply_listing(root, vec![directory])
        .expect("directory snapshot");
    tree.toggle(&path).expect("expand directory");
    tree.apply_listing(root, vec![]).expect("external removal");
    assert!(tree.apply_listing(&path, vec![]).is_err());
    assert!(tree.rows().is_empty());
    assert!(tree.missing_listings().is_empty());
}

/// Real workspace snapshots retain hidden,
///  quoted,
///  Unicode,
///  and non-UTF-8 names in dirent order.
#[test]
fn workspace_snapshots_keep_native_names_and_order() {
    let fixture = tempfile::tempdir().expect("disposable project");
    // What: from_vec constructs a Unix native filename from arbitrary bytes, unlike UTF-8 String.
    // Why: UI display conversion must never become the identity used to open a file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = Buffer.from([0x62, 0x61, 0x64, 0xff]);
    // ```
    let native = OsString::from_vec(vec![0x62, 0x61, 0x64, 0xff]);
    for name in [
        OsString::from(".hidden"),
        OsString::from("猫 '\";$.rs"),
        native,
    ] {
        fs::write(fixture.path().join(name), "fixture").expect("source fixture");
    }
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let snapshot = workspace
        .list(workspace.root())
        .expect("native directory snapshot");
    let mut tree = FileTree::new(workspace.root());
    // Clone preserves expected native metadata while the model takes ownership of the other snapshot.
    tree.apply_listing(workspace.root(), snapshot.clone())
        .expect("apply native snapshot");
    // Consume the returned rows and extract their original metadata for an exact ordered comparison.
    let mut rows = Vec::new();
    for row in tree.rows() {
        rows.push(row.entry);
    }
    assert_eq!(rows, snapshot);
}
