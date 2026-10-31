//! What:     String-scan validators for crate names and version requirements, written as plain character loops.
//! Why:      The catalog parser and the sync need to decide "is this a Cargo name / a `>=` floor / a bounded
//!           requirement" without a regex or semver crate (neither is approved for this library).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // syntax.ts
//! export function isCrateName(name: string): boolean { ... }
//! export function isVersion(version: string): boolean { ... }
//! export function isFloorRequirement(requirement: string): boolean { ... }
//! export function isBoundedRequirement(requirement: string): boolean { ... }
//! ```

/// What:     `const MAX_CRATE_NAME_LENGTH: usize = 64;` is a named constant. `usize` is the unsigned integer wide enough
///           to index memory (siblings: `u32`, `u64`, `i32`).
/// Why:      crates.io rejects longer names; `usize` and not `u32` because it is compared with `str::len()`, which
///           returns `usize`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_CRATE_NAME_LENGTH = 64;
/// ```
const MAX_CRATE_NAME_LENGTH: usize = 64;

/// The most dot-separated numeric parts a version may have (major, minor, patch).
const MAX_VERSION_PARTS: usize = 3;

/// What:     `fn is_name_character(character: char) -> bool` declares a private function. `char` is one Unicode
///           scalar value (sibling: `u8` is one raw byte).
/// Why:      Crate names allow ASCII letters, digits, `-` and `_`; `char` and not `u8` because we iterate `str::chars()`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNameCharacter(character: string): boolean {
///   return /[A-Za-z0-9_-]/.test(character); // written without a regex in the TS source too
/// }
/// ```
fn is_name_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '-' || character == '_';
}

/// What:     `pub fn is_crate_name(name: &str) -> bool` declares a public function. `&str` is a borrowed read-only
///           view of text (sibling: `String` owns its text).
/// Why:      Used by the catalog parser and the manifest reader to refuse names that Cargo or crates.io would not
///           accept (`../escape`, `9lives`, `__proto__`); `&str` and not `String` because it only reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isCrateName(name: string): boolean
/// ```
pub fn is_crate_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_CRATE_NAME_LENGTH {
        return false;
    }
    // `chars().next()` yields `Option<char>`: the first character, or `None` for empty text (excluded above).
    // `Some(first)` is the "present" case, like `first !== undefined`.
    let first: Option<char> = name.chars().next();
    if let Some(first_character) = first
        && !first_character.is_ascii_alphabetic()
    {
        return false;
    }
    for character in name.chars() {
        if !is_name_character(character) {
            return false;
        }
    }
    return true;
}

/// What:     `fn is_label(label: &str) -> bool` declares a private function checking a prerelease or build label.
/// Why:      Semver allows ASCII letters, digits, `.` and `-` in labels, and the label must not be empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isLabel(label: string): boolean
/// ```
fn is_label(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }
    for character in label.chars() {
        if !(character.is_ascii_alphanumeric() || character == '.' || character == '-') {
            return false;
        }
    }
    return true;
}

/// What:     `fn is_numeric_component(part: &str) -> bool` declares a private function checking one dot-separated
///           number.
/// Why:      Semver numeric identifiers are digit runs without a leading zero (except `0` itself).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNumericComponent(part: string): boolean
/// ```
fn is_numeric_component(part: &str) -> bool {
    if part.is_empty() {
        return false;
    }
    if part.len() > 1 && part.starts_with('0') {
        return false;
    }
    for character in part.chars() {
        if !character.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// What:     `pub fn is_version(version: &str) -> bool` declares a public function accepting a Cargo version with
///           one to three numeric parts and optional `-prerelease` and `+build` labels (`1`, `0.4`, `1.0.0-alpha.5`).
/// Why:      Cargo requirements usually carry partial versions, so the catalog must accept them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isVersion(version: string): boolean
/// ```
pub fn is_version(version: &str) -> bool {
    // `split_once('+')` returns `Option<(&str, &str)>`: the text before and after the first `+`, or `None`.
    let (without_build, build): (&str, Option<&str>) = match version.split_once('+') {
        Some((before, after)) => (before, Some(after)),
        None => (version, None),
    };
    if let Some(build_label) = build
        && !is_label(build_label)
    {
        return false;
    }
    let (core, prerelease): (&str, Option<&str>) = match without_build.split_once('-') {
        Some((before, after)) => (before, Some(after)),
        None => (without_build, None),
    };
    if let Some(prerelease_label) = prerelease
        && !is_label(prerelease_label)
    {
        return false;
    }
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() > MAX_VERSION_PARTS {
        return false;
    }
    for part in parts {
        if !is_numeric_component(part) {
            return false;
        }
    }
    return true;
}

/// What:     `pub fn is_floor_requirement(requirement: &str) -> bool` accepts exactly one `>=` comparator on a valid
///           version, such as `>=1.0.104`.
/// Why:      The default catalog holds only floors, so the parser checks every registry entry with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isFloorRequirement(requirement: string): boolean
/// ```
pub fn is_floor_requirement(requirement: &str) -> bool {
    return match requirement.strip_prefix(">=") {
        Some(rest) => is_version(rest.trim()),
        None => false,
    };
}

/// What:     `pub fn is_bounded_requirement(requirement: &str) -> bool` accepts a bare caret, `^`, `~` or `=`
///           comparator on a valid version (`0.58`, `^0.58`, `~0.58.1`, `=0.58.0`).
/// Why:      Named catalogs hold bounded requirements; a `>=` floor there would be pulled to the newest release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isBoundedRequirement(requirement: string): boolean
/// ```
pub fn is_bounded_requirement(requirement: &str) -> bool {
    for operator in ["=", "^", "~"] {
        if let Some(rest) = requirement.strip_prefix(operator) {
            return is_version(rest.trim());
        }
    }
    return is_version(requirement.trim());
}

/// What:     `pub fn floor_version(requirement: &str) -> Option<&str>` returns the version text of a `>=` floor.
///           `Option<&str>` is "borrowed text or nothing" (TS `string | undefined`); the result borrows from the
///           input, so no copy is made.
/// Why:      Callers that need the bare version of a default-catalog entry get `None` for anything that is not a
///           floor, instead of a thrown error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function floorVersion(requirement: string): string | undefined
/// ```
pub fn floor_version(requirement: &str) -> Option<&str> {
    if !is_floor_requirement(requirement) {
        return None;
    }
    return requirement.strip_prefix(">=").map(str::trim);
}
