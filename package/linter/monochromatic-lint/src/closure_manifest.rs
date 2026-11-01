//! What:
//!  The per-package prerequisite manifest of the closure check.
//! Why:
//!  A package may need machine facts before `cargo check` can compile it (a system library,
//! a tool,
//! prepared artifacts).
//!  The manifest names them from a fixed list of kinds that this program implements;
//!  it never holds a command,
//! so repository JSONC still cannot choose an executable to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Prerequisite = { kind: 'pkg-config'; modules: string[] } | { kind: 'executables'; names: string[] }
//!   | { kind: 'qmake' } | { kind: 'ide-runtime-artifacts' } | { kind: 'android-ndk' };
//! function readPrerequisites(directory: Path): Prerequisite[]; // throws string
//! ```

/// Import the shared data validation and string readers of the configuration.
use crate::config_data::{key_text, strings, validate_data};
/// Import the exact JSONC parser and value model.
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue, parse_jsonc};
/// Import the path types that locate the manifest beside a package's `Cargo.toml`.
use std::path::Path;

/// What:
///  The file name of the manifest,
///  looked up in the directory of the package's `Cargo.toml`.
/// Why:
///  A package without the file has no prerequisites beyond the generic ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PREREQUISITES_FILE = 'monochromatic-lint.prerequisites.jsonc';
/// ```
pub const PREREQUISITES_FILE: &str = "monochromatic-lint.prerequisites.jsonc";

/// What:
///  The kinds a manifest may name.
/// Why:
///  An unknown key is an error that names this list,
///  so a misspelling cannot silently drop a prerequisite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KINDS = ['pkg-config', 'executables', 'qmake', 'ide-runtime-artifacts', 'android-ndk'] as const;
/// ```
pub const KINDS: &[&str] = &[
    "pkg-config",
    "executables",
    "qmake",
    "ide-runtime-artifacts",
    "android-ndk",
];

/// What:
///  One prerequisite of a package.
/// Why:
///  Each variant is a check or a preparation that this program implements;
///  the manifest only selects among them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prerequisite = { kind: string; names?: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Prerequisite {
    /// Every named `pkg-config` module is installed.
    PkgConfig(
        /// The module names,
        ///  each a plain name.
        Vec<String>,
    ),
    /// Every named program is on `PATH`.
    Executables(
        /// The program names,
        ///  each a plain name.
        Vec<String>,
    ),
    /// A Qt 6 `qmake` is available and `QMAKE` points at it.
    Qmake,
    /// The IDE's runtime and licenses artifacts are prepared,
    ///  and the build directory links to them.
    IdeRuntimeArtifacts,
    /// The Android NDK is installed and the cross-compilation variables are set.
    AndroidNdk,
}

/// What:
///  True for a module or program name that cannot be read as an option or a path.
/// Why:
///  The names reach the argument vector of `pkg-config` and a `PATH` search,
///  so a leading dash,
/// a separator or a space is refused instead of escaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function validName(name: string): boolean { return /^[A-Za-z0-9][A-Za-z0-9._+-]{0,127}$/.test(name); }
/// ```
fn valid_name(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() || name.len() > 128 {
        return false;
    }
    for character in characters {
        if !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | '+' | '-') {
            return false;
        }
    }
    return true;
}

/// What:
///  Read a non-empty list of valid names for one kind.
/// Why:
///  An empty list would name no prerequisite,
///  which is almost certainly a mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function names(kind: string, value: JsoncValue): string[]; // throws string
/// ```
fn names(kind: &str, value: &JsoncValue) -> Result<Vec<String>, String> {
    let list: Vec<String> = match strings(value, kind) {
        Ok(found) => found,
        Err(error) => return Err(format!("{kind}: {error}")),
    };
    if list.is_empty() {
        return Err(format!(
            "{kind} names no entry; remove the key or list a name."
        ));
    }
    for name in &list {
        if !valid_name(name) {
            return Err(format!(
                "{kind} entry {name:?} is not a plain name (letters, digits, dot, underscore, plus and dash; no separator or leading dash)."
            ));
        }
    }
    return Ok(list);
}

/// What:
///  Accept only the literal `true` for a kind that takes no argument.
/// Why:
///  `false` or a list would read as a setting this kind does not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onlyTrue(kind: string, value: JsoncValue): void; // throws string
/// ```
fn only_true(kind: &str, value: &JsoncValue) -> Result<(), String> {
    if let JsoncKind::Boolean { value: true } = value.kind {
        return Ok(());
    }
    return Err(format!("{kind} takes the value true."));
}

/// What:
///  Parse the text of a manifest.
/// Why:
///  The text is data only:
///  an object whose keys are kinds from the fixed list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePrerequisites(source: string): Prerequisite[]; // throws string
/// ```
pub fn parse_prerequisites(source: &str) -> Result<Vec<Prerequisite>, String> {
    let root: JsoncValue = match parse_jsonc(source) {
        Ok(value) => value,
        Err(error) => return Err(format!("it is not valid JSONC: {error:?}")),
    };
    if let Err(error) = validate_data(&root) {
        return Err(format!("{error}"));
    }
    let Some(entries) = root.entries() else {
        return Err(String::from(
            "it must be an object whose keys name prerequisites.",
        ));
    };
    let mut found: Vec<Prerequisite> = Vec::<Prerequisite>::new();
    for entry in entries {
        let key: String = match key_text(&entry.key.units) {
            Ok(text) => text,
            Err(error) => return Err(format!("{error}")),
        };
        match key.as_str() {
            "pkg-config" => found.push(Prerequisite::PkgConfig(names(&key, &entry.value)?)),
            "executables" => found.push(Prerequisite::Executables(names(&key, &entry.value)?)),
            "qmake" => {
                only_true(&key, &entry.value)?;
                found.push(Prerequisite::Qmake);
            }
            "ide-runtime-artifacts" => {
                only_true(&key, &entry.value)?;
                found.push(Prerequisite::IdeRuntimeArtifacts);
            }
            "android-ndk" => {
                only_true(&key, &entry.value)?;
                found.push(Prerequisite::AndroidNdk);
            }
            _ => {
                return Err(format!(
                    "{key:?} is not a prerequisite this linter implements; the kinds are {}.",
                    KINDS.join(", ")
                ));
            }
        }
    }
    return Ok(found);
}

/// What:
///  Read the manifest of a package directory,
///  or no prerequisites when the file is absent.
/// Why:
///  A manifest that exists but cannot be read or understood must stop the package,
///  not be treated as empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPrerequisites(directory: Path): Prerequisite[]; // throws string
/// ```
pub fn read_prerequisites(directory: &Path) -> Result<Vec<Prerequisite>, String> {
    let path = directory.join(PREREQUISITES_FILE);
    let source: String = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(Vec::<Prerequisite>::new());
            }
            return Err(format!("{PREREQUISITES_FILE} cannot be read: {error}."));
        }
    };
    return match parse_prerequisites(source.as_str()) {
        Ok(found) => Ok(found),
        Err(reason) => Err(format!("{PREREQUISITES_FILE}: {reason}")),
    };
}

/// Manifest grammar controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_manifest_tests.rs"]
mod tests;
