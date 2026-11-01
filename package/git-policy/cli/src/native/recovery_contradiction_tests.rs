//! Controls for the refusal to discard a transaction whose branch moved without a usable landing record.

use super::*;
use crate::recovery_landing::tests::{ID, Scene};
use crate::recovery_landing::{RecoveryAction, recover_dead_transaction};
use crate::test_support::git_text;
use crate::transaction_journal::{LandingOperation, PREPARING_FILENAME};
use crate::transaction_journal_encode::{encode_preparing, encode_ref_updated};

/// The owner process ID recovery is told died.
fn dead_owner() -> i64 {
    return i64::from(std::process::id());
}

/// A landed transaction as the harness left it at the `ref-updated` phase: capture, locked index, landing record,
///  moved branch and marker.
fn landed(name: &str) -> Scene {
    let scene: Scene = Scene::new(name, false);
    scene.capture();
    scene.land(LandingOperation::Commit, Vec::new());
    scene.move_branch();
    std::fs::write(scene.directory.join(REF_UPDATED_FILENAME), encode_ref_updated(scene.prepared.as_str())).expect("marker");
    return scene;
}

/// A branch that moved and a directory whose landing record is gone is refused with the evidence, the directory and
///  the steps named, and nothing is removed.
#[test]
fn a_moved_branch_without_its_landing_record_is_refused() {
    let scene: Scene = landed("contradiction-landing-deleted");
    std::fs::remove_file(scene.directory.join("landing-1.json")).expect("delete the landing record");
    let message: String = recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, dead_owner())
        .expect_err("refused")
        .0;
    assert!(
        message.starts_with(&format!(
            "Transaction directory {} has {REF_UPDATED_FILENAME} but no landing-<n>.json, so the commit of this transaction \
             is already on refs/heads/main",
            scene.directory.display()
        )),
        "{message}"
    );
    assert!(message.contains("run `git reset`"), "{message}");
    assert!(message.contains(&format!("Recovery retained the transaction at {}", scene.directory.display())), "{message}");
    assert!(scene.directory.join(REF_UPDATED_FILENAME).exists());
    assert!(scene.git_dir.join("index.lock").exists());
    assert_eq!(git_text(scene.repo.as_path(), &["rev-parse", "HEAD"]), scene.prepared);
    scene.finish();
}

/// With the marker lost together with the landing record, the transaction's own reflog entry is the evidence.
#[test]
fn a_reflog_entry_without_any_landing_file_is_refused() {
    let scene: Scene = landed("contradiction-reflog");
    std::fs::remove_file(scene.directory.join("landing-1.json")).expect("delete the landing record");
    std::fs::remove_file(scene.directory.join(REF_UPDATED_FILENAME)).expect("delete the marker");
    let message: String = recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, dead_owner())
        .expect_err("refused")
        .0;
    assert!(
        message.starts_with(&format!("The reflog of refs/heads/main has an entry of transaction {ID} naming {}", scene.prepared)),
        "{message}"
    );
    assert!(message.contains(&scene.directory.display().to_string()), "{message}");
    assert!(scene.directory.exists());
    scene.finish();
}

/// A directory with later records and no `preparing.json` is refused, naming what it holds.
#[test]
fn later_records_without_the_capture_record_are_refused() {
    let scene: Scene = landed("contradiction-preparing-deleted");
    std::fs::remove_file(scene.directory.join(PREPARING_FILENAME)).expect("delete the capture record");
    let message: String = recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, dead_owner())
        .expect_err("refused")
        .0;
    assert!(
        message.starts_with(&format!(
            "Transaction directory {} has landing-1.json and {REF_UPDATED_FILENAME} but no {PREPARING_FILENAME}.",
            scene.directory.display()
        )),
        "{message}"
    );
    assert!(message.contains("any index.lock file in the repository's Git directory"), "{message}");
    assert!(scene.directory.exists());
    scene.finish();
}

/// Neither a finished install nor an attempt that never moved its branch is refused: both are discarded as before.
#[test]
fn installed_and_unmoved_transactions_are_discarded_as_before() {
    let installed: Scene = landed("contradiction-installed");
    std::fs::remove_file(installed.directory.join("landing-1.json")).expect("delete the landing record");
    std::fs::write(installed.directory.join(INDEX_INSTALLED_FILENAME), b"").expect("install marker");
    assert_eq!(
        recover_dead_transaction(&installed.place(), installed.directory.as_path(), ID, dead_owner()).expect("discarded"),
        RecoveryAction::CommitNotCreated
    );
    assert!(!installed.directory.exists());
    installed.finish();
    // The target never moved, so its reflog holds no entry of the transaction.
    let unmoved: Scene = Scene::new("contradiction-unmoved", false);
    unmoved.capture();
    unmoved.land(LandingOperation::Commit, Vec::new());
    std::fs::remove_file(unmoved.directory.join("landing-1.json")).expect("delete the landing record");
    assert_eq!(
        recover_dead_transaction(&unmoved.place(), unmoved.directory.as_path(), ID, dead_owner()).expect("discarded"),
        RecoveryAction::CommitNotCreated
    );
    assert!(!unmoved.directory.exists());
    unmoved.finish();
    // A target whose reflog cannot be read is no evidence; the transaction is treated as before.
    let unreadable: Scene = Scene::new("contradiction-unreadable", false);
    let mut preparing: PreparingRecord = unreadable.preparing();
    preparing.target_ref = String::from("refs/heads/does-not-exist");
    std::fs::write(unreadable.directory.join(PREPARING_FILENAME), encode_preparing(&preparing)).expect("preparing");
    assert_eq!(
        recover_dead_transaction(&unreadable.place(), unreadable.directory.as_path(), ID, dead_owner()).expect("discarded"),
        RecoveryAction::CommitNotCreated
    );
    unreadable.finish();
}
