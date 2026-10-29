//! What:     Unit tests of the entry-name splitting and the version order in `version.rs`.
//! Why:      The prune deletes files by what these functions say, so a wrong split or order would delete the wrong copy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // version.unit.test.ts
//! describe('splitEntryName', () => { it('splits a crate name from its version', () => { ... }); });
//! ```

/// What:     `use crate::version::{...};` imports the items under test from this crate (`crate` means "the package I
///           am in").
/// Why:      The tests call them directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compareVersions, splitEntryName } from './version.ts';
/// ```
use crate::version::{EntryName, compare_versions, split_entry_name};

/// Imports the comparison result type the order function returns.
use std::cmp::Ordering;

/// What:     `#[test]` is an attribute marking the next function as a test case the runner should run. The function
///           takes nothing and passes if it does not panic.
/// Why:      Rust's built-in test runner (here driven by nextest) discovers tests by this attribute.
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

/// A pre-release hyphen belongs to the version, not the crate name.
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

/// Parts compare as numbers, not text.
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
