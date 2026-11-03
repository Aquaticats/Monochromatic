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

/// The owner's rule of 2026-11-03:
/// a local policy refusal that no older release can settle stops the command,
/// an operational
/// failure (and a frozen refusal) still warns and runs it with `--locked`,
/// and everything else runs it as typed.
#[test]
fn a_refusal_stops_the_command_and_a_failure_runs_it_locked() {
    use crate::install_step::StepOutcome;
    use crate::shadow_step::{Preflight, preflight_of};

    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(preflight_of(StepOutcome::Refused("dep@1.0.1 refused".to_string())), Some(Preflight::Refused("dep@1.0.1 refused".to_string())));
    // `Some`: the present or the absent value of an `Option`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(preflight_of(StepOutcome::Failed("no network".to_string())), Some(Preflight::Failed("no network".to_string())));
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(preflight_of(StepOutcome::Passed), None);
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(preflight_of(StepOutcome::FastPath), None);
}
