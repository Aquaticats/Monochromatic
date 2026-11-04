//! What:
//!  Reading and validating `cli-git.config.jsonc` and the Git configuration one invocation is judged by.
//! Why:
//!  Settings reach every other subsystem through these modules,
//!  so they are easy to find in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // config/index.ts: one `export * from './file'` per module below.
//! ```

/// The one failure type for rejected configuration content or files.
pub mod config_error;

/// Repository-root discovery and bounded reading of `cli-git.config.jsonc`.
pub mod config_file;

/// Lazy configuration loading for known inspection versus mutation/ambiguous commands.
pub mod config_loading;

/// Whole-document JSONC validation into typed settings.
pub mod config_parse;

/// Validation of the `policies` section and per-policy options.
pub(crate) mod config_policies;

/// The `rulesFile` option:
///  a repository-relative name that stays inside the repository.
pub mod config_rules_file;

/// Typed settings and their defaults.
pub mod config_schema;

/// One `git config --list` read, parsed, and the lookups the commit path makes against it.
pub mod config_snapshot;

/// Key-naming typed readers over parsed JSONC values.
pub(crate) mod config_values;

/// Configuration of the repository one invocation selects.
pub mod invocation_config;
