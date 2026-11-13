//! What:
//!  The three answers a bounded metadata lookup gives:
//!  the entry's own metadata,
//!  the lookup's error,
//!  and no answer at the ceiling.
//! Why:
//!  The guard tells a file from a directory and compares a marked edit with its index entry by
//!  metadata,
//!  so an entry that is there and a path that is not must stay distinguishable,
//!  and neither may be confused with a lookup still waiting at the ceiling,
//!  which says nothing about the worktree.
//!  The ceiling itself is controlled in
//!  `work_awaited_within_a_ceiling_tests.rs`,
//!  where a job that never answers is held past the ceiling on purpose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // symlinkMetadataWithin(5s, 'link') === { ok: true, metadata }  // the link itself
//! // symlinkMetadataWithin(5s, 'gone') === { ok: false, error: ENOENT }
//! ```

use super::symlink_metadata_within;
use crate::command_test_support::{fixture, remove};
use std::path::PathBuf;
use std::time::Duration;

/// The ceiling a control uses when the answer is expected:
/// long enough that only a stopped mount could miss it.
const CEILING: Duration = Duration::from_secs(30);

/// An entry that is there answers with its own metadata,
/// a link answers for the link and not for what it names,
/// and a path that is gone answers with the lookup's own error.
#[test]
fn an_entry_is_its_own_answer_and_a_link_is_not_followed() {
    let root: PathBuf = fixture("metadata-within-a-ceiling");
    std::fs::write(root.join("a.txt"), b"one\n").expect("the file");
    std::fs::create_dir(root.join("nested")).expect("the directory");
    std::os::unix::fs::symlink(root.join("nested"), root.join("link")).expect("the link");
    let file = symlink_metadata_within(CEILING, root.join("a.txt").as_path()).expect("the file answers");
    assert!(file.expect("the metadata").file_type().is_file());
    let directory = symlink_metadata_within(CEILING, root.join("nested").as_path()).expect("the directory answers");
    assert!(directory.expect("the metadata").file_type().is_dir());
    let link = symlink_metadata_within(CEILING, root.join("link").as_path()).expect("the link answers");
    assert!(link.expect("the metadata").file_type().is_symlink(), "the link is the entry, not what it names");
    let gone = symlink_metadata_within(CEILING, root.join("gone").as_path()).expect("a missing path still answers");
    assert_eq!(
        gone.expect_err("nothing is there").kind(),
        std::io::ErrorKind::NotFound,
        "a path that is not there is the lookup's NotFound, not an entry and not a wait"
    );
    remove(root.as_path());
}
