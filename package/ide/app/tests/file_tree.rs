//! Lazy expansion consumes directory snapshots without performing any filesystem operations.

/// The production model and reader share native directory-entry values.
use ide_app::{file_tree::FileTree, workspace::DirectoryEntry};
/// Synthetic absolute paths make tests independent of cwd and human-owned directories.
use std::path::Path;

/// Construct native metadata without creating the represented file or directory.
fn entry(parent: &str, name: &str, is_directory: bool) -> DirectoryEntry {
    // What: into copies a borrowed string into an owned native filename.
    // Why: The snapshot may outlive the helper's borrowed input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { name, path: join(parent, name), isDirectory };
    // ```
    return DirectoryEntry {
        name: name.into(),
        path: Path::new(parent).join(name),
        is_directory,
    };
}

/// Initial root loading and child expansion are separate requests,
///  preserving supplied entry order.
#[test]
fn root_and_expanded_children_load_lazily_in_snapshot_order() {
    let root = Path::new("/project");
    // What: a mutable local owns the tree while assertions borrow its state.
    // Why: No represented path needs to exist on the filesystem.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const tree = new FileTree('/project');
    // ```
    let mut tree = FileTree::new(root);
    assert!(tree.rows().is_empty());
    assert_eq!(tree.missing_listings(), [root]);
    let entries = vec![
        entry("/project", "z", false),
        entry("/project", "src", true),
        entry("/project", ".hidden", false),
    ];
    // Clone the expected snapshot before transferring its counterpart into the model.
    tree.apply_listing(root, entries.clone())
        .expect("root snapshot");
    // The closure extracts only metadata so the comparison tests ordering, not UI strings.
    let mut actual = Vec::new();
    for row in tree.rows() {
        actual.push(row.entry);
    }
    assert_eq!(actual, entries);
    assert!(tree.missing_listings().is_empty());
    let src = root.join("src");
    assert!(tree.toggle(&src).expect("expand src"));
    assert_eq!(tree.missing_listings(), [src.as_path()]);
    tree.apply_listing(
        &src,
        vec![
            entry("/project/src", "猫.ts", false),
            entry("/project/src", "a.rs", false),
        ],
    )
    .expect("child snapshot");
    let rows = tree.rows();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[2].entry.name, "猫.ts");
    assert_eq!(rows[3].entry.name, "a.rs");
    assert_eq!(rows[2].depth, 1);
    assert_eq!(rows[4].depth, 0);
    assert!(rows[1].expanded);
    assert!(tree.missing_listings().is_empty());
}

/// Collapsing an ancestor hides descendants and their missing requests without losing expansion state.
#[test]
fn collapse_retains_descendant_expansion_but_stops_hidden_loading() {
    let root = Path::new("/project");
    let src = root.join("src");
    let nested = src.join("nested");
    let mut tree = FileTree::new(root);
    tree.apply_listing(root, vec![entry("/project", "src", true)])
        .expect("root snapshot");
    tree.toggle(&src).expect("expand src");
    tree.apply_listing(&src, vec![entry("/project/src", "nested", true)])
        .expect("child snapshot");
    tree.toggle(&nested).expect("expand nested");
    assert_eq!(tree.missing_listings(), [nested.as_path()]);
    assert!(!tree.toggle(&src).expect("collapse src"));
    assert_eq!(tree.rows().len(), 1);
    assert!(tree.missing_listings().is_empty());
    assert!(tree.toggle(&src).expect("reopen src"));
    assert_eq!(tree.missing_listings(), [nested]);
    assert!(tree.rows()[1].expanded);
}

/// Detached subtrees lose cached data;
///  prefix siblings and surviving directories keep their state.
#[test]
fn refresh_prunes_removed_subtrees_without_pruning_prefix_siblings() {
    let root = Path::new("/project");
    let src = root.join("src");
    let sibling = root.join("src-other");
    let mut tree = FileTree::new(root);
    tree.apply_listing(
        root,
        vec![
            entry("/project", "src", true),
            entry("/project", "src-other", true),
        ],
    )
    .expect("root snapshot");
    for path in [&src, &sibling] {
        tree.toggle(path).expect("expand directory");
        // to_str lends UTF-8 from these explicitly ASCII test paths; expect rejects accidental fixture changes.
        tree.apply_listing(
            path,
            vec![entry(
                path.to_str().expect("ASCII fixture"),
                "old.rs",
                false,
            )],
        )
        .expect("child snapshot");
    }
    tree.apply_listing(root, vec![entry("/project", "src-other", true)])
        .expect("external removal");
    assert_eq!(tree.rows().len(), 2);
    assert!(tree.rows()[0].expanded);
    tree.apply_listing(
        root,
        vec![
            entry("/project", "src", true),
            entry("/project", "src-other", true),
        ],
    )
    .expect("recreated directory");
    assert!(!tree.rows()[0].expanded);
    tree.toggle(&src).expect("expand recreated directory");
    assert_eq!(tree.missing_listings(), [src]);
    assert_eq!(tree.rows().len(), 3);
}

/// Replacing a directory with a file must prevent stale descendants from reappearing later.
#[test]
fn directory_to_file_replacement_discards_expansion_and_cache() {
    let root = Path::new("/project");
    let path = root.join("changed");
    let mut tree = FileTree::new(root);
    tree.apply_listing(root, vec![entry("/project", "changed", true)])
        .expect("root snapshot");
    tree.toggle(&path).expect("expand directory");
    tree.apply_listing(&path, vec![entry("/project/changed", "old.rs", false)])
        .expect("child snapshot");
    tree.apply_listing(root, vec![entry("/project", "changed", false)])
        .expect("file replacement");
    assert_eq!(tree.rows().len(), 1);
    assert!(!tree.rows()[0].expanded);
    assert!(tree.toggle(&path).is_err());
    assert!(tree.apply_listing(&path, vec![]).is_err());
    tree.apply_listing(root, vec![entry("/project", "changed", true)])
        .expect("directory replacement");
    tree.toggle(&path).expect("expand replacement directory");
    assert_eq!(tree.missing_listings(), [path]);
}

/// An empty loaded root is not a missing snapshot and cannot be toggled as a visible row.
#[test]
fn empty_root_and_unknown_entries_remain_distinct() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    assert!(!tree.has_listing(root));
    tree.apply_listing(root, vec![])
        .expect("empty root snapshot");
    assert!(tree.has_listing(root));
    assert!(tree.rows().is_empty());
    assert!(tree.missing_listings().is_empty());
    assert!(tree.toggle(root).is_err());
    assert!(tree.toggle(&root.join("missing")).is_err());
    assert!(tree.apply_listing(&root.join("missing"), vec![]).is_err());
}
