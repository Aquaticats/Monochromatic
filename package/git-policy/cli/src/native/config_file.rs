//! What:
//!  Find and read `cli-git.config.jsonc` at a repository root.
//! Why:
//!  Configuration is one data file at one fixed place;
//!  legacy executable
//!      configuration is reported for migration and never read,
//!  run or silently ignored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // loadRepositoryConfig(root) returns { config, source, ignoredLegacy } or throws ConfigError.
//! ```

/// What:
///  `use super::...` imports sibling modules of this crate.
/// Why:
///   Loading combines the error type,
///  the document parser and the typed settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ConfigError } from './config-error.ts';
/// ```
use super::config_error::ConfigError;
use super::config_parse::parse_config;
use super::config_schema::CliGitConfig;
/// What:
///  Import the trait that gives files `.take(..)` and `.read_to_end(..)`.
///       A trait is an interface;
///  its methods exist only while it is in scope.
/// Why:
///   The file is read through a byte cap instead of trusting its reported size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open } from 'node:fs/promises';
/// ```
use std::io::Read;
/// What:
///  `Path` is a borrowed filesystem path;
///  `PathBuf` is its owned form.
///       The same pair as `&str` and `String`,
///  but holding raw OS bytes.
/// Why:
///   Repository paths need not be UTF-8,
///  so they are never converted to text
///       except inside a diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { join } from 'node:path';
/// ```
use std::path::{Path, PathBuf};

/// The only configuration file the native wrapper reads.
pub const CONFIG_FILE_NAME: &str = "cli-git.config.jsonc";

/// What:
///  Executable configuration names of the TypeScript wrapper,
///  in its lookup order.
///       `&[&str]` is a borrowed list of borrowed strings compiled into the executable.
/// Why:
///   Their presence must produce a migration diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LEGACY_CONFIG_FILE_NAMES = ['cli-git.config.mjs', 'cli-git.config.ts'] as const;
/// ```
pub const LEGACY_CONFIG_FILE_NAMES: &[&str] = &["cli-git.config.mjs", "cli-git.config.ts"];

/// What:
///  Largest accepted configuration file,
///  one mebibyte.
///       `u64` is an unsigned 64-bit integer,
///  the type of file sizes (sibling `usize`).
/// Why:
///   A repository file must not control how much memory every Git command reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_CONFIG_BYTES = 1024 * 1024;
/// ```
pub const MAX_CONFIG_BYTES: u64 = 1_048_576;

/// What:
///  The outcome of loading one repository's configuration.
///       `Option<PathBuf>` is "an owned path or nothing";
///  `Vec<PathBuf>` is an owned list.
/// Why:
///   Callers need the settings,
///  the file they came from for diagnostics,
///  and any
///       legacy file that was left beside an authoritative JSONC file so they can
///       report it instead of ignoring it silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LoadedConfig = { config: CliGitConfig; source?: string; ignoredLegacy: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedConfig {
    /// Validated settings;
    ///  `CliGitConfig::defaults()` when the repository has no file.
    pub config: CliGitConfig,
    /// The JSONC file that was read,
    ///  absent when the repository has none.
    pub source: Option<PathBuf>,
    /// Legacy files present beside the JSONC file;
    ///  never read,
    ///  to be reported.
    pub ignored_legacy: Vec<PathBuf>,
}

/// What:
///  Report whether a directory entry exists,
///  without following a final symlink.
///       `Result<bool, ConfigError>` is "true/false or an error".
/// Why:
///   Only "not found" means absent;
///  any other failure (for example a permission
///       error) must stop the command instead of pretending no file exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function entryExists(path: string): Promise<boolean>;
/// ```
fn entry_exists(path: &Path) -> Result<bool, ConfigError> {
    // What: `match` on the metadata `Result`: `Ok(_)` ignores the metadata itself.
    //       `error.kind()` classifies the OS error; `::` walks module paths.
    // Why:  Existence is all that matters for a legacy file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { await lstat(path); return true; } catch (e) { if (e.code === 'ENOENT') return false; throw e; }
    // ```
    match std::fs::symlink_metadata(path) {
        // `Ok(true)` is the success variant carrying the answer.
        Ok(_) => return Ok(true),
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(false);
            }
            // `Err(...)` is the failure variant; `path.display()` renders lossy text.
            return Err(ConfigError::new(
                format!("Cannot inspect {}: {error}.", path.display()).as_str(),
            ));
        }
    }
}

/// What:
///  Explain a legacy configuration file that has no JSONC replacement yet.
/// Why:
///   The author needs to know it was not run,
///  where the replacement goes and
///       which parts have no equivalent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function migrationRequired(legacy: string, target: string): ConfigError;
/// ```
fn migration_required(legacy: &Path, target: &Path) -> ConfigError {
    return ConfigError::new(
        format!(
            "Legacy configuration {} is not executed or read by the native cli-git. \
             Translate its settings into {} (data only: policies, hooks, indexLock, landing), \
             then remove the legacy file. Plugins, executable paths, command arrays and trust \
             settings have no JSONC equivalent because every policy is built in.",
            legacy.display(),
            target.display()
        )
        .as_str(),
    );
}

/// What:
///  Describe a legacy file that sits beside an authoritative JSONC file.
///       `String` is owned text returned to the caller.
/// Why:
///   The executable prints this so the stale file is never ignored silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoredLegacyNotice(legacy: string, source: string): string;
/// ```
pub fn ignored_legacy_notice(legacy: &Path, source: &Path) -> String {
    return format!(
        "Legacy configuration {} is ignored: {} is authoritative for the native cli-git. \
         Remove the legacy file once no TypeScript cli-git reads it.",
        legacy.display(),
        source.display()
    );
}

