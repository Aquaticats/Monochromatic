//! What:
//!  Measurement of the repository-pinned tool version beside a rejected configuration.
//! Why:
//!  "The resolved binary may be stale" is a hypothesis,
//!  and this module states it only when a
//! lockfile pin measures a version newer than the running binary;
//!  an unmeasured pin produces no hint at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // staleBinaryHint(configDirectory) returns text only for a measured newer pin.
//! ```

/// Import the running build's version for the measured comparison.
use crate::run_finish::VERSION;
/// Import native paths for ancestor walking and lockfile names.
use std::path::Path;

/// What:
///  The lockfile mise writes beside the configuration file that declares its tools.
/// Why:
///  The pin that governs which binary a repository resolves lives there,
///  not in the tool's
/// own manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOCK_NAME = 'mise.lock';
/// ```
pub const LOCK_NAME: &str = "mise.lock";

/// What:
///  The lockfile table header that owns this tool's resolved version.
/// Why:
///  Only this tool's entry is evidence about this binary;
///  another tool's pin must not reach the
/// hint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TOOL_TABLE = '[[tools."cargo:monochromatic-lint"]]';
/// ```
pub const TOOL_TABLE: &str = "[[tools.\"cargo:monochromatic-lint\"]]";

/// What:
///  The running binary's stale-binary hint for one rejected configuration directory.
/// Why:
///  Production callers get the same measurement the injected-reader controls exercise,
///  and any
/// read failure is indistinguishable from an absent lockfile:
///  no measurement means no hint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleBinaryHint(configDirectory: Path): string | undefined;
/// ```
pub(crate) fn stale_binary_hint(config_directory: &Path) -> Option<String> {
    return stale_binary_hint_with(config_directory, VERSION, &read_lock_text);
}

/// What:
///  One native lockfile read,
///  mapping every failure to absence.
/// Why:
///  A missing,
///  unreadable or non-text lockfile cannot measure a pin,
///  so it must stay silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLockText(path: Path): string | undefined { return tryReadText(path); }
/// ```
fn read_lock_text(path: &Path) -> Option<String> {
    return std::fs::read_to_string(path).ok();
}

/// What:
///  The stale-binary hint built from the nearest lockfile at or above one configuration directory.
/// Why:
///  The nearest lockfile is the one governing the rejected configuration's repository;
///  a
/// farther lockfile belongs to another repository and is not evidence here.
///  The running version is a parameter so that a control can run
/// with a plain `x.y.z` version when the manifest's own version is a pre-release the comparison refuses to order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleBinaryHintWith(configDirectory: Path, read): string | undefined;
/// ```
fn stale_binary_hint_with(
    config_directory: &Path,
    own_version: &str,
    read: &dyn Fn(&Path) -> Option<String>,
) -> Option<String> {
    for directory in config_directory.ancestors() {
        let lock_path = directory.join(LOCK_NAME);
        // What: The first lockfile found governs, whatever it contains.
        // Why: Walking past a lockfile that omits this tool would borrow another repository's pin.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const text = read(lockPath); if (text === undefined) continue;
        // ```
        let Some(lock_text) = read(&lock_path) else {
            continue;
        };
        let pin = pinned_tool_version(lock_text.as_str())?;
        if !older_than_pin(own_version, pin.as_str()) {
            return None;
        }
        return Some(format!(
            "The resolved binary may be stale: {} pins cargo:monochromatic-lint {}, newer than this binary.",
            lock_path.display(),
            pin
        ));
    }
    return None;
}

/// What:
///  The pinned tool version recorded in one lockfile's text.
/// Why:
///  Only a plain `version` key inside this tool's exact table is a pin;
///  any other shape returns
/// absence instead of a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pinnedToolVersion(lockText: string): string | undefined;
/// ```
fn pinned_tool_version(lock_text: &str) -> Option<String> {
    let mut in_tool_table: bool = false;
    for line in lock_text.lines() {
        let trimmed: &str = line.trim();
        if trimmed == TOOL_TABLE {
            in_tool_table = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_tool_table = false;
            continue;
        }
        if !in_tool_table {
            continue;
        }
        if let Some(version) = version_value(trimmed) {
            return Some(version);
        }
    }
    return None;
}

/// What:
///  The quoted value of one `version = "..."` key line.
/// Why:
///  TOML accepts many spellings;
///  accepting only a double-quoted plain value keeps a differently
/// shaped lockfile silent instead of misread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function versionValue(line: string): string | undefined;
/// ```
fn version_value(trimmed: &str) -> Option<String> {
    let (key, value) = trimmed.split_once('=')?;
    if key.trim() != "version" {
        return None;
    }
    let quoted: &str = value.trim();
    let inner = quoted.strip_prefix('"')?.strip_suffix('"')?;
    let plain: bool = !inner.is_empty() && inner.bytes().all(plain_version_byte);
    if !plain {
        return None;
    }
    return Some(String::from(inner));
}

