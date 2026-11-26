//! Controls for the fill's own decisions:
//!  whether its restore still has to run,
//!  and the trash reading:
//!  no trash is none,
//!  one trash with its own record is the fill's,
//!  a trash without one refuses,
//!  and two trashes refuse.

use super::{
    the_restore_still_runs,
    the_trash_of_a_filled_clone,
};
use crate::fs_beneath::identity_at;
use crate::test_support::{
    fixture,
    remove,
};
use crate::worktree_trash::{
    TRASH_PARENT,
    TrashManifest,
    is_trash_name,
};
use crate::worktree_trash_own_manifest;
use std::path::PathBuf;

/// What:
///  The restore runs while the private index stands and stops as soon as the
///  publication took that name away.
/// Why:
///  A restore pointed at a private name that the publication removed would read
///  an empty index, so the state the publication leaves must be told from the
///  one before it.
///  The exchange is past the restore for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theRestoreStillRuns(admin, clone, false)).toBe(true); // index.pending stands
/// expect(theRestoreStillRuns(admin, clone, false)).toBe(false); // index published
/// ```
#[test]
fn the_restore_stops_once_the_publication_took_the_private_name() {
    let root: PathBuf = fixture("fill-restore-guard");
    let admin: PathBuf = root.join("registration");
    let clone: PathBuf = root.join("clone");
    std::fs::create_dir_all(admin.as_path()).expect("the registration");
    std::fs::create_dir_all(clone.join(".git")).expect("the copied administration");
    // Before the fill writes anything: the copied `.git` stands and no index does.
    assert!(
        the_restore_still_runs(admin.as_path(), clone.as_path(), false),
        "the copied administration is the state before the private index"
    );
    std::fs::write(
        admin.join(crate::worktree_clone_add_route::add_index::PENDING_INDEX_NAME),
        b"index",
    )
    .expect("the private index");
    assert!(
        the_restore_still_runs(admin.as_path(), clone.as_path(), false),
        "the private name stands, so the restore runs"
    );
    std::fs::rename(
        admin.join(crate::worktree_clone_add_route::add_index::PENDING_INDEX_NAME),
        admin.join(crate::worktree_clone_add_route::add_index::INDEX_NAME),
    )
    .expect("the publication");
    assert!(
        !the_restore_still_runs(admin.as_path(), clone.as_path(), false),
        "the publication removed the private name, so the restore is behind the fill"
    );
    assert!(
        !the_restore_still_runs(admin.as_path(), clone.as_path(), true),
        "an add resumed after the exchange is past the restore"
    );
    remove(root.as_path());
}

/// A clone root with a trash place.
fn a_clone_root(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    std::fs::create_dir_all(root.join(TRASH_PARENT)).expect("the trash place");
    return root;
}

/// Make one trash below the root and answer its name.
fn make_a_trash(
    root: &std::path::Path,
    fill: u8,
) -> String {
    let name: String = std::iter::repeat_n(format!("{fill:02x}"), 20).collect();
    assert!(is_trash_name(name.as_str()));
    let folder: PathBuf = root.join(TRASH_PARENT).join(name.as_str());
    std::fs::create_dir(folder.as_path()).expect("the trash");
    let parent: std::fs::File =
        std::fs::File::open(root.join(TRASH_PARENT)).expect("the parent opens");
    let identity = identity_at(&parent, std::ffi::OsStr::new(name.as_str())).expect("the identity");
    let manifest: TrashManifest = TrashManifest {
        name: name.clone(),
        trash: identity,
        entries: Vec::new(),
    };
    worktree_trash_own_manifest::write(root, &manifest).expect("the manifest");
    return name;
}

#[test]
fn a_clone_with_no_trash_answers_none() {
    let root: PathBuf = a_clone_root("fill-no-trash");
    assert_eq!(
        the_trash_of_a_filled_clone(root.as_path()).expect("no trash is no refusal"),
        None
    );
    remove(root.as_path());
}

#[test]
fn a_trash_with_its_own_record_is_the_fill_s() {
    let root: PathBuf = a_clone_root("fill-one-trash");
    let name: String = make_a_trash(root.as_path(), 0xab);
    let found: TrashManifest = the_trash_of_a_filled_clone(root.as_path())
        .expect("the record reads")
        .expect("one trash stands");
    assert_eq!(found.name, name);
    remove(root.as_path());
}

#[test]
fn a_trash_without_its_record_and_two_trashes_refuse() {
    let root: PathBuf = a_clone_root("fill-bad-trash");
    let name: String = std::iter::repeat_n(String::from("cd"), 20).collect();
    std::fs::create_dir(root.join(TRASH_PARENT).join(name.as_str()))
        .expect("a trash with no record");
    let refusal: String = the_trash_of_a_filled_clone(root.as_path())
        .expect_err("a trash with no record proves nothing");
    assert!(
        refusal.contains("holds no record this add wrote"),
        "{refusal}"
    );
    std::fs::remove_dir_all(root.join(TRASH_PARENT).join(name.as_str()))
        .expect("the recordless trash goes");
    make_a_trash(root.as_path(), 0x11);
    make_a_trash(root.as_path(), 0x22);
    let two: String =
        the_trash_of_a_filled_clone(root.as_path()).expect_err("two trashes are two add records");
    assert!(two.contains("more than one trash"), "{two}");
    remove(root.as_path());
}
