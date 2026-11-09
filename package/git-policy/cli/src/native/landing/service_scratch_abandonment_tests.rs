//! What:
//!  Which commit scratch trees a later command removes:
//!  the ones whose marker names a process that is gone,
//!  the ones no marker was ever written for once they are old,
//!  and nothing else.
//! Why:
//!  A rule that removed a live command's tree would destroy the private indexes and the typed message of a commit that is
//!  still running,
//! and one that removed nothing would let a killed command's trees pile up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(removeAbandonedTrees(directory)).toBe(1);
//! ```

/// The rules under control.
use super::{OWNER_MARKER, SCRATCH_TREES_REMOVED_PER_COMMAND, UNMARKED_SCRATCH_TREE_AGE, mark_the_owner, remove_abandoned_trees};
use crate::service_index_marker::{holder_is_gone, marker_of, parse_marker};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Set an entry's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path).expect("open").set_modified(SystemTime::now() - back).expect("set modification time");
}

/// One scratch tree of the wrapper's own name,
/// holding a file.
fn tree(directory: &Path, identity: &str) -> PathBuf {
    let path: PathBuf = directory.join(format!("monochromatic-git-policy-cli-service-{identity}"));
    std::fs::create_dir_all(&path).expect("tree");
    std::fs::write(path.join("commit.index"), b"index").expect("a file in the tree");
    return path;
}

/// A tree this command marks belongs to it,
/// so a later command leaves it alone however old it is,
/// and a tree marked by a process that is gone is removed at once.
#[test]
fn a_tree_whose_owner_runs_is_kept_and_one_whose_owner_is_gone_goes() {
    let root: PathBuf = fixture("scratch-abandonment");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let mine: PathBuf = tree(directory.as_path(), "mine");
    mark_the_owner(mine.as_path());
    age(mine.as_path(), UNMARKED_SCRATCH_TREE_AGE * 2);
    let marker = marker_of(mine.join(OWNER_MARKER).as_path()).expect("the marker this command wrote");
    assert!(!holder_is_gone(marker.holder.as_ref()), "the marker names this live process");
    assert_eq!(remove_abandoned_trees(directory.as_path()), 0);
    assert!(mine.exists(), "a live command's tree is kept however old it is");

    // A tree marked by a process that cannot be running: the number leaf carries no process, so a marker naming it names a
    //  holder that is gone.
    let dead: PathBuf = tree(directory.as_path(), "dead");
    let text: String = String::from("cli-git-service v2 0 0 1.1:0\n");
    std::fs::write(dead.join(OWNER_MARKER), text.as_bytes()).expect("a marker of a process that is gone");
    let parsed: Option<crate::service_index_marker::Marked> = parse_marker(text.as_str());
    assert!(parsed.is_some(), "a control needs a marker the reader parses");
    assert!(holder_is_gone(parsed.expect("parsed").holder.as_ref()), "the number leaf names no live process");
    assert_eq!(remove_abandoned_trees(directory.as_path()), 1);
    assert!(!dead.exists(), "a tree of a process that is gone goes");
    assert!(mine.exists(), "the live command's tree stays");
    remove(root.as_path());
}

/// A tree no marker was written for goes once it is old,
/// and a fresh one and a name that is not a tree's are left alone.
#[test]
fn an_unmarked_tree_goes_by_age_and_a_fresh_one_stays() {
    let root: PathBuf = fixture("scratch-abandonment-age");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let old: PathBuf = tree(directory.as_path(), "old");
    let fresh: PathBuf = tree(directory.as_path(), "fresh");
    let foreign: PathBuf = directory.join("notes");
    std::fs::create_dir_all(&foreign).expect("another directory");
    age(old.as_path(), UNMARKED_SCRATCH_TREE_AGE + Duration::from_secs(60));
    assert_eq!(remove_abandoned_trees(directory.as_path()), 1);
    assert!(!old.exists(), "an old tree with no marker goes");
    assert!(fresh.exists(), "a fresh tree may be a command's own");
    assert!(foreign.exists(), "a name that is not a commit scratch tree's is left alone");
    remove(root.as_path());
}

/// One command removes at most the number of trees it is given,
/// so collecting them never becomes the work the command does.
#[test]
fn one_command_removes_only_the_trees_it_is_told_to() {
    let root: PathBuf = fixture("scratch-abandonment-bound");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let aged: Vec<PathBuf> = (0..3).map(|at| { return tree(directory.as_path(), format!("old-{at}").as_str()); }).collect();
    for path in &aged {
        age(path.as_path(), UNMARKED_SCRATCH_TREE_AGE + Duration::from_secs(60));
    }
    assert_eq!(remove_abandoned_trees(directory.as_path()), SCRATCH_TREES_REMOVED_PER_COMMAND.min(3));
    assert_eq!(aged.iter().filter(|path| { return path.exists(); }).count(), 0, "three fit inside the bound");
    remove(root.as_path());
}
