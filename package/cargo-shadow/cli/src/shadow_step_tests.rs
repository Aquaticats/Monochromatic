//! The unit tests of the fallback after a failed install step:
//! `--locked` is added to the user's command,
//! once,
//! before `--`.

use std::ffi::OsString;

use crate::shadow_step::with_locked;

fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

#[test]
fn locked_is_added_after_the_command_words() {
    assert_eq!(with_locked(&arguments(&["check", "-q"])), arguments(&["check", "-q", "--locked"]));
}

#[test]
fn locked_goes_before_a_double_dash() {
    assert_eq!(with_locked(&arguments(&["run", "--", "--flag"])), arguments(&["run", "--locked", "--", "--flag"]));
}

#[test]
fn a_user_who_already_asked_for_locked_or_frozen_gets_nothing_added() {
    for option in ["--locked", "--frozen"] {
        let given: Vec<OsString> = arguments(&["check", option]);
        assert_eq!(with_locked(&given), given, "{option}");
    }
}

#[test]
fn a_locked_after_the_double_dash_belongs_to_the_program_and_does_not_count() {
    assert_eq!(with_locked(&arguments(&["run", "--", "--locked"])), arguments(&["run", "--locked", "--", "--locked"]));
}

/// The owner's rule of 2026-11-03: a local policy refusal that no older release can settle stops the command, an operational
/// failure (and a frozen refusal) still warns and runs it with `--locked`, and everything else runs it as typed.
#[test]
fn a_refusal_stops_the_command_and_a_failure_runs_it_locked() {
    use crate::install_step::StepOutcome;
    use crate::shadow_step::{Preflight, preflight_of};

    assert_eq!(preflight_of(StepOutcome::Refused("dep@1.0.1 refused".to_string())), Some(Preflight::Refused("dep@1.0.1 refused".to_string())));
    assert_eq!(preflight_of(StepOutcome::Failed("no network".to_string())), Some(Preflight::Failed("no network".to_string())));
    assert_eq!(preflight_of(StepOutcome::Passed), None);
    assert_eq!(preflight_of(StepOutcome::FastPath), None);
}
