//! What:
//!  Configuration discovery with explicit filesystem ownership.
//! Why:
//!  Nearest-file lookup and --config have different pattern bases and must be tested independently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // discoverConfiguration({ file, cwd, explicitConfig }) returns one configuration, not an ancestor merge.
//! ```

/// Import typed parsing and setup failures.
use crate::config_error::ConfigError;
/// Import the measured staleness hint that names a stale resolved binary.
use crate::config_pin::stale_binary_hint;
/// Import the block model and parser that turn each configuration file into rules.
use crate::configuration::{ConfigBlock, parse_configuration};
/// Import native path values rather than requiring filesystem names to be UTF-8 strings.
use std::path::{Path, PathBuf};

/// What:
///  The sole discovered configuration filename.
/// Why:
///  HCL and executable configuration are not alternate formats for this implementation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONFIG_NAME = 'monochromatic-lint.config.jsonc';
/// ```
pub const CONFIG_NAME: &str = "monochromatic-lint.config.jsonc";

/// What:
///  One selected source and the directory its patterns are relative to.
/// Why:
///  An explicit temporary config uses cwd as its base,
///  unlike a discovered config.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigurationSource = { path: Path; base: Path; blocks: ConfigBlock[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationSource {
    /// Selected source file,
    ///  retained for diagnostics.
    pub path: PathBuf,
    /// Pattern base,
    ///  deliberately independent from the selected source path.
    pub base: PathBuf,
    /// Parsed blocks in authored order.
    pub blocks: Vec<ConfigBlock>,
}

/// What:
///  The filesystem operation configuration discovery needs.
/// Why:
///  A memory-backed test adapter proves root and missing-file behavior without reading real user configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ConfigFilesystem { read(path: Path): string | undefined }
/// ```
pub(crate) trait ConfigFilesystem {
    /// Return absence only when no file exists;
    ///  unreadable or non-file inputs are failures.
    fn read(&self, path: &Path) -> Result<Option<String>, ConfigError>;
}

/// What:
///  The native read-only configuration adapter.
/// Why:
///  Production discovery reads the actual filesystem,
///  while tests can supply isolated trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NativeConfigFilesystem implements ConfigFilesystem {}
/// ```
struct NativeConfigFilesystem;

/// What:
///  Implement native reads without accepting directories or other special entries as configuration.
/// Why:
///  A malformed nearest configuration must not silently fall through to an ancestor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// read(path: Path): string | undefined;
/// ```
impl ConfigFilesystem for NativeConfigFilesystem {
    /// Read a regular configuration file or return a typed filesystem diagnostic.
    fn read(&self, path: &Path) -> Result<Option<String>, ConfigError> {
        // What: Distinguish a present file, absence, and an actual metadata failure.
        // Why: Only absence permits discovery to continue toward an ancestor.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const metadata = statOrMissing(path);
        // ```
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound
                    || error.kind() == std::io::ErrorKind::NotADirectory
                {
                    // Missing virtual-file directories are normal during ancestor lookup.
                    return Ok(None);
                }
                return Err(ConfigError::new(
                    format!("Cannot inspect configuration {}: {error}", path.display()).as_str(),
                ));
            }
        };
        if !metadata.is_file() {
            return Err(ConfigError::new(
                format!("Configuration {} is not a regular file.", path.display()).as_str(),
            ));
        }
        // What: Read owned UTF-8 source, preserving I/O errors instead of treating them as absent configuration.
        // Why: Invalid encoding must not disable configured rules by causing an ancestor fallback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return readUtf8OrThrow(path);
        // ```
        match std::fs::read_to_string(path) {
            Ok(source) => return Ok(Some(source)),
            Err(error) => {
                return Err(ConfigError::new(
                    format!("Cannot read configuration {}: {error}", path.display()).as_str(),
                ));
            }
        }
    }
}

/// What:
///  Resolve one possibly relative path against the caller's explicit working directory.
/// Why:
///  Lookup tests and the executable must not depend on a hidden process cwd read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absolutePath(path: Path, cwd: Path): Path;
/// ```
fn absolute_path(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        // Copy the borrowed path into result-owned storage without converting it to text.
        return path.to_path_buf();
    }
    return cwd.join(path);
}

/// What:
///  Parse a selected file and attach its original path to syntax or schema failures.
/// Why:
///  Users need to know which of several ancestor configurations rejected their input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selected(path: Path, base: Path, source: string): ConfigurationSource;
/// ```
fn selected(
    path: PathBuf,
    base: PathBuf,
    source: &str,
) -> Result<ConfigurationSource, ConfigError> {
    // Inspect the typed parser result and retain the affected source path on failure.
    match parse_configuration(source) {
        Ok(blocks) => return Ok(ConfigurationSource { path, base, blocks }),
        Err(error) => {
            return Err(rejected(&path, error));
        }
    }
}

