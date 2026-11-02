//! Controls for the startup step of a command.

use super::*;
use crate::test_support::{fixture, git, git_context, remove};

/// A canonical transaction name an earlier release gave its commit directories.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// The identity of the main worktree of the repository at `root`.
fn identity(root: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: root.join(".git"),
        git_dir: root.join(".git"),
        worktree_root: root.to_path_buf(),
    };
}

/// A repository with nothing left by an earlier release goes on, and so does a command outside a worktree.
#[test]
fn a_clean_repository_and_a_place_outside_a_worktree_go_on() {
    let root: PathBuf = fixture("startup-gate-clean");
    git(&root, &["init", "--quiet"]);
    assert_eq!(recover_for_command(&git_context(), &identity(root.as_path())), Gate::Clear);
    assert_eq!(recover_for_command(&git_context(), &WorktreeIdentity::OutsideWorktree), Gate::Clear);
    remove(root.as_path());
}

/// The single-journal directory of the previous executable stops the command with the way to recover it, and nothing is touched.
#[test]
fn the_previous_executables_journal_directory_stops_the_command() {
    let root: PathBuf = fixture("startup-gate-legacy");
    git(&root, &["init", "--quiet"]);
    let legacy: PathBuf = root.join(".git").join(LEGACY_JOURNAL_DIRECTORY_NAME);
    std::fs::create_dir(&legacy).expect("legacy directory");
    assert_eq!(
        recover_for_command(&git_context(), &identity(root.as_path())),
        Gate::Stop(legacy_notice(legacy.as_path()))
    );
    assert!(legacy.is_dir(), "nothing was removed");
    remove(root.as_path());
}

/// A registry that holds only locks and staging names has nothing to look at; a transaction directory in it is named.
#[test]
fn a_registry_note_appears_only_for_a_transaction_directory() {
    let root: PathBuf = fixture("startup-gate-registry");
    assert_eq!(registry_note(root.join("absent").as_path()), None, "no registry, no note");
    let registry: PathBuf = root.join(REGISTRY_DIRECTORY_NAME);
    std::fs::create_dir_all(registry.join("landing.lock")).expect("lock directory");
    std::fs::create_dir_all(registry.join(format!("{TRANSACTION}.pending"))).expect("staging directory");
    assert_eq!(registry_note(registry.as_path()), None, "locks and staging names are not commits");
    std::fs::create_dir_all(registry.join(TRANSACTION)).expect("transaction directory");
    let note: String = registry_note(registry.as_path()).expect("a note");
    assert!(note.contains(registry.to_string_lossy().as_ref()), "{note}");
    assert!(note.contains("git status"), "the way to see whether the commit was made: {note}");
    assert!(registry.join(TRANSACTION).is_dir(), "nothing was removed");
    remove(root.as_path());
}

/// A transaction directory an earlier release left does not stop the command: the note goes to standard error and the command goes on.
#[test]
fn a_registry_of_an_earlier_release_does_not_stop_the_command() {
    let root: PathBuf = fixture("startup-gate-registry-goes-on");
    git(&root, &["init", "--quiet"]);
    std::fs::create_dir_all(root.join(".git").join(REGISTRY_DIRECTORY_NAME).join(TRANSACTION)).expect("transaction directory");
    assert_eq!(recover_for_command(&git_context(), &identity(root.as_path())), Gate::Clear);
    remove(root.as_path());
}
