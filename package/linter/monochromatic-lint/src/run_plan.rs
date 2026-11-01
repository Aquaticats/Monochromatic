//! What:
//!  Decide,
//!  before any rule runs,
//!  what each input file is and which configuration governs it.
//! Why:
//!  Configuration and setup errors must stop the run with status 2 before a single file is
//! read for linting or rewritten,
//!  and worker threads then need only immutable plans.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // planFile(store, path) -> { absolute, display, relative, language, config, root } | skipped
//! ```

/// Import configuration loading,
///  matching and typed rule selection.
use crate::{
    config_error::ConfigError,
    config_lookup::{ConfigurationSource, discover_configuration, load_configuration_at},
    config_match::{FileConfiguration, PreparedConfiguration, prepare_configuration},
    directive_options::DirectiveOptions,
    markdown_rule_settings::{MarkdownRuleSettings, markdown_rule_settings},
    run_paths::{Language, absolute_normal, display_name, language_of, relative_from},
    rust_rule_settings::{RustRuleSettings, rust_rule_settings},
};
/// Import an ordered map and shared immutable ownership for configurations used by many files.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

/// What:
///  The rules selected for a host file itself,
///  already typed for its language.
/// Why:
///  A host with no matching block still has its embedded virtual files checked,
///  so "no root
/// rules" is a normal plan,
///  not a skipped file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootRules = { kind: 'none' } | { kind: 'rust'; settings } | { kind: 'markdown'; settings };
/// ```
#[derive(Clone, Debug)]
pub enum RootRules {
    /// No configuration block selects the host file.
    None,
    /// Rules for a `.rs` host.
    Rust {
        /// Typed selection of the Rust rules.
        settings: RustRuleSettings,
    },
    /// Rules for a `.md` or `.mdx` host.
    Markdown {
        /// Typed selection of the Markdown rules.
        settings: MarkdownRuleSettings,
    },
}

/// What:
///  Everything a worker needs to lint one file,
///  with no further setup that can fail.
/// Why:
///  Plans are immutable and shareable across threads;
///  only file contents remain to be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FilePlan = { absolute: Path; display: string; relative: Path; language: Language; config: PreparedConfiguration; root: RootRules };
/// ```
#[derive(Clone)]
pub struct FilePlan {
    /// Absolute,
    ///  lexically normal location used for reads,
    ///  writes and repository discovery.
    pub absolute: PathBuf,
    /// Name reported in findings,
    ///  relative to the working directory when inside it.
    pub display: String,
    /// Logical path relative to the configuration's base,
    ///  used to match virtual files.
    pub relative: PathBuf,
    /// Language chosen by the file extension.
    pub language: Language,
    /// The single configuration governing this file and its virtual files.
    pub config: Arc<PreparedConfiguration>,
    /// Rules for the host file itself.
    pub root: RootRules,
    /// Whether and how loudly unused suppression directives are reported;
    ///  the same for every file of a run.
    pub directives: DirectiveOptions,
}

/// What:
///  Whether this plan needs the Cargo-backed semantic engine.
/// Why:
///  Semantic sessions are owned by one thread;
///  such plans are processed there,
///  after the workers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plan.needsSemanticEngine()
/// ```
impl FilePlan {
    /// True only for a Rust host whose own rules select `rust/require-explicit-types`.
    pub fn needs_semantic_engine(&self) -> bool {
        if let RootRules::Rust { settings } = &self.root {
            return settings.explicit_types.is_some();
        }
        return false;
    }
}

/// What:
///  The outcome of planning one input path.
/// Why:
///  A skipped file is distinguished by reason,
///  so the run can tell "nothing configured anywhere"
/// (a setup error) from "this file is deliberately ignored".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Planned = { kind: 'lint'; plan } | { kind: 'ignored' } | { kind: 'no-configuration' } | { kind: 'unsupported' };
/// ```
pub enum Planned {
    /// The file is linted under this plan.
    Lint {
        /// The complete plan;
        ///  `Box` keeps this enum small next to its field-less variants.
        plan: Box<FilePlan>,
    },
    /// An ignores-only block removes the file from linting entirely.
    Ignored,
    /// No configuration file exists in the file's directory or any ancestor.
    NoConfiguration,
    /// The extension is not `.rs`,
    ///  `.md` or `.mdx`.
    Unsupported,
}

/// What:
///  Per-run configuration loading with one parse per configuration file.
/// Why:
///  Thousands of files share a few configurations;
///  lookup is memoized by directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConfigStore { private byDirectory = new Map<Path, PreparedConfiguration | undefined>(); }
/// ```
pub struct ConfigStore {
    /// Working directory:
    ///  the base for `--config` patterns and for display names.
    cwd: PathBuf,
    /// The `--config` file,
    ///  used for every input when present.
    explicit: Option<Arc<PreparedConfiguration>>,
    /// Nearest configuration for each directory already looked up;
    ///  `None` records "none found".
    by_directory: BTreeMap<PathBuf, Option<Arc<PreparedConfiguration>>>,
    /// Unused-directive settings copied into every plan.
    directives: DirectiveOptions,
}

