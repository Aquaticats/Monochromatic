//! Consumer checks of linked-worktree registration observation against real Git.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// A bare repository with no worktree.
fn bare(parent: &Path, name: &str) -> PathBuf {
    let root: PathBuf = parent.join(name);
    git(parent, &["init", "--quiet", "--bare", name]);
    return std::fs::canonicalize(root).expect("canonical bare repository");
}

/// The common directory of a repository whose `.git` is a directory.
fn common_dir_of(root: &Path) -> PathBuf {
    return std::fs::canonicalize(root.join(".git")).expect("canonical common directory");
}

/// A linked worktree's identity as the native identity query reports it.
fn linked(common_dir: &Path, worktree_root: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::LinkedWorktree {
        common_dir: common_dir.to_path_buf(),
        git_dir: common_dir.join("worktrees").join("unused"),
        worktree_root: worktree_root.to_path_buf(),
    };
}

/// A non-directory path has no administrative listing and fails the inspection.
#[test]
fn admin_listing_of_a_file_fails_with_its_path() {
    let root: PathBuf = fixture("registry-admin-file");
    let file: PathBuf = root.join("worktrees");
    std::fs::write(file.as_path(), b"not a directory").expect("file");
    let failure: WorktreeCopyError = read_admin_ids(file.as_path()).expect_err("file listing");
    assert!(
        failure
            .0
            .starts_with("cli-git: could not inspect linked-worktree administration at ")
    );
    assert!(failure.0.contains(" Cause: "));
    remove(root.as_path());
}

/// A missing administration directory is an empty identity set.
#[test]
fn missing_admin_directory_is_empty() {
    let root: PathBuf = fixture("registry-admin-missing");
    let ids: BTreeSet<String> = read_admin_ids(root.join("worktrees").as_path()).expect("empty");
    assert!(ids.is_empty());
    remove(root.as_path());
}

/// Only directories count as identities; files and symbolic links to directories do not.
#[test]
fn admin_listing_keeps_only_real_directories() {
    let root: PathBuf = fixture("registry-admin-kinds");
    let admin: PathBuf = root.join("worktrees");
    std::fs::create_dir_all(admin.join("topic")).expect("directory");
    std::fs::write(admin.join("stray-file"), b"x").expect("file");
    #[cfg(unix)]
    std::os::unix::fs::symlink(admin.join("topic"), admin.join("linked")).expect("symlink");
    let ids: BTreeSet<String> = read_admin_ids(admin.as_path()).expect("listing");
    assert_eq!(ids, BTreeSet::from([String::from("topic")]));
    remove(root.as_path());
}

/// Outside a repository and in the main worktree nothing is observed.
#[test]
fn outside_and_main_worktrees_are_not_applicable() {
    let root: PathBuf = fixture("registry-observe-main");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let main_identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: common.clone(),
        git_dir: common,
        worktree_root: main.clone(),
    };
    assert_eq!(
        observe_worktree_repository(&WorktreeIdentity::OutsideWorktree, root.as_path()),
        Ok(None)
    );
    assert_eq!(
        observe_worktree_repository(&main_identity, main.as_path()),
        Ok(None)
    );
    remove(root.as_path());
}

/// A main worktree is observed only for named paths: it has the main root as source and selects just those paths.
#[test]
fn main_worktree_is_observed_for_named_paths_only() {
    let root: PathBuf = fixture("registry-observe-main-paths");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let main_identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: common.clone(),
        git_dir: common.clone(),
        worktree_root: main.clone(),
    };
    assert_eq!(
        observe_copy_source(&main_identity, main.as_path(), &[]),
        Ok(None)
    );
    let named: Vec<String> = vec![String::from(".cache/rules.txt")];
    let observation: WorktreeCopyObservation =
        observe_copy_source(&main_identity, main.as_path(), named.as_slice())
            .expect("observation")
            .expect("applicable");
    assert_eq!(observation.source_root, Some(main.clone()));
    assert_eq!(observation.common_dir, common);
    assert_eq!(observation.selection, CopySelection::Paths(named));
    remove(root.as_path());
}

