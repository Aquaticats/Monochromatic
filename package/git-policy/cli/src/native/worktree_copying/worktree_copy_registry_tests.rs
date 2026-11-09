//! Consumer checks of linked-worktree registration observation against real Git.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_prune::Prune;

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

/// A main worktree is observed through `observe_copy_source` only:
/// it has the main root as source and the main-checkout selection.
#[test]
fn main_worktree_is_observed_with_the_main_checkout_selection() {
    let root: PathBuf = fixture("registry-observe-main");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let main_identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: common.clone(),
        git_dir: common.clone(),
        worktree_root: main.clone(),
    };
    let observation: WorktreeCopyObservation = observe_copy_source(&main_identity, main.as_path())
        .expect("observation")
        .expect("applicable");
    assert_eq!(observation.source_root, Some(main.clone()));
    assert_eq!(observation.common_dir, common);
    assert_eq!(observation.selection, CopySelection::MainCheckout);
    assert_eq!(observation.selection.prune(), Prune::main_checkout());
    remove(root.as_path());
}

/// Linked and bare repositories keep the standard selection,
/// which leaves nothing out.
#[test]
fn linked_and_bare_observations_keep_the_standard_selection() {
    let root: PathBuf = fixture("registry-observe-standard");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let topic: PathBuf = root.join("topic");
    git(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")],
    );
    let topic_root: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical topic");
    let linked_observation: WorktreeCopyObservation = observe_copy_source(
        &linked(common.as_path(), topic_root.as_path()),
        topic_root.as_path(),
    )
    .expect("observation")
    .expect("applicable");
    assert_eq!(linked_observation.selection, CopySelection::Ignored);
    assert_eq!(linked_observation.selection.prune(), Prune::Never);
    let bare_root: PathBuf = bare(root.as_path(), "shared.git");
    let bare_observation: WorktreeCopyObservation = observe_copy_source(
        &WorktreeIdentity::BareRepository {
            common_dir: bare_root.clone(),
            git_dir: bare_root.clone(),
        },
        bare_root.as_path(),
    )
    .expect("observation")
    .expect("applicable");
    assert_eq!(bare_observation.selection, CopySelection::Ignored);
    assert_eq!(bare_observation.source_root, None);
    remove(root.as_path());
}

/// A bare repository is observed without a source root,
/// and a linked worktree is observed with one.
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
    assert_eq!(linked_observation.common_dir, common);
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
    remove(root.as_path());
}

/// The observation of the main checkout used by the finds below.
fn main_observation(main: &Path, common: &Path) -> WorktreeCopyObservation {
    return WorktreeCopyObservation {
        common_dir: common.to_path_buf(),
        effective_cwd: main.to_path_buf(),
        source_root: Some(main.to_path_buf()),
        selection: CopySelection::Ignored,
    };
}

/// Adds the worktree `name` beside the repository with real Git and returns its canonical destination.
fn add_worktree(root: &Path, main: &Path, name: &str) -> PathBuf {
    let destination: PathBuf = root.join(name);
    git(main, &["worktree", "add", "--quiet", "-b", name, destination.to_str().expect("utf8")]);
    return std::fs::canonicalize(destination).expect("canonical destination");
}

/// A destination that Git did not register is not created,
/// and asking reads nothing else.
#[test]
fn an_unregistered_destination_creates_nothing() {
    let root: PathBuf = fixture("registry-find-none");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let absent: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("absent");
    assert!(!destination_is_registered(common.as_path(), absent.as_path()));
    let found: CreatedWorktrees =
        find_created_worktree(&git_context(), &main_observation(main.as_path(), common.as_path()), absent.as_path(), false, "test-command").expect("found");
    assert_eq!(
        found,
        CreatedWorktrees {
            created: Vec::new(),
            registered_roots: Vec::new(),
            other_commands: None
        }
    );
    remove(root.as_path());
}

/// A worktree added by real Git at the named destination is found with its canonical root and every registered root.
#[test]
fn added_worktree_is_found_with_its_root() {
    let root: PathBuf = fixture("registry-find-added");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let observation: WorktreeCopyObservation = main_observation(main.as_path(), common.as_path());
    let future: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let registered_before: bool = destination_is_registered(common.as_path(), future.as_path());
    assert!(!registered_before);
    let topic_root: PathBuf = add_worktree(root.as_path(), main.as_path(), "topic");
    assert_eq!(topic_root, future);
    let found: CreatedWorktrees =
        find_created_worktree(&git_context(), &observation, topic_root.as_path(), registered_before, "test-command").expect("found");
    assert_eq!(found.created, vec![CreatedWorktree { root: topic_root.clone(), binding: binding_of(common.as_path(), topic_root.as_path()) }]);
    assert!(found.registered_roots.contains(&topic_root));
    assert!(found.registered_roots.contains(&std::fs::canonicalize(main.as_path()).expect("main")));
    remove(root.as_path());
}

