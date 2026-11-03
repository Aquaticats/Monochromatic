//! The unit tests of the fallback after a failed install step: `--locked` is added to the user's command, once, before `--`.

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