/// What:
///  Wrap one rejected configuration document with its path and any measured staleness context.
/// Why:
///  The reader must see which file failed,
///  the inner message already ends with the running binary's
/// identity,
///  and a stale-binary hint belongs only to an unregistered rule name
///  beside a repository pin
/// that measures a newer version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rejected(path: Path, error: ConfigError): ConfigError;
/// ```
fn rejected(path: &Path, error: ConfigError) -> ConfigError {
    // What: Measure staleness only for an unregistered rule name.
    // Why: A syntax or option failure says nothing about which rules another version registers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hint = error.unknownRule !== undefined ? staleBinaryHint(dirname(path)) : undefined;
    // ```
    let hint: Option<String> = match (error.unknown_rule.is_some(), path.parent()) {
        (true, Some(directory)) => stale_binary_hint(directory),
        _ => None,
    };
    let mut message: String = format!("Configuration {}: {}", path.display(), error.message);
    if let Some(text) = hint {
        message.push(' ');
        message.push_str(text.as_str());
    }
    return ConfigError::composed(message, error.unknown_rule);
}

/// What:
///  Discover one configuration through a supplied filesystem adapter.
/// Why:
///  Explicit config and nearest-ancestor lookup share error handling but not base-directory rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function discoverWithFilesystem(file: Path, cwd: Path, explicit: Path | undefined, fs: ConfigFilesystem);
/// ```
pub(crate) fn discover_with_filesystem(
    file: &Path,
    cwd: &Path,
    explicit: Option<&Path>,
    filesystem: &impl ConfigFilesystem,
) -> Result<Option<ConfigurationSource>, ConfigError> {
    if !cwd.is_absolute() {
        return Err(ConfigError::new(
            "Configuration lookup requires an absolute working directory.",
        ));
    }
    // What: A present override bypasses discovery, even when its file is outside the repository.
    // Why: The commit adapter supplies a temporary one-rule config whose patterns still target cwd.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (explicit !== undefined) return loadExplicit(explicit, cwd);
    // ```
    if let Some(explicit_path) = explicit {
        let path = absolute_path(explicit_path, cwd);
        let Some(source) = filesystem.read(&path)? else {
            return Err(ConfigError::new(
                format!("Explicit configuration {} does not exist.", path.display()).as_str(),
            ));
        };
        let result = selected(path, cwd.to_path_buf(), source.as_str())?;
        return Ok(Some(result));
    }
    let path = absolute_path(file, cwd);
    // What: Start at the linted file's parent and walk only toward the filesystem root.
    // Why: Config files from multiple directories are never merged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const directory of ancestors(dirname(file))) { ... }
    // ```
    if let Some(parent) = path.parent() {
        for directory in parent.ancestors() {
            let candidate = directory.join(CONFIG_NAME);
            if let Some(source) = filesystem.read(&candidate)? {
                let result = selected(candidate, directory.to_path_buf(), source.as_str())?;
                return Ok(Some(result));
            }
        }
    }
    return Ok(None);
}

/// What:
///  Read the configuration file of exactly one directory,
///  without consulting its ancestors.
/// Why:
///  The executable memoizes nearest-file lookup per directory,
///  so each candidate file is read
/// and parsed once per run instead of once per linted file.
/// Its patterns resolve against that directory,
///  as in ancestor discovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadConfigurationAt(directory: Path): ConfigurationSource | undefined;
/// ```
pub fn load_configuration_at(directory: &Path) -> Result<Option<ConfigurationSource>, ConfigError> {
    if !directory.is_absolute() {
        return Err(ConfigError::new(
            "Configuration lookup requires an absolute directory.",
        ));
    }
    let candidate: PathBuf = directory.join(CONFIG_NAME);
    let Some(source): Option<String> = NativeConfigFilesystem.read(&candidate)? else {
        return Ok(None);
    };
    let result: ConfigurationSource =
        selected(candidate, directory.to_path_buf(), source.as_str())?;
    return Ok(Some(result));
}

/// What:
///  Production configuration lookup using the native read-only filesystem.
/// Why:
///  All runtime calls use the same discovery implementation exercised by the memory-backed tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function discoverConfiguration(file: Path, cwd: Path, explicit?: Path): ConfigurationSource | undefined;
/// ```
pub fn discover_configuration(
    file: &Path,
    cwd: &Path,
    explicit: Option<&Path>,
) -> Result<Option<ConfigurationSource>, ConfigError> {
    return discover_with_filesystem(file, cwd, explicit, &NativeConfigFilesystem);
}

/// Keep filesystem fixtures out of release artifacts.
#[cfg(test)]
#[path = "config_lookup_tests.rs"]
mod tests;