/// What:
///  Build the store and answer per-file configuration questions.
/// Why:
///  `--config` skips lookup and resolves patterns against the working directory,
///  so a
/// temporary file outside the repository still matches repository paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ConfigStore(cwd, explicitPath).plan(path)
/// ```
impl ConfigStore {
    /// What:
    ///  Load the explicit configuration now,
    ///  if one was given.
    /// Why:
    ///  A missing or malformed `--config` file is a setup error before any file is planned.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(cwd: Path, explicit?: Path)
    /// ```
    pub fn new(cwd: &Path, explicit: Option<&Path>) -> Result<ConfigStore, ConfigError> {
        let mut prepared: Option<Arc<PreparedConfiguration>> = None;
        if let Some(path) = explicit {
            // The file argument is unused when an explicit path is supplied.
            let source: Option<ConfigurationSource> =
                discover_configuration(Path::new("."), cwd, Some(path))?;
            let selected: ConfigurationSource =
                source.expect("explicit configuration is present or an error");
            prepared = Some(Arc::new(prepare_configuration(selected)?));
        }
        return Ok(ConfigStore {
            cwd: cwd.to_path_buf(),
            explicit: prepared,
            by_directory: BTreeMap::<PathBuf, Option<Arc<PreparedConfiguration>>>::new(),
            directives: DirectiveOptions::default(),
        });
    }

    /// What:
    ///  Set the unused-directive reporting that every later plan carries.
    /// Why:
    ///  The command line fixes it for the whole run,
    ///  and planning is the one place that builds file plans.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// store.withDirectives(options)
    /// ```
    pub fn with_directives(mut self, directives: DirectiveOptions) -> ConfigStore {
        self.directives = directives;
        return self;
    }

    /// What:
    ///  The nearest configuration for a directory,
    ///  walking toward the filesystem root.
    /// Why:
    ///  Each directory on the way is remembered,
    ///  so siblings and descendants reuse the answer.
    /// The nearest file is used alone;
    ///  configurations are never merged across directories.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private nearest(directory: Path): PreparedConfiguration | undefined
    /// ```
    fn nearest(
        &mut self,
        directory: &Path,
    ) -> Result<Option<Arc<PreparedConfiguration>>, ConfigError> {
        let mut visited: Vec<PathBuf> = Vec::<PathBuf>::new();
        let mut found: Option<Arc<PreparedConfiguration>> = None;
        for ancestor in directory.ancestors() {
            if let Some(known) = self.by_directory.get(ancestor) {
                found = known.clone();
                break;
            }
            visited.push(ancestor.to_path_buf());
            if let Some(source) = load_configuration_at(ancestor)? {
                found = Some(Arc::new(prepare_configuration(source)?));
                break;
            }
        }
        for ancestor in visited {
            self.by_directory.insert(ancestor, found.clone());
        }
        return Ok(found);
    }

    /// What:
    ///  The configuration governing one absolute file path,
    ///  explicit or nearest.
    /// Why:
    ///  Both `--print-config` and linting ask the same question through the same code.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// configurationFor(absolute: Path): PreparedConfiguration | undefined
    /// ```
    pub fn configuration_for(
        &mut self,
        absolute: &Path,
    ) -> Result<Option<Arc<PreparedConfiguration>>, ConfigError> {
        if let Some(explicit) = &self.explicit {
            return Ok(Some(Arc::clone(explicit)));
        }
        let Some(directory): Option<&Path> = absolute.parent() else {
            return Ok(None);
        };
        return self.nearest(directory);
    }

    /// What:
    ///  Plan one input path given by the user or the walker.
    /// Why:
    ///  Language,
    ///  names,
    ///  governing configuration and typed root rules are all fixed here,
    /// so every configuration error surfaces before linting starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(path: Path): Planned
    /// ```
    pub fn plan(&mut self, path: &Path) -> Result<Planned, ConfigError> {
        let Some(language): Option<Language> = language_of(path) else {
            return Ok(Planned::Unsupported);
        };
        let absolute: PathBuf = absolute_normal(path, &self.cwd);
        let Some(config): Option<Arc<PreparedConfiguration>> = self.configuration_for(&absolute)?
        else {
            return Ok(Planned::NoConfiguration);
        };
        let relative: PathBuf = relative_from(&config.base, &absolute);
        let root: RootRules = match config.resolve(&relative)? {
            FileConfiguration::Ignored => return Ok(Planned::Ignored),
            FileConfiguration::Unconfigured => RootRules::None,
            FileConfiguration::Configured { rules } => {
                if language == Language::Yaml {
                    // A workflow has no host rules; its steps are linted as virtual shell files.
                    RootRules::None
                } else if language == Language::Rust {
                    RootRules::Rust {
                        settings: rust_rule_settings(&rules)?,
                    }
                } else {
                    RootRules::Markdown {
                        settings: markdown_rule_settings(&rules)?,
                    }
                }
            }
        };
        return Ok(Planned::Lint {
            plan: Box::new(FilePlan {
                display: display_name(&absolute, &self.cwd),
                absolute,
                relative,
                language,
                config,
                root,
                directives: self.directives,
            }),
        });
    }
}

/// Lookup,
///  memoization and classification controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_plan_tests.rs"]
mod tests;
