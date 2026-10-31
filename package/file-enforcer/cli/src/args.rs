//! What:
//!  The command-line grammar,
//! declared with clap's derive macros.
//! Why:
//!  clap owns parsing,
//! help,
//! version and error text;
//! `output.rs` turns its help,
//! version and errors into JSON lines
//!  so standard output stays one JSON object per line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! program.command('write').option('--outputs <scope>'); program.command('check').option('--outputs <scope>').option('--expected-dir <dir>');
//! ```

#![cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(
        avoidable_closure,
        reason = "clap's derive generates the missing-argument closures in its FromArgMatches implementation with the field type as their span, so no author-written closure exists to refactor"
    )
)]

/// Imports `Parser`,
/// `Subcommand`,
/// `ValueEnum` from `clap`.
use clap::{Parser, Subcommand, ValueEnum};
/// Imports `OutputScope` from `monochromatic_file_enforcer::output_scope`.
use monochromatic_file_enforcer::output_scope::OutputScope;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// What:
///  The whole command line:
/// global options and one subcommand.
///  `Option<PathBuf>` is "a path or nothing" (TS:
/// `string | undefined`).
/// Why:
///  `--config` picks the repository config;
/// without it the nearest `file-enforcer.config.jsonc` above the current
///  directory is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cli = { config?: string; userConfig?: string; command: Command };
/// ```
#[derive(Debug, Parser)]
#[command(name = "file-enforcer", version, about = "Generate and check the files this repository derives from other files.")]
pub struct Cli {
    /// Path of the repository config;
    /// the worktree root is its directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Path of the per-user config used by the `user` commands.
    #[arg(long, global = true, value_name = "PATH")]
    pub user_config: Option<PathBuf>,

    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// What:
///  The words `--outputs` accepts.
///  A `ValueEnum` is a closed set of words clap parses and lists in its help (TS:
/// a string literal union).
/// Why:
///  A fresh checkout holds every committed output but none of the gitignored ones,
/// so CI checks `committed` first and `local` after generating;
/// one single-valued option with a default avoids repeated or delimited forms,
/// and the unscoped default stays `all`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OutputsArg = 'all' | 'committed' | 'local';
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputsArg {
    /// Every output (the default).
    #[default]
    All,
    /// Outputs that no `.gitignore` rule matches.
    Committed,
    /// Outputs that a `.gitignore` rule matches.
    Local,
}

/// What:
///  The engine's scope for a command-line word.
/// Why:
///  The command line owns the words and the engine owns the behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outputScopeOf(arg: OutputsArg): OutputScope
/// ```
impl OutputsArg {
    /// What:
    ///  Convert to the engine's `OutputScope`.
    /// Why:
    ///  The request sent to the server carries the engine's word.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scope(): OutputScope
    /// ```
    pub fn scope(self) -> OutputScope {
        match self {
            OutputsArg::All => return OutputScope::All,
            OutputsArg::Committed => return OutputScope::Committed,
            OutputsArg::Local => return OutputScope::Local,
        }
    }
}

/// What:
///  The subcommands.
///  An enum with data is a tagged union (TS:
/// a discriminated union).
/// Why:
///  Every command except the hidden `serve` is a client of the per-worktree server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Command = { kind: 'write'; outputs: OutputsArg } | { kind: 'check'; outputs: OutputsArg; expectedDir?: string } | { kind: 'watch' } | { kind: 'explain'; path: string } | { kind: 'user'; command: UserCommand } | { kind: 'serve'; ... };
/// ```
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Regenerate every drifted file.
    Write {
        /// Which outputs to regenerate:
        /// committed ones (no `.gitignore` rule matches),
        /// local ones (a rule matches),
        /// or all.
        #[arg(long, value_enum, default_value_t = OutputsArg::All, value_name = "SCOPE")]
        outputs: OutputsArg,
    },
    /// Report drift without writing;
    /// exit 1 when anything differs.
    Check {
        /// Which outputs to check:
        /// committed ones (no `.gitignore` rule matches),
        /// local ones (a rule matches),
        /// or all.
        /// Drift in the other outputs is not reported and does not change the exit code.
        #[arg(long, value_enum, default_value_t = OutputsArg::All, value_name = "SCOPE")]
        outputs: OutputsArg,

        /// Write paired actual/ and expected/ trees here,
        /// for `diff -ru` or `git diff --no-index`.
        #[arg(long, value_name = "DIR")]
        expected_dir: Option<PathBuf>,
    },
    /// Keep the generated files current and warn when one is edited by hand.
    Watch,
    /// Say which rules read or write a path.
    Explain {
        /// Path to explain,
        /// relative to the worktree root or absolute.
        path: String,
    },
    /// Rules that write outside the repository,
    /// from the per-user config.
    #[command(subcommand)]
    User(
        /// The wrapped value.
        UserCommand,
    ),
    /// Run the per-worktree server;
    /// started automatically by the other commands.
    #[command(hide = true)]
    Serve {
        /// Worktree root (or home directory for the per-user server).
        #[arg(long)]
        root: PathBuf,
        /// Unix socket path to listen on.
        #[arg(long)]
        socket: PathBuf,
        /// Seconds without a client before the server exits.
        #[arg(long, default_value_t = 120)]
        idle_seconds: u64,
        /// Serve the per-user config instead of a worktree.
        #[arg(long)]
        user: bool,
    },
}

/// What:
///  The `user` subcommands.
/// Why:
///  They mirror `write` and `check` for the per-user config.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UserCommand = 'write' | 'check';
/// ```
#[derive(Debug, Subcommand)]
pub enum UserCommand {
    /// Apply the per-user rules.
    Write,
    /// Report drift in what the per-user rules manage.
    Check,
}
