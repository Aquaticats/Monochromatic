//! What:
//!  The accepted native unified-linter command grammar.
//! Why:
//!  Flags,
//!  help and usage validation share the incumbent's CLI parser instead of divergent manual scanning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Declare typed command options; execution consumes native paths without lossy string conversion.
//! ```

#![cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(
        avoidable_closure,
        reason = "clap's derive generates the parser closures in its FromArgMatches implementation with the field type as their span, so no author-written closure exists to refactor"
    )
)]

/// Import the unused-directive settings the two reporting options resolve to.
use crate::directive_options::{DirectiveOptions, UnusedDirectiveSeverity};
/// Import the incumbent CLI parser's trait and declaration macro.
use clap::Parser;
/// Import the non-zero count type so a worker limit of zero cannot be expressed.
use std::num::NonZeroUsize;
/// Import native path storage and a strictly positive worker-count type.
use std::path::PathBuf;

/// What:
///  Parsed command options,
///  with absence retained for discovery and default selection.
/// Why:
///  Repository configuration remains data-only and cannot add executables,
///  plugins or a rule-selection CLI.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CliOptions = { paths: Path[]; config?: Path; fix: boolean; stdin: boolean; ... };
/// ```
#[derive(Parser, Debug, Eq, PartialEq)]
#[command(
    name = "monochromatic-lint",
    version,
    about = "Lint Rust, Markdown and MDX with repository-owned policies.",
    // Without this, the parser would print this struct's source documentation as the long help text.
    long_about = None
)]
pub struct CliOptions {
    /// Literal paths or path globs;
    ///  execution supplies '.'
    ///  only when this list is empty outside stdin mode.
    #[arg(value_name = "PATH", conflicts_with = "stdin")]
    pub paths: Vec<PathBuf>,
    /// Use exactly this JSONC configuration instead of searching ancestors.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// Apply compatible source fixes and recheck the resulting input.
    #[arg(long, conflicts_with_all = ["print_config", "init", "rules"])]
    pub fix: bool,
    /// Read one source from stdin instead of the filesystem.
    #[arg(long, requires = "stdin_filename", conflicts_with_all = ["print_config", "init", "rules"])]
    pub stdin: bool,
    /// Logical filename selecting language,
    ///  configuration and reported host identity for stdin.
    #[arg(long, value_name = "FILE", requires = "stdin")]
    pub stdin_filename: Option<PathBuf>,
    /// Maximum accepted warning findings;
    ///  absence does not impose a warning threshold.
    #[arg(long, value_name = "COUNT")]
    pub max_warnings: Option<usize>,
    /// Suppress warning diagnostics while retaining exit-status accounting.
    #[arg(long)]
    pub quiet: bool,
    /// Suppress diagnostics while retaining exit status and required fixed-source output.
    #[arg(long)]
    pub silent: bool,
    /// Print the effective data-only configuration for one file without linting it.
    #[arg(long, value_name = "FILE", conflicts_with_all = ["init", "rules", "paths"])]
    pub print_config: Option<PathBuf>,
    /// Create a starter configuration without overwriting an existing file.
    #[arg(long, conflicts_with_all = ["rules", "paths"])]
    pub init: bool,
    /// List the shipped rule identifiers and their documented capabilities.
    #[arg(long, conflicts_with = "paths")]
    pub rules: bool,
    /// Positive worker limit;
    ///  absence uses the operating system's available parallelism.
    #[arg(long, value_name = "COUNT")]
    pub concurrency: Option<NonZeroUsize>,
    /// Additional traversal exclusion glob,
    ///  repeatable without a delimiter mini-language.
    #[arg(long = "ignore-pattern", value_name = "GLOB")]
    pub ignore_patterns: Vec<String>,
    /// Additional ignore file,
    ///  repeatable and retained as a native path.
    #[arg(long = "ignore-path", value_name = "FILE")]
    pub ignore_paths: Vec<PathBuf>,
    /// Disable traversal ignore sources;
    ///  explicit configuration matching still determines selected rules.
    #[arg(long)]
    pub no_ignore: bool,
    /// Permit a command-line path pattern with no matches.
    #[arg(long)]
    pub no_error_on_unmatched_pattern: bool,
    /// Emit execution debug information separately from JSONL findings.
    #[arg(long)]
    pub debug: bool,
    /// Report disable directives that suppressed nothing,
    ///  as warnings.
    #[arg(long, conflicts_with = "report_unused_disable_directives_severity")]
    pub report_unused_disable_directives: bool,
    /// Report disable directives that suppressed nothing at this severity;
    ///  `allow` and `off` report nothing.
    #[arg(long, value_name = "SEVERITY", value_enum)]
    pub report_unused_disable_directives_severity: Option<UnusedDirectiveSeverity>,
}

/// What:
///  Resolve the unused-directive options of one invocation.
/// Why:
///  Planning copies the result into every file plan,
///  so rule code never reads the command line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// options.directiveOptions()
/// ```
impl CliOptions {
    /// Combine the plain flag and the severity option.
    pub fn directive_options(&self) -> DirectiveOptions {
        return DirectiveOptions::from_flags(
            self.report_unused_disable_directives,
            self.report_unused_disable_directives_severity,
        );
    }
}

/// Argument grammar controls do not ship in release code.
#[cfg(test)]
#[path = "cli_options_tests.rs"]
mod tests;
