//! What:
//! Orders and compares full SemVer versions for the set-aside rule,
//! on top of the catalog library's version parser.
//! Why:
//! A refused version is replaced by the newest older release in the same caret class,
//! so the update needs "newest first"
//! ordering and the caret-compatibility test.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // version-order.ts
//! export function compareVersions({ left, right }): number
//! export function caretCompatible({ version, candidate }): boolean
//! ```

/// What:
/// `use std::cmp::Ordering;` imports `Ordering` from the standard library so this file can name it unqualified.
/// Why:
/// `Ordering` is used by `compare_texts`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Ordering } from 'std/cmp';
/// ```
use std::cmp::Ordering;

/// What:
/// `use monochromatic_dependency_catalog::version::{ParsedVersion, compare_versions, parse_version};` imports `ParsedVersion`,
/// `compare_versions`,
/// `parse_version` from the external crate `monochromatic_dependency_catalog` so this file can name them unqualified.
/// Why:
/// `ParsedVersion` is used by `parsed`;
/// `compare_versions` is used by `compare_texts`;
/// `parse_version` is used by `parsed`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ParsedVersion, compare_versions, parse_version } from 'monochromatic_dependency_catalog/version';
/// ```
use monochromatic_dependency_catalog::version::{
    ParsedVersion,
    compare_versions,
    parse_version,
};

/// Parses a full version;
/// `None` when the text is not a version.
pub fn parsed(version: &str) -> Option<ParsedVersion> {
    return parse_version(version).ok();
}

/// Orders two version texts;
/// text that is not a version sorts first.
pub fn compare_texts(
    left: &str,
    right: &str,
) -> Ordering {
    return match (parsed(left), parsed(right)) {
        (Some(a), Some(b)) => compare_versions(&a, &b),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    };
}

/// What:
/// `fn has_prerelease(found: ParsedVersion) -> bool` tells whether a parsed version carries a pre-release part (`1.0.0-beta`).
/// Why:
/// A version that is not a version at all counts as no pre-release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hasPrerelease = (found: ParsedVersion) => found.prerelease.length > 0;
/// ```
fn has_prerelease(found: ParsedVersion) -> bool {
    return !found.prerelease.is_empty();
}

/// Whether the version has a prerelease part.
pub fn is_prerelease_text(version: &str) -> bool {
    return parsed(version).is_some_and(has_prerelease);
}

/// What:
/// `pub fn caret_compatible(version: &str, candidate: &str) -> bool` says whether `candidate` is in the same caret class as
/// `version`:
/// the same major when major is above zero,
/// else the same minor when minor is above zero,
/// else the same patch.
/// Why:
/// A replacement for a refused release must satisfy every requirement the refused release satisfied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function caretCompatible({ version, candidate }): boolean
/// ```
pub fn caret_compatible(
    version: &str,
    candidate: &str,
) -> bool {
    let (a, b) = match (parsed(version), parsed(candidate)) {
        (Some(first), Some(second)) => (first, second),
        _ => return false,
    };
    if a.major != b.major {
        return false;
    }
    if a.major > 0 {
        return true;
    }
    if a.minor != b.minor {
        return false;
    }
    if a.minor > 0 {
        return true;
    }
    return a.patch == b.patch;
}

/// What:
/// `#[cfg(test)] mod version_order_tests;` declares the tests in `src/version_order_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the version ordering,"
/// so the behavior of `version_order` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // version_order_tests.test.ts, run by the test runner only
/// ```
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "version_order_tests.rs"]
mod version_order_tests;