/// Linked and bare repositories keep the standard selection, and the named paths never narrow them.
#[test]
fn linked_and_bare_observations_ignore_named_paths() {
    let root: PathBuf = fixture("registry-observe-paths-ignored");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let topic: PathBuf = root.join("topic");
    git(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")],
    );
    let topic_root: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical topic");
    let named: Vec<String> = vec![String::from(".cache/rules.txt")];
    let linked_observation: WorktreeCopyObservation = observe_copy_source(
        &linked(common.as_path(), topic_root.as_path()),
        topic_root.as_path(),
        named.as_slice(),
    )
    .expect("observation")
    .expect("applicable");
    assert_eq!(linked_observation.selection, CopySelection::Ignored);
    let bare_root: PathBuf = bare(root.as_path(), "shared.git");
    let bare_observation: WorktreeCopyObservation = observe_copy_source(
        &WorktreeIdentity::BareRepository {
            common_dir: bare_root.clone(),
            git_dir: bare_root.clone(),
        },
        bare_root.as_path(),
        named.as_slice(),
    )
    .expect("observation")
    .expect("applicable");
    assert_eq!(bare_observation.selection, CopySelection::Ignored);
    assert_eq!(bare_observation.source_root, None);
    remove(root.as_path());
}

/// A bare repository is observed without a source root, and a linked worktree is observed with one.
#[test]
fn bare_and_linked_repositories_are_observed() {
    let root: PathBuf = fixture("registry-observe-kinds");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let topic: PathBuf = root.join("topic");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    let topic_root: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical topic");
    let linked_observation: WorktreeCopyObservation = observe_worktree_repository(
        &linked(common.as_path(), topic_root.as_path()),
        topic_root.as_path(),
    )
    .expect("observation")
    .expect("applicable");
    assert_eq!(linked_observation.source_root, Some(topic_root.clone()));
    assert_eq!(linked_observation.admin_root, common.join("worktrees"));
    assert_eq!(linked_observation.common_dir, common);
    assert_eq!(
        linked_observation.before_admin_ids,
        BTreeSet::from([String::from("topic")])
    );
    let bare_root: PathBuf = bare(root.as_path(), "shared.git");
    let bare_identity: WorktreeIdentity = WorktreeIdentity::BareRepository {
        common_dir: bare_root.clone(),
        git_dir: bare_root.clone(),
    };
    let bare_observation: WorktreeCopyObservation =
        observe_worktree_repository(&bare_identity, bare_root.as_path())
            .expect("observation")
            .expect("applicable");
    assert_eq!(bare_observation.source_root, None);
    assert!(bare_observation.before_admin_ids.is_empty());
    remove(root.as_path());
}

/// A linked-worktree `.git` file is resolved to the basename of its administrative directory,
/// whether that path is absolute or relative to the worktree root.
#[test]
fn linked_pointer_names_its_administrative_directory() {
    let root: PathBuf = fixture("registry-pointer");
    let main: PathBuf = repository(root.as_path(), "main");
    let topic: PathBuf = root.join("topic");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    assert_eq!(
        read_linked_admin_id(topic.as_path()).expect("absolute pointer"),
        Some(String::from("topic"))
    );
    let relative: &[u8] = b"gitdir: ../main/.git/worktrees/topic\n";
    std::fs::write(topic.join(".git"), relative).expect("relative pointer");
    assert_eq!(
        read_linked_admin_id(topic.as_path()).expect("relative pointer"),
        Some(String::from("topic"))
    );
    remove(root.as_path());
}

/// A main worktree's `.git` directory and a `.git` file without the gitdir prefix are the main worktree.
#[test]
fn main_worktree_and_foreign_pointer_have_no_linked_identity() {
    let root: PathBuf = fixture("registry-pointer-main");
    let main: PathBuf = repository(root.as_path(), "main");
    assert_eq!(read_linked_admin_id(main.as_path()).expect("main"), None);
    let other: PathBuf = root.join("other");
    std::fs::create_dir(other.as_path()).expect("other");
    std::fs::write(other.join(".git"), b"something else\n").expect("foreign pointer");
    assert_eq!(
        read_linked_admin_id(other.as_path()).expect("foreign"),
        None
    );
    remove(root.as_path());
}

