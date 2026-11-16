//! What:
//!  Controls for the line a pruned kept reference leaves in the repository's log.
//! Why:
//!  The reference was the only handle on a commit no branch reaches,
//!  so the line is what a person has afterwards,
//!  and a line that leaves out the commit,
//!  the worktree it was built in,
//!  its age or the reason says nothing they can act on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(lineOf(pruned, now, limits)).toContain('"worktree":"/a worktree"');
//! ```

use super::{PRUNED_KEPT_REFERENCE_LOG, line_of, log_path};
use crate::kept_reference_limits::{KEPT_REFERENCE_FLOOR, KEPT_REFERENCE_MAX_AGE, KEPT_REFERENCE_MAX_BYTES, KeptReferenceLimits};
use crate::kept_reference_pass::{Pruned, WhyPruned};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The limits of the release,
/// as one pass reads them.
fn the_limits() -> KeptReferenceLimits {
    return KeptReferenceLimits { max_age: KEPT_REFERENCE_MAX_AGE, max_bytes: KEPT_REFERENCE_MAX_BYTES, floor: KEPT_REFERENCE_FLOOR };
}

/// One pruned reference that reached no branch.
fn a_pruning(why: WhyPruned) -> Pruned {
    return Pruned {
        reference: String::from("refs/cli-git/kept/1794071191963902"),
        commit: String::from("deb10b333be2e7c350d4d5cce35b8d474161fbbc"),
        batch: 1794071191963902,
        rescued: false,
        worktree: Some(String::from("/a worktree/with a space")),
        kept_at_ms: 1_791_000_000_000,
        age_ms: 6 * 24 * 60 * 60 * 1000,
        why,
    };
}

/// The line carries the commit,
/// the reference,
/// the age,
/// the worktree and batch it came from,
/// and why it went.
#[test]
fn the_line_carries_what_a_person_needs_to_find_a_commit_that_went() {
    let line: String = line_of(&a_pruning(WhyPruned::TooOld), "pruned", 1_791_500_000_000, the_limits());
    let parsed: Value = serde_json::from_str(line.trim()).expect("one JSON line");
    assert_eq!(parsed["reference"], "refs/cli-git/kept/1794071191963902");
    assert_eq!(parsed["commit"], "deb10b333be2e7c350d4d5cce35b8d474161fbbc");
    assert_eq!(parsed["batch"], 1_794_071_191_963_902_u64);
    assert_eq!(parsed["worktree"], "/a worktree/with a space");
    assert_eq!(parsed["age_seconds"], 6 * 24 * 60 * 60);
    assert_eq!(parsed["why"], "too-old");
    assert_eq!(parsed["outcome"], "pruned");
    assert!(parsed["why_words"].as_str().is_some_and(|words| { return words.contains("reached no branch"); }), "{line}");
    assert!(parsed["time"].as_str().is_some_and(|time| { return time.ends_with('Z'); }), "{line}");
}

/// Each of the three reasons says whether anything was lost.
#[test]
fn each_reason_says_whether_the_commit_was_lost() {
    let reached: String = line_of(&a_pruning(WhyPruned::ReachedABranch), "pruned", 1_791_500_000_000, the_limits());
    assert!(reached.contains("so nothing was lost"), "{reached}");
    let too_much: String = line_of(&a_pruning(WhyPruned::TooMuchKept), "pruned", 1_791_500_000_000, the_limits());
    assert!(too_much.contains("weighed more than"), "{too_much}");
}

/// The log lives in the common Git directory,
/// beside the wrapper's other logs.
#[test]
fn the_log_sits_in_the_common_git_directory() {
    let common: &Path = Path::new("/a repository/.git");
    let expected: PathBuf = common.join(PRUNED_KEPT_REFERENCE_LOG);
    assert_eq!(log_path(common), expected);
    assert_eq!(log_path(common).file_name().and_then(|name| { return name.to_str(); }), Some(PRUNED_KEPT_REFERENCE_LOG));
}
