// What:     Unit tests for `color_scheme.rs`, pulled in by `#[cfg(test)] #[path = "color_scheme_tests.rs"] mod tests;`.
// Why:      Each accepted value maps to its fixed override, and a typo is an error, not a silent system scheme.

// What:     `use super::*;` brings the override function and its constants into the test scope.
// Why:      The tests call `scheme_override` and compare against the named constants.
use super::*;

#[test]
fn unset_follows_the_system() {
    assert_eq!(scheme_override(None), Ok(FOLLOW_SYSTEM));
}

#[test]
fn light_forces_light() {
    assert_eq!(scheme_override(Some("light")), Ok(LIGHT));
}

#[test]
fn dark_forces_dark() {
    assert_eq!(scheme_override(Some("dark")), Ok(DARK));
}

#[test]
fn other_text_is_an_error_naming_the_value() {
    let error = scheme_override(Some("Dark")).expect_err("capitalised names are not accepted");
    assert_eq!(error, ColorSchemeError::Unknown("Dark".to_string()));
    assert!(error.to_string().contains("\"Dark\""));
}

#[test]
fn an_empty_variable_counts_as_unset() {
    assert_eq!(scheme_override(Some("")), Ok(FOLLOW_SYSTEM));
}

#[test]
fn a_blank_variable_counts_as_unset() {
    assert_eq!(scheme_override(Some("  ")), Ok(FOLLOW_SYSTEM));
}
