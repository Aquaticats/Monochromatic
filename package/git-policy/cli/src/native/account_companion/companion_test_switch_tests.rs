//! Controls of the test-only switch and of the shipped deadline:
//!  every value the controls use parses to what it asks for,
//!  a value this build does not know is no switch,
//!  and the deadline ships with its margin over the measured run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parse('answer-attach-EPERM')).toEqual({ kind: 'fixture', fixture: { step: 'attach', errno: 1 } });
//! expect(parse('answer-attach-NOPE')).toEqual({ kind: 'none' });
//! ```

use super::{Switch, parse};
use crate::account_companion::companion_buffers::Fixture;
use crate::account_companion::companion_deadline::{COMPANION_DEADLINE, COMPANION_DEADLINE_MARGIN, the_deadline};
use crate::account_companion::companion_steps::CompanionStep;

/// What:
///  The control arm,
/// the deadline arm and the two arms that end before any
///  record all parse to what they ask for.
#[test]
fn each_value_the_controls_use_parses_to_what_it_asks_for() {
    assert_eq!(parse(None), Switch::None);
    assert_eq!(parse(Some("disabled")), Switch::Disabled);
    assert_eq!(parse(Some("hold-walk")), Switch::Hold { step: CompanionStep::Walk });
    assert_eq!(parse(Some("no-status")), Switch::NoStatus);
    assert_eq!(parse(Some("signal-9")), Switch::Signal { number: 9 });
    assert_eq!(parse(Some("answer-attach-EPERM")), Switch::Fixture { fixture: Fixture { step: CompanionStep::Attach, errno: libc::EPERM } });
}

/// What:
///  A step's own name holds hyphens,
///  so the errno is read from the end of the value.
#[test]
fn a_step_name_with_hyphens_still_reads_its_errno() {
    let parsed: Switch = parse(Some("answer-user-namespace-ENOSPC"));
    assert_eq!(parsed, Switch::Fixture { fixture: Fixture { step: CompanionStep::UserNamespace, errno: libc::ENOSPC } });
    assert_eq!(parsed.fixture().map(|fixture| return fixture.step), Some(CompanionStep::UserNamespace));
}

/// What:
///  A mistyped control runs the shipped check rather than a quieter one.
#[test]
fn a_value_this_build_does_not_know_is_no_switch() {
    for word in ["answer-attach-NOPE", "answer-nosuchstep-EPERM", "hold-nosuchstep", "signal-", "DISABLED", ""] {
        assert_eq!(parse(Some(word)), Switch::None, "{word} must not be read as a switch");
    }
}

/// What:
///  A shipped run waits the shipped deadline,
///  which sits above the margin over the measured run.
#[test]
fn the_shipped_deadline_sits_above_its_margin() {
    assert!(the_deadline(false) == COMPANION_DEADLINE);
    assert!(COMPANION_DEADLINE > COMPANION_DEADLINE_MARGIN, "the bound must be larger than the margin it carries");
    assert!(!Switch::None.is_set(), "a shipped run sets no switch, so no test deadline is read");
    assert!(Switch::Disabled.is_set());
}
