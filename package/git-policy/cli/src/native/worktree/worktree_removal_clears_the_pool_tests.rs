//! Controls for the pool a removed worktree's commits were served from:
//! it goes with the worktree,
//! and a worktree that is still there,
//! the main checkout and a pool that is already gone keep
//!  theirs.

use super::*;
use crate::test_support::{fixture, git, remove, repository};

/// A repository,
/// its canonical common directory and the pool a linked worktree of it fills.
/// Why:
///   The pool's name is worked out from the administrative directory the removal deletes,
/// so a control seeds it where the
///  product looks for it.
struct Fixture {
    root: PathBuf,
    common: PathBuf,
    admin: PathBuf,
    pool: PathBuf,
    member: PathBuf,
}

/// A repository with one linked worktree whose pool holds one finished member.
fn fixture_with_a_pooled_worktree(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("canonical common directory");
    let admin: PathBuf = std::fs::canonicalize(common.join("worktrees/wt")).expect("the administrative directory");
    let pool: PathBuf = crate::shadow_pool::pool_root_for(common.as_path(), admin.as_path());
    let member: PathBuf = pool.join("member-finished-ahead");
    std::fs::create_dir_all(member.as_path()).expect("a finished member");
    return Fixture { root, common, admin, pool, member };
}

/// The pool of a worktree whose Git directory is gone is removed with the links of every member.
#[test]
fn the_pool_of_a_worktree_that_is_gone_is_removed() {
    let fixture: Fixture = fixture_with_a_pooled_worktree("removal-pool-gone");
    std::fs::remove_dir_all(fixture.admin.as_path()).expect("the administrative directory goes as Git's removal takes it");
    clear_the_pool_of_a_removed_worktree(fixture.common.as_path(), fixture.admin.as_path());
    assert!(!fixture.member.exists(), "no finished member is left behind");
    assert!(!fixture.pool.exists(), "and no pool folder either");
    remove(fixture.root.as_path());
}

/// A worktree that is still registered keeps its pool,
/// so a removal Git refused loses nothing the worktree would have taken.
#[test]
fn the_pool_of_a_worktree_that_is_still_there_stays() {
    let fixture: Fixture = fixture_with_a_pooled_worktree("removal-pool-present");
    clear_the_pool_of_a_removed_worktree(fixture.common.as_path(), fixture.admin.as_path());
    assert!(fixture.member.is_dir(), "the finished member stays for the worktree that can claim it");
    remove(fixture.root.as_path());
}

/// The main checkout's pool is never taken:
/// no removal removes the main checkout,
/// and its pool is the one every commit there claims from.
#[test]
fn the_pool_of_the_main_checkout_is_never_taken() {
    let root: PathBuf = fixture("removal-pool-main");
    let main: PathBuf = repository(root.as_path(), "main");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("canonical common directory");
    let pool: PathBuf = crate::shadow_pool::pool_root_for(common.as_path(), common.as_path());
    let member: PathBuf = pool.join("member-finished-ahead");
    std::fs::create_dir_all(member.as_path()).expect("a finished member");
    clear_the_pool_of_a_removed_worktree(common.as_path(), common.as_path());
    assert!(member.is_dir(), "the main checkout keeps the pool its commits claim from");
    remove(root.as_path());
}

/// A pool that is not there is nothing to remove,
/// and nothing about it is reported as a failure.
#[test]
fn a_pool_that_is_already_gone_is_done() {
    let fixture: Fixture = fixture_with_a_pooled_worktree("removal-pool-absent");
    std::fs::remove_dir_all(fixture.pool.as_path()).expect("the pool goes first");
    std::fs::remove_dir_all(fixture.admin.as_path()).expect("the administrative directory goes as Git's removal takes it");
    clear_the_pool_of_a_removed_worktree(fixture.common.as_path(), fixture.admin.as_path());
    assert!(!fixture.pool.exists(), "nothing is made again in its place");
    remove(fixture.root.as_path());
}

/// A commit's claim of a member is never undone by the pool's removal,
/// whether it came before the removal or exactly when the deletion starts.
/// Why:
///   A claim is one rename of a member out of the pool,
/// and a deletion keeps unlinking through the directory it already opened,
/// so a
///  claim that lands after the deletion opened the member loses the ref store the commit wrote there,
/// and the commit
///  reports a failure real Git did not have.
/// The pool is therefore taken out of its own name before anything is deleted, so a claim
///  either comes first or finds no pool.
#[test]
fn a_member_a_commit_claims_is_never_taken_with_the_pool() {
    // Granting the member to a commit: what a claim does, and the ref store a commit writes after it.
    let claim = |from: &Path, shadow: &Path| -> bool {
        if std::fs::rename(from, shadow).is_err() {
            return false;
        }
        std::fs::create_dir_all(shadow.join("refs/heads")).expect("the ref store a commit writes after its claim");
        std::fs::write(shadow.join("refs/heads/main"), b"0123456789abcdef0123456789abcdef01234567\n").expect("the ref of the commit it wrote");
        return true;
    };
    // The positive control: a claim made before the removal is served, and the removal takes nothing it claimed.
    let before: Fixture = fixture_with_a_pooled_worktree("removal-pool-claim-before");
    std::fs::remove_dir_all(before.admin.as_path()).expect("the administrative directory goes as Git's removal takes it");
    let claimed_before: PathBuf = before.root.join("commit-shadow");
    assert!(claim(before.member.as_path(), claimed_before.as_path()), "the fixture's member is one a commit can claim");
    clear_the_pool_of_a_removed_worktree(before.common.as_path(), before.admin.as_path());
    assert!(claimed_before.join("refs/heads/main").is_file(), "the removal of the pool takes nothing of the shadow a commit claimed before it");
    assert!(!before.pool.exists(), "and the pool is gone");
    remove(before.root.as_path());

    // The claim that lands exactly when the deletion of the pool starts.
    let raced: Fixture = fixture_with_a_pooled_worktree("removal-pool-claim-during");
    std::fs::remove_dir_all(raced.admin.as_path()).expect("the administrative directory goes as Git's removal takes it");
    let claimed_during: PathBuf = raced.root.join("commit-shadow");
    let member: PathBuf = raced.member.clone();
    let shadow: PathBuf = claimed_during.clone();
    let claimed: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let remove_while_claiming = |taken: &Path| {
        claimed.set(claim(member.as_path(), shadow.as_path()));
        return remove_tree(taken);
    };
    clear_the_pool_of_a_removed_worktree_with(raced.common.as_path(), raced.admin.as_path(), &remove_while_claiming);
    assert!(!claimed.get(), "no commit claims a member from the pool at the moment the pool is deleted: the pool's name is free before anything in it is removed");
    assert!(!raced.pool.exists(), "and the pool is gone");
    remove(raced.root.as_path());
}
