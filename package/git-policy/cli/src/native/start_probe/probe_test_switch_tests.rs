//! What:
//!  Controls of the test-only switch's parse:
//!  no value is no switch,
//!  `disabled` is the timing control,
//!  `hold-<step>` names a step of this build,
//!  and anything else is no switch rather than a guess.
//! Why:
//!  A value this build does not know must not become a hold the deadline never
//!  ends,
//!  and a shipped run sets nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseTestSwitch(undefined)).toEqual({ kind: 'none' }); expect(parseTestSwitch('hold-move')).toEqual({ kind: 'hold', step: 'move' });
//! ```

use super::{TestSwitch, parse_test_switch};
use super::super::probe_arms::ProbeStep;

/// No value is no switch,
/// and `disabled` is the control arm.
#[test]
fn no_value_is_no_switch_and_disabled_is_the_control() {
    assert_eq!(parse_test_switch(None), TestSwitch::None);
    assert_eq!(parse_test_switch(Some("disabled")), TestSwitch::Disabled);
}

/// A hold names a step of this build,
/// and an unknown step or word is no switch.
#[test]
fn a_hold_names_a_step_and_anything_else_is_no_switch() {
    assert_eq!(parse_test_switch(Some("hold-move")), TestSwitch::Hold { step: ProbeStep::Move });
    assert_eq!(parse_test_switch(Some("hold-teardown")), TestSwitch::Hold { step: ProbeStep::Teardown });
    assert_eq!(parse_test_switch(Some("hold-no-such-step")), TestSwitch::None);
    assert_eq!(parse_test_switch(Some("please")), TestSwitch::None);
}
