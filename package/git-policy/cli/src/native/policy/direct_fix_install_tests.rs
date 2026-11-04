//! What:
//!  Controls for installing corrections:
//!  all files replaced with their modes and the
//!       index untouched,
//!  or no file changed,
//!  whichever step fails.
//! Why:
//!  A fix that left half its files replaced,
//!  a stray sibling file,
//!  or a changed index
//!      would damage the worktree it was asked to tidy.
//!  Every failure is provoked for real
//!      on a disposable directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await expect(installCorrections(root, index, changes)).rejects.toThrow('changed while');
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{
    INSTALL_SIBLING_PREFIX, InstallChange, Replacement, install_corrections, install_prepared,
    prepare_replacements, read_index,
};
use crate::candidate_object::CandidateMode;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A worktree with a regular file at the top,
///  an executable one below,
///  and an index file.
struct Tree {
    /// The disposable fixture,
    ///  removed at the end.
    root: PathBuf,
    /// The worktree's top level.
    top: PathBuf,
    /// The index file.
    index: PathBuf,
}

/// A fresh tree under a fixture named `name`.
fn tree(name: &str) -> Tree {
    let root: PathBuf = fixture(name);
    let top: PathBuf = root.join("top");
    std::fs::create_dir_all(top.join("sub")).expect("worktree");
    std::fs::write(top.join("a.txt"), b"a").expect("a");
    std::fs::write(top.join("sub/run.sh"), b"#!/bin/sh").expect("script");
    std::fs::set_permissions(
        top.join("sub/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("executable");
    let index: PathBuf = root.join("index");
    std::fs::write(&index, b"index bytes").expect("index");
    return Tree { root, top, index };
}

/// The two corrections of a tree.
fn changes() -> Vec<InstallChange> {
    return vec![
        InstallChange {
            path: b"a.txt".to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(&b"a"[..]),
            replacement: Rc::from(&b"a\n"[..]),
        },
        InstallChange {
            path: b"sub/run.sh".to_vec(),
            mode: CandidateMode::Executable,
            original: Rc::from(&b"#!/bin/sh"[..]),
            replacement: Rc::from(&b"#!/bin/sh\n"[..]),
        },
    ];
}

/// Every file name below `directory` that starts with the sibling prefix.
fn siblings(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("directory") {
        let path: PathBuf = entry.expect("entry").path();
        if path.is_dir() {
            found.extend(siblings(path.as_path()));
            continue;
        }
        let name: String = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        if name.starts_with(INSTALL_SIBLING_PREFIX) {
            found.push(name);
        }
    }
    return found;
}

/// The two worktree files' bytes,
///  in a fixed order.
fn contents(top: &Path) -> (Vec<u8>, Vec<u8>) {
    return (
        std::fs::read(top.join("a.txt")).expect("a"),
        std::fs::read(top.join("sub/run.sh")).expect("script"),
    );
}

/// The untouched bytes of a tree.
fn untouched() -> (Vec<u8>, Vec<u8>) {
    return (b"a".to_vec(), b"#!/bin/sh".to_vec());
}

/// Every file is replaced with its mode,
///  the index is the same,
///  and nothing is left beside them.
#[test]
fn corrections_are_installed_with_their_modes() {
    let fixture_tree: Tree = tree("install-ok");
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Ok(())
    );
    assert_eq!(
        contents(&fixture_tree.top),
        (b"a\n".to_vec(), b"#!/bin/sh\n".to_vec())
    );
    let regular: u32 = std::fs::metadata(fixture_tree.top.join("a.txt"))
        .expect("a")
        .permissions()
        .mode();
    let executable: u32 = std::fs::metadata(fixture_tree.top.join("sub/run.sh"))
        .expect("script")
        .permissions()
        .mode();
    assert_eq!(regular & 0o111, 0, "{regular:o}");
    assert_eq!(executable & 0o100, 0o100, "{executable:o}");
    assert_eq!(
        std::fs::read(&fixture_tree.index).expect("index"),
        b"index bytes"
    );
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    // A repository without an index file still has none afterwards, and nothing to install
    // installs nothing.
    std::fs::remove_file(&fixture_tree.index).expect("no index");
    assert_eq!(read_index(&fixture_tree.index).expect("absent"), None);
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, &[]),
        Ok(())
    );
    let again: Vec<InstallChange> = vec![InstallChange {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a\n"[..]),
        replacement: Rc::from(&b"b\n"[..]),
    }];
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, again.as_slice()),
        Ok(())
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"b\n"
    );
    assert!(!fixture_tree.index.exists());
    remove(fixture_tree.root.as_path());
}

/// A file that changed under the fix,
///  is missing,
///  or has no name stops the fix before
/// any file changes,
///  and leaves nothing beside the files.
#[test]
fn a_file_that_is_not_as_read_stops_before_any_change() {
    let fixture_tree: Tree = tree("install-stale");
    std::fs::write(fixture_tree.top.join("sub/run.sh"), b"edited").expect("edit");
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Err(String::from(
            "cli-git fix could not install its corrections: a selected file changed while cli-git fix ran, so no file was changed. Run cli-git fix again."
        ))
    );
    assert_eq!(
        contents(&fixture_tree.top),
        (b"a".to_vec(), b"edited".to_vec())
    );
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    let missing: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("missing file");
    assert!(
        missing.starts_with(
            "cli-git fix could not install its corrections: reading a corrected file failed ("
        ) && missing.ends_with("), so no file was changed."),
        "{missing}"
    );
    assert!(!missing.contains("run.sh"), "{missing}");
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    let mut nameless: Vec<InstallChange> = changes();
    nameless[1].path = Vec::new();
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, nameless.as_slice()),
        Err(String::from(
            "cli-git fix could not install its corrections: Git named a corrected file this system cannot represent, so no file was changed."
        ))
    );
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    remove(fixture_tree.root.as_path());
}

