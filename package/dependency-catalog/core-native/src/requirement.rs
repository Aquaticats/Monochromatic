//! What:     Single-comparator Cargo requirement splitting and matching: bare and `^` caret, `~` tilde, `=` exact and
//!           `>=` floor, each on a partial or full version.
//! Why:      Adoption must prove that a shared `>=` floor satisfies every literal a crate wrote, and that a named
//!           catalog entry can replace a literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // semver.ts (requirement half)
//! export function splitRequirement(requirement: string): { operator: '>=' | '^' | '~' | '='; version: ParsedVersion }
//! export function satisfies(args: { version: string; requirement: string }): boolean
//! ```

use std::cmp::Ordering;

use crate::version::{ABSENT_PART, ParsedVersion, VersionError, compare_versions, parse_version};

/// What:     `pub enum Operator { ... }` declares a closed set of named choices (a TS string-literal union).
///           `Copy` lets it be passed around by value without moving ownership.
/// Why:      A requirement has exactly these four comparison shapes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Operator = '>=' | '^' | '~' | '=';
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// `>=` floor.
    AtLeast,
    /// `^` caret, also the meaning of a bare version.
    Caret,
    /// `~` tilde.
    Tilde,
    /// `=` exact.
    Exact,
}

/// A requirement split into its operator and parsed version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// Comparison shape.
    pub operator: Operator,
    /// Version the comparison names.
    pub version: ParsedVersion,
}

/// What:     `pub fn split_requirement(requirement: &str) -> Result<Requirement, VersionError>` splits text such as
///           `>=1.0`, `^0.4`, `~1.2.3`, `=1.18.1` or bare `1` (a bare version means caret).
/// Why:      Both adoption and the satisfaction check start from the operator and the version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function splitRequirement(requirement: string): { operator: Operator; version: ParsedVersion } // throws
/// ```
pub fn split_requirement(requirement: &str) -> Result<Requirement, VersionError> {
    let trimmed: &str = requirement.trim();
    let operators: [(&str, Operator); 4] = [
        (">=", Operator::AtLeast),
        ("^", Operator::Caret),
        ("~", Operator::Tilde),
        ("=", Operator::Exact),
    ];
    for (text, operator) in operators {
        if let Some(rest) = trimmed.strip_prefix(text) {
            // `?` returns early with the error when the parse fails, like an uncaught `throw` propagating.
            let version: ParsedVersion = parse_version(rest.trim())?;
            return Ok(Requirement { operator, version });
        }
    }
    let version: ParsedVersion = parse_version(trimmed)?;
    return Ok(Requirement {
        operator: Operator::Caret,
        version,
    });
}

/// Builds a release version (no prerelease) from three parts.
fn release_version(major: i64, minor: i64, patch: i64) -> ParsedVersion {
    return ParsedVersion {
        major,
        minor,
        patch,
        prerelease: Vec::new(),
    };
}

/// What:     `fn upper_bound(operator: Operator, floor: &ParsedVersion) -> ParsedVersion` returns the smallest
///           version a caret or tilde comparator excludes.
/// Why:      `^0.4` allows `0.4.x` only, `^1.2` allows `1.x`; the satisfaction check needs the exclusive top.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upperBound(operator: '^' | '~', floor: ParsedVersion): ParsedVersion
/// ```
fn upper_bound(operator: Operator, floor: &ParsedVersion) -> ParsedVersion {
    if operator == Operator::Tilde {
        if floor.minor == ABSENT_PART {
            return release_version(floor.major + 1, 0, 0);
        }
        return release_version(floor.major, floor.minor + 1, 0);
    }
    if floor.major > 0 || floor.minor == ABSENT_PART {
        return release_version(floor.major + 1, 0, 0);
    }
    if floor.minor > 0 || floor.patch == ABSENT_PART {
        return release_version(0, floor.minor + 1, 0);
    }
    return release_version(0, 0, floor.patch + 1);
}

/// What:     `pub fn satisfies(version: &str, requirement: &str) -> Result<bool, VersionError>` says whether a full
///           version satisfies a single-comparator requirement, by Cargo's documented rules (a prerelease only
///           matches a comparator that names a prerelease of the same `major.minor.patch`).
/// Why:      Adoption refuses to raise a shared floor above what a crate's own literal allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function satisfies(args: { version: string; requirement: string }): boolean // throws TypeError
/// ```
pub fn satisfies(version: &str, requirement: &str) -> Result<bool, VersionError> {
    let candidate: ParsedVersion = parse_version(version)?;
    let split: Requirement = split_requirement(requirement)?;
    let floor: &ParsedVersion = &split.version;
    if !candidate.prerelease.is_empty() {
        let same_release: bool = !floor.prerelease.is_empty()
            && floor.major == candidate.major
            && floor.minor == candidate.minor
            && floor.patch == candidate.patch;
        if !same_release {
            return Ok(false);
        }
    }
    let from_floor: Ordering = compare_versions(&candidate, floor);
    if split.operator == Operator::AtLeast {
        return Ok(from_floor != Ordering::Less);
    }
    if split.operator == Operator::Exact {
        if floor.minor == ABSENT_PART {
            return Ok(candidate.major == floor.major);
        }
        if floor.patch == ABSENT_PART {
            return Ok(candidate.major == floor.major && candidate.minor == floor.minor);
        }
        return Ok(from_floor == Ordering::Equal);
    }
    let top: ParsedVersion = upper_bound(split.operator, floor);
    return Ok(from_floor != Ordering::Less && compare_versions(&candidate, &top) == Ordering::Less);
}
