//! What:
//!  The controls of the seam a removal that a holder stops asks through:
//!  the handler's own rollback words,
//!  and the clause each stop point prints.
//! Why:
//!  The words this file compares against are the mount handler's,
//!  copied from the arm that files them,
//!  so a rename there has to be seen here rather than in a removal that
//!  silently stops refusing to cancel.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isTheHandlersRollback('bind-gone')).toBe(true);
//! ```

use super::{
    THE_ROLLBACK_STEPS,
    TheStopPoint,
    is_the_handlers_rollback,
};

/// The two words the mount handler files beside an unmount a busy mount rolled
/// back.
const THE_HANDLERS_ROLLBACK_WORDS: &[&str] = &["bind-gone", "bind-present"];

#[test]
fn the_rollback_steps_are_the_handlers_own_two_words() {
    assert_eq!(
        THE_ROLLBACK_STEPS, THE_HANDLERS_ROLLBACK_WORDS,
        "the seam's words must be the ones `worktree_clone_mount_handler/sweep_unmount_answers.rs` files"
    );
}

#[test]
fn a_step_word_that_is_not_the_handlers_rollback_is_no_stop_point() {
    for word in [
        "unmount",
        "receipt",
        "move-mount",
        "bind",
        "bind-gone-again",
    ] {
        assert!(
            !is_the_handlers_rollback(word),
            "`{word}` is not one of the handler's rollback words"
        );
    }
}

#[test]
fn every_stop_point_prints_a_clause_of_its_own() {
    let points: [TheStopPoint; 3] = [
        TheStopPoint::InStaging,
        TheStopPoint::ByAScanBeforeADelete,
        TheStopPoint::AtTheHeadDestroy,
    ];
    let mut clauses: Vec<&str> = Vec::new();
    for point in points {
        let clause: &str = point.clause();
        assert!(!clause.is_empty(), "{point:?} prints no clause");
        clauses.push(clause);
    }
    clauses.sort_unstable();
    clauses.dedup();
    assert_eq!(
        clauses.len(),
        points.len(),
        "two stop points print the same clause, so a report cannot say which one stopped the removal"
    );
}
