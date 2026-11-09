//! What:
//!  How a scratch entry is removed within its ceiling,
//!  and which commit scratch trees a later command takes for abandoned.
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
use super::{COMMIT_SCRATCH_TREE_ABANDONED_AFTER, Removal, SCRATCH_REMOVAL_CEILING, remove_abandoned_commit_scratch_trees, remove_within};
/// The record's count of removals that did not finish.
use crate::time_spent_outside_the_phases::outside_the_phases;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Set an entry's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path).expect("open").set_modified(SystemTime::now() - back).expect("set modification time");
}

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

/// A tree older than the longest a commit can live belongs to no live command,
/// so it goes long before the sweep's day,
/// and a fresh one and a name that is not a tree's are left alone.
#[test]
fn an_abandoned_scratch_tree_goes_before_the_sweeps_day() {
    let root: PathBuf = fixture("scratch-commit-abandoned");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let abandoned: PathBuf = directory.join("monochromatic-git-policy-cli-service-abandoned");
    let fresh: PathBuf = directory.join("monochromatic-git-policy-cli-service-fresh");
    let foreign: PathBuf = directory.join("notes");
    for path in [&abandoned, &fresh] {
        std::fs::create_dir_all(path).expect("tree");
        std::fs::write(path.join("commit.index"), b"index").expect("a file in the tree");
    }
    std::fs::create_dir_all(&foreign).expect("another directory");
    age(abandoned.as_path(), COMMIT_SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    assert_eq!(remove_abandoned_commit_scratch_trees(directory.as_path(), COMMIT_SCRATCH_TREE_ABANDONED_AFTER, 8), 1);
    assert!(!abandoned.exists(), "the abandoned tree goes");
    assert!(fresh.exists(), "a tree a live command may own is kept");
    assert!(foreign.exists(), "a name that is not a commit scratch tree's is left alone");
    remove(root.as_path());
}

/// One command removes at most the number of abandoned trees it is given,
/// so collecting them never becomes the work the command does.
#[test]
fn one_command_removes_only_the_abandoned_trees_it_is_told_to() {
    let root: PathBuf = fixture("scratch-commit-bounded");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let aged: Vec<PathBuf> = (0..3)
        .map(|at| { return directory.join(format!("monochromatic-git-policy-cli-service-abandoned-{at}")); })
        .collect();
    for path in &aged {
        std::fs::create_dir_all(path).expect("tree");
        age(path.as_path(), COMMIT_SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    }
    assert_eq!(remove_abandoned_commit_scratch_trees(directory.as_path(), COMMIT_SCRATCH_TREE_ABANDONED_AFTER, 2), 2);
    assert_eq!(aged.iter().filter(|path| { return path.exists(); }).count(), 1, "one is left for the next command");
    remove(root.as_path());
}
