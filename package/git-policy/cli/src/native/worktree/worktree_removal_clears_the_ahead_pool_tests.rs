//! What:
//!  Controls for taking the snapshots the wrapper made ahead of an add out of the administrative directory of a
//! worktree that is being removed.
//! Why:
//!   The pool is the wrapper's own cache of that worktree,
//! and the removal is the only thing that takes it,
//! so a pool that
//!  stayed behind was a registration nothing could empty (a Sonnet finding,
//! F1).
//! These run on a plain directory tree:
//! a member that is no subvolume is still a name the wrapper made,
//! and the pool goes
//!  the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(clearTheSnapshotsAhead(admin)).toBeUndefined(); expect(pool.exists()).toBe(false);
//! ```

use super::*;
use crate::private_storage::remove_tree;
use crate::test_support::fixture;

/// The name of one member of a pool,
/// in the shape the service makes it,
/// so a control never mistakes its own name for a snapshot's.
const A_MEMBER: &str = ".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef";

/// What:
///  The pool of a worktree that is going away goes with its members,
/// and nothing of it is left under the name a fill reads.
/// Why:
///   The removal empties the registration after the destroy,
/// and that emptying refuses a nested subvolume,
///  so the pool has to be gone before it (a Sonnet finding,
/// F1).
#[test]
fn the_pool_of_a_worktree_that_is_going_away_goes_with_its_members() {
    let root: PathBuf = fixture("ahead-pool-cleared");
    let admin: PathBuf = root.join("worktrees/wt");
    let pool: PathBuf = pool_folder(admin.as_path());
    let member: PathBuf = pool.join(A_MEMBER);
    std::fs::create_dir_all(member.as_path()).expect("a snapshot ahead of an add");
    std::fs::write(member.join("tracked.txt"), b"the worktree's own file\n").expect("its files");
    assert_eq!(clear_the_snapshots_ahead(admin.as_path()), Ok(()), "the pool of a worktree that is going away goes");
    assert!(!pool.exists(), "no fill finds a pool under its own name any more");
    assert!(!member.exists(), "and no member is left");
}

/// What:
///  A worktree that never was the source of an add has no pool,
/// and the clear is nothing at all.
/// Why:
///   The clear runs on every removal of a subvolume worktree,
/// and the one that is not a source must not
///  make anything or fail.
#[test]
fn a_worktree_with_no_pool_is_nothing_to_clear() {
    let root: PathBuf = fixture("ahead-pool-absent");
    let admin: PathBuf = root.join("worktrees/wt");
    std::fs::create_dir_all(admin.as_path()).expect("an administrative directory");
    assert_eq!(clear_the_snapshots_ahead(admin.as_path()), Ok(()));
    assert!(!admin.join("monochromatic-git-policy-cli").exists(), "nothing was made for a pool that never was");
}

/// What:
///  A member that cannot go is the reason the removal refuses,
/// and the report names it.
/// Why:
///   A refusal that did not say which snapshot stood in the way leaves the person with a removal that cannot run
///  and nothing to act on.
#[test]
fn a_member_that_cannot_go_is_the_reason_the_removal_refuses() {
    let root: PathBuf = fixture("ahead-pool-blocked");
    let taken: PathBuf = root.join("removing-taken");
    let member: PathBuf = taken.join(A_MEMBER);
    std::fs::create_dir_all(member.as_path()).expect("a snapshot ahead of an add");
    let refused: Result<(), String> = clear_the_snapshots_ahead_with(
        taken.as_path(),
        &|_member: &Path| { return Err(std::io::Error::from_raw_os_error(libc::EACCES)); },
        &remove_tree,
    );
    let reason: String = match refused {
        Ok(()) => panic!("a member that cannot go is the reason the removal refuses"),
        Err(reason) => reason,
    };
    assert!(reason.contains(member.to_str().expect("UTF-8")), "the reason names the snapshot that stands there: {reason}");
}
