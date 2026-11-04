//! What:
//!  The controls of R8-8 for merges: refusal after the checkout with a concurrent edit, and a lost swap.
//! Why:
//!  Revision 8 leaves the checkout in place after a refusal and a checkout for a stale base after a lost swap; revision 9's MG6
//!  and MG5 close both without losing the person's edit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision eight: a refused merge leaves its checkout', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_merge::{Rules, check, end_state, initial, invariant, steps};

/// The steps to a checkout with a concurrent edit, shared by the tests.
const CHECKED_OUT_EDITED: [&str; 3] = [
    "MG1 compute: computes the merge M",
    "MG3 check out: installs M in the worktree under the marker",
    "person: saves an edit to the merged file after the checkout",
];

/// Revision 8 (red): the hook refuses after the checkout and nothing restores the worktree.
#[test]
fn eight_leaves_the_checkout_in_place_after_a_refusal() {
    let mut schedule: Vec<&str> = CHECKED_OUT_EDITED.to_vec();
    schedule.push("environment: a person's hook refuses the swap");
    let reached = replay(initial(Rules::Eight), steps, &schedule);
    assert_eq!(invariant(&reached), Some("a refused merge left its checkout in the worktree".to_string()));
}

/// Revision 8 (red): the swap is lost after the checkout and nothing retargets it.
#[test]
fn eight_leaves_a_checkout_for_a_stale_base_after_a_lost_swap() {
    let reached = replay(
        initial(Rules::Eight),
        steps,
        &[
            "MG1 compute: computes the merge M",
            "MG3 check out: installs M in the worktree under the marker",
            "foreign: moves the tip",
            "MG4 swap: the swap is lost because the tip moved, P4",
        ],
    );
    assert!(end_state(&reached).is_some());
}

/// Revision 9 (green): MG6 restores and keeps the edit, MG5 retargets, in every interleaving.
#[test]
fn nine_restores_after_a_refusal_and_retargets_after_a_lost_swap() {
    let mut schedule: Vec<&str> = CHECKED_OUT_EDITED.to_vec();
    schedule.push("environment: a person's hook refuses the swap");
    schedule.push("MG6 restore: installs the old tip's content with M as the old side and keeps every displaced inode");
    let restored = replay(initial(Rules::Nine), steps, &schedule);
    assert!(restored.edit_saved && restored.edit_in_worktree);
    assert_eq!(invariant(&restored), None);
    let report: Report = check(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("merge nine: {} states, deepest {}", report.states, report.deepest);
    assert!(check(Rules::Eight).violation.is_some());
}
