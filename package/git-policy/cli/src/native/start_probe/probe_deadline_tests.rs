//! What:
//!  Controls of the deadline's arithmetic:
//!  a run inside the bound has time left,
//!  the bound itself has none,
//!  the margin is the part of the value that protects against noise,
//!  and a build the release ships keeps the shipped bound with the test-only
//!  variable set.
//! Why:
//!  The mode's wait and its kill must agree on where the boundary is,
//!  and a margin larger than the value would be a bound that fires at once.
//!  The variable's read sits behind a Cargo feature, so a release build never
//!  reads it;
//!  the last control starts a child of this test binary with the variable set,
//!  which is the only way a build without the feature is asked the question,
//!  and a plant that removes the gating `cfg` lines makes it fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(remaining(start, start)).toBeGreaterThan(0); expect(deadlinePassed(start, start + PROBE_DEADLINE)).toBe(true);
//! ```

use super::{DEADLINE_SWITCH_VARIABLE, PROBE_DEADLINE, PROBE_DEADLINE_MARGIN, deadline_from_words, deadline_passed, remaining, the_bound};
use std::ffi::OsString;
use std::time::{Duration, Instant};

/// The variable that tells the child it is the child,
/// set beside the deadline variable the child reads.
/// Why:
///  The control runs the same test name in a child,
///  and the child must do the child's one assertion rather than start another
///  child.
const THE_DEADLINE_CHILD: &str = "CLI_GIT_TEST_ONLY_PROBE_DEADLINE_CHILD";

/// A just-started run has time left,
/// and the bound itself has none.
#[test]
fn the_bound_is_where_the_wait_ends() {
    let started: Instant = Instant::now();
    assert!(!deadline_passed(started, started, PROBE_DEADLINE));
    assert!(remaining(started, started, PROBE_DEADLINE).is_some_and(|left| return left == PROBE_DEADLINE));
    assert!(deadline_passed(started, started + PROBE_DEADLINE, PROBE_DEADLINE));
    assert_eq!(remaining(started, started + PROBE_DEADLINE, PROBE_DEADLINE), None);
    assert_eq!(remaining(started, started + PROBE_DEADLINE + Duration::from_millis(1), PROBE_DEADLINE), None);
}

/// The margin is part of the value,
/// never larger than it.
#[test]
fn the_margin_is_part_of_the_value() {
    assert!(PROBE_DEADLINE > Duration::ZERO);
    assert!(PROBE_DEADLINE_MARGIN < PROBE_DEADLINE);
}

/// The test-only deadline override takes a positive number of milliseconds
/// and leaves the shipped bound alone otherwise.
#[test]
fn the_test_only_deadline_is_a_positive_number() {
    assert_eq!(deadline_from_words(None), PROBE_DEADLINE);
    assert_eq!(deadline_from_words(Some("900000")), std::time::Duration::from_millis(900000));
    assert_eq!(deadline_from_words(Some("0")), PROBE_DEADLINE);
    assert_eq!(deadline_from_words(Some("soon")), PROBE_DEADLINE);
}

/// A build the release ships uses the shipped bound,
/// whatever the environment says,
/// because the variable's read sits behind the feature a control's build turns
/// on.
#[test]
fn the_shipped_bound_is_what_this_build_uses() {
    assert_eq!(the_bound(), PROBE_DEADLINE, "the release's probe path is the shipped bound; only `--features start-probe-raised-deadline` reads the variable");
}

/// The bound is still the shipped one with the test-only variable set,
/// asked of a build that was not made for a control.
/// Why:
///  A unit's `Environment=` line can set a variable,
///  and a deadline that can be raised turns a missed deadline into a pass;
///  the variable is read only by a build made with
///  `--features start-probe-raised-deadline`,
///  and only a child of this test binary can carry it without changing the
///  environment every other control in this process reads.
#[cfg(not(feature = "start-probe-raised-deadline"))]
#[test]
fn the_shipped_bound_holds_with_the_test_variable_set() {
    if std::env::var_os(THE_DEADLINE_CHILD).is_some() {
        assert_eq!(
            the_bound(),
            PROBE_DEADLINE,
            "a build without `start-probe-raised-deadline` keeps the shipped bound with {DEADLINE_SWITCH_VARIABLE}=900000"
        );
        return;
    }
    let output: std::process::Output = crate::test_support::child_test_process::run_with_environment(
        "start_probe::probe_deadline::tests::the_shipped_bound_holds_with_the_test_variable_set",
        &[
            (OsString::from(THE_DEADLINE_CHILD), OsString::from("1")),
            (OsString::from(DEADLINE_SWITCH_VARIABLE), OsString::from("900000")),
        ],
    );
    let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(output.status.success(), "a release build read {DEADLINE_SWITCH_VARIABLE} and took the raised bound:\n{said}");
    assert!(said.contains("1 passed"), "the child ran no test:\n{said}");
}
