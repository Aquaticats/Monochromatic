//! Controls for the recovery entry point.

use super::*;
use crate::capture_records::{LandedCaptureRecord, encode_landed_capture_record};
use crate::recovery_landing::tests::Scene;
use crate::test_support::git_context;

/// The identity of a scene's main worktree.
fn main_identity(scene: &Scene) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: scene.git_dir.clone(),
        git_dir: scene.git_dir.clone(),
        worktree_root: scene.repo.clone(),
    };
}

/// Only worktrees hold transactions.
#[test]
fn only_worktrees_are_recovered() {
    assert_eq!(
        recovery_directories(&WorktreeIdentity::OutsideWorktree),
        None
    );
    assert_eq!(
        recovery_directories(&WorktreeIdentity::BareRepository {
            common_dir: PathBuf::from("/b"),
            git_dir: PathBuf::from("/b"),
        }),
        None
    );
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/c"),
        git_dir: PathBuf::from("/c/worktrees/w"),
        worktree_root: PathBuf::from("/w"),
    };
    assert_eq!(
        recovery_directories(&linked),
        Some((Path::new("/c/worktrees/w"), Path::new("/c")))
    );
    assert_eq!(
        recover_commit_transactions(
            &git_context(),
            Path::new("/"),
            &WorktreeIdentity::OutsideWorktree
        ),
        Ok(Vec::new())
    );
}

/// The legacy directory stops the command with the owner's instructions.
#[test]
fn legacy_directories_stop_with_instructions() {
    let scene: Scene = Scene::new("commit-recovery-legacy", false);
    let legacy: PathBuf = scene.git_dir.join(LEGACY_TRANSACTION_DIRECTORY_NAME);
    std::fs::create_dir(&legacy).expect("legacy");
    assert_eq!(
        recover_commit_transactions(&git_context(), scene.repo.as_path(), &main_identity(&scene)),
        Err(RecoveryStop::Legacy(legacy.clone()))
    );
    assert!(scene.directory.exists());
    assert_eq!(
        legacy_notice(legacy.as_path()),
        format!(
            "cli-git: {} holds a commit journal in a format this executable does not recover. \
             Nothing was changed. Run any git command once with the previous cli-git executable to \
             recover it, then run this command again.\n",
            legacy.display()
        )
    );
    scene.finish();
}

/// An absent or empty registry needs no further work;
///  a registry that is a file is unsafe.
#[test]
fn empty_registries_cost_one_read() {
    let scene: Scene = Scene::new("commit-recovery-empty", false);
    std::fs::remove_dir_all(&scene.directory).expect("empty registry");
    assert_eq!(registry_is_empty(scene.registry.as_path()), Ok(true));
    assert_eq!(
        recover_commit_transactions(&git_context(), scene.repo.as_path(), &main_identity(&scene)),
        Ok(Vec::new())
    );
    std::fs::remove_dir(&scene.registry).expect("absent registry");
    assert_eq!(registry_is_empty(scene.registry.as_path()), Ok(true));
    std::fs::write(&scene.registry, b"").expect("registry file");
    assert_eq!(
        registry_is_empty(scene.registry.as_path()),
        Err(RecoveryError(format!(
            "Unsafe transaction registry: {}",
            scene.registry.display()
        )))
    );
    assert!(matches!(
        recover_commit_transactions(&git_context(), scene.repo.as_path(), &main_identity(&scene)),
        Err(RecoveryStop::Failure(_))
    ));
    std::fs::remove_file(&scene.registry).expect("remove file");
    std::fs::create_dir(&scene.registry).expect("registry");
    std::fs::create_dir(scene.registry.join("landing.lock")).expect("lock only");
    assert_eq!(registry_is_empty(scene.registry.as_path()), Ok(false));
    scene.finish();
}

/// A dead transaction is recovered,
///  outcomes are reported,
///  and landed records are pruned.
#[test]
fn dead_transactions_are_recovered_and_records_pruned() {
    let scene: Scene = Scene::new("commit-recovery-dead", false);
    let directory: PathBuf =
        crate::capture_records::landed_record_directory(scene.git_dir.as_path());
    std::fs::create_dir_all(&directory).expect("landed directory");
    std::fs::write(
        directory.join("c0.json"),
        encode_landed_capture_record(&LandedCaptureRecord {
            commit: String::from("c0"),
            transaction_id: String::from("t"),
            worktree_id: String::from("w"),
            sequence: 1,
            next_sequence_after_landing: 2,
            worktree_paths: Vec::new(),
        }),
    )
    .expect("landed record");
    let outcomes: Vec<RecoveryOutcome> =
        recover_commit_transactions(&git_context(), scene.repo.as_path(), &main_identity(&scene))
            .expect("recovered");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].action, RecoveryAction::CommitNotCreated);
    assert!(has_recovered(outcomes.as_slice()));
    assert!(!leaves_live_transactions(outcomes.as_slice()));
    assert!(!directory.join("c0.json").exists());
    assert_eq!(
        outcomes_json(outcomes.as_slice()),
        format!(
            "[{{\"directory\":{},\"action\":\"commit-not-created\"}}]",
            crate::json_record::quote(scene.directory.to_str().expect("UTF-8"))
        )
    );
    assert_eq!(outcomes_json(&[]), "[]");
    // A failing transaction stops with the failure.
    std::fs::create_dir(&scene.directory).expect("published again");
    assert!(matches!(
        recover_commit_transactions(&git_context(), scene.repo.as_path(), &main_identity(&scene)),
        Err(RecoveryStop::Failure(_))
    ));
    scene.finish();
}

/// A live owner is skipped,
///  counts as live,
///  and pins its landed records.
#[test]
fn live_transactions_are_skipped() {
    let scene: Scene = Scene::new("commit-recovery-live", true);
    let outcomes: Vec<RecoveryOutcome> =
        recover_commit_transactions(&git_context(), scene.repo.as_path(), &main_identity(&scene))
            .expect("skipped");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].action, RecoveryAction::OwnerActive);
    assert!(!has_recovered(outcomes.as_slice()));
    assert!(leaves_live_transactions(outcomes.as_slice()));
    assert!(scene.directory.exists());
    let unattributed: Vec<RecoveryOutcome> = vec![RecoveryOutcome {
        directory: PathBuf::from("/x"),
        action: RecoveryAction::StagingUnattributed,
    }];
    assert!(leaves_live_transactions(unattributed.as_slice()));
    let retained: Vec<RecoveryOutcome> = vec![RecoveryOutcome {
        directory: PathBuf::from("/x"),
        action: RecoveryAction::StagingRetained,
    }];
    assert!(!leaves_live_transactions(retained.as_slice()));
    scene.finish();
}
