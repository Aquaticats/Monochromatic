//! What:
//!  The controls of `process_wide_state_child_test_support.rs`:
//!  that a state of the whole process is refused where other tests run,
//!  and that the child which holds this test alone knows it is that child.
//! Why:
//!  The refusal and the marker it reads are what keeps a control from setting such state beside its module's tests,
//!  so both are shown here rather than only by the controls that use them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => refuseOutsideTheChild('the hold a control sets')).toThrow('belongs to the whole process');
//! ```

/// The call that starts one control in a child of its own,
/// the refusal that call makes possible,
/// and the question both ask.
use super::{
    refuse_outside_the_child,
    started_within_a_child_of_its_own,
    this_process_is_the_child,
};

/// What:
///  A control that sets state of the whole process where other tests run is refused,
///  and the refusal names the starter that gives it a process of its own.
/// Why:
///  A control that forgot its child would otherwise change every test of the run,
///  and this is the one place that failure is shown to be a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => refuseOutsideTheChild('the hold a control sets')).toThrow('run this control in a child of its own');
/// ```
#[test]
#[should_panic(expected = "belongs to the whole process")]
fn setting_that_state_outside_a_child_of_its_own_is_refused() {
    refuse_outside_the_child("the hold a control of the size read's module sets");
}

/// What:
///  The child this test starts is the only test of its process, and the refusal lets it set its state.
/// Why:
///  The parent's half is every control that starts such a child;
///  this is the child's half, so a marker that never reached the child would fail here instead of in the control that
///  waited for state it could not set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (startedWithinAChildOfItsOwn(import.meta.url, 'the_child_holds_this_test_alone')) return;
/// expect(thisProcessIsTheChild()).toBe(true);
/// refuseOutsideTheChild('the hold a control sets');
/// ```
#[test]
fn the_child_holds_this_test_alone() {
    // What: the parent starts this same test in a child and asserts that it passed, then returns; the child runs the
    //  body below. `true` in the parent, `false` in the child.
    // Why: this control's own subject is the child's answer, which only the child can give.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (startedWithinAChildOfItsOwn(import.meta.url, 'the_child_holds_this_test_alone')) return;
    // ```
    if started_within_a_child_of_its_own(module_path!(), "the_child_holds_this_test_alone") {
        return;
    }
    // What: `assert!` ends the test with the message when the child did not learn it is the child.
    // Why: every control that sets such state relies on this answer to keep it out of the parent process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!thisProcessIsTheChild()) throw new Error('the child did not see its marker');
    // ```
    assert!(
        this_process_is_the_child(),
        "the child of this control holds this test alone"
    );
    // What: the refusal that a parent process fails on, which the child passes.
    // Why: the pair of this test and the one above is what shows the refusal reads the marker and nothing else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // refuseOutsideTheChild('the hold a control sets');
    // ```
    refuse_outside_the_child("the hold a control of the size read's module sets");
}
