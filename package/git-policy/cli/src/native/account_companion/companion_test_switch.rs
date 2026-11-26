//! What:
//!  The one test-only switch the companion reads,
//!  and the test deadline beside it.
//! Why:
//!  M11's controls need a run that does nothing,
//!  a run held past its deadline,
//!  a child that answers a chosen errno at one step,
//!  and a child that ends before its first record,
//!  and none of them may exist in a shipped run
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  M11;
//! the switch's decision is the project's own in
//!  `doc/planning/cli-git-rust-open-decisions.md`).
//!  A filter for `mount` answering `EPERM` cannot single out one `write` of
//!  three,
//!  so the fixtures are a seam of their own,
//!  and a value this build does not know is no switch rather than a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const switch = parse('answer-attach-EPERM'); // { fixture: { step: 'attach', errno: 1 } }
//! ```

use super::companion_buffers::Fixture;
use super::companion_steps::CompanionStep;

/// The variable the switch is read from,
/// which no shipped run sets.
pub const TEST_SWITCH_VARIABLE: &str = "CLI_GIT_TEST_ONLY_ACCOUNT_COMPANION";

/// What:
///  What one run's switch asks for.
/// Why:
///  The timing control needs a run that does nothing,
///  the deadline arms need a run that sleeps after a step,
///  the refusal arms need a chosen errno at one step,
///  and a shipped run needs no switch at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Switch = { kind: 'none' } | { kind: 'disabled' } | { kind: 'hold'; step: Step } | { kind: 'fixture'; fixture: Fixture } | { kind: 'no-status' } | { kind: 'signal'; number: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Switch {
    /// No switch,
    /// which is every shipped run.
    None,
    /// The companion disabled:
    /// the control arm that runs nothing and lets the add go ahead.
    Disabled,
    /// The child sleeps after this step,
    /// so the parent's deadline fires.
    Hold {
        /// The step whose passing record the child wrote last.
        step: CompanionStep,
    },
    /// The child answers this errno at this step,
    /// instead of making the step's call.
    Fixture {
        /// The step and the errno.
        fixture: Fixture,
    },
    /// The child ends before it writes any record.
    NoStatus,
    /// The child kills itself with this signal before it writes any record.
    Signal {
        /// The signal's number.
        number: i32,
    },
}

impl Switch {
    /// Whether any switch at all was set,
    /// which is what lets a control name a test deadline.
    pub fn is_set(self) -> bool {
        return self != Switch::None;
    }

    /// The fixture this switch asks for,
    /// where it asks for one.
    pub fn fixture(self) -> Option<Fixture> {
        return match self {
            Switch::Fixture { fixture } => Some(fixture),
            _other => None,
        };
    }

    /// The step the child sleeps after,
    /// where the switch names one.
    pub fn hold(self) -> Option<CompanionStep> {
        return match self {
            Switch::Hold { step } => Some(step),
            _other => None,
        };
    }
}

/// What:
///  The switch one value asks for.
/// Why:
///  A value this build does not know is no switch,
///  so a mistyped control runs the shipped check rather than a quieter one.
pub fn parse(value: Option<&str>) -> Switch {
    let Some(word) = value else {
        return Switch::None;
    };
    if word == "disabled" {
        return Switch::Disabled;
    }
    if word == "no-status" {
        return Switch::NoStatus;
    }
    if let Some(number) = word.strip_prefix("signal-").and_then(|text| return text.parse::<i32>().ok()) {
        return Switch::Signal { number };
    }
    if let Some(step_name) = word.strip_prefix("hold-").and_then(|text| return CompanionStep::from_name(text)) {
        return Switch::Hold { step: step_name };
    }
    if let Some(rest) = word.strip_prefix("answer-") {
        return parse_fixture(rest);
    }
    return Switch::None;
}

/// What:
///  The fixture one `answer-<step>-<errno>` value names.
/// Why:
///  The step comes first and the errno last,
///  and a step's own name holds hyphens,
///  so the errno is read from the end.
fn parse_fixture(rest: &str) -> Switch {
    let Some((step_part, errno_part)) = rest.rsplit_once('-') else {
        return Switch::None;
    };
    let (Some(step), Some(errno)) = (CompanionStep::from_name(step_part), errno_number(errno_part)) else {
        return Switch::None;
    };
    return Switch::Fixture { fixture: Fixture { step, errno } };
}

/// What:
///  The number one errno name stands for,
///  among the names the design's skip list and the controls use.
/// Why:
///  The switch's own vocabulary is closed,
///  so a name this build does not know is no switch.
fn errno_number(word: &str) -> Option<i32> {
    return match word {
        "EPERM" => Some(libc::EPERM),
        "EACCES" => Some(libc::EACCES),
        "ENOSPC" => Some(libc::ENOSPC),
        "EINVAL" => Some(libc::EINVAL),
        "ENOMEM" => Some(libc::ENOMEM),
        "EBUSY" => Some(libc::EBUSY),
        "ENOENT" => Some(libc::ENOENT),
        "EOPNOTSUPP" => Some(libc::EOPNOTSUPP),
        _other => None,
    };
}

/// The switch this process's environment asks for,
/// which a release build never reads.
pub fn from_environment() -> Switch {
    // The switch a control set, or nothing in every shipped build.
    let word: Option<String> = crate::test_only_switches::text(TEST_SWITCH_VARIABLE);
    return parse(word.as_deref());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_test_switch_tests.rs"]
mod tests;
