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

/// What:
///  A path is followed to its target when the caller asks through links,
///  and a lookup that does not answer within its ceiling answers nothing.
/// Why:
///  The bounded lookup is what keeps a command from waiting on a mount whose server stopped
///  answering,
///  and the two readings differ for a link:
///  `symlink_metadata_within` reports the link itself and `metadata_through_links_within` the
///  directory it names.
///  A control that stalls the job shows the second answer:
///  no answer, never "not a directory".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// metadataThroughLinksWithin(30s, 'link') === { ok: true, metadata: { isDirectory: true } }
/// workWithin(100ms, () => sleep(5s)) === undefined
/// ```
#[test]
fn a_path_is_followed_when_the_caller_asks_through_links() {
    let root: PathBuf = fixture("metadata-through-links-within");
    std::fs::create_dir(root.join("nested")).expect("the directory");
    std::os::unix::fs::symlink(root.join("nested"), root.join("link")).expect("the link");
    let followed = super::metadata_through_links_within(CEILING, root.join("link").as_path()).expect("the link answers");
    assert!(
        followed.expect("the metadata").file_type().is_dir(),
        "the target of the link is what a directory question asks about"
    );
    let entry = symlink_metadata_within(CEILING, root.join("link").as_path()).expect("the entry answers");
    assert!(entry.expect("the metadata").file_type().is_symlink(), "the entry itself is still the link");
    remove(root.as_path());
}

/// What:
///  A metadata read held past its ceiling answers nothing and returns at the ceiling.
/// Why:
///  A stalled mount is not going to answer,
///  and a guard that waited for it would hold the command with no end;
///  the ceiling is what turns that into an answer the caller refuses on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = Date.now(); workWithin(100ms, () => sleep(5s)); Date.now() - started < 2s
/// ```
#[test]
fn a_read_held_past_its_ceiling_answers_nothing() {
    let ceiling = Duration::from_millis(100);
    let started = std::time::Instant::now();
    let answer: Option<std::io::Result<std::fs::Metadata>> =
        crate::work_awaited_within_a_ceiling::work_within(ceiling, || {
            std::thread::sleep(Duration::from_secs(5));
            return std::fs::metadata("/");
        });
    let waited = started.elapsed();
    assert!(answer.is_none(), "a read that did not answer is no answer");
    assert!(waited < Duration::from_secs(2), "the answer came at the ceiling, not after {waited:?}");
}
