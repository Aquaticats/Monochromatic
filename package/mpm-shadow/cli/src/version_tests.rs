//! What:
//! Unit tests of the entry-name splitting and the version order in `version.rs`.
//! Why:
//! The prune deletes files by what these functions say,
//! so a wrong split or order would delete the wrong copy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // version.unit.test.ts
//! describe('splitEntryName', () => { it('splits a crate name from its version', () => { ... }); });
//! ```

/// What:
/// `use crate::version::{...};` imports the items under test from this crate (`crate` means "the package I
///           am in").
/// Why:
/// The tests call them directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compareVersions, splitEntryName } from './version.ts';
/// ```
use crate::version::{EntryName, compare_versions, split_entry_name};

/// Imports the comparison result type the order function returns.
use std::cmp::Ordering;

/// What:
/// `#[test]` is an attribute marking the next function as a test case the runner should run.
/// The function
///           takes nothing and passes if it does not panic.
/// Why:
/// Rust's built-in test runner (here driven by nextest) discovers tests by this attribute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('splits a crate name from its version', () => { ... });
/// ```
#[test]
fn splits_a_crate_name_from_its_version() {
    // What:     `assert_eq!(left, right)` is a macro (the `!`) that panics, failing the test, when the two values
    //           differ. `Some(EntryName { ... })` is the present case of `Option` wrapping the expected record.
    // Why:      This is the plain case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(splitEntryName('monochromatic-lint-0.1.0')).toEqual({ crateName: 'monochromatic-lint', version: '0.1.0' });
    // ```
    assert_eq!(
        split_entry_name("monochromatic-lint-0.1.0"),
        Some(EntryName {
            crate_name: "monochromatic-lint".to_string(),
            version: "0.1.0".to_string(),
        })
    );
}

/// A pre-release hyphen belongs to the version,
/// not the crate name.
#[test]
fn keeps_a_pre_release_suffix_in_the_version() {
    assert_eq!(
        split_entry_name("monochromatic-lint-0.5.0-rc.1"),
        Some(EntryName {
            crate_name: "monochromatic-lint".to_string(),
            version: "0.5.0-rc.1".to_string(),
        })
    );
}

/// A longer crate name that shares a prefix stays whole.
#[test]
fn keeps_a_longer_crate_name_whole() {
    assert_eq!(
        split_entry_name("monochromatic-lint-closure-0.1.0"),
        Some(EntryName {
            crate_name: "monochromatic-lint-closure".to_string(),
            version: "0.1.0".to_string(),
        })
    );
}

/// A name with no hyphen followed by a whole version is not an entry name.
#[test]
fn returns_none_when_no_hyphen_is_followed_by_a_version() {
    // `None` is the absent case of `Option`; `assert_eq!` compares it with the function's answer.
    assert_eq!(split_entry_name("monochromatic-lint"), None);
    assert_eq!(split_entry_name("monochromatic-lint-0.1"), None);
}

/// Parts compare as numbers,
/// not text.
#[test]
fn orders_by_numeric_core_not_text() {
    // `Ordering::Greater` means the left version is newer than the right one.
    assert_eq!(compare_versions("0.1.10", "0.1.9"), Ordering::Greater);
    assert_eq!(compare_versions("0.2.13", "0.2.8"), Ordering::Greater);
}

/// A release is newer than its own pre-release.
#[test]
fn ranks_a_release_above_its_pre_release() {
    assert_eq!(compare_versions("0.5.0", "0.5.0-rc.1"), Ordering::Greater);
    assert_eq!(compare_versions("0.5.0-rc.1", "0.5.0"), Ordering::Less);
}

/// The same version compares equal.
#[test]
fn reports_equal_versions_as_equal() {
    assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
}

/// Semantic Versioning 2.0.0, item 11, lists this chain in ascending precedence:
/// every version is older than the ones after it,
/// and a list in that order is the order `compare_versions` must give for each pair.
#[test]
fn orders_the_pre_release_chain_of_the_semver_specification() {
    let ascending: [&str; 8] =
        ["1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-alpha.beta", "1.0.0-beta", "1.0.0-beta.2", "1.0.0-beta.11", "1.0.0-rc.1", "1.0.0"];
    for (earlier_index, earlier) in ascending.iter().enumerate() {
        for (later_index, later) in ascending.iter().enumerate() {
            assert_eq!(compare_versions(earlier, later), earlier_index.cmp(&later_index), "{earlier} against {later}");
        }
    }
}

/// Identifiers made only of digits compare as numbers,
/// so the number after `alpha` with more digits is the larger one,
/// even when it starts with a smaller digit.
#[test]
fn compares_numeric_pre_release_identifiers_as_numbers() {
    assert_eq!(compare_versions("0.2.7-alpha.10", "0.2.7-alpha.9"), Ordering::Greater);
    assert_eq!(compare_versions("0.2.7-alpha.9", "0.2.7-alpha.10"), Ordering::Less);
    assert_eq!(compare_versions("0.2.7-alpha.2.1", "0.2.7-alpha.10.0"), Ordering::Less);
}

/// A number longer than any machine integer still compares as a number, and never overflows into an equal one.
#[test]
fn compares_numeric_identifiers_longer_than_a_machine_integer() {
    let smaller: &str = "1.0.0-rc.99999999999999999999999999";
    let larger: &str = "1.0.0-rc.100000000000000000000000000";
    assert_eq!(compare_versions(larger, smaller), Ordering::Greater);
    assert_eq!(compare_versions(smaller, larger), Ordering::Less);
}

/// A numeric identifier is lower than an identifier with letters or hyphens,
/// and the identifiers with letters compare as ASCII text (capitals before lower case letters).
#[test]
fn ranks_numeric_identifiers_below_alphanumeric_ones_and_compares_those_as_ascii_text() {
    assert_eq!(compare_versions("1.0.0-1", "1.0.0-a"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0-99999", "1.0.0--"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0-Beta", "1.0.0-alpha"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0-alpha.beta", "1.0.0-alpha.1"), Ordering::Greater);
}

/// A larger set of identifiers is higher when every earlier identifier is equal.
#[test]
fn ranks_a_longer_identifier_set_above_a_shorter_one_with_equal_leading_identifiers() {
    assert_eq!(compare_versions("1.0.0-alpha.1", "1.0.0-alpha"), Ordering::Greater);
    assert_eq!(compare_versions("1.0.0-alpha.1.1", "1.0.0-alpha.1"), Ordering::Greater);
    assert_eq!(compare_versions("1.0.0-alpha", "1.0.0-alpha.1"), Ordering::Less);
}

/// Build metadata after a plus sign never figures into precedence.
#[test]
fn ignores_build_metadata() {
    assert_eq!(compare_versions("1.0.0-alpha.1+build.9", "1.0.0-alpha.1+build.10"), Ordering::Equal);
    assert_eq!(compare_versions("1.0.0+a", "1.0.0"), Ordering::Equal);
}
