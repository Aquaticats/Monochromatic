//! The controls of the holds themselves:
//! each hold answers the point it names while another hold waits in the same process,
//! and a second hold of one point is refused.
//! Why:
//!  `cargo test --lib` runs a crate's tests as threads of one process,
//!  and the holds once stood in one slot of that process,
//!  so the second arming dropped the first control's hold and two controls answered for each other
//!  (issue 6497: three heartbeat controls of `service_server_connection_handling_tests.rs` failed together).
//!  The first control below fails on that slot,
//!  the second on a hold registry that would let one point be armed twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdThePoint('a-point-one-control-owns'); holdThePoint('another-point-another-control-owns');
//! expect(takenByItsOwnPoint()).toBe(true);
//! ```

use super::{
    hold_the_point,
    run_on_any_thread,
};

/// The point one control of this file owns.
const THE_FIRST_POINT: &str = "the-first-point-of-this-file";

/// The point another control of this file owns.
const THE_SECOND_POINT: &str = "the-second-point-of-this-file";

/// How long a hold's reach may take before the control that waits for it fails.
/// It is the window `finished_within` gives a read of a pipe that never answers,
/// which is more than a thread of a test binary needs to reach a point it is told to.
const REACH_DEADLINE: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  Whether the thread that reached a held point said so within `REACH_DEADLINE`,
///  and nothing when it did not.
/// Why:
///   A hold that was dropped by a later arming never sends,
/// so the control that waits for it needs a bound to fail under instead of waiting for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = reachedWithin(REACH_DEADLINE, reached); // undefined when nothing said
/// ```
fn reached(
    reached: tokio::sync::oneshot::Receiver<()>
) -> Option<()> {
    return crate::test_support::finished_within_at(REACH_DEADLINE, move || {
        return reached.blocking_recv().ok();
    })
    .flatten();
}

/// What:
///  Two holds armed in one process are each taken by a thread that reaches the point its own control named.
/// Why:
///  The holds stand for work the control's own thread does not run,
///  and a run of this crate's tests holds points of several controls side by side;
///  one slot for the whole process dropped the earlier hold here,
///  which is what made the three heartbeat controls of issue 6497 fail together.
///  Both waits are let go before any assertion,
/// so a run that saw neither still ends and still reads which hold answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = holdThePoint(A), second = holdThePoint(B);
/// expect(reach(A)).toBe(true); expect(reach(B)).toBe(true); first.letGo(); second.letGo();
/// ```
#[test]
fn each_hold_is_taken_by_the_point_it_names_while_another_is_held() {
    let (first_reached, first_go) = hold_the_point(THE_FIRST_POINT);
    // What: the second arming happens while the first hold has not been taken yet.
    // Why:  that is the state one slot for the whole process could not hold: it replaced the first hold here.
    let (second_reached, second_go) = hold_the_point(THE_SECOND_POINT);
    let taking_the_first = std::thread::spawn(|| {
        return run_on_any_thread(THE_FIRST_POINT);
    });
    let taking_the_second = std::thread::spawn(|| {
        return run_on_any_thread(THE_SECOND_POINT);
    });
    let first_said: Option<()> = reached(first_reached);
    let second_said: Option<()> = reached(second_reached);
    // Both holds are let go before any assertion, so the threads they wait on end whatever this control saw.
    let _first_let_go = first_go.send(());
    let _second_let_go = second_go.send(());
    let _first_ended = taking_the_first.join();
    let _second_ended = taking_the_second.join();
    assert!(
        first_said.is_some(),
        "the thread that reached {THE_FIRST_POINT} said so while {THE_SECOND_POINT} was held beside it"
    );
    assert!(
        second_said.is_some(),
        "the thread that reached {THE_SECOND_POINT} said so while {THE_FIRST_POINT} was held beside it"
    );
}

/// What:
///  A second hold of a point this process already holds is refused,
///  and the hold that was there still answers it.
/// Why:
///   A hold answers the first thread that reaches its point,
///  so two controls that arm one point answer for each other:
///  the refusal fails the control that forgot to name the instance it owns at its own first line,
///  every time it runs,
///  instead of leaving the other control's hold dropped and a run of the module failing where the reason is not.
///  The first hold is taken and let go before any assertion,
/// so nothing of this control's own is left in the registry for the controls that run beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdThePoint(A); expect(() => holdThePoint(A)).toThrow(); expect(reach(A)).toBe(true); held.letGo();
/// ```
#[test]
fn a_second_hold_of_one_point_in_one_process_is_refused() {
    let (first_reached, first_go) = hold_the_point(THE_FIRST_POINT);
    let refused: std::thread::Result<()> = std::panic::catch_unwind(|| {
        let (_second_reach, _second_go) = hold_the_point(THE_FIRST_POINT);
        return ();
    });
    // The hold that was there before the refusal is the one this process answers, and it is taken and let go here.
    let taking_the_first = std::thread::spawn(|| {
        return run_on_any_thread(THE_FIRST_POINT);
    });
    let first_said: Option<()> = reached(first_reached);
    let _first_let_go = first_go.send(());
    let _first_ended = taking_the_first.join();
    assert!(
        refused.is_err(),
        "a second hold of {THE_FIRST_POINT} in one process is refused, because a hold answers the first thread that \
         reaches its point and two controls of one run would answer for each other"
    );
    assert!(
        first_said.is_some(),
        "the hold armed before the refusal still answers the point it names"
    );
}
