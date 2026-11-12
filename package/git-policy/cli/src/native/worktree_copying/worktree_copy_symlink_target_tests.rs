//! What:
//!  Controls for the link texts a copied tree holds and for the rule that tells two links naming one place.
//! Why:
//!  The behaviour under test is what a fresh worktree's `node_modules` links name,
//!  so every control builds a real tree with real links and asks the kernel where the copied link reaches,
//!  instead of comparing text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copied = copyOneLeaf(sourceRoot, stagePath, entry); symlink(readLink(stage), destinationLink); assert(canonicalize(destinationLink) === canonicalize(sourceLink));
//! ```

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry, retained_mode};
use crate::worktree_copy_stage_leaf_copying::{CopiedLeaf, copy_one_leaf};
use std::path::{Path, PathBuf};

/// The link the controls copy: a package's entry in one workspace package's `node_modules`.
const THE_LINK: &[u8] = b"package/x/node_modules/zod";

/// A clone callback for a link,
/// which never runs: only files are cloned.
fn never_cloned(_source: &std::fs::File, _stage: &std::fs::File) -> bool {
    return false;
}

/// One link as the manifest describes it.
fn link_entry(source_root: &Path, relative_path: &[u8]) -> WorktreeCopyEntry {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(source_root.join(crate::path_bytes::os_from_path_bytes(relative_path))).expect("metadata");
    return WorktreeCopyEntry { kind: EntryKind::Symlink, mode: retained_mode(&metadata), relative_path: relative_path.to_vec() };
}

/// What:
///  One link of the source copied into a stage,
///  then installed at a destination the caller places,
///  which is the two steps the add takes.
/// Why:
///   The stage and the destination sit at different depths,
///   so a control that only compared texts would miss the climb this file is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const install = (sourceRoot, stageRoot, destinationRoot, relativePath) => { copyOneLeaf(...); const stage = join(stageRoot, relativePath); const destination = join(destinationRoot, relativePath); mkdir(dirname(destination)); symlink(readLink(stage), destination); return destination };
/// ```
fn copied_link(source_root: &Path, stage_root: &Path, destination_root: &Path, relative_path: &[u8]) -> PathBuf {
    let entry: WorktreeCopyEntry = link_entry(source_root, relative_path);
    let stage_path: PathBuf = stage_root.join(crate::path_bytes::os_from_path_bytes(relative_path));
    std::fs::create_dir_all(stage_path.parent().expect("stage parent")).expect("stage directories");
    let leaf: CopiedLeaf = copy_one_leaf(source_root, stage_path.as_path(), &entry, never_cloned).expect("copy");
    assert!(matches!(leaf, CopiedLeaf::Staged(_)), "the link was not staged");
    let destination_path: PathBuf = destination_root.join(crate::path_bytes::os_from_path_bytes(relative_path));
    std::fs::create_dir_all(destination_path.parent().expect("destination parent")).expect("destination directories");
    let text: PathBuf = std::fs::read_link(stage_path.as_path()).expect("staged text");
    std::os::unix::fs::symlink(text, destination_path.as_path()).expect("install");
    return destination_path;
}

/// What:
///  A link from inside a checkout into a store beside it,
///  the shape pnpm writes for a package it holds in its store.
/// Why:
///   The store sits outside the copied tree,
///   so the link's text counts levels to it from the checkout's own depth,
///   and a copy at greater depth climbs past it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const storeBeside = (root) => { mkdir(root/store/links/@/zod/node_modules/zod); mkdir(root/checkout/package/x/node_modules); symlink('../../../../store/...', root/checkout/package/x/node_modules/zod) };
/// ```
fn store_beside(root: &Path) -> PathBuf {
    let package: PathBuf = root.join("store/links/@/zod/node_modules/zod");
    std::fs::create_dir_all(package.as_path()).expect("store entry");
    std::fs::write(package.join("package.json"), b"{}").expect("store entry file");
    let modules: PathBuf = root.join("checkout/package/x/node_modules");
    std::fs::create_dir_all(modules.as_path()).expect("modules");
    std::os::unix::fs::symlink("../../../../store/links/@/zod/node_modules/zod", modules.join("zod")).expect("store link");
    return modules.join("zod");
}

/// What:
///  A link into pnpm's store reaches the same store entry after the copy is installed three levels deeper.
/// Why:
///   This is the failure a fresh worktree was left with:
///   the copied link climbed from the worktree's depth to a directory that holds no `home`,
///   so pnpm's own install had a link it could not resolve.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(canonicalize(copied) === canonicalize(sourceLink));
/// ```
#[test]
fn a_link_into_a_store_outside_the_tree_reaches_it_from_the_deeper_worktree() {
    let root: PathBuf = fixture("symlink-target-store");
    let source_link: PathBuf = store_beside(root.as_path());
    let destination_root: PathBuf = root.join("deeper/one/two/three/checkout");
    let copied: PathBuf = copied_link(root.join("checkout").as_path(), root.join("stage-of-the-copy").as_path(), destination_root.as_path(), THE_LINK);
    let reached: PathBuf = std::fs::canonicalize(copied.as_path()).expect("the copied link reaches the store");
    assert_eq!(reached, std::fs::canonicalize(source_link.as_path()).expect("the source link reaches the store"));
    remove(root.as_path());
}

