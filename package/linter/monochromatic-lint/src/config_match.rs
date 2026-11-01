//! What:
//!  Compile and apply configuration-relative file patterns.
//! Why:
//!  Each selected configuration keeps its own pattern base and ordered rule settings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prepareConfiguration(source).resolve(relativeFile) returns ignored, unconfigured, or effective rules.
//! ```

/// Import ordered merging and validated source blocks.
use crate::config_error::ConfigError;
/// Import the discovered configuration record that each candidate path is matched against.
use crate::config_lookup::ConfigurationSource;
/// Import the value merge that layers matching configuration blocks into one effective value.
use crate::config_merge::merge_values;
/// Import the block type whose path globs decide which files a configuration covers.
use crate::configuration::ConfigBlock;
/// Import the rule completion step that fills in defaults after merging.
use crate::resolved_rules::complete_rules;
/// Import the incumbent Rust glob compiler;
///  it matches native path bytes.
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
/// Import the JSONC value type that matched configuration is held in before merging.
use monochromatic_jsonc_edit::JsoncValue;
/// Import native paths and the configuration value model.
use std::path::{Path, PathBuf};

/// What:
///  One compiled block and its validated settings.
/// Why:
///  Repeated files reuse pattern compilation instead of compiling each glob again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CompiledBlock = { files: GlobSet; ignores: GlobSet; block: ConfigBlock };
/// ```
struct CompiledBlock {
    /// Positive file selectors.
    files: GlobSet,
    /// Exclusions,
    ///  including directory descendants for trailing-slash patterns.
    ignores: GlobSet,
    /// Original ordered settings and global-ignore classification.
    block: ConfigBlock,
}

/// What:
///  A prepared configuration with explicit source and matching base.
/// Why:
///  Callers can report source paths and derive relative candidate names without rereading configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PreparedConfiguration { path: Path; base: Path; blocks: CompiledBlock[] }
/// ```
pub struct PreparedConfiguration {
    /// Source path used for diagnostics.
    pub path: PathBuf,
    /// Directory that candidate names are relative to.
    pub base: PathBuf,
    /// Compiled blocks retained in authored order.
    blocks: Vec<CompiledBlock>,
}

/// What:
///  Distinguish global exclusion,
///  no matching block,
///  and configured files.
/// Why:
///  An empty effective rule set is not the same state as an unmatched file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileConfiguration = { kind: 'ignored' } | { kind: 'unconfigured' }
///   | { kind: 'configured'; rules: JsoncValue };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileConfiguration {
    /// A global ignores-only block removes this candidate entirely.
    Ignored,
    /// No ordinary block selected this candidate.
    Unconfigured,
    /// At least one block selected the file;
    ///  absent rules remain absent.
    Configured {
        /// Final validated settings with per-rule option defaults.
        rules: JsoncValue,
    },
}

/// What:
///  Compile one OR-set of configuration patterns.
/// Why:
///  A single * does not cross directories,
///  and the pattern grammar is consistent across platforms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compilePatterns(patterns: string[], directories: boolean): GlobSet;
/// ```
fn compile_patterns(patterns: &[String], directories: bool) -> Result<GlobSet, ConfigError> {
    let mut compiled = GlobSetBuilder::new();
    for pattern in patterns {
        // A trailing slash explicitly selects a directory's descendants.
        let expanded = if directories && pattern.ends_with('/') {
            format!("{pattern}**")
        } else {
            pattern.clone()
        };
        let mut builder = GlobBuilder::new(expanded.as_str());
        builder.literal_separator(true);
        builder.backslash_escape(true);
        let glob = match builder.build() {
            Ok(glob) => glob,
            Err(error) => {
                return Err(ConfigError::new(
                    format!("Invalid configuration pattern {pattern:?}: {error}").as_str(),
                ));
            }
        };
        compiled.add(glob);
    }
    match compiled.build() {
        Ok(pattern_set) => return Ok(pattern_set),
        Err(error) => {
            return Err(ConfigError::new(
                format!("Configuration pattern compilation failed: {error}").as_str(),
            ));
        }
    }
}

/// What:
///  Compile patterns once for the selected source.
/// Why:
///  This consumes validated blocks without making a second rule registry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareConfiguration(source: ConfigurationSource): PreparedConfiguration;
/// ```
pub fn prepare_configuration(
    source: ConfigurationSource,
) -> Result<PreparedConfiguration, ConfigError> {
    let mut blocks = Vec::with_capacity(source.blocks.len());
    for block in source.blocks {
        let files = compile_patterns(block.files.as_slice(), false)?;
        let ignores = compile_patterns(block.ignores.as_slice(), true)?;
        blocks.push(CompiledBlock {
            files,
            ignores,
            block,
        });
    }
    return Ok(PreparedConfiguration {
        path: source.path,
        base: source.base,
        blocks,
    });
}

/// What:
///  Resolve one config-relative logical path,
///  including virtual-file suffixes.
/// Why:
///  The same operation selects rules for real files,
///  fenced Rust and rustdoc Markdown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolve(relativeFile: Path): FileConfiguration;
/// ```
impl PreparedConfiguration {
    /// Select matching blocks and finalize their full-group merge.
    pub fn resolve(&self, relative_file: &Path) -> Result<FileConfiguration, ConfigError> {
        // An absolute path would silently use the wrong pattern base.
        // Parent components remain valid for files explicitly selected outside cwd with --config.
        if relative_file.is_absolute() {
            return Err(ConfigError::new(
                "Configuration matching requires a candidate path relative to the configuration base.",
            ));
        }
        for compiled in &self.blocks {
            if compiled.block.global_ignore && compiled.ignores.is_match(relative_file) {
                return Ok(FileConfiguration::Ignored);
            }
        }
        let mut settings = Vec::new();
        for compiled in &self.blocks {
            if compiled.block.global_ignore {
                continue;
            }
            if compiled.files.is_match(relative_file) && !compiled.ignores.is_match(relative_file) {
                settings.push(&compiled.block.rules);
            }
        }
        if settings.is_empty() {
            return Ok(FileConfiguration::Unconfigured);
        }
        let rules = complete_rules(merge_values(settings.as_slice()))?;
        return Ok(FileConfiguration::Configured { rules });
    }
}

/// Keep matching regressions in the verification build.
#[cfg(test)]
#[path = "config_match_tests.rs"]
mod tests;
