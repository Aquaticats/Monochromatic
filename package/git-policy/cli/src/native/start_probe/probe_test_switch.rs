//! What:
//!  The one test-only switch the probe reads and what it can ask for:
//!  the probe disabled for the timing control arm,
//!  or a step the worker is held after so a run really misses its deadline.
//! Why:
//!  M4's control arm is the same binary with the probe disabled,
//!  and M3's held arm needs a run that passes its deadline on purpose,
//!  which no shipped switch may give:
//!  a variable that turned a refusal into a pass would be the force flag the
//!  design forbids
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "The deadline's margin";
//!  the test-only switch is the project's own decision in
//!  `doc/planning/cli-git-rust-open-decisions.md`).
//!  So the disabled arm records a skipped probe and enables nothing,
//!  and the switch is inert unless set.
//!  The value is parsed before the fork,
//!  so no child reads the environment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mode = parseTestSwitch(process.env.CLI_GIT_TEST_ONLY_START_PROBE); // 'none' | 'disabled' | { hold: 'move' }
//! ```

use super::probe_arms::ProbeStep;

/// The variable the switch is read from,
/// which no shipped run sets.
pub const TEST_SWITCH_VARIABLE: &str = "CLI_GIT_TEST_ONLY_START_PROBE";

/// The test-only variable that keeps one descriptor on the attached mount,
/// so M9's fixture can make the unmount arm answer `EBUSY`.
/// It is a second variable rather than a third value of the switch above,
/// whose values the project fixed at `disabled` and `hold-<step>`.
pub const KEEP_DESCRIPTOR_VARIABLE: &str = "CLI_GIT_TEST_ONLY_START_PROBE_KEEP_DESCRIPTOR";

/// What:
///  What a run's test switch asks for.
/// Why:
///  The timing control needs a run that does nothing,
///  the deadline arms need a run that wedges after a named step,
///  and everything else needs no switch at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TestSwitch = { kind: 'none' } | { kind: 'disabled' } | { kind: 'hold'; step: Step };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestSwitch {
    /// No switch,
    /// which is every shipped run.
    None,
    /// The probe disabled:
    /// the control arm that writes a skipped record and mounts nothing.
    Disabled,
    /// The worker held after this step for twice the shipped deadline,
    /// so the mode's deadline fires and names the step in flight,
    /// and M3's raised-deadline control can see the step finish.
    Hold {
        /// The step whose passing line the worker wrote last.
        step: ProbeStep,
    },
}

/// What:
///  The switch one value asks for.
/// Why:
///  A value this build does not know is no switch rather than a guess,
///  and a shipped run sets nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = parseTestSwitch('hold-move'); // { kind: 'hold', step: 'move' }
/// ```
pub fn parse_test_switch(value: Option<&str>) -> TestSwitch {
    let Some(word) = value else {
        return TestSwitch::None;
    };
    if word == "disabled" {
        return TestSwitch::Disabled;
    }
    let Some(step_name) = word.strip_prefix("hold-") else {
        return TestSwitch::None;
    };
    return match ProbeStep::from_name(step_name) {
        Some(step) => TestSwitch::Hold {
            step,
        },
        None => TestSwitch::None,
    };
}

/// The switch this process's environment asks for,
/// which a release build never reads.
pub fn test_switch_from_environment() -> TestSwitch {
    // The switch a control set, or nothing in every shipped build.
    let word: Option<String> = crate::test_only_switches::text(TEST_SWITCH_VARIABLE);
    return parse_test_switch(word.as_deref());
}

/// Whether this run keeps one descriptor on the attached mount,
/// which only M9's fixture asks for.
pub fn keep_descriptor_from_words(value: Option<&str>) -> bool {
    return value == Some("1");
}

/// Whether this process's environment keeps one descriptor on the attached
/// mount,
/// which a release build never reads.
pub fn keep_descriptor_from_environment() -> bool {
    // The value a control set, or nothing in every shipped build.
    let word: Option<String> = crate::test_only_switches::text(KEEP_DESCRIPTOR_VARIABLE);
    return keep_descriptor_from_words(word.as_deref());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_test_switch_tests.rs"]
mod tests;
