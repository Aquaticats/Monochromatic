//! What:
//! Unit tests of version parsing,
//! ordering,
//! formatting and requirement matching.
//! Why:
//! Adoption's correctness rests on these;
//! the cases are the TypeScript semver tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // semver.unit.test.ts
//! describe('semver', () => { it('parseVersion', () => { ... }); });
//! ```

/// Imports `std::cmp::Ordering`.
use std::cmp::Ordering;

/// Imports `crate::requirement::{Operator, satisfies, split_requirement}`.
use crate::requirement::{Operator, satisfies, split_requirement};
/// Imports `crate::version::{ParsedVersion, compare_versions, format_version, parse_version}`.
use crate::version::{ParsedVersion, compare_versions, format_version, parse_version};

/// Builds a `ParsedVersion` for assertions.
fn parsed(major: i64, minor: i64, patch: i64, prerelease: &[&str]) -> ParsedVersion {
    return ParsedVersion {
        major,
        minor,
        patch,
        prerelease: prerelease.iter().map(|label: &&str| return (*label).to_string()).collect(),
    };
}

/// Verdicts of `satisfies` for one requirement across several versions.
fn matches(requirement: &str, versions: &[&str]) -> Vec<bool> {
    let mut verdicts: Vec<bool> = Vec::new();
    for version in versions {
        verdicts.push(satisfies(version, requirement).unwrap_or(false));
    }
    return verdicts;
}

/// Parsing keeps partial widths and prerelease labels,
/// and rejects non-versions.
#[test]
fn parse_version_shapes() {
    assert_eq!(parse_version("1.0.0-rc.1+b7"), Ok(parsed(1, 0, 0, &["rc", "1"])));
    assert_eq!(parse_version("0.4"), Ok(parsed(0, 4, -1, &[])));
    assert_eq!(parse_version("2"), Ok(parsed(2, -1, -1, &[])));
    assert!(parse_version("1.x").is_err());
}

/// The semver specification's own prerelease ordering example,
/// then releases.
#[test]
fn compare_versions_order() {
    let ascending: [&str; 12] = [
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
        "1.0.99",
        "1.0.104",
        "1.10.0",
        "2.0.0",
    ];
    let mut shuffled: Vec<&str> = ascending.to_vec();
    shuffled.reverse();
    shuffled.sort_by(|left: &&str, right: &&str| {
        let left_version: ParsedVersion = parse_version(left).unwrap_or(parsed(0, 0, 0, &[]));
        let right_version: ParsedVersion = parse_version(right).unwrap_or(parsed(0, 0, 0, &[]));
        return compare_versions(&left_version, &right_version);
    });
    assert_eq!(shuffled, ascending.to_vec());
    let short: ParsedVersion = parsed(1, 0, -1, &[]);
    let full: ParsedVersion = parsed(1, 0, 0, &[]);
    assert_eq!(compare_versions(&short, &full), Ordering::Equal);
}

/// Splitting finds the operator,
/// and a bare version means caret.
#[test]
fn split_requirement_operators() {
    assert_eq!(split_requirement(">=1.0.104").map(|split| return split.operator), Ok(Operator::AtLeast));
    assert_eq!(split_requirement("=1.18.1").map(|split| return split.operator), Ok(Operator::Exact));
    assert_eq!(split_requirement("~0.58.1").map(|split| return split.operator), Ok(Operator::Tilde));
    assert_eq!(split_requirement("^0.4").map(|split| return split.operator), Ok(Operator::Caret));
    assert_eq!(split_requirement("0.4").map(|split| return split.operator), Ok(Operator::Caret));
    assert!(split_requirement("*").is_err());
}

/// Caret requirements follow Cargo's rules for major,
/// zero-major and zero-minor versions.
#[test]
fn caret_requirements() {
    assert_eq!(matches("1", &["1.0.0", "1.99.0", "2.0.0", "0.9.0"]), vec![true, true, false, false]);
    assert_eq!(matches("1.2.3", &["1.2.3", "1.9.0", "1.2.2", "2.0.0"]), vec![true, true, false, false]);
    assert_eq!(matches("0.58", &["0.58.0", "0.58.9", "0.62.0", "0.57.0"]), vec![true, true, false, false]);
    assert_eq!(matches("^0.0.3", &["0.0.3", "0.0.4"]), vec![true, false]);
    assert_eq!(matches("^0.0", &["0.0.9", "0.1.0"]), vec![true, false]);
    assert_eq!(matches("^0", &["0.9.0", "1.0.0"]), vec![true, false]);
}

/// Tilde,
/// exact and floor requirements.
#[test]
fn tilde_exact_and_floor_requirements() {
    assert_eq!(matches("~1.2.3", &["1.2.9", "1.3.0"]), vec![true, false]);
    assert_eq!(matches("~1", &["1.9.0", "2.0.0"]), vec![true, false]);
    assert_eq!(matches("=1.18.1", &["1.18.1", "1.18.2"]), vec![true, false]);
    assert_eq!(matches("=1.18", &["1.18.7", "1.19.0"]), vec![true, false]);
    assert_eq!(matches("=1", &["1.5.0", "2.0.0"]), vec![true, false]);
    assert_eq!(matches(">=1.0.104", &["1.0.104", "2.0.0", "1.0.103"]), vec![true, true, false]);
}

/// A prerelease only matches a comparator naming a prerelease of the same release.
#[test]
fn prerelease_requirements() {
    assert_eq!(matches(">=1.0.0", &["1.1.0-alpha"]), vec![false]);
    assert_eq!(matches(">=1.0.0-alpha.5", &["1.0.0-alpha.6", "1.0.1-alpha.1"]), vec![true, false]);
}

/// Formatting pads absent parts with zero and drops build labels.
#[test]
fn format_version_text() {
    assert_eq!(parse_version("0.4").map(|version| return format_version(&version)), Ok("0.4.0".to_string()));
    assert_eq!(parse_version("1.0.0-rc.1+b").map(|version| return format_version(&version)), Ok("1.0.0-rc.1".to_string()));
}