/// What:
///  Read at most `MAX_CONFIG_BYTES` of a regular,
///  non-symlink file as UTF-8 text.
/// Why:
///   A symbolic link could point outside the repository,
///  a directory or device is
///       not configuration,
///  and an oversized or non-UTF-8 file is rejected by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readConfigText(path: string): Promise<string>;
/// ```
fn read_config_text(path: &Path) -> Result<String, ConfigError> {
    // What: `let x = match ... { Ok(v) => v, Err(e) => return Err(...) };` unwraps or exits.
    // Why:  Metadata without following the link reveals a symlink before it is opened.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entry = await lstat(path);
    // ```
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) => {
            return Err(ConfigError::new(
                format!("Cannot inspect {}: {error}.", path.display()).as_str(),
            ));
        }
    };
    if metadata.file_type().is_symlink() {
        return Err(ConfigError::new(
            format!(
                "Configuration path must not be a symbolic link: {}. Replace it with a regular file.",
                path.display()
            )
            .as_str(),
        ));
    }
    if !metadata.is_file() {
        return Err(ConfigError::new(
            format!(
                "Configuration path must be a regular file: {}.",
                path.display()
            )
            .as_str(),
        ));
    }
    let file: std::fs::File = match std::fs::File::open(path) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(ConfigError::new(
                format!("Cannot open {}: {error}.", path.display()).as_str(),
            ));
        }
    };
    // What: `Vec::<u8>::new()` is an empty owned byte list (`u8` is one byte, 0 to 255).
    // Why:  Bytes are validated as UTF-8 only after the size cap has been enforced.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bytes = Buffer.alloc(0);
    // ```
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    // What: `.take(n)` stops reading after `n` bytes; `&mut bytes` lends the buffer for
    //       appending. One byte past the cap proves the file is too large.
    // Why:  The cap holds even if the file grows after its metadata was read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { bytesRead } = await handle.read(buffer, 0, MAX_CONFIG_BYTES + 1, 0);
    // ```
    if let Err(error) = file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes) {
        return Err(ConfigError::new(
            format!("Cannot read {}: {error}.", path.display()).as_str(),
        ));
    }
    // `as u64` widens the list length for comparison with the byte cap.
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(ConfigError::new(
            format!(
                "Configuration file {} is larger than {MAX_CONFIG_BYTES} bytes.",
                path.display()
            )
            .as_str(),
        ));
    }
    // `String::from_utf8` takes ownership of the bytes and checks them without copying.
    match String::from_utf8(bytes) {
        Ok(text) => return Ok(text),
        Err(error) => {
            return Err(ConfigError::new(
                format!(
                    "Configuration file {} is not UTF-8 text: {error}.",
                    path.display()
                )
                .as_str(),
            ));
        }
    }
}

/// What:
///  Load the configuration of the repository whose top level is `repository_root`.
/// Why:
///   This is the single entry point for policy settings.
///  Nothing in a repository
///       is executed:
///  the JSONC file is parsed as data,
///  and legacy `.mjs`/`.ts`
///       configuration is only detected by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function loadRepositoryConfig(repositoryRoot: string): Promise<LoadedConfig>;
/// ```
pub fn load_repository_config(repository_root: &Path) -> Result<LoadedConfig, ConfigError> {
    // `.join(name)` appends one path segment and returns an owned path.
    let source: PathBuf = repository_root.join(CONFIG_FILE_NAME);
    // `Vec::<PathBuf>::new()` is an empty owned list; `mut` allows pushing.
    let mut legacy: Vec<PathBuf> = Vec::<PathBuf>::new();
    // `for name in LEGACY_CONFIG_FILE_NAMES` visits each compiled-in name.
    for name in LEGACY_CONFIG_FILE_NAMES {
        let candidate: PathBuf = repository_root.join(name);
        // A trailing `?` returns the helper's error to our caller, or unwraps its value.
        if entry_exists(candidate.as_path())? {
            legacy.push(candidate);
        }
    }
    if !entry_exists(source.as_path())? {
        // `.first()` is `Some(&item)` for a non-empty list, `None` otherwise.
        if let Some(first) = legacy.first() {
            return Err(migration_required(first.as_path(), source.as_path()));
        }
        return Ok(LoadedConfig {
            config: CliGitConfig::defaults(),
            // `None` records that no file was read.
            source: None,
            ignored_legacy: legacy,
        });
    }
    let text: String = read_config_text(source.as_path())?;
    // What: `.strip_prefix(..)` is `Some(rest)` when the text starts with the marker.
    //       `.unwrap_or(default)` picks `rest`, or the whole text when there is no marker.
    // Why:  A UTF-8 byte order mark is an editor artefact, not configuration content.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const body = text.startsWith('﻿') ? text.slice(1) : text;
    // ```
    let body: &str = text.strip_prefix('\u{feff}').unwrap_or(text.as_str());
    match parse_config(body) {
        Ok(config) => {
            return Ok(LoadedConfig {
                config,
                // `Some(source)` records the file the settings came from.
                source: Some(source),
                ignored_legacy: legacy,
            });
        }
        Err(error) => {
            return Err(ConfigError::new(
                format!("{}: {}", source.display(), error.message).as_str(),
            ));
        }
    }
}

/// Disposable-directory controls stay out of the release executable.
#[cfg(test)]
#[path = "config_file_tests.rs"]
mod tests;

/// Controls on the repository's own configuration file stay out of the release executable.
#[cfg(test)]
#[path = "config_repository_tests.rs"]
mod repository_tests;
