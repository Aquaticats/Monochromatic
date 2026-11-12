//! What:
//!  Which scratch entries a command may remove:
//!  a commit tree only once nothing that named it can still be reading it,
//!  and everything else of a scratch place by the age its caller gives.
//! Why:
//!  A rule that removed a recent tree would take the private index and the typed message of a command that is
//!  still running,
//! and of a landing that a killed client left queued;
//!  one that removed nothing would let a killed command's trees pile up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(mayGo(tree, metadata, now, day)).toBe(false);
//! ```

/// The rule under control,
/// and the record of the trees this service's requests hold.
use super::{SCRATCH_TREE_ABANDONED_AFTER, may_go};
use crate::service_scratch_trees_of_queued_requests::{note, release};
use crate::service_index_marker::{LOCK_MARKER, MARKER_VERSION};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Set an entry's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path).expect("open").set_modified(SystemTime::now() - back).expect("set modification time");
}

/// A commit tree of the wrapper's own name,
/// holding the private index.
fn tree(directory: &Path, identity: &str) -> PathBuf {
    let path: PathBuf = directory.join(format!("monochromatic-git-policy-cli-service-{identity}"));
    std::fs::create_dir_all(&path).expect("tree");
    std::fs::write(path.join("commit.index"), b"index").expect("a file in the tree");
    return path;
}

/// Whether `path` may go by the age the sweep of a repository scratch directory gives its other entries.
fn may_go_by_the_day(path: &Path) -> bool {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("metadata");
    return may_go(path, &metadata, SystemTime::now(), crate::scratch_location::SCRATCH_STALE_AFTER);
}

/// The identity the control's request is queued under.
const A_QUEUED_REQUEST: &str = "scratch-abandonment-queued-request";

/// A tree a request this service still holds is kept however old it is,
/// and goes by the age rule once the service is done with that request.
/// Why:
///  A client killed while its request is queued renews nothing,
///  so the age a collection sees of the tree,
///  not the liveness of a process,
///  is what it judges;
///  the landing that request waits for reads the private index and the conclusion copies in that tree,
///  and losing them leaves the real conclusion state where the next commit concludes the same merge again.
#[test]
fn a_tree_a_queued_request_holds_is_kept_past_the_age_rule() {
    let root: PathBuf = fixture("scratch-abandonment-held");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let held: PathBuf = tree(directory.as_path(), "held1");
    age(held.as_path(), SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    note(A_QUEUED_REQUEST, held.as_path());
    assert!(!may_go_by_the_day(held.as_path()), "the landing a killed client left queued has still to read this tree");
    release(A_QUEUED_REQUEST);
    assert!(may_go_by_the_day(held.as_path()), "once the request is over the tree goes by its age");
    remove(root.as_path());
}

/// A recent commit tree is kept however it is marked,
/// because a landing a killed client left queued may still be reading it;
/// an old one goes,
/// and so does anything else of the place once it reaches the caller's age.
#[test]
fn a_tree_goes_when_nothing_that_named_it_can_still_read_it() {
    let root: PathBuf = fixture("scratch-abandonment");
    let directory: PathBuf = root.join("scratch");
    std::fs::create_dir_all(&directory).expect("scratch directory");
    let recent: PathBuf = tree(directory.as_path(), "recent1");
    assert!(!may_go_by_the_day(recent.as_path()), "a tree written a moment ago may still be read by a landing");
    // The marker of a process that is gone is no evidence at all: what a request still names does not depend on it.
    let gone: String = format!("{LOCK_MARKER}{MARKER_VERSION}0 0 1.1:0\n");
    std::fs::write(recent.join("owner"), gone.as_bytes()).expect("a marker naming a process that is gone");
    assert!(!may_go_by_the_day(recent.as_path()), "a marker neither adds nor removes a reason to keep the tree");
    age(recent.as_path(), SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    assert!(may_go_by_the_day(recent.as_path()), "a tree no command has written for two hours goes");
    // A tree one second inside the age stays.
    let fresh: PathBuf = tree(directory.as_path(), "fresh1");
    age(fresh.as_path(), SCRATCH_TREE_ABANDONED_AFTER - Duration::from_secs(1));
    assert!(!may_go_by_the_day(fresh.as_path()), "an entry exactly inside its age is kept");
    // An entry that is not a commit tree's goes by the age its caller gives it, which here is a day.
    let other: PathBuf = directory.join("cli-git-markdown-stale.jsonc");
    std::fs::write(other.as_path(), b"[]").expect("an entry of the place");
    age(other.as_path(), SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    assert!(!may_go_by_the_day(other.as_path()), "an entry of the place is kept until the age the caller gives");
    age(other.as_path(), crate::scratch_location::SCRATCH_STALE_AFTER + Duration::from_secs(60));
    assert!(may_go_by_the_day(other.as_path()), "and goes at that age");
    remove(root.as_path());
}
