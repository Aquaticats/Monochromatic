//! What:
//!  The controls of C2's `bound`: exposure before the client lock (control 2) and exits on both sides of the link (control 3).
//! Why:
//!  Red under revision 13, where the request can be linked before the lock is held; green under 14, where the lock comes first and the
//!  link is the bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the request is linked before the client lock is held', ...);
//! ```

use crate::model_bound_r14::{Client, Rules, check, initial, invariant, steps};
use crate::model_explore::{Report, replay};

/// Control 2: the request is linked in `inbox/` before the client lock is held; the owner adopts it as clientless, the command then
/// reaches its bound and withdraws with an error, and the owner's `commit` stands (red).
#[test]
fn exposure_before_the_client_lock_lets_the_owner_decide_a_live_command_as_clientless() {
    let reached = replay(
        initial(Rules::Thirteen),
        steps,
        &[
            "client: links the request into inbox/",
            "owner: adopts the request, sees no client lock and decides it as clientless",
            "client: takes the client lock",
            "W1: the bound is reached; the command withdraws and returns the error",
        ],
    );
    assert_eq!(invariant(&reached), Some("the command returned the withdrawal error and the owner decided the request as clientless".to_string()));
    assert!(check(Rules::Thirteen).violation.is_some());
}

/// Control 3: under 14 the lock is taken first, so no step lets an owner adopt a linked request without a lock; an exit before the link
/// leaves no request, and an exit after it is an exit of a command that holds its lock.
#[test]
fn fourteen_takes_the_lock_before_the_link_and_exits_on_both_sides_are_clean() {
    let report: Report = check(Rules::Fourteen);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return found.trace; }));
    let before = replay(initial(Rules::Fourteen), steps, &["client: takes the client lock", "client: exits holding the lock and before the link"]);
    assert!(!before.linked && before.client == Client::ExitedBeforeLink);
    let after = replay(initial(Rules::Fourteen), steps, &["client: takes the client lock", "client: links the request into inbox/ (bound)"]);
    assert!(after.linked && after.lock_held, "after the bound the request is linked and the lock is held");
    assert!(steps(&after).iter().all(|step| { return !step.0.starts_with("owner:"); }), "an owner sees an attached command");
}
