//! Controls for landed-transaction disposal: the removal order, idempotence, and pruning after removal.

use super::*;
use crate::capture_records::{
    CAPTURED_FILENAME, CapturedRecord, encode_captured_record, landed_record_directory,
    record_landed_capture,
};
use crate::recovery_landing::tests::{ID, Scene};

/// Landed commit the pruning controls record.
const LANDED: &str = "0123456789abcdef0123456789abcdef01234567";

/// Paths of a scene's transaction, in the form disposal takes.
fn paths(scene: &Scene) -> LandedTransactionPaths<'_> {
    return LandedTransactionPaths {
        shadow: scene.shadow.as_path(),
        directory: scene.directory.as_path(),
        git_dir: scene.git_dir.as_path(),
        registry_root: scene.registry.as_path(),
    };
}

/// Writes the captured record a landed transaction must have to be recorded.
fn write_captured(scene: &Scene) {
    std::fs::write(
        scene.directory.join(CAPTURED_FILENAME),
        encode_captured_record(&CapturedRecord {
            worktree_id: String::from("w"),
            sequence: 1,
            next_sequence_before_base: 1,
            worktree_paths: vec![String::from("added.txt")],
        }),
    )
    .expect("captured record");
}

/// Disposal removes the shadow repository and the transaction directory, and reports no pruned record.
#[test]
fn disposal_removes_the_shadow_and_the_directory() {
    let scene: Scene = Scene::new("landed-disposal-removes", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let pruned: usize = dispose_landed_transaction(paths(&scene)).expect("disposed");
    assert_eq!(pruned, 0);
    assert!(!scene.shadow.exists());
    assert!(!scene.directory.exists());
    scene.finish();
}

/// Disposal syncs no directory: removal needs no durability because recovery finishes what a power loss leaves.
#[test]
fn disposal_syncs_no_directory() {
    let scene: Scene = Scene::new("landed-disposal-sync", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    crate::private_storage::take_directory_syncs();
    dispose_landed_transaction(paths(&scene)).expect("disposed");
    assert_eq!(crate::private_storage::take_directory_syncs(), Vec::<std::path::PathBuf>::new());
    scene.finish();
}

/// A shadow an earlier run already removed is not an error, so a recovery can repeat disposal.
#[test]
fn disposal_is_idempotent_when_the_shadow_is_already_gone() {
    let scene: Scene = Scene::new("landed-disposal-idempotent", false);
    assert!(!scene.shadow.exists());
    dispose_landed_transaction(paths(&scene)).expect("first disposal");
    assert!(!scene.directory.exists());
    scene.finish();
}

/// Pruning runs after the directory is gone, so the landed record its transaction pinned is removed.
#[test]
fn pruning_runs_after_the_directory_is_removed() {
    let scene: Scene = Scene::new("landed-disposal-prunes", false);
    write_captured(&scene);
    let record: std::path::PathBuf =
        landed_record_directory(scene.git_dir.as_path()).join(format!("{LANDED}.json"));
    assert!(
        record_landed_capture(
            scene.git_dir.as_path(),
            scene.directory.as_path(),
            ID,
            LANDED,
        )
        .expect("recorded")
    );
    assert!(record.exists());
    let pruned: usize = dispose_landed_transaction(paths(&scene)).expect("disposed");
    assert_eq!(pruned, 1);
    assert!(!record.exists());
    scene.finish();
}

/// A shadow that cannot be removed fails disposal before the directory or any record is touched,
/// so recovery still finds everything it needs.
#[cfg(unix)]
#[test]
fn a_failed_shadow_removal_keeps_the_directory_and_the_records() {
    use std::os::unix::fs::PermissionsExt;
    let scene: Scene = Scene::new("landed-disposal-keeps", false);
    write_captured(&scene);
    assert!(
        record_landed_capture(
            scene.git_dir.as_path(),
            scene.directory.as_path(),
            ID,
            LANDED,
        )
        .expect("recorded")
    );
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let parent: &std::path::Path = scene.shadow.parent().expect("shadow parent");
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o555))
        .expect("read-only parent");
    let failed: bool = dispose_landed_transaction(paths(&scene)).is_err();
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755))
        .expect("restored parent");
    assert!(failed);
    assert!(scene.directory.exists());
    assert!(
        landed_record_directory(scene.git_dir.as_path())
            .join(format!("{LANDED}.json"))
            .exists()
    );
    scene.finish();
}

/// A directory that cannot be removed fails disposal after the shadow went, and no landed record is pruned,
/// so a recovery that finds the directory still has the records it needs.
#[cfg(unix)]
#[test]
fn a_failed_directory_removal_keeps_the_records() {
    use std::os::unix::fs::PermissionsExt;
    let scene: Scene = Scene::new("landed-disposal-directory-fails", false);
    write_captured(&scene);
    assert!(
        record_landed_capture(
            scene.git_dir.as_path(),
            scene.directory.as_path(),
            ID,
            LANDED,
        )
        .expect("recorded")
    );
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    std::fs::set_permissions(
        scene.registry.as_path(),
        std::fs::Permissions::from_mode(0o555),
    )
    .expect("read-only registry");
    let failed: bool = dispose_landed_transaction(paths(&scene)).is_err();
    std::fs::set_permissions(
        scene.registry.as_path(),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("restored registry");
    assert!(failed);
    assert!(
        !scene.shadow.exists(),
        "the shadow goes before the directory"
    );
    assert!(scene.directory.exists());
    assert!(
        landed_record_directory(scene.git_dir.as_path())
            .join(format!("{LANDED}.json"))
            .exists()
    );
    scene.finish();
}
