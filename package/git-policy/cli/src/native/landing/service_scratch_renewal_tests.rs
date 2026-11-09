//! What:
//!  The renewal of a commit's scratch tree:
//!  that it keeps a tree fresh while the command lives,
//!  and that it stops when the command lets go.
//! Why:
//!  A renewal that never runs lets the sweep take a live tree,
//!  and one that never stops keeps a killed command's tree alive after it is gone,
//!  which is the pile the abandonment rule exists to collect.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(ageOf(tree)).toBeLessThan(1000);
//! ```

/// The renewal under control.
use super::ScratchRenewal;
use crate::scratch_location::SCRATCH_STALE_AFTER;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The pace a control renews at,
/// short enough that a test ends in a moment.
const QUICK: Duration = Duration::from_millis(20);

/// How long a control waits for a renewal to happen.
const PATIENCE: Duration = Duration::from_secs(15);

/// How old `path` is now.
fn age_of(path: &Path) -> Duration {
    let modified: SystemTime = std::fs::symlink_metadata(path).expect("metadata").modified().expect("a time");
    return SystemTime::now().duration_since(modified).unwrap_or_default();
}

/// A tree that starts an age old loses that age while the renewal lives,
/// and a tree whose renewal went ages again.
#[test]
fn a_live_tree_is_kept_fresh_and_a_released_one_is_not() {
    let root: PathBuf = fixture("scratch-renewal");
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(&tree).expect("tree");
    std::fs::File::open(&tree).expect("open").set_modified(SystemTime::now() - SCRATCH_STALE_AFTER).expect("age it");
    let renewal: ScratchRenewal = ScratchRenewal::start(tree.as_path(), QUICK);
    let began: std::time::Instant = std::time::Instant::now();
    while age_of(tree.as_path()) > SCRATCH_STALE_AFTER / 2 {
        assert!(began.elapsed() < PATIENCE, "the renewal never came: {:?}", age_of(tree.as_path()));
        std::thread::sleep(QUICK);
    }
    drop(renewal);
    // The tree ages again from the moment the renewal went.
    let released: Duration = age_of(tree.as_path());
    std::thread::sleep(QUICK * 3);
    assert!(age_of(tree.as_path()) > released, "nothing renews a released tree");
    remove(root.as_path());
}