/// What:
///  Whether one version-value byte may appear in a plain lockfile version.
/// Why:
///  A named predicate keeps the byte filter a reusable function path instead of a
/// capture-free closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plainVersionByte(byte: number): boolean;
/// ```
fn plain_version_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || b".+-_".contains(&byte);
}

/// What:
///  Whether one semantic version is older than another.
/// Why:
///  "Stale" claims a direction;
///  a version this module cannot order is unmeasured,
///  and an unmeasured direction must not produce a hint.
///  A pre-release ranks below its own release and above the release before it,
///  as the semantic versioning specification orders them,
///  so a binary of an alpha series is measured like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function olderThanPin(own: string, pinned: string): boolean;
/// ```
fn older_than_pin(own: &str, pinned: &str) -> bool {
    let Some(own_version) = parse_version(own) else {
        return false;
    };
    let Some(pin_version) = parse_version(pinned) else {
        return false;
    };
    return compare_versions(&own_version, &pin_version) == std::cmp::Ordering::Less;
}

/// What:
///  A version split into its numeric core and its pre-release identifiers.
/// Why:
///  Build metadata after `+` never orders versions,
///  so it is dropped when the version is parsed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Version = { core: [number, number, number]; preRelease: string[] };
/// ```
struct Version {
    /// Major,
    ///  minor and patch numbers.
    core: (u64, u64, u64),
    /// The dot-separated identifiers after `-`,
    ///  empty for a release.
    pre_release: Vec<String>,
}

/// What:
///  Parse `x.y.z`,
///  `x.y.z-identifiers` or either with `+metadata` into a version.
/// Why:
///  Lexicographic text comparison would rank 0.10.0 below 0.9.0,
///  and a shape this function cannot read is rejected rather than guessed:
///  a missing component,
///  an empty pre-release or an empty identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseVersion(version: string): Version | undefined;
/// ```
fn parse_version(version: &str) -> Option<Version> {
    let without_build: &str = version.split('+').next()?;
    let (core_text, pre_text): (&str, Option<&str>) = match without_build.split_once('-') {
        Some((core_text, pre_text)) => (core_text, Some(pre_text)),
        None => (without_build, None),
    };
    let mut parts = core_text.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    let patch = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let mut pre_release: Vec<String> = Vec::<String>::new();
    if let Some(text) = pre_text {
        for identifier in text.split('.') {
            if identifier.is_empty() {
                return None;
            }
            pre_release.push(String::from(identifier));
        }
    }
    return Some(Version {
        core: (decimal(major)?, decimal(minor)?, decimal(patch)?),
        pre_release,
    });
}

/// What:
///  The order of two parsed versions.
/// Why:
///  The numeric cores decide first;
///  with equal cores a release is above any pre-release,
///  and two pre-releases compare identifier by identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareVersions(left: Version, right: Version): -1 | 0 | 1;
/// ```
fn compare_versions(left: &Version, right: &Version) -> std::cmp::Ordering {
    let by_core = left.core.cmp(&right.core);
    if by_core != std::cmp::Ordering::Equal {
        return by_core;
    }
    return match (left.pre_release.is_empty(), right.pre_release.is_empty()) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => compare_identifiers(&left.pre_release, &right.pre_release),
    };
}

/// What:
///  The order of two non-empty pre-release identifier lists.
/// Why:
///  Identifiers of digits compare as numbers and rank below identifiers with other characters,
///  which compare as text,
///  and a list that is a prefix of another ranks below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareIdentifiers(left: string[], right: string[]): -1 | 0 | 1;
/// ```
fn compare_identifiers(left: &[String], right: &[String]) -> std::cmp::Ordering {
    for (own, other) in left.iter().zip(right.iter()) {
        let ordered = match (decimal(own), decimal(other)) {
            (Some(own_number), Some(other_number)) => own_number.cmp(&other_number),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => own.as_str().cmp(other.as_str()),
        };
        if ordered != std::cmp::Ordering::Equal {
            return ordered;
        }
    }
    return left.len().cmp(&right.len());
}

/// What:
///  Parse one run of ASCII digits as a number.
/// Why:
///  A sign,
///  space or empty part would turn the comparison back into a text comparison.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decimal(text: string): number | undefined;
/// ```
fn decimal(text: &str) -> Option<u64> {
    if text.is_empty() || !text.bytes().all(ascii_digit) {
        return None;
    }
    return text.parse::<u64>().ok();
}

/// What:
///  Whether one byte is an ASCII decimal digit.
/// Why:
///  A named predicate keeps the digit filter a reusable function path instead of a
/// capture-free closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function asciiDigit(byte: number): boolean;
/// ```
fn ascii_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Pin-measurement controls stay outside release artifacts.
#[cfg(test)]
#[path = "config_pin_tests.rs"]
mod tests;
