//! What:
//!  Find the newest installed JetBrains product's `options` directory through the virtual view.
//! Why:
//!  The IDE keeps one directory per product and version under `.config/JetBrains`
//!  (for example `IntelliJIdea2026.2`).
//!  The policy applies to the newest version of the requested products,
//! and a newer install without the settings files is a clear miss,
//! never a reason to edit an older install.
//!  Listing goes through the view so the directory is a recorded input.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function latestOptionsDirectory(view: View, prefixes: string[]): string | undefined
//! ```

/// Imports `OsStr` from `std::ffi`.
use std::ffi::OsStr;
/// Imports `Component`,
/// `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Component, Path, PathBuf};
/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `EntryKind` from `monochromatic_file_enforcer::real_fs`.
use monochromatic_file_enforcer::real_fs::EntryKind;
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;

/// The JetBrains configuration directory when `XDG_CONFIG_HOME` does not name one,
/// relative to the home directory the per-user view is rooted at.
pub const CONFIG_DIRECTORY: &str = ".config/JetBrains";

/// The IDE's language server settings file name inside a product's `options` directory.
pub const LANGUAGE_FILE: &str = "LanguageServersSettings.xml";

/// The IDE's user-defined language server settings file name inside a product's `options` directory.
pub const USER_FILE: &str = "UserDefinedLanguageServerSettings.xml";

/// What:
///  The part of an absolute directory below the view root,
/// or nothing when the directory is not below it.
/// Why:
///  The view reads only paths below its root.
///  The lexical comparison is tried first so a configuration directory that is itself a link is not resolved away;
/// the resolved comparison then covers a home reached through a different spelling (`/home` and `/var/home`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function belowRoot(root: string, directory: string): string | undefined
/// ```
fn below_root(root: &Path, directory: &Path) -> Option<PathBuf> {
    if let Ok(rest) = directory.strip_prefix(root) {
        return Some(rest.to_path_buf());
    }
    let real_root = root.canonicalize().ok()?;
    let real_directory = directory.canonicalize().ok()?;
    let rest = real_directory.strip_prefix(&real_root).ok()?;
    return Some(rest.to_path_buf());
}

/// What:
///  The home-relative path of the `JetBrains` directory for a given `XDG_CONFIG_HOME` value.
/// Why:
///  The retired TypeScript tool read `XDG_CONFIG_HOME` when it was set, else `~/.config`.
///  This follows the XDG Base Directory rule for the value:
/// an unset, empty or relative value is ignored and `.config` under the home directory is used.
///  An absolute value must lie below the view root, because the view cannot read outside it;
/// anything else is an error naming both paths, so the rule warns instead of editing a different directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jetbrainsConfigDirectory(root: string, xdgConfigHome: string | undefined): string // throws
/// ```
pub fn jetbrains_config_directory(root: &Path, xdg_config_home: Option<&OsStr>) -> Result<String, String> {
    let Some(value) = xdg_config_home else { return Ok(CONFIG_DIRECTORY.to_string()) };
    let configured = Path::new(value);
    if !configured.is_absolute() {
        return Ok(CONFIG_DIRECTORY.to_string());
    }
    let Some(rest) = below_root(root, configured) else {
        return Err(format!(
            "XDG_CONFIG_HOME is {}, which is outside the home directory {} that this run is rooted at",
            configured.display(),
            root.display()
        ));
    };
    let mut segments: Vec<&str> = Vec::new();
    for component in rest.components() {
        let Component::Normal(segment) = component else {
            return Err(format!("XDG_CONFIG_HOME is {}, which contains a path segment other than a plain name", configured.display()));
        };
        let Some(text) = segment.to_str() else {
            return Err(format!("XDG_CONFIG_HOME is {}, which is not valid UTF-8", configured.display()));
        };
        segments.push(text);
    }
    segments.push("JetBrains");
    return Ok(segments.join("/"));
}

/// What:
///  The numeric version of a product directory name,
/// when it starts with one of the prefixes and the rest is dot-separated decimal digits.
/// Why:
///  `IntelliJIdea2026.2-backup` and `IntelliJIdeaCE` are not versioned products.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function versionParts(name: string, prefixes: string[]): number[] | undefined
/// ```
pub fn version_parts(name: &str, prefixes: &[String]) -> Option<Vec<u64>> {
    let prefix = prefixes.iter().find(|candidate| return name.starts_with(candidate.as_str()))?;
    let suffix = &name[prefix.len()..];
    let mut parts: Vec<u64> = Vec::new();
    for segment in suffix.split('.') {
        if segment.is_empty() || !segment.bytes().all(|byte| return byte.is_ascii_digit()) {
            return None;
        }
        parts.push(segment.parse::<u64>().ok()?);
    }
    return Some(parts);
}

/// What:
///  Compare two version tuples element by element,
/// padding the shorter with zeros.
/// Why:
///  `2026.2` and `2026.2.0` are the same version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareVersionParts(left: number[], right: number[]): number
/// ```
pub fn compare_versions(left: &[u64], right: &[u64]) -> std::cmp::Ordering {
    let length = left.len().max(right.len());
    for index in 0..length {
        let left_part = left.get(index).copied().unwrap_or(0);
        let right_part = right.get(index).copied().unwrap_or(0);
        if left_part != right_part {
            return left_part.cmp(&right_part);
        }
    }
    return std::cmp::Ordering::Equal;
}

/// What:
///  The home-relative `options` directory of the newest matching product,
/// or nothing when no product matches.
/// Why:
///  Equal versions are broken by the larger directory name,
/// so the choice never depends on listing order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function latestOptionsDirectory(view: View, configDirectory: string, prefixes: string[]): string | undefined
/// ```
pub fn latest_options_directory(view: &mut View, config_directory: &str, prefixes: &[String]) -> Result<Option<String>, Diagnostic> {
    let mut best: Option<(Vec<u64>, String)> = None;
    for (name, kind) in view.list(config_directory)? {
        if kind != EntryKind::Directory {
            continue;
        }
        let Some(version) = version_parts(&name, prefixes) else { continue };
        let newer = match &best {
            None => true,
            Some((best_version, best_name)) => match compare_versions(&version, best_version) {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Equal => name > *best_name,
                std::cmp::Ordering::Less => false,
            },
        };
        if newer {
            best = Some((version, name));
        }
    }
    return Ok(best.map(|(_, name)| return format!("{config_directory}/{name}/options")));
}

/// Tests for the discovery helpers.
#[cfg(test)]
#[path = "jetbrains_discovery_tests.rs"]
mod tests;