/// A sibling that cannot be written stops the fix before any change.
#[test]
fn an_unwritable_directory_stops_before_any_change() {
    let fixture_tree: Tree = tree("install-unwritable");
    let sub: PathBuf = fixture_tree.top.join("sub");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o555)).expect("read-only");
    let probe: std::io::Result<std::fs::File> = std::fs::File::create(sub.join("probe"));
    // Positive control: a process that ignores permissions cannot run this control.
    if probe.is_ok() {
        std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).expect("restore");
        remove(fixture_tree.root.as_path());
        panic!("the read-only directory accepted a new file, so this control cannot run here");
    }
    let refused: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("unwritable");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert!(
        refused.starts_with("cli-git fix could not install its corrections: writing a corrected file beside its original failed (")
            && refused.ends_with("), so no file was changed."),
        "{refused}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// An index that changed,
///  cannot be read again,
///  or cannot be read at all restores every file.
#[test]
fn an_index_that_is_not_as_read_restores_every_file() {
    let fixture_tree: Tree = tree("install-index");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    assert_eq!(
        install_prepared(prepared.as_slice(), &fixture_tree.index, &before),
        Err(String::from(
            "cli-git fix could not install its corrections: the index changed while cli-git fix ran, so every corrected file was restored. Run cli-git fix again."
        ))
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    // An index that turned into a directory cannot be read again.
    std::fs::remove_file(&fixture_tree.index).expect("remove index");
    std::fs::create_dir(&fixture_tree.index).expect("index directory");
    let prepared_again: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    let unreadable: String =
        install_prepared(prepared_again.as_slice(), &fixture_tree.index, &before)
            .expect_err("unreadable index");
    assert!(
        unreadable.starts_with(
            "cli-git fix could not install its corrections: reading the index again failed ("
        ) && unreadable
            .ends_with("), so every corrected file was restored. Run cli-git fix again."),
        "{unreadable}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    // The first read failing changes nothing at all.
    let first: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("unreadable index");
    assert!(
        first.starts_with(
            "cli-git fix could not install its corrections: reading the index failed ("
        ) && first.ends_with("), so no file was changed."),
        "{first}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// A rename that fails restores the files already replaced;
///  a restore that fails keeps
/// the backup it needs and says so.
#[test]
fn a_failed_replacement_restores_and_a_failed_restore_keeps_its_backup() {
    let fixture_tree: Tree = tree("install-rename");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    // The second destination becomes a non-empty directory, so renaming over it fails.
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    std::fs::create_dir_all(fixture_tree.top.join("sub/run.sh/inside")).expect("directory");
    let failed: String = install_prepared(prepared.as_slice(), &fixture_tree.index, &before)
        .expect_err("rename fails");
    assert!(
        failed.starts_with(
            "cli-git fix could not install its corrections: replacing a corrected file failed ("
        ) && failed.ends_with("), so every corrected file was restored. Run cli-git fix again."),
        "{failed}"
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    // Back to two files; this time the first backup vanishes before the index changes.
    std::fs::remove_dir_all(fixture_tree.top.join("sub/run.sh")).expect("directory");
    std::fs::write(fixture_tree.top.join("sub/run.sh"), b"#!/bin/sh").expect("script");
    let lost: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    let first_backup: PathBuf = top_backup(&fixture_tree.top).expect("first backup");
    std::fs::remove_file(&first_backup).expect("lose the backup");
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    let unrestored: String =
        install_prepared(lost.as_slice(), &fixture_tree.index, &before).expect_err("restore fails");
    assert!(
        unrestored.starts_with(
            "cli-git fix could not install its corrections: the index changed while cli-git fix ran, and restoring a corrected file failed ("
        ) && unrestored.ends_with(
            "); its original bytes remain beside it in a file whose name starts with .cli-git-direct-fix-."
        ),
        "{unrestored}"
    );
    // The later file was restored first; its backup was used, and no prepared file is left.
    assert_eq!(
        std::fs::read(fixture_tree.top.join("sub/run.sh")).expect("script"),
        b"#!/bin/sh"
    );
    for name in siblings(&fixture_tree.top) {
        assert!(!name.ends_with(".new"), "{name}");
    }
    remove(fixture_tree.root.as_path());
}

/// The backup the fix wrote beside the top-level file,
///  if any.
fn top_backup(directory: &Path) -> Option<PathBuf> {
    for entry in std::fs::read_dir(directory).expect("directory") {
        let path: PathBuf = entry.expect("entry").path();
        let name: String = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        if name.starts_with(INSTALL_SIBLING_PREFIX) && name.ends_with(".old") {
            return Some(path);
        }
    }
    return None;
}
