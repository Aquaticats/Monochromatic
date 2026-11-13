//! Controls of the lease one filling holds on one build directory's names:
//! one holder at a time,
//! a dead holder's lease taken by the next,
//! and two build directories that never meet.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  One filling's lease is refused to the next,
///  in the same process as well as in another,
///  so a filling that is still at work is never taken for a dead one.
/// Why:
///   Two fillings of one build directory run in one commit service,
///  so the proof has to hold between two descriptors of one process:
///  `flock` is held per open file description, and a second open contends with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = claimTheNames(staging); expect(first.kind).toBe('held'); expect(claimTheNames(staging).kind).toBe('taken')
/// ```
#[test]
fn a_lease_one_filling_holds_is_refused_to_the_next() {
    let root: PathBuf = fixture("prewarm-claim-held");
    let staging: PathBuf = root.join("worktree-build.monochromatic-prewarm");
    let first: TheClaim = claim_the_names(staging.as_path());
    assert!(matches!(first, TheClaim::Held(_)), "the first filling takes the lease");
    assert!(matches!(claim_the_names(staging.as_path()), TheClaim::Taken), "the second filling may not have it");
    drop(first);
    remove(root.as_path());
}

/// What:
///  A lease a filling no longer holds is taken by the next one,
///  so the leftovers of a killed filling are the next filling's to complete.
/// Why:
///   A kill leaves the lease file on disk and no descriptor holding it,
///  which is exactly the state the next filling has to read as "nobody is at work here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = claimTheNames(staging); drop(first); expect(claimTheNames(staging).kind).toBe('held')
/// ```
#[test]
fn a_lease_no_filling_holds_is_taken_by_the_next() {
    let root: PathBuf = fixture("prewarm-claim-free");
    let staging: PathBuf = root.join("worktree-build.monochromatic-prewarm");
    {
        let first: TheClaim = claim_the_names(staging.as_path());
        assert!(matches!(first, TheClaim::Held(_)), "the first filling takes the lease");
    }
    assert!(lease_of(staging.as_path()).is_file(), "the file a killed filling leaves stands where it stood");
    let next: TheClaim = claim_the_names(staging.as_path());
    assert!(matches!(next, TheClaim::Held(_)), "a lease nobody holds is the next filling's");
    remove(root.as_path());
}

/// What:
///  Two build directories never contend,
///  because one filling's lease says nothing about another directory's names.
/// Why:
///   A commit service fills many workspaces of one record,
///  and one workspace's slow clean must never make another workspace's names look taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(claimTheNames(first).kind).toBe('held'); expect(claimTheNames(second).kind).toBe('held')
/// ```
#[test]
fn two_build_directories_have_two_leases() {
    let root: PathBuf = fixture("prewarm-claim-two-directories");
    let first: PathBuf = root.join("one/worktree-build.monochromatic-prewarm");
    let second: PathBuf = root.join("two/worktree-build.monochromatic-prewarm");
    std::fs::create_dir_all(first.parent().expect("a directory")).expect("the first directory");
    std::fs::create_dir_all(second.parent().expect("a directory")).expect("the second directory");
    let first_claim: TheClaim = claim_the_names(first.as_path());
    let second_claim: TheClaim = claim_the_names(second.as_path());
    assert!(matches!(first_claim, TheClaim::Held(_)), "the first directory's lease is this run's");
    assert!(matches!(second_claim, TheClaim::Held(_)), "and the second directory's is too");
    remove(root.as_path());
}

/// What:
///  A lease file this run cannot make is refused and named,
///  never mistaken for another filling's lock.
/// Why:
///   A parent that is a regular file makes the open fail,
///  and a caller that read that as "another filling is at work" would leave the names alone for
///  ever;
///  the refusal names the path so the caller can say why it took names of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync('blocker', ''); expect(claimTheNames('blocker/x.monochromatic-prewarm').kind).toBe('refused')
/// ```
#[test]
fn a_lease_that_cannot_be_made_is_refused_and_named() {
    let root: PathBuf = fixture("prewarm-claim-refused");
    std::fs::write(root.join("blocker"), b"a file where a directory would have to be\n").expect("the blocker");
    let staging: PathBuf = root.join("blocker/worktree-build.monochromatic-prewarm");
    let refused: TheClaim = claim_the_names(staging.as_path());
    match refused {
        TheClaim::Refused(reason) => assert!(reason.contains("blocker"), "the refusal names the path: {reason}"),
        other => panic!("a lease that cannot be made is refused: {other:?}"),
    }
    remove(root.as_path());
}
