//! Controls for the lines the isolated merge's apply writes when another program holds the index.

use super::*;

/// What:
///  The line for a merge that met another program's `index.lock` for longer than the wait's ceiling names the lock,
///  the wait,
///  what was not applied and what becomes of the merge.
/// Why:
///  The refusal used to name a cause that never happened ("the branch moved or the index lock was taken in 20 merges
///  in a row") and advise a command that could not work,
/// so a person could not tell what was wrong
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aHeldIndexFinding('feature', '/r/.git/index.lock', 60s)).toContain('index.lock');
/// ```
#[test]
fn a_held_index_line_says_what_blocked_the_merge_and_what_becomes_of_it() {
    let said: String = a_held_index_finding("feature", Path::new("/r/.git/index.lock"), Duration::from_secs(60));
    assert!(said.starts_with("monochromatic-git-policy-cli: git merge feature: "), "{said}");
    assert!(said.contains("/r/.git/index.lock"), "the lock is named: {said}");
    assert!(said.contains("for 60 seconds"), "the wait is named: {said}");
    assert!(said.contains("was not applied") && said.contains("the branch did not move"), "{said}");
    assert!(said.contains("the next git command in this worktree finishes it"), "what becomes of the merge: {said}");
    assert!(said.contains("If no program is using this repository"), "the way out of a lock nothing releases: {said}");
    assert!(!said.contains("branch moved"), "it does not name a cause that never happened: {said}");
    assert!(!said.contains("Run the merge again"), "it does not advise a command that cannot work: {said}");
}
