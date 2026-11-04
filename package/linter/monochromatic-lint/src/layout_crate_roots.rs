//! What:
//!  Find the files where a Cargo package's code starts:
//!  the library root and every executable root.
//! Why:
//!  The two layout rules follow the real module tree,
//!  and the tree starts at these files.
//!  `Cargo.toml` is read line by line instead of with a TOML parser because the crate has no TOML dependency
//!  and only two keys matter.
//!  Known limit:
//!  `path` written as a dotted key (`lib.path = "..."`) or an inline table is not read,
//!  and test, example and benchmark targets are left out because they are test-only code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function crateRootFiles(manifest: Path): Path[];
//! ```

/// Import the lexical path normalizer,
///  which never follows a symbolic link.
use crate::markdown_lfs_target::lexical_normal;
/// Import ordered sets so the result is sorted and has no duplicates,
///  and native paths.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// What:
///  The table a line of `Cargo.toml` is inside.
/// Why:
///  Only `path` under `[lib]` and under each `[[bin]]` names a root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Table = 'lib' | 'bin' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Table {
    /// The `[lib]` table.
    Library,
    /// A `[[bin]]` table.
    Executable,
    /// Any other table,
    ///  or no table yet.
    Other,
}

/// What:
///  Read the string after the `=` of a `path = "..."` line.
/// Why:
///  Both TOML string forms are accepted,
///  and a trailing comment after the closing quote is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathValue(line: string): string | undefined;
/// ```
fn path_value(line: &str) -> Option<String> {
    let trimmed: &str = line.trim();
    let after_key: &str = trimmed.strip_prefix("path")?;
    let after_equals: &str = after_key.trim_start().strip_prefix('=')?;
    let value: &str = after_equals.trim_start();
    let quote: char = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inside: &str = &value[quote.len_utf8()..];
    let end: usize = inside.find(quote)?;
    return Some(String::from(&inside[..end]));
}

/// What:
///  The manifest's explicit root paths,
///  relative to the package directory.
/// Why:
///  `[lib] path` and `[[bin]] path` override the default locations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explicitRootPaths(text: string): string[];
/// ```
fn explicit_root_paths(manifest_text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    let mut table: Table = Table::Other;
    for line in manifest_text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            // A table header line switches the current table.
            let header: &str = trimmed.split('#').next().unwrap_or("").trim();
            table = match header {
                "[lib]" => Table::Library,
                "[[bin]]" => Table::Executable,
                _ => Table::Other,
            };
            continue;
        }
        if table == Table::Other {
            continue;
        }
        if let Some(value) = path_value(line) {
            found.push(value);
        }
    }
    return found;
}

/// What:
///  The `.rs` files directly inside a directory,
///  sorted by name.
/// Why:
///  `src/bin/*.rs` are executable roots by Cargo's convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rustFilesIn(directory: Path): Path[];
/// ```
fn rust_files_in(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::<PathBuf>::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        // A link is never followed, so a `uses` link cannot add a second copy of a root.
        let Ok(kind) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if kind.is_file() && path.extension().is_some_and(|extension| return extension == "rs") {
            found.push(path);
        }
    }
    found.sort();
    return found;
}

/// What:
///  The directories directly inside `src/bin`,
///  which hold a `main.rs` each.
/// Why:
///  `src/bin/<name>/main.rs` is an executable root by Cargo's convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function binaryFolders(directory: Path): Path[];
/// ```
fn binary_folders(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::<PathBuf>::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let Ok(kind) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if kind.is_dir() {
            found.push(path);
        }
    }
    found.sort();
    return found;
}

/// What:
///  Every root file of the package whose manifest is given,
///  sorted and without duplicates.
/// Why:
///  A package without any root has nothing to check,
///  and the caller says so instead of reporting a clean package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function crateRootFiles(manifest: Path): Path[];
/// ```
pub fn crate_root_files(manifest: &Path) -> Result<Vec<PathBuf>, String> {
    let text: String = match std::fs::read_to_string(manifest) {
        Ok(read) => read,
        Err(error) => {
            return Err(format!("cannot read {}: {error}", manifest.display()));
        }
    };
    let Some(package): Option<&Path> = manifest.parent() else {
        return Err(format!("{} has no parent folder", manifest.display()));
    };
    let mut candidates: Vec<PathBuf> = Vec::<PathBuf>::new();
    let source: PathBuf = package.join("src");
    candidates.push(source.join("lib.rs"));
    candidates.push(source.join("main.rs"));
    candidates.extend(rust_files_in(&source.join("bin")));
    for folder in binary_folders(&source.join("bin")) {
        candidates.push(folder.join("main.rs"));
    }
    for explicit in explicit_root_paths(text.as_str()) {
        candidates.push(package.join(explicit));
    }
    let mut roots: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for candidate in candidates {
        let normal: PathBuf = lexical_normal(&candidate);
        // Only a regular file can be a root; a missing default is normal.
        if std::fs::metadata(&normal).is_ok_and(|kind| return kind.is_file()) {
            roots.insert(normal);
        }
    }
    return Ok(roots.into_iter().collect::<Vec<PathBuf>>());
}

/// Controls of root discovery stay outside release artifacts.
#[cfg(test)]
#[path = "layout_crate_roots_tests.rs"]
mod tests;
