//! What:     Unit tests of the cargo home lookup in `cargo_home.rs`.
//! Why:      Both the install record and the registry are found through it; a wrong answer would prune the wrong
//!           directory or none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cargo-home.unit.test.ts
//! describe('cargoHome', () => { it('prefers CARGO_HOME', () => { ... }); });
//! ```

/// Imports the function under test from this crate.
use crate::cargo_home::cargo_home;

/// Imports the owned path type for the expected answers.
use std::path::PathBuf;

/// `CARGO_HOME` wins over `HOME`.
#[test]
fn prefers_cargo_home_over_home() {
    assert_eq!(
        cargo_home(Some("/custom".to_string()), Some("/home/me".to_string())),
        Some(PathBuf::from("/custom"))
    );
}

/// Without `CARGO_HOME`, the default is `HOME/.cargo`.
#[test]
fn falls_back_to_home_dot_cargo() {
    assert_eq!(cargo_home(None, Some("/home/me".to_string())), Some(PathBuf::from("/home/me/.cargo")));
}

/// An empty `CARGO_HOME` counts as unset.
#[test]
fn treats_an_empty_cargo_home_as_unset() {
    assert_eq!(
        cargo_home(Some(String::new()), Some("/home/me".to_string())),
        Some(PathBuf::from("/home/me/.cargo"))
    );
}

/// With neither variable there is no directory.
#[test]
fn returns_none_when_neither_variable_is_usable() {
    assert_eq!(cargo_home(None, None), None);
    assert_eq!(cargo_home(Some(String::new()), Some(String::new())), None);
}
