//! Controls for recovery as a command step.

use super::*;
use crate::recovery_landing::tests::Scene;
use crate::test_support::git_context;

/// The identity of a scene's main worktree.
fn identity(scene: &Scene) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: scene.git_dir.clone(),
        git_dir: scene.git_dir.clone(),
        worktree_root: scene.repo.clone(),
    };
}

/// Live transactions,
///  recovered ones,
///  legacy directories and failures each give their gate.
#[test]
fn gates_follow_recovery() {
    let live: Scene = Scene::new("transaction-gate-live", true);
    assert_eq!(
        recover_for_command(&git_context(), &identity(&live)),
        Gate::Live(live.registry.clone())
    );
    live.finish();
    let dead: Scene = Scene::new("transaction-gate-dead", false);
    assert_eq!(
        recover_for_command(&git_context(), &identity(&dead)),
        Gate::Clear
    );
    assert!(!dead.directory.exists());
    assert_eq!(
        recover_for_command(&git_context(), &WorktreeIdentity::OutsideWorktree),
        Gate::Clear
    );
    std::fs::create_dir(dead.registry.join("stray")).expect("stray");
    let Gate::Stop(stderr) = recover_for_command(&git_context(), &identity(&dead)) else {
        panic!("an unexpected entry stops the command");
    };
    assert_eq!(
        stderr,
        render_engine_failure_with_hint(
            0,
            EngineFailureCode::ContentUnavailable,
            format!(
                "Unexpected transaction registry entry: {}",
                dead.registry.join("stray").display()
            )
            .as_str(),
            None
        )
    );
    // The code's generic hint (fetch history) does not apply to transaction state, so none is printed.
    assert!(!stderr.contains("\"hint\""), "{stderr}");
    assert!(!stderr.contains("git fetch"), "{stderr}");
    std::fs::remove_dir(dead.registry.join("stray")).expect("remove stray");
    let legacy: PathBuf = dead.git_dir.join("cli-git-transaction");
    std::fs::create_dir(&legacy).expect("legacy");
    assert_eq!(
        recover_for_command(&git_context(), &identity(&dead)),
        Gate::Stop(legacy_notice(legacy.as_path()))
    );
    dead.finish();
}
