//! The servers' folder scan agrees with the search's ignore rules,
//!  on disposable projects.

/// The functions under test.
use super::{bases, scan};
/// What:
///  `BTreeSet` is an ordered set;
///  `fs` creates the fixture;
///  `PathBuf` is an owned path.
/// Why:
///  Results are compared as sets of folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mkdirSync, writeFileSync } from 'node:fs';
/// ```
use std::{collections::BTreeSet, fs, path::PathBuf};

/// What:
///  A disposable project inside a git work tree:
///  a `.git` folder makes ripgrep honour `.gitignore`.
///       Returns the temporary directory (removed when dropped) and the canonical root.
/// Why:
///  Every test needs ignored,
///  pruned,
///  hidden,
///  empty,
///  and source folders side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function project(): [TempDir, string]
/// ```
fn project() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("disposable project");
    let root = directory.path().canonicalize().expect("canonical root");
    for folder in [
        ".git",
        "src/deep",
        "src/empty",
        "dist/sub",
        "node_modules/pkg",
        "target/debug",
        ".hidden",
        "only-ignored",
    ] {
        fs::create_dir_all(root.join(folder)).expect("fixture folder");
    }
    for (file, text) in [
        (".gitignore", "dist/\n*.log\n"),
        ("src/a.ts", "a"),
        ("src/deep/b.ts", "b"),
        ("dist/out.js", "x"),
        ("dist/sub/out.js", "x"),
        ("node_modules/pkg/index.js", "x"),
        ("target/debug/build.rs", "x"),
        (".hidden/c.ts", "c"),
        ("only-ignored/run.log", "x"),
    ] {
        fs::write(root.join(file), text).expect("fixture file");
    }
    return (directory, root);
}

/// Source folders are the ones the search lists files from;
///  ignored,
///  pruned,
///  and hidden ones are left out,
/// and an empty folder is watched provisionally.
#[test]
fn source_folders_follow_the_search_ignore_rules() {
    let (_directory, root) = project();
    let found = scan(&root, &root, true);
    let mut expected: BTreeSet<PathBuf> = BTreeSet::new();
    for relative in ["", "src", "src/deep"] {
        expected.insert(root.join(relative));
    }
    // `root.join("")` is the root itself.
    assert_eq!(found.directories, expected);
    assert_eq!(
        found.provisional,
        BTreeSet::from([root.join("src/empty")]),
        "the empty folder was not watched provisionally"
    );
    assert!(found.initial);
    assert!(
        found.files.contains(&root.join("src/deep/b.ts")),
        "listed files: {:?}",
        found.files
    );
}

/// A folder ripgrep lists files from is still not watched below `node_modules` or `target`,
///  even when no
/// ignore file names them.
#[test]
fn pruned_folders_are_left_out_without_an_ignore_file() {
    let directory = tempfile::tempdir().expect("disposable project");
    let root = directory.path().canonicalize().expect("canonical root");
    fs::create_dir_all(root.join("lib/node_modules/pkg")).expect("fixture folder");
    fs::create_dir_all(root.join("lib/target")).expect("fixture folder");
    fs::write(root.join("lib/node_modules/pkg/index.js"), "x").expect("fixture file");
    fs::write(root.join("lib/target/out.rs"), "x").expect("fixture file");
    fs::write(root.join("lib/main.rs"), "x").expect("fixture file");
    let found = scan(&root, &root, true);
    assert_eq!(
        found.directories,
        BTreeSet::from([root.clone(), root.join("lib")])
    );
}

/// A new folder asks for its parent to be scanned,
///  a new file asks for nothing,
///  and a request inside
/// another requested folder is covered by that one.
#[test]
fn new_folders_are_classified_by_their_parent() {
    let (_directory, root) = project();
    let candidates = BTreeSet::from([
        root.join("src/deep"),
        root.join("src/a.ts"),
        root.join("node_modules"),
        root.join("missing"),
    ]);
    assert_eq!(
        bases(&root, &candidates, &BTreeSet::new()),
        vec![root.join("src")]
    );
    let rescans = BTreeSet::from([root.clone()]);
    assert_eq!(
        bases(&root, &candidates, &rescans),
        vec![root.clone()],
        "a request inside the root was not covered by the root's scan"
    );
}

/// Scanning an ignored folder's parent leaves it out;
///  scanning the folder itself would not,
///  which is
/// why new folders are classified by their parent.
#[test]
fn an_ignored_folder_is_left_out_when_its_parent_is_scanned() {
    let (_directory, root) = project();
    let parent = scan(&root, &root, false);
    assert!(!parent.directories.contains(&root.join("dist")));
    let itself = scan(&root, &root.join("dist"), false);
    assert!(
        itself.directories.contains(&root.join("dist/sub")),
        "ripgrep no longer lists an ignored folder given on its command line; the classification by parent may be unnecessary"
    );
}
