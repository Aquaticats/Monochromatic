//! What:     Cargo version parsing, ordering and formatting for partial and full versions with prerelease labels.
//! Why:      Adoption picks the highest literal version across crates and checks requirements against it, with no
//!           semver crate available.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // semver.ts (parse, compare, format half)
//! export type ParsedVersion = { major: number; minor: number; patch: number; prerelease: string[] };
//! export function parseVersion(text: string): ParsedVersion
//! export function compareVersions(left: ParsedVersion, right: ParsedVersion): number
//! export function formatVersion(version: ParsedVersion): string
//! ```

use std::cmp::Ordering;
use std::fmt;

use crate::syntax::is_version;

/// What:     `pub const ABSENT_PART: i64 = -1;` marks a minor or patch part the text leaves out. `i64` is a signed
///           64-bit integer (siblings: `u64` cannot hold `-1`, `i32` is narrower).
/// Why:      Partial requirements keep their width (`1` is not `1.0.0`), which the caret and tilde bounds need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ABSENT_PART = -1;
/// ```
pub const ABSENT_PART: i64 = -1;

/// What:     `pub struct ParsedVersion { ... }` declares a record with four public fields; `#[derive(...)]` asks the
///           compiler to generate `Debug` printing, `Clone` (deep copy), `PartialEq`/`Eq` (`==`).
/// Why:      Holds one parsed version; `i64` parts so `ABSENT_PART` fits, and `Vec<String>` (not `&str`) so the
///           struct owns its labels and can outlive the input text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedVersion = { major: number; minor: number; patch: number; prerelease: string[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVersion {
    /// Major part.
    pub major: i64,
    /// Minor part, or `ABSENT_PART`.
    pub minor: i64,
    /// Patch part, or `ABSENT_PART`.
    pub patch: i64,
    /// Dot-separated prerelease identifiers; empty for a release.
    pub prerelease: Vec<String>,
}

/// What:     `pub struct VersionError { ... }` is the failure value returned when text is not a Cargo version or
///           requirement.
/// Why:      Callers turn it into a diagnostic naming the offending requirement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class VersionError extends Error {}
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionError {
    /// Human-readable reason.
    pub message: String,
}

impl fmt::Display for VersionError {
    /// Prints the reason.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.message);
    }
}

/// What:     `pub fn parse_version(text: &str) -> Result<ParsedVersion, VersionError>` parses version text without an
///           operator. `Result<T, E>` is Rust's "returns a value or an error" type (no exceptions).
/// Why:      Fails with a `VersionError` for text `is_version` rejects, so callers can report instead of panicking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseVersion(text: string): ParsedVersion // throws TypeError
/// ```
pub fn parse_version(text: &str) -> Result<ParsedVersion, VersionError> {
    if !is_version(text) {
        // `Err(...)` is the failure variant of `Result`; `format!` builds an owned `String` like a template literal.
        return Err(VersionError {
            message: format!("{text:?} is not a Cargo version."),
        });
    }
    let without_build: &str = text.split('+').next().unwrap_or(text);
    let (core, prerelease): (&str, Vec<String>) = match without_build.split_once('-') {
        Some((before, after)) => (before, after.split('.').map(str::to_string).collect()),
        None => (without_build, Vec::new()),
    };
    let mut numbers: Vec<i64> = Vec::new();
    for part in core.split('.') {
        // `is_version` already proved every part is digits; a part too long for `i64` saturates instead of failing.
        numbers.push(part.parse::<i64>().unwrap_or(i64::MAX));
    }
    return Ok(ParsedVersion {
        major: numbers.first().copied().unwrap_or(0),
        minor: numbers.get(1).copied().unwrap_or(ABSENT_PART),
        patch: numbers.get(2).copied().unwrap_or(ABSENT_PART),
        prerelease,
    });
}

/// Whether a prerelease identifier is only ASCII digits.
fn is_numeric_identifier(identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }
    for character in identifier.chars() {
        if !character.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// What:     `fn compare_identifier(left: &str, right: &str) -> Ordering` orders two prerelease identifiers by semver
///           precedence: numbers compare as numbers and sort before words; words compare as text.
/// Why:      `Ordering` (Less, Equal, Greater) is Rust's comparison result, the counterpart of a negative, zero or
///           positive number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareIdentifier(left: string, right: string): number
/// ```
fn compare_identifier(left: &str, right: &str) -> Ordering {
    let left_numeric: bool = is_numeric_identifier(left);
    let right_numeric: bool = is_numeric_identifier(right);
    if left_numeric && right_numeric {
        let left_number: i64 = left.parse::<i64>().unwrap_or(i64::MAX);
        let right_number: i64 = right.parse::<i64>().unwrap_or(i64::MAX);
        return left_number.cmp(&right_number);
    }
    if left_numeric {
        return Ordering::Less;
    }
    if right_numeric {
        return Ordering::Greater;
    }
    return left.cmp(right);
}

/// What:     `pub fn compare_versions(left: &ParsedVersion, right: &ParsedVersion) -> Ordering` orders two versions;
///           absent parts count as 0 and build labels never reach this type.
/// Why:      Adoption needs "the highest version" and the requirement check needs "at or above the floor".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function compareVersions(left: ParsedVersion, right: ParsedVersion): number
/// ```
pub fn compare_versions(left: &ParsedVersion, right: &ParsedVersion) -> Ordering {
    let pairs: [(i64, i64); 3] = [
        (left.major, right.major),
        (left.minor.max(0), right.minor.max(0)),
        (left.patch.max(0), right.patch.max(0)),
    ];
    for (left_part, right_part) in pairs {
        if left_part != right_part {
            return left_part.cmp(&right_part);
        }
    }
    if left.prerelease.is_empty() && right.prerelease.is_empty() {
        return Ordering::Equal;
    }
    if left.prerelease.is_empty() {
        return Ordering::Greater;
    }
    if right.prerelease.is_empty() {
        return Ordering::Less;
    }
    let shared: usize = left.prerelease.len().min(right.prerelease.len());
    for index in 0..shared {
        let order: Ordering = compare_identifier(&left.prerelease[index], &right.prerelease[index]);
        if order != Ordering::Equal {
            return order;
        }
    }
    return left.prerelease.len().cmp(&right.prerelease.len());
}

/// What:     `pub fn format_version(version: &ParsedVersion) -> String` prints the full `major.minor.patch` core
///           (absent parts as 0) plus `-prerelease` when present.
/// Why:      The requirement check compares a full candidate version, so a floor like `0.4` becomes `0.4.0`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function formatVersion(version: ParsedVersion): string
/// ```
pub fn format_version(version: &ParsedVersion) -> String {
    let core: String = format!("{}.{}.{}", version.major, version.minor.max(0), version.patch.max(0));
    if version.prerelease.is_empty() {
        return core;
    }
    return format!("{core}-{}", version.prerelease.join("."));
}