/// A pointer to a missing administrative directory and a root without `.git` both fail with the root named.
#[test]
fn unresolvable_pointer_fails_with_its_root() {
    let root: PathBuf = fixture("registry-pointer-broken");
    let dangling: PathBuf = root.join("dangling");
    std::fs::create_dir(dangling.as_path()).expect("dangling");
    std::fs::write(dangling.join(".git"), b"gitdir: /nonexistent/admin\n").expect("pointer");
    let failure: WorktreeCopyError =
        read_linked_admin_id(dangling.as_path()).expect_err("dangling");
    assert!(
        failure
            .0
            .starts_with("cli-git: could not resolve worktree identity for ")
    );
    let bare_root: PathBuf = root.join("no-dot-git");
    std::fs::create_dir(bare_root.as_path()).expect("no dot git");
    assert!(read_linked_admin_id(bare_root.as_path()).is_err());
    remove(root.as_path());
}

/// No registration after Git returned means nothing is created.
#[test]
fn unchanged_administration_creates_nothing() {
    let root: PathBuf = fixture("registry-find-none");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        admin_root: common.join("worktrees"),
        before_admin_ids: BTreeSet::new(),
        common_dir: common.clone(),
        effective_cwd: main.clone(),
        source_root: Some(main.clone()),
        selection: CopySelection::Ignored,
    };
    let found: CreatedWorktrees =
        find_created_worktrees(&git_context(), &observation).expect("found");
    assert_eq!(
        found,
        CreatedWorktrees {
            created: Vec::new(),
            registered_roots: Vec::new()
        }
    );
    remove(root.as_path());
}

/// A worktree added by real Git is found with its canonical root and every registered root.
#[test]
fn added_worktree_is_found_with_its_root() {
    let root: PathBuf = fixture("registry-find-added");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        admin_root: common.join("worktrees"),
        before_admin_ids: BTreeSet::new(),
        common_dir: common.clone(),
        effective_cwd: main.clone(),
        source_root: Some(main.clone()),
        selection: CopySelection::Ignored,
    };
    let topic: PathBuf = root.join("topic");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    let found: CreatedWorktrees =
        find_created_worktrees(&git_context(), &observation).expect("found");
    let topic_root: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical");
    assert_eq!(
        found.created,
        vec![CreatedWorktree {
            admin_id: String::from("topic"),
            root: topic_root.clone()
        }]
    );
    assert!(found.registered_roots.contains(&topic_root));
    assert!(
        found
            .registered_roots
            .contains(&std::fs::canonicalize(main.as_path()).expect("main"))
    );
    remove(root.as_path());
}

/// A worktree Git registered before its `post-checkout` hook failed is still found.
#[cfg(unix)]
#[test]
fn worktree_registered_before_failing_hook_is_found() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("registry-find-hook");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        admin_root: common.join("worktrees"),
        before_admin_ids: BTreeSet::new(),
        common_dir: common.clone(),
        effective_cwd: main.clone(),
        source_root: Some(main.clone()),
        selection: CopySelection::Ignored,
    };
    let hooks: PathBuf = common.join("hooks");
    std::fs::create_dir_all(hooks.as_path()).expect("hooks");
    let hook: PathBuf = hooks.join("post-checkout");
    std::fs::write(hook.as_path(), b"#!/bin/sh\nexit 1\n").expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("mode");
    let topic: PathBuf = root.join("topic");
    let output = crate::test_support::git_output(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    assert!(!output.status.success(), "the hook must fail the command");
    let found: CreatedWorktrees =
        find_created_worktrees(&git_context(), &observation).expect("found");
    assert_eq!(found.created.len(), 1);
    assert_eq!(found.created[0].admin_id, "topic");
    remove(root.as_path());
}

/// A registration whose directory was deleted has no root to copy into, so the copy fails.
#[test]
fn registration_without_root_fails() {
    let root: PathBuf = fixture("registry-find-missing-root");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        admin_root: common.join("worktrees"),
        before_admin_ids: BTreeSet::new(),
        common_dir: common.clone(),
        effective_cwd: main.clone(),
        source_root: Some(main.clone()),
        selection: CopySelection::Ignored,
    };
    let topic: PathBuf = root.join("topic");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    std::fs::remove_dir_all(topic.as_path()).expect("delete root");
    let failure: WorktreeCopyError =
        find_created_worktrees(&git_context(), &observation).expect_err("unresolved root");
    assert_eq!(
        failure.0,
        "cli-git: linked worktree \"topic\" was registered but its root could not be resolved."
    );
    remove(root.as_path());
}
