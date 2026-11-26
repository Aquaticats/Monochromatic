//! What:
//!  The controls of `alone_in_its_process_child_test_support.rs`:
//!  that a seam armed in a process holding other tests is refused,
//!  and that the child which holds this test alone knows it is that child.
//! Why:
//!  The refusal and the marker it reads are what keeps a control from arming a seam of the whole process beside its
//!  module's tests (issue 6493),
//!  so both are shown here rather than only by the controls that use them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => refuseOutsideTheChild('THE_READ_IS_HELD')).toThrow('belongs to the whole process');
//! ```

/// The rule that arms a seam,
/// the refusal it makes outside a child,
/// the question a child answers,
/// and the call that starts one.
use super::{
    ran_alone_in_a_child_of_its_own,
    refuse_outside_the_child,
    this_process_holds_this_test_alone,
};

/// What:
///  A control that arms a seam of the whole process where other tests run is refused,
///  and the refusal names the starter that gives it a process of its own.
/// Why:
///  A control that forgot its child would otherwise change every test of the run,
///  and this is the one place that failure is shown to be a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => refuseOutsideTheChild('THE_READ_IS_HELD')).toThrow('run this control in a child of its own');
/// ```
#[test]
#[should_panic(expected = "belongs to the whole process")]
fn arming_a_seam_outside_a_child_of_its_own_is_refused() {
    refuse_outside_the_child("THE_READ_IS_HELD, the hold a control of this module arms");
}

/// What:
///  The child this test starts is the only test of its process, and the refusal lets it arm its seam.
/// Why:
///  The parent's half is every control that starts such a child;
///  this is the child's half, so a marker that never reached the child would fail here instead of in the control that
///  waited for a hold it could not arm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (ranAloneInAChildOfItsOwn(import.meta.url, 'the_child_holds_this_test_alone')) return;
/// expect(thisProcessHoldsThisTestAlone()).toBe(true);
/// refuseOutsideTheChild('THE_READ_IS_HELD');
/// ```
#[test]
fn the_child_holds_this_test_alone() {
    // What: the parent starts this same test in a child and asserts that it passed, then returns; the child runs the
    //  body below. `true` in the parent, `false` in the child.
    // Why: this control's own subject is the child's answer, which only the child can give.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (ranAloneInAChildOfItsOwn(import.meta.url, 'the_child_holds_this_test_alone')) return;
    // ```
    if ran_alone_in_a_child_of_its_own(module_path!(), "the_child_holds_this_test_alone") {
        return;
    }
    // What: `assert!` ends the test with the message when the child did not learn it is the child.
    // Why: every control of the size read's module relies on this answer to keep its seam out of the parent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!thisProcessHoldsThisTestAlone()) throw new Error('the child did not see its marker');
    // ```
    assert!(
        this_process_holds_this_test_alone(),
        "the child of this control holds this test alone"
    );
    // What: the refusal that a parent process would fail on, which the child passes.
    // Why: the pair of this test and the one above is what shows the refusal reads the marker and nothing else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // refuseOutsideTheChild('THE_READ_IS_HELD');
    // ```
    refuse_outside_the_child("THE_READ_IS_HELD, which this child may arm by itself");
}
