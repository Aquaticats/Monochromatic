//! What:
//!  Controls for the text `git monochromatic-git-policy-cli kept-references` prints.
//! Why:
//!  A person reads the listing to find a commit they gave up on,
//!  and to know what the next pruning takes before it goes,
//!  so every block has to name the commit,
//!  the age,
//!  the worktree and the batch,
//!  and the verdict has to be one of the two sentences a pass means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theListing(pass, limits)).toContain(`${PRUNED_NEXT} it is older than the 30-day age limit`);
//! ```

use super::{KEPT_NEXT, PRUNED_NEXT, the_listing, the_pruning_report};
use crate::kept_reference_limits::{KEPT_REFERENCE_FLOOR, KEPT_REFERENCE_MAX_AGE, KEPT_REFERENCE_MAX_BYTES, KeptReferenceLimits};
use crate::kept_reference_pass::{Kept, Pruned, WhyNotPruned, WhyPruned};
use crate::kept_reference_pass_answers::ThePass;
use std::path::Path;

/// The limits of the release.
fn the_limits() -> KeptReferenceLimits {
    return KeptReferenceLimits { max_age: KEPT_REFERENCE_MAX_AGE, max_bytes: KEPT_REFERENCE_MAX_BYTES, floor: KEPT_REFERENCE_FLOOR };
}

/// A pass that prunes one reference and keeps another.
fn a_pass() -> ThePass {
    return ThePass {
        pruned: vec![Pruned {
            reference: String::from("refs/cli-git/kept/1794071191963902"),
            commit: String::from("deb10b333be2e7c350d4d5cce35b8d474161fbbc"),
            batch: 1794071191963902,
            rescued: false,
            worktree: Some(String::from("/a worktree/with a space")),
            kept_at_ms: 1_791_000_000_000,
            age_ms: 31 * 24 * 60 * 60 * 1000,
            why: WhyPruned::TooOld,
        }],
        kept: vec![Kept {
            reference: String::from("refs/cli-git/kept/1794071191963903-rescue"),
            commit: String::from("4abc7d6a39b8f76b741705ae772910419412a60e"),
            batch: 1794071191963903,
            rescued: true,
            worktree: None,
            kept_at_ms: 1_791_500_000_000,
            age_ms: 60 * 1000,
            why_not: WhyNotPruned::YoungerThanTheLimit,
        }],
        bytes_before: 7_573,
        bytes_after: 0,
        dry_run: true,
        stopped_early: false,
    };
}

/// Every block names the commit,
/// the age,
/// the worktree and the batch,
/// and says which of the two things the next pass does.
#[test]
fn a_listing_names_what_is_kept_and_what_the_next_pass_takes() {
    let text: String = the_listing(&a_pass(), the_limits());
    assert!(text.starts_with("2 kept references hold objects only they reach, 7573 bytes on disk."), "{text}");
    assert!(text.contains("refs/cli-git/kept/1794071191963902"), "{text}");
    assert!(text.contains("commit deb10b333be2e7c350d4d5cce35b8d474161fbbc"), "{text}");
    assert!(text.contains("31 days 0 hours ago"), "{text}");
    assert!(text.contains("worktree /a worktree/with a space"), "{text}");
    assert!(text.contains("batch 1794071191963902\n"), "{text}");
    assert!(text.contains(PRUNED_NEXT), "{text}");
    assert!(text.contains("it reached no branch and was older than the age limit of 30 days"), "{text}");
    assert!(text.contains("refs/cli-git/kept/1794071191963903-rescue"), "{text}");
    assert!(text.contains(", the name beside the batch's own"), "{text}");
    assert!(text.contains("worktree not recorded"), "{text}");
    assert!(text.contains(KEPT_NEXT), "{text}");
    assert!(text.contains("The next pass prunes 1 of them, leaving 0 bytes."), "{text}");
}

/// A listing of references the pass takes none of says so.
#[test]
fn a_listing_with_nothing_to_prune_says_the_objects_stay() {
    let mut pass: ThePass = a_pass();
    pass.pruned.clear();
    let text: String = the_listing(&pass, the_limits());
    assert!(text.contains("The next pass prunes nothing, so the objects only kept references reach stay at 7573 bytes."), "{text}");
    assert!(!text.contains(PRUNED_NEXT), "{text}");
}

/// The report of a pruning that ran names what went and where the line about it is.
#[test]
fn a_pruning_report_names_where_the_line_is() {
    let mut pass: ThePass = a_pass();
    pass.dry_run = false;
    let text: String = the_pruning_report(&pass, the_limits(), Path::new("/a repository/.git/monochromatic-git-policy-cli-kept-references.jsonl"));
    assert!(text.contains("pruned refs/cli-git/kept/1794071191963902"), "{text}");
    assert!(text.contains("commit deb10b333be2e7c350d4d5cce35b8d474161fbbc"), "{text}");
    assert!(text.contains("/a repository/.git/monochromatic-git-policy-cli-kept-references.jsonl"), "{text}");
}
