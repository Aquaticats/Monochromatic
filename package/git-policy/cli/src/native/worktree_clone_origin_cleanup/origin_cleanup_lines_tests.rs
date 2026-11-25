//! What:
//!  The controls of the cleanup's lines:
//!  a destruction's line,
//!  a block's line,
//!  and the rule that a block speaks only when its reason changed.
//! Why:
//!  The project decided one line when it destroys and one when a reason changes,
//!  and silence while nothing changed,
//!  so the rule is a value and not a caller's choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theReasonChanged(previous, next)).toBe(true);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::an_owed_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_lines::{line_of_a_block, line_of_a_destruction, the_reason_changed};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{CleanupBlock, OriginCleanupRecord};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use std::path::Path;

#[test]
fn a_destruction_names_the_snapshot_and_what_the_checkout_kept() {
    let line: String = line_of_a_destruction("tank/mono-checkout@git-worktree-clone-origin-00");
    assert!(line.contains("destroyed the snapshot `tank/mono-checkout@git-worktree-clone-origin-00`"));
    assert!(line.contains("the checkout keeps every other snapshot it holds"));
}

#[test]
fn a_block_names_the_reason_the_detail_and_the_record() {
    let blocked: OriginCleanupRecord = an_owed_record().blocked(CleanupBlock { reason: BlockReason::Dependency, errno: 0, clones: 1, holds: 0 }, "2026-11-25T09:15:01-05:00");
    let line: String = line_of_a_block(&blocked, Path::new("/repo/.git/monochromatic-git-policy-cli/origin-cleanups/git-worktree-clone-origin-00"), "1 clone(s) and 0 hold(s) pin the snapshot");
    assert!(line.contains("it is still pinned"));
    assert!(line.contains("1 clone(s) and 0 hold(s)"));
    assert!(line.contains("nothing expires it"));
}

#[test]
fn a_block_speaks_only_when_its_reason_or_its_detail_changed() {
    let owed: OriginCleanupRecord = an_owed_record();
    let one: OriginCleanupRecord = owed.blocked(CleanupBlock { reason: BlockReason::Dependency, errno: 0, clones: 1, holds: 0 }, "2026-11-25T09:15:01-05:00");
    let same: OriginCleanupRecord = owed.blocked(CleanupBlock { reason: BlockReason::Dependency, errno: 0, clones: 1, holds: 0 }, "2026-11-25T09:20:01-05:00");
    let more: OriginCleanupRecord = owed.blocked(CleanupBlock { reason: BlockReason::Dependency, errno: 0, clones: 2, holds: 0 }, "2026-11-25T09:25:01-05:00");
    assert!(the_reason_changed(Some(&owed), &one), "an owed record that becomes blocked speaks");
    assert!(!the_reason_changed(Some(&one), &same), "the same counts read again say nothing");
    assert!(the_reason_changed(Some(&one), &more), "one clone becoming two speaks");
}