/// What:
///  A link into the tree itself reaches the copy's own directory after the copy is installed deeper.
/// Why:
///   The copy keeps the tree's shape,
///   so a text that stays inside it names the same place at any depth,
///   and rewriting it would send the worktree's own links back to the checkout it was copied from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(canonicalize(copied) === canonicalize(destinationRoot + '/package/x/y'));
/// ```
#[test]
fn a_link_inside_the_tree_reaches_the_copys_own_directory() {
    let root: PathBuf = fixture("symlink-target-inside");
    let source_root: PathBuf = root.join("checkout");
    std::fs::create_dir_all(source_root.join("package/x/y").as_path()).expect("target");
    std::fs::create_dir_all(source_root.join("package/x/node_modules").as_path()).expect("modules");
    std::os::unix::fs::symlink("../y", source_root.join("package/x/node_modules/zod")).expect("inside link");
    let destination_root: PathBuf = root.join("elsewhere/the worktree");
    std::fs::create_dir_all(destination_root.join("package/x/y").as_path()).expect("the copy's own directory");
    let copied: PathBuf = copied_link(source_root.as_path(), root.join("stage-of-the-copy").as_path(), destination_root.as_path(), THE_LINK);
    assert_eq!(
        std::fs::canonicalize(copied.as_path()).expect("the copied link reaches the copy's own directory"),
        std::fs::canonicalize(destination_root.join("package/x/y").as_path()).expect("the copy's own directory")
    );
    assert_ne!(
        std::fs::canonicalize(copied.as_path()).expect("the copied link reaches the copy's own directory"),
        std::fs::canonicalize(source_root.join("package/x/y").as_path()).expect("the checkout's directory")
    );
    remove(root.as_path());
}

/// What:
///  An absolute link into the tree becomes a climb that reaches the copy's own directory.
/// Why:
///   pnpm records `link:` overrides as absolute paths into the checkout that declared them,
///   and a copy that kept the text would make the worktree use the other checkout's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(isRelative(readLink(copied))); assert(canonicalize(copied) === canonicalize(destinationRoot + '/package/x/y'));
/// ```
#[test]
fn an_absolute_link_into_the_tree_names_the_copys_own_directory() {
    let root: PathBuf = fixture("symlink-target-absolute-inside");
    let source_root: PathBuf = root.join("checkout");
    std::fs::create_dir_all(source_root.join("package/x/y").as_path()).expect("target");
    std::fs::create_dir_all(source_root.join("package/x/node_modules").as_path()).expect("modules");
    std::os::unix::fs::symlink(source_root.join("package/x/y"), source_root.join("package/x/node_modules/zod")).expect("absolute link");
    let destination_root: PathBuf = root.join("elsewhere/the worktree");
    std::fs::create_dir_all(destination_root.join("package/x/y").as_path()).expect("the copy's own directory");
    let copied: PathBuf = copied_link(source_root.as_path(), root.join("stage-of-the-copy").as_path(), destination_root.as_path(), THE_LINK);
    assert!(std::fs::read_link(copied.as_path()).expect("text").is_relative(), "the copy kept an absolute path into the checkout");
    assert_eq!(
        std::fs::canonicalize(copied.as_path()).expect("the copied link reaches the copy's own directory"),
        std::fs::canonicalize(destination_root.join("package/x/y").as_path()).expect("the copy's own directory")
    );
    remove(root.as_path());
}

/// What:
///  A text that leaves the tree is written as the absolute path it resolved to,
///  and one that stays inside keeps the source's text.
/// Why:
///   The two answers are the whole rule of this feature:
///   only a climb that leaves the tree needs a spelling that does not count levels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(targetForTheNewPlace('../../..', link, root) === resolve(link, '../../..')); assert(targetForTheNewPlace('../y', link, root) === '../y');
/// ```
#[test]
fn only_a_text_that_leaves_the_tree_is_made_absolute() {
    let root: PathBuf = fixture("symlink-target-rule");
    let source_root: PathBuf = root.join("checkout");
    let link: PathBuf = source_root.join("package/x/node_modules/zod");
    let outside: &Path = Path::new("../../../../store/links/zod");
    assert_eq!(
        target_for_the_new_place(outside, link.as_path(), source_root.as_path()),
        normalized_path(link.parent().expect("parent").join(outside).as_path())
    );
    let inside: &Path = Path::new("../y");
    assert_eq!(target_for_the_new_place(inside, link.as_path(), source_root.as_path()), PathBuf::from("../y"));
    remove(root.as_path());
}

/// What:
///  A stage's link and the source's link name one place when one text was made absolute,
///  and two store entries are two places.
/// Why:
///   The check that the source still matches the stage compares links,
///   and it must accept the text the copy had to change while still refusing another package's entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(namesTheSamePlace('../a/store/zod', sourceLink, sourceRoot, '/abs/store/zod', stageLink, stageRoot)); assert(!namesTheSamePlace(... other entry ...));
/// ```
#[test]
fn a_made_absolute_text_and_the_source_text_name_one_place() {
    let root: PathBuf = fixture("symlink-target-same-place");
    let source_root: PathBuf = root.join("checkout");
    let source_link: PathBuf = source_root.join("package/x/node_modules/zod");
    let stage_root: PathBuf = root.join("elsewhere/stage");
    let stage_link: PathBuf = stage_root.join("package/x/node_modules/zod");
    let reached: PathBuf = normalized_path(source_link.parent().expect("parent").join("../../../../store/links/zod").as_path());
    assert!(names_the_same_place(
        Path::new("../../../../store/links/zod"),
        source_link.as_path(),
        source_root.as_path(),
        reached.as_path(),
        stage_link.as_path(),
        stage_root.as_path()
    ));
    assert!(!names_the_same_place(
        Path::new("../../../../store/links/zod"),
        source_link.as_path(),
        source_root.as_path(),
        Path::new("/abs/store/links/other"),
        stage_link.as_path(),
        stage_root.as_path()
    ));
    remove(root.as_path());
}
