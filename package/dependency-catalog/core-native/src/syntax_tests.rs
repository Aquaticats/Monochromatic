//! What:
//! Unit tests of the validators in `syntax.rs`.
//! Why:
//! Every accepted and rejected shape the TypeScript tests covered stays covered here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // syntax.unit.test.ts
//! describe('syntax', () => { it('isCrateName', () => { ... }); });
//! ```

/// Imports `crate::syntax::{floor_version, is_bounded_requirement, is_crate_name, is_floor_requirement, is_version}`.
use crate::syntax::{floor_version, is_bounded_requirement, is_crate_name, is_floor_requirement, is_version};

/// What:
/// `fn all(validate: fn(&str) -> bool, inputs: &[&str], expected: bool) -> bool` checks one validator
/// against a list.
/// `fn(&str) -> bool` is a function pointer,
/// like passing a TS function as a value.
/// Why:
/// Each test states its accepted and rejected lists once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const all = (validate, inputs, expected) => inputs.every((input) => validate(input) === expected);
/// ```
fn all(validate: fn(&str) -> bool, inputs: &[&str], expected: bool) -> bool {
    for input in inputs {
        if validate(input) != expected {
            return false;
        }
    }
    return true;
}

/// Crate names are 1 to 64 ASCII letters,
/// digits,
/// `-` or `_`,
/// starting with a letter.
#[test]
fn crate_names() {
    let long_ok: String = "a".repeat(64);
    let too_long: String = "a".repeat(65);
    assert!(all(is_crate_name, &["anyhow", "tracing-subscriber", "ra_ap_hir", "a", &long_ok], true));
    let rejected: [&str; 11] = ["", "9lives", "_x", "-x", &too_long, "a b", "a.b", "../x", "a\nb", "\u{e9}", "\u{1F980}"];
    assert!(all(is_crate_name, &rejected, false));
}

/// Versions have one to three numeric parts and optional labels.
#[test]
fn versions() {
    assert!(all(is_version, &["1", "0.4", "1.0.104", "1.0.0-alpha.5", "1.0.0+build.7", "1.0.0-rc.1+b"], true));
    let rejected: [&str; 10] = ["", "1.x", "01", "1.02", "1.2.3.4", "1.0.0-", "1.0.0+", "v1", "1..2", "1.0.0-a b"];
    assert!(all(is_version, &rejected, false));
}

/// Floors are exactly one `>=` comparator.
#[test]
fn floor_requirements() {
    assert!(all(is_floor_requirement, &[">=1", ">=1.0.104", ">= 0.4"], true));
    assert!(all(is_floor_requirement, &["1", ">1", ">=1, <2", ">=", "<=1", "*"], false));
}

/// Bounded requirements are caret,
/// tilde or exact.
#[test]
fn bounded_requirements() {
    assert!(all(is_bounded_requirement, &["0.58", "^0.58", "~0.58.1", "=1.18.1"], true));
    assert!(all(is_bounded_requirement, &[">=0.58", "*", "<1", "0.58, <1", ""], false));
}

/// `floor_version` returns the version of a floor and nothing for other requirements.
#[test]
fn floor_version_text() {
    assert_eq!(floor_version(">=1.0.104"), Some("1.0.104"));
    assert_eq!(floor_version("1.0.104"), None);
}
