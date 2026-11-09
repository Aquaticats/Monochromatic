//! Controls for the lines the isolated merge's apply writes when another program holds the index.

use super::*;

/// What:
///  A merge that meets an index lock it may not clear says where its result is,
///  that the checkout is still to be finished,
///  what finishes it and what happens to the person's own edits meanwhile.
/// Why:
///  The command exits 0 without the merge in the checkout,
/// so the line must say that plainly
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 3;
/// the owner's answer for a live foreign
///  `index.lock` at `SPEC.md:2187`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aDeferredSaveFinding('feature', lock, 'refs/cli-git/kept/17', Some(60s))).toContain('refs/cli-git/kept/17');
/// ```
#[test]
fn the_line_of_a_saved_merge_names_the_side_ref_and_what_finishes_the_checkout() {
    let said: String =
        a_deferred_save_finding("feature", Path::new("/r/.git/index.lock"), "refs/cli-git/kept/1794200000000", Some(Duration::from_secs(60)));
    assert!(said.starts_with("monochromatic-git-policy-cli: git merge feature: "), "{said}");
    assert!(said.contains("/r/.git/index.lock"), "the lock is named: {said}");
    assert!(said.contains("did not let go within 60 seconds"), "the wait is named: {said}");
    assert!(said.contains("saved on refs/cli-git/kept/1794200000000"), "the side ref is named: {said}");
    assert!(said.contains("exits 0") && said.contains("branch has not moved"), "the state is stated: {said}");
    assert!(said.contains("the next git command in this worktree finishes the checkout"), "{said}");
    assert!(said.contains("edits you make meanwhile to files the merge does not change are kept"), "{said}");
    assert!(said.contains("If no program is using this repository"), "the way out of a lock nothing releases: {said}");
    // A lock this command may not clear at all waited no ceiling, so no seconds are claimed.
    let untouched: String = a_deferred_save_finding("feature", Path::new("/r/.git/index.lock"), "refs/cli-git/kept/1", None);
    assert!(untouched.contains("is not one this command may clear"), "{untouched}");
    assert!(!untouched.contains("seconds"), "no wait is claimed: {untouched}");
}

/// What:
///  A merge whose commit could not be kept on a side ref is refused,
///  with nothing changed,
///  and never promises a later finish it may not reach.
/// Why:
///  Without the name the commit can be collected,
/// so the merge must not exit 0 claiming it is saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aMergeThatCouldNotBeSaved('feature', lock)).toContain('could not be kept on a side ref');
/// ```
#[test]
fn the_line_of_a_merge_that_could_not_be_saved_changes_nothing() {
    let said: String = a_merge_that_could_not_be_saved("feature", Path::new("/r/.git/index.lock"));
    assert!(said.contains("/r/.git/index.lock"), "{said}");
    assert!(said.contains("could not be kept on a side ref"), "{said}");
    assert!(said.contains("Nothing was changed"), "{said}");
    assert!(said.contains("Run the merge again"), "{said}");
    assert!(!said.contains("the next git command"), "it promises no finish: {said}");
}
