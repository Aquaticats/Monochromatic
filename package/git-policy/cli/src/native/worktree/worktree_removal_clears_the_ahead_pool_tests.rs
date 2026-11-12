//! What:
//!  Controls for taking the pools of snapshots the wrapper made ahead of an add out of the administrative directory
//! of a worktree that is being removed.
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
//! expect(clearThePoolsAheadOfAWorktree(admin)).toBeUndefined(); expect(pool.exists()).toBe(false);
//! ```

use super::*;
use crate::private_storage::remove_tree;
use crate::test_support::fixture;
use crate::worktree_snapshot_ahead::pool_folder;

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
    assert_eq!(clear_the_pools_ahead_of_a_worktree(admin.as_path()), Ok(()), "the pool of a worktree that is going away goes");
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
    assert_eq!(clear_the_pools_ahead_of_a_worktree(admin.as_path()), Ok(()));
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
    let refused: Result<(), String> = clear_one_pool_ahead(
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

/// The name a killed clear leaves a pool it had already renamed,
/// which is the identity that name was made of.
const TAKEN_BY_A_CLEAR: &str = "removing-0f0e0d0c-0b0a-4908-8706-050403020100";

/// What:
///  A pool a clear renamed and was killed before emptying is taken by the next clear,
///  with its members,
///  and the wrapper's folder goes with it.
/// Why:
///   A clear renames a pool to a name no fill reads before it deletes a member,
/// and a command killed between
///  those two steps leaves that member under the new name;
/// a clear that took only the pool's own name would then
///  refuse its own leftover for ever,
/// so the registration could never be emptied and every later command would print
///  the notice about it.
#[test]
fn a_pool_a_killed_clear_left_under_its_taken_name_goes_with_the_next_clear() {
    let root: PathBuf = fixture("ahead-pool-left-taken");
    let admin: PathBuf = root.join("worktrees/wt");
    let folder: PathBuf = admin.join("monochromatic-git-policy-cli");
    let left: PathBuf = folder.join(TAKEN_BY_A_CLEAR);
    let member: PathBuf = left.join(A_MEMBER);
    std::fs::create_dir_all(member.as_path()).expect("the member of a pool a killed clear left");
    std::fs::write(member.join("tracked.txt"), b"the worktree's own file\n").expect("its files");
    // The pool a fill made after that clear, which is the one the removal was written for.
    let pool: PathBuf = pool_folder(admin.as_path());
    std::fs::create_dir_all(pool.join(A_MEMBER).as_path()).expect("the pool of the worktree that is going away");
    assert_eq!(clear_the_pools_ahead_of_a_worktree(admin.as_path()), Ok(()));
    assert!(!left.exists(), "the leftover of the killed clear goes");
    assert!(!member.exists(), "with the member it still held");
    assert!(!pool.exists(), "and the pool a fill made goes as it always did");
    assert!(!folder.exists(), "the data folder is free again, because it held nothing but those pools");
}

/// What:
///  The records a person reads stay where they are when the pools of a worktree that is going away are taken,
///  and a name that is no identity this program made is not taken for a pool.
/// Why:
///   The data folder holds the notices of a push that failed,
/// the files a pull kept out of the way and the service's
///  own work queue beside the pool,
/// so a clear that took the folder whole would delete what a person has not read yet,
///  and one that took every name beginning `removing-` would take a directory somebody else named.
#[test]
fn the_records_a_person_reads_stay_and_a_name_this_program_did_not_make_is_no_pool() {
    let root: PathBuf = fixture("ahead-pool-keeps-the-records");
    let admin: PathBuf = root.join("worktrees/wt");
    let folder: PathBuf = admin.join("monochromatic-git-policy-cli");
    let notices: PathBuf = folder.join("notices/push+refs%2Fheads%2Ftopic+0123456789abcdef0123456789abcdef01234567.txt");
    let kept: PathBuf = folder.join("pull-kept/notes.txt");
    let queued: PathBuf = folder.join("work/an-item");
    for path in [&notices, &kept, &queued] {
        std::fs::create_dir_all(path.parent().expect("a folder a person reads")).expect("the folder");
        std::fs::write(path.as_path(), b"what a person reads\n").expect("its bytes");
    }
    // A directory of the person's own whose name begins like a taken pool but is no identity this program makes.
    let named_like_a_pool: PathBuf = folder.join("removing-not-an-identity");
    std::fs::create_dir_all(named_like_a_pool.as_path()).expect("a folder of the person's own");
    std::fs::write(named_like_a_pool.join("notes.txt"), b"mine\n").expect("its file");
    // The pool a fill reads, and the pool a killed clear left.
    let pool: PathBuf = pool_folder(admin.as_path());
    std::fs::create_dir_all(pool.join(A_MEMBER).as_path()).expect("the pool of the worktree that is going away");
    let left: PathBuf = folder.join(TAKEN_BY_A_CLEAR);
    std::fs::create_dir_all(left.join(A_MEMBER).as_path()).expect("the pool a killed clear left");
    assert_eq!(clear_the_pools_ahead_of_a_worktree(admin.as_path()), Ok(()));
    assert!(!pool.exists(), "the pool a fill reads goes");
    assert!(!left.exists(), "and so does the pool a killed clear left");
    for path in [&notices, &kept, &queued] {
        assert!(path.is_file(), "the records a person reads stay: {}", path.display());
    }
    assert_eq!(std::fs::read(named_like_a_pool.join("notes.txt")).expect("the person's file"), b"mine\n", "a name this program did not make is no pool");
}