/// A destination that was a registered worktree before Git ran is not this command's creation,
/// whatever Git did with it.
#[test]
fn a_destination_registered_before_is_not_created() {
    let root: PathBuf = fixture("registry-find-before");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let topic_root: PathBuf = add_worktree(root.as_path(), main.as_path(), "topic");
    assert!(destination_is_registered(common.as_path(), topic_root.as_path()));
    let found: CreatedWorktrees =
        find_created_worktree(&git_context(), &main_observation(main.as_path(), common.as_path()), topic_root.as_path(), true, "test-command").expect("found");
    assert!(found.created.is_empty());
    assert_eq!(found.other_commands, None);
    remove(root.as_path());
}

/// The registration a worktree Git made is bound to,
/// for the expectations of what a find returns.
fn binding_of(common: &Path, destination: &Path) -> crate::worktree_add_binding::Binding {
    return crate::worktree_add_binding::bind_registration(common, destination).expect("binding").binding;
}

/// Two worktrees registered around one command are told apart by their destinations:
/// each find returns only its own,
/// so a
/// command never copies into a worktree another add registered while it ran.
#[test]
fn concurrent_registrations_are_told_apart_by_destination() {
    let root: PathBuf = fixture("registry-find-two");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let observation: WorktreeCopyObservation = main_observation(main.as_path(), common.as_path());
    let first: PathBuf = add_worktree(root.as_path(), main.as_path(), "first");
    let second: PathBuf = add_worktree(root.as_path(), main.as_path(), "second");
    let for_first: CreatedWorktrees = find_created_worktree(&git_context(), &observation, first.as_path(), false, "test-command").expect("first");
    let for_second: CreatedWorktrees = find_created_worktree(&git_context(), &observation, second.as_path(), false, "test-command").expect("second");
    assert_eq!(for_first.created, vec![CreatedWorktree { root: first.clone(), binding: binding_of(common.as_path(), first.as_path()) }]);
    assert_eq!(for_second.created, vec![CreatedWorktree { root: second.clone(), binding: binding_of(common.as_path(), second.as_path()) }]);
    remove(root.as_path());
}

/// A directory whose `.git` file points at an administrative directory of another repository,
/// or at nothing,
/// is no
/// registration of this one.
#[test]
fn a_foreign_or_dangling_pointer_is_not_a_registration() {
    let root: PathBuf = fixture("registry-pointer-foreign");
    let main: PathBuf = repository(root.as_path(), "main");
    let other: PathBuf = repository(root.as_path(), "other");
    let common: PathBuf = common_dir_of(main.as_path());
    let elsewhere: PathBuf = add_worktree(root.as_path(), other.as_path(), "elsewhere");
    assert!(!destination_is_registered(common.as_path(), elsewhere.as_path()));
    let dangling: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("dangling");
    std::fs::create_dir(dangling.as_path()).expect("dangling");
    std::fs::write(dangling.join(".git"), b"gitdir: /nonexistent/admin\n").expect("pointer");
    assert!(!destination_is_registered(common.as_path(), dangling.as_path()));
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
    let observation: WorktreeCopyObservation = main_observation(main.as_path(), common.as_path());
    let hooks: PathBuf = common.join("hooks");
    std::fs::create_dir_all(hooks.as_path()).expect("hooks");
    let hook: PathBuf = hooks.join("post-checkout");
    std::fs::write(hook.as_path(), b"#!/bin/sh\nexit 1\n").expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("mode");
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let output = crate::test_support::git_output(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")],
    );
    assert!(!output.status.success(), "the hook must fail the command");
    let found: CreatedWorktrees = find_created_worktree(&git_context(), &observation, topic.as_path(), false, "test-command").expect("found");
    assert_eq!(found.created, vec![CreatedWorktree { root: topic.clone(), binding: binding_of(common.as_path(), topic.as_path()) }]);
    remove(root.as_path());
}

/// A registration whose directory was deleted is no worktree to copy into:
/// nothing is created and nothing fails.
#[test]
fn a_registration_whose_directory_is_gone_creates_nothing() {
    let root: PathBuf = fixture("registry-find-missing-root");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let topic: PathBuf = add_worktree(root.as_path(), main.as_path(), "topic");
    std::fs::remove_dir_all(topic.as_path()).expect("delete root");
    let found: CreatedWorktrees =
        find_created_worktree(&git_context(), &main_observation(main.as_path(), common.as_path()), topic.as_path(), false, "test-command").expect("found");
    assert!(found.created.is_empty());
    remove(root.as_path());
}

/// A destination registered with relative paths is found like any other,
/// so its copy is made.
#[test]
fn a_destination_registered_with_relative_paths_is_found() {
    let root: PathBuf = fixture("registry-relative-paths");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = common_dir_of(main.as_path());
    let destination: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    git(main.as_path(), &["worktree", "add", "--quiet", "--relative-paths", "-b", "topic", destination.to_str().expect("utf8")]);
    assert!(destination_is_registered(common.as_path(), destination.as_path()));
    let found: CreatedWorktrees =
        find_created_worktree(&git_context(), &main_observation(main.as_path(), common.as_path()), destination.as_path(), false, "test-command").expect("found");
    assert_eq!(found.created, vec![CreatedWorktree { root: destination.clone(), binding: binding_of(common.as_path(), destination.as_path()) }]);
    remove(root.as_path());
}
