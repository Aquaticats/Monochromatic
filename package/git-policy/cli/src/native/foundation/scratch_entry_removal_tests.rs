//! What:
//!  How a scratch entry is removed within its ceiling.
//! Why:
//!  A removal that waits for a stalled device is what the commit latency record measured,
//!  and one that removes a name it did not make,
//!  or a tree a live command still owns,
//!  would destroy work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(removeWithin(tree, 1)).toBe('unfinished');
//! ```

/// The rules under control.
use super::{Removal, SCRATCH_REMOVAL_CEILING, remove_within, unfinished_removals};
/// Holding the process's places for blocking work to one,
/// so a stalled removal has nowhere to go.
use crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to;
/// The record's count of removals that did not finish.
use crate::time_spent_outside_the_phases::outside_the_phases;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A removal of an entry that is already gone is finished,
/// a removal that is refused says so,
/// and a removal that cannot finish within its ceiling is counted as unfinished.
#[test]
fn a_removal_says_what_became_of_it() {
    let root: PathBuf = fixture("scratch-removal-outcomes");
    std::fs::create_dir_all(root.as_path()).expect("fixture");
    assert!(remove_within(root.join("absent").as_path(), SCRATCH_REMOVAL_CEILING).finished(), "an entry that is gone is removed");

    let sealed: PathBuf = root.join("sealed");
    let inside: PathBuf = sealed.join("tree");
    std::fs::create_dir_all(&inside).expect("tree");
    std::fs::write(inside.join("file"), b"x").expect("a file in the tree");
    std::fs::set_permissions(sealed.as_path(), std::fs::Permissions::from_mode(0o500)).expect("seal");
    let refused: Removal = remove_within(inside.as_path(), SCRATCH_REMOVAL_CEILING);
    std::fs::set_permissions(sealed.as_path(), std::fs::Permissions::from_mode(0o700)).expect("unseal");
    assert!(matches!(&refused, Removal::Failed(_)), "a refusal is named: {refused:?}");
    assert!(inside.exists(), "a refused removal leaves the tree for the next sweep");

    // A tree of many entries removed within a microsecond cannot finish, and the record counts it.
    let many: PathBuf = root.join("many");
    std::fs::create_dir_all(&many).expect("tree");
    for at in 0..500 {
        std::fs::write(many.join(format!("entry-{at}")), b"x").expect("an entry");
    }
    let before: u64 = outside_the_phases().removals_that_did_not_finish;
    let late: Removal = remove_within(many.as_path(), Duration::from_micros(1));
    assert!(matches!(&late, Removal::Unfinished), "a removal that outlasts its ceiling says so: {late:?}");
    assert_eq!(outside_the_phases().removals_that_did_not_finish, before + 1, "the record counts it");
    remove(root.as_path());
}

/// A tree of `entries` files,
/// whose removal takes longer than a microsecond ceiling.
fn a_tree_of(root: &Path, name: &str, entries: usize) -> PathBuf {
    let tree: PathBuf = root.join(name);
    std::fs::create_dir_all(&tree).expect("tree");
    for at in 0..entries {
        std::fs::write(tree.join(format!("entry-{at}")), b"x").expect("an entry");
    }
    return tree;
}

/// A removal that outlasts its ceiling keeps its thread,
/// so the number of places those threads may take is bounded:
///  a removal that finds every place held takes none,
///  returns at once and leaves its entry to a later sweep,
///  and a place comes back when the thread that took it ends.
#[test]
fn removals_that_outlast_their_ceiling_hold_a_bounded_number_of_places() {
    // One place, set before any blocking job of this process could have made the budget without the cap.
    hold_blocking_jobs_to(1);
    let root: PathBuf = fixture("scratch-removal-places");
    let first: PathBuf = a_tree_of(root.as_path(), "first", 500);
    let second: PathBuf = a_tree_of(root.as_path(), "second", 500);
    let began: Instant = Instant::now();
    let late: Removal = remove_within(first.as_path(), Duration::from_micros(1));
    assert!(matches!(&late, Removal::Unfinished), "a removal that outlasts its ceiling says so: {late:?}");
    assert!(unfinished_removals() >= 1, "the stalled removal holds a place");
    let refused: Removal = remove_within(second.as_path(), SCRATCH_REMOVAL_CEILING);
    assert!(matches!(&refused, Removal::Unfinished), "a removal that finds every place held says so: {refused:?}");
    assert!(second.exists(), "an entry no place was free for is left to a later sweep");
    assert!(began.elapsed() < SCRATCH_REMOVAL_CEILING, "a removal with no place does not wait for one");
    // The place comes back with the thread that took it.
    let waited: Instant = Instant::now();
    while unfinished_removals() > 0 {
        assert!(waited.elapsed() < Duration::from_secs(30), "the stalled removal never gave its place back");
        std::thread::sleep(Duration::from_millis(10));
    }
    let again: Removal = remove_within(second.as_path(), SCRATCH_REMOVAL_CEILING);
    assert!(again.finished(), "the place came back with the thread: {again:?}");
    assert!(!second.exists(), "the entry goes once a place is free");
    remove(root.as_path());
}
