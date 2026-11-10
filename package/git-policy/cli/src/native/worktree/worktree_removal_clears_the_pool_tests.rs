//! Controls for the pool a removed worktree's commits were served from:
//! it goes with the worktree,
//! and a worktree that is still there,
//! the main checkout and a pool that is already gone keep
//!  theirs.

use super::*;
use crate::test_support::{fixture, git, remove, repository};

/// A repository, its canonical common directory and the pool a linked worktree of it fills.
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
