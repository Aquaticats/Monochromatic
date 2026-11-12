//! What:
//!  The pool of finished shadow repositories of a worktree Git no longer lists,
//! through the built executable:
//! a wrapped command removes it,
//! and one of a worktree Git still lists never,
//! with what a member links to left alone.
//! Why:
//!  A worktree removed by a route the wrapper cannot see
//! (`git worktree prune`,
//! a registration deleted by hand)
//!  leaves the members of the pool for a worktree that is gone.
//! They are private repositories with their links into this one,
//! and no commit can ever claim one,
//!  so a command in the repository is the named event that collects them
//! (`shadow_pool.rs`,
//! `sweep_gone_worktrees`).
//!  The folder is the repository's own state and no control of a library call can show that a real `git status`
//!  removes it,
//! so these controls run the program a person runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // mkdir(join(repo, '.git/monochromatic-git-policy-cli-shadow-pool/worktree-wt/member-a'));
//! // runWrapped(fixture, repo, ['status', '--short']); expect(poolOfGoneWorktree).not.toExist();
//! ```

/// The fixture and the bounded wrapped command.
use super::support::{Fixture, Observed, fixture, git, remove, run_wrapped, stderr_of};
use git_policy_cli::wrapper_names_old_and_new::{SHADOW_DRAWN_FOLDER, SHADOW_POOL_FOLDER};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  One member of a worktree's pool,
///  with a link where the creation puts one,
///  as a commit that took a member ahead left it.
fn a_pool_folder(common: &Path, key: &str) -> PathBuf {
    let member: PathBuf = common.join(SHADOW_POOL_FOLDER).join(key).join("member-a-finished-shadow");
    std::fs::create_dir_all(member.join("objects/info").as_path()).expect("a member");
    std::fs::write(member.join("config").as_path(), b"[core]\n\trepositoryformatversion = 0\n").expect("the member's config");
    std::fs::write(member.join("HEAD").as_path(), b"ref: refs/heads/main\n").expect("the member's placeholder head");
    // The member's own entries are links to this repository's, so a removal that followed one would reach them.
    std::os::unix::fs::symlink(common.join("objects").as_path(), member.join("objects-kept").as_path()).expect("a link into the repository");
    return common.join(SHADOW_POOL_FOLDER).join(key);
}

/// What:
///  A wrapped command removes the pool of a worktree Git no longer lists.
/// Why:
///   `git worktree prune` runs in the repository and removes the registration alone,
///  so the pool of that worktree is what a later command in the repository collects.
///  The member's link points at the repository's object directory,
///  which is still there afterwards:
///  nothing here follows a link.
#[test]
fn a_wrapped_command_removes_the_pool_of_a_worktree_git_no_longer_lists() {
    let fixture: Fixture = fixture("pool-of-a-pruned-worktree");
    let repo: PathBuf = super::support::repository(&fixture, OsStr::new("repo"));
    let common: PathBuf = repo.join(".git");
    let gone: PathBuf = a_pool_folder(common.as_path(), "worktree-a-worktree-that-was-pruned");
    let main: PathBuf = a_pool_folder(common.as_path(), "main");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--short"]);
    assert_eq!(observed.code, Some(0), "the command the person asked for still runs: {}", stderr_of(&observed));
    assert!(std::fs::symlink_metadata(gone.as_path()).is_err(), "the pool of the worktree Git no longer lists is gone");
    assert!(main.join("member-a-finished-shadow").is_dir(), "the main checkout's pool is untouched");
    assert!(std::fs::symlink_metadata(common.join("objects").as_path()).is_ok(), "what the member linked to is still there");
    remove(&fixture);
}

/// What:
///  A wrapped command removes a drawn shadow nothing renewed and leaves one a commit holds.
/// Why:
///  A drawn tree is used where it stands, so no other sweep reaches it:
///  the pool's own removal takes the pool's folder, and the release of a commit's scratch takes the scratch tree.
///  The command is the named event that collects what a killed commit left,
///  and the age is the scratch tree's own, so a tree a live command renews is never old enough.
#[test]
fn a_wrapped_command_removes_a_drawn_shadow_nothing_renewed_and_leaves_a_fresh_one() {
    let fixture: Fixture = fixture("drawn-shadow-swept");
    let repo: PathBuf = super::support::repository(&fixture, OsStr::new("repo"));
    let common: PathBuf = repo.join(".git");
    let drawn: PathBuf = common.join(SHADOW_DRAWN_FOLDER).join("main");
    let held: PathBuf = drawn.join("drawn-a-tree-a-commit-holds");
    let stopped: PathBuf = drawn.join("drawn-a-tree-a-command-that-stopped");
    std::fs::create_dir_all(held.as_path()).expect("a tree a commit holds");
    std::fs::create_dir_all(stopped.as_path()).expect("a tree nothing renews");
    let abandoned: std::time::Duration = std::time::Duration::from_secs(3 * 60 * 60);
    std::fs::File::open(stopped.as_path()).expect("the stopped tree").set_modified(std::time::SystemTime::now() - abandoned).expect("an age");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--short"]);
    assert_eq!(observed.code, Some(0), "the command the person asked for still runs: {}", stderr_of(&observed));
    assert!(std::fs::symlink_metadata(stopped.as_path()).is_err(), "the tree nothing renewed is gone");
    assert!(held.is_dir(), "the tree a command is renewing stays");
    remove(&fixture);
}

/// What:
///  A wrapped command keeps the pool of a worktree Git still lists.
/// Why:
///   Git's own list still names a linked worktree whose checkout is gone until `git worktree prune` runs,
///  and it names every worktree Git is adding before its checkout exists.
///  Its pool is the pool of a worktree that is there,
///  so a sweep of a command that runs while the worktree stands removes nothing.
#[test]
fn a_wrapped_command_keeps_the_pool_of_a_worktree_git_lists() {
    let fixture: Fixture = fixture("pool-of-a-listed-worktree");
    let repo: PathBuf = super::support::repository(&fixture, OsStr::new("repo"));
    let common: PathBuf = repo.join(".git");
    git(&fixture, repo.as_path(), &["worktree", "add", "../linked", "--quiet", "-b", "topic"]);
    let listed: PathBuf = a_pool_folder(common.as_path(), "worktree-linked");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--short"]);
    assert_eq!(observed.code, Some(0), "the command the person asked for still runs: {}", stderr_of(&observed));
    assert!(listed.join("member-a-finished-shadow").is_dir(), "the pool of a worktree Git lists stays");
    remove(&fixture);
}
