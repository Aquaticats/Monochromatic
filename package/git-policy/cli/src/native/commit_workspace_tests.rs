//! Controls for the commit workspace: publication with an owner record, and disposal that honors recovery.

use super::*;
use crate::private_storage::take_directory_syncs;
use crate::test_support::{fixture, remove};
use crate::transaction_journal::RefFormat;
use std::path::Path;

/// A layout whose registry and shadow area live under `root`.
fn layout(root: &Path) -> InvocationLayout {
    return InvocationLayout {
        symbolic_head: crate::transaction_journal::SymbolicHead::Branch(String::from("refs/heads/main")),
        target_ref: String::from("refs/heads/main"),
        conclusion: crate::transaction_journal::Conclusion::None,
        repository_root: root.to_path_buf(),
        git_dir: root.join(".git"),
        common_dir: root.join(".git"),
        real_index_path: root.join(".git/index"),
        object_directory: root.join(".git/objects"),
        registry_root: root.join(".git/cli-git-transactions"),
        ref_format: RefFormat::Files,
        object_format: String::from("sha1"),
        empty_tree_oid: "4".repeat(40),
        invoked_at: String::from("2026-10-11T00:00:00.000Z"),
    };
}

/// Publication creates the owned directory with its owner record and derives every path from the transaction ID.
#[test]
fn publication_creates_an_owned_directory_and_derives_the_paths() {
    let root: PathBuf = fixture("workspace-publish");
    std::fs::create_dir_all(root.join(".git")).expect("git dir");
    let mut workspace: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("workspace");
    let id: &str = workspace.landing.transaction_id.as_str();
    assert_eq!(workspace.landing.directory, root.join(".git/cli-git-transactions").join(id));
    assert!(workspace.landing.directory.join("owner.json").exists());
    assert_eq!(workspace.landing.commit_index, workspace.landing.directory.join("commit.index"));
    assert_eq!(workspace.landing.captured_index, workspace.landing.directory.join("captured.index"));
    assert_eq!(workspace.landing.shadow, root.join(".git/cli-git/shadow").join(id));
    assert_eq!(workspace.object_directory, workspace.landing.shadow.join("objects"));
    assert_eq!(workspace.hooks_directory, workspace.landing.directory.join("hooks"));
    assert!(!workspace.landing.preserved);
    assert_eq!(workspace.git_dir(), root.join(".git"));
    assert_eq!(workspace.registry_root(), root.join(".git/cli-git-transactions"));
    workspace.dispose().expect("dispose");
    remove(root.as_path());
}

/// Two workspaces never share an ID or a directory.
#[test]
fn every_workspace_has_its_own_identity() {
    let root: PathBuf = fixture("workspace-unique");
    std::fs::create_dir_all(root.join(".git")).expect("git dir");
    let first: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("first");
    let second: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("second");
    assert_ne!(first.landing.transaction_id, second.landing.transaction_id);
    assert_ne!(first.landing.directory, second.landing.directory);
    remove(root.as_path());
}

/// The Git directory is synced once, by the workspace that creates the registry, and not by a workspace that finds it.
#[test]
fn only_the_workspace_that_creates_the_registry_syncs_the_git_directory() {
    let root: PathBuf = fixture("workspace-registry-sync");
    let git_dir: PathBuf = root.join(".git");
    std::fs::create_dir_all(&git_dir).expect("git dir");
    take_directory_syncs();
    let first: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("first");
    let creating: Vec<PathBuf> = take_directory_syncs();
    assert_eq!(creating.iter().filter(|synced| return **synced == git_dir).count(), 1, "{creating:?}");
    let second: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("second");
    let finding: Vec<PathBuf> = take_directory_syncs();
    assert!(!finding.contains(&git_dir), "{finding:?}");
    // Publication still syncs the candidate and the registry for each workspace.
    assert!(finding.contains(&git_dir.join("cli-git-transactions")), "{finding:?}");
    drop((first, second));
    remove(root.as_path());
}

/// Disposal removes the shadow and the directory; a preserved workspace keeps both for recovery; later calls do nothing.
#[test]
fn disposal_removes_unless_a_landing_record_handed_the_directory_to_recovery() {
    let root: PathBuf = fixture("workspace-dispose");
    std::fs::create_dir_all(root.join(".git")).expect("git dir");
    let mut removed: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("removed");
    std::fs::create_dir_all(removed.landing.shadow.as_path()).expect("shadow");
    removed.dispose().expect("dispose");
    assert!(!removed.landing.directory.exists());
    assert!(!removed.landing.shadow.exists());
    removed.dispose().expect("a later call does nothing");

    let mut kept: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("kept");
    std::fs::create_dir_all(kept.landing.shadow.as_path()).expect("shadow");
    kept.landing.preserved = true;
    kept.dispose().expect("dispose");
    assert!(kept.landing.directory.exists());
    assert!(kept.landing.shadow.exists());

    let mut finished: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("finished");
    finished.landing.preserved = true;
    finished.finish_transaction();
    assert!(!finished.landing.preserved);
    finished.dispose().expect("dispose");
    assert!(!finished.landing.directory.exists());

    let mut handed: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("handed");
    handed.mark_disposed();
    handed.dispose().expect("dispose");
    assert!(handed.landing.directory.exists(), "a workspace marked disposed removes nothing again");
    // Dropping a live workspace removes it.
    let directory: PathBuf = {
        let dropped: CommitWorkspace = create_commit_workspace(&layout(root.as_path())).expect("dropped");
        dropped.landing.directory.clone()
    };
    assert!(!directory.exists());
    remove(root.as_path());
}

/// Each failing step keeps its own words in the message.
#[test]
fn each_failure_displays_its_own_words() {
    assert_eq!(
        WorkspaceError::Registry(crate::recovery_error::RecoveryError(String::from("registry words"))).to_string(),
        "registry words"
    );
    assert_eq!(WorkspaceError::Random(String::from("random words")).to_string(), "random words");
    let identity: String = WorkspaceError::Identity(crate::process_identity::ProcessIdentityError::MalformedStat { pid: 7 }).to_string();
    assert!(identity.contains("MalformedStat") && identity.contains('7'), "{identity}");
}
