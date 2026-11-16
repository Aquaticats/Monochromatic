//! What:
//!  Controls for which reference names count as a kept commit.
//! Why:
//!  The pruning removes references,
//!  and `refs/cli-git/` holds the carried work of an isolated merge and a private fetch of a push,
//!  so a name the wrapper never keeps a commit on must never be one the pruning answers for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isAKeptReference('refs/cli-git/carried/7')).toBe(false);
//! ```

use super::{batch_and_rescued_of, is_a_kept_reference};

/// The batch's own name and the name beside it are both kept references,
/// and each carries its batch.
#[test]
fn the_two_forms_a_commit_is_kept_on_are_both_kept_references() {
    assert_eq!(batch_and_rescued_of("refs/cli-git/kept/1794071191963902"), Some((1794071191963902, false)));
    assert_eq!(batch_and_rescued_of("refs/cli-git/kept/1794071191963902-rescue"), Some((1794071191963902, true)));
    assert!(is_a_kept_reference("refs/cli-git/kept/7"));
    assert!(is_a_kept_reference("refs/cli-git/kept/7-rescue"));
}

/// Every other name under `refs/cli-git/` is no kept reference:
/// the carried work of a merge,
/// a private fetch of a push,
/// a name that is no number,
/// and a name another folder deep.
#[test]
fn what_else_lives_under_the_wrapper_refs_is_no_kept_reference() {
    for name in [
        "refs/cli-git/carried/7",
        "refs/cli-git/auto-push/fetch-4242",
        "refs/cli-git/kept/",
        "refs/cli-git/kept/seven",
        "refs/cli-git/kept/7-other",
        "refs/cli-git/kept/7/another",
        "refs/cli-git/kept/-rescue",
        "refs/cli-git/kept/18446744073709551616",
        "refs/heads/kept/7",
        "",
    ] {
        assert!(!is_a_kept_reference(name), "{name:?} is no kept reference");
        assert_eq!(batch_and_rescued_of(name), None, "{name:?} names no batch");
    }
}
