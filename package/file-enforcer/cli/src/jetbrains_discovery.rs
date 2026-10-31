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

/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `EntryKind` from `monochromatic_file_enforcer::real_fs`.
use monochromatic_file_enforcer::real_fs::EntryKind;
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;

/// The JetBrains configuration directory, relative to the home directory the per-user view is rooted at.
pub const CONFIG_DIRECTORY: &str = ".config/JetBrains";

/// The IDE's language server settings file name inside a product's `options` directory.
pub const LANGUAGE_FILE: &str = "LanguageServersSettings.xml";

/// The IDE's user-defined language server settings file name inside a product's `options` directory.
pub const USER_FILE: &str = "UserDefinedLanguageServerSettings.xml";

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
/// function latestOptionsDirectory(view: View, prefixes: string[]): string | undefined
/// ```
pub fn latest_options_directory(view: &mut View, prefixes: &[String]) -> Result<Option<String>, Diagnostic> {
    let mut best: Option<(Vec<u64>, String)> = None;
    for (name, kind) in view.list(CONFIG_DIRECTORY)? {
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
    return Ok(best.map(|(_, name)| return format!("{CONFIG_DIRECTORY}/{name}/options")));
}

/// Tests for the discovery helpers.
#[cfg(test)]
#[path = "jetbrains_discovery_tests.rs"]
mod tests;
