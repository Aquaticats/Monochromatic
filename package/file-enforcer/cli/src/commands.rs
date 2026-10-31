//! What:
//!  Turn a parsed command line into a run request and execute it.
//! Why:
//!  This is the single seam between clap and the engine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function dispatch(cli: Cli, sink: EventSink): Promise<number>
//! ```

use crate::args::{Cli, Command};
use crate::discover::resolve_repository;
use monochromatic_file_enforcer::config_decode::ConfigScope;
use monochromatic_file_enforcer::event::EventSink;
use monochromatic_file_enforcer::registry::RuleRegistry;
use monochromatic_file_enforcer::run::{EXIT_USAGE, RunMode, RunRequest, run};

/// What:
///  Run the chosen command and return its exit code.
/// Why:
///  Config discovery failures are usage errors (exit 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dispatch(cli: Cli, sink: EventSink): Promise<number>
/// ```
pub fn dispatch(cli: &Cli, sink: &mut dyn EventSink) -> i32 {
    let mode = match &cli.command {
        Command::Write => RunMode::Write,
        Command::Check { expected_dir } => RunMode::Check { expected_dir: expected_dir.clone() },
        Command::Explain { path } => RunMode::Explain { path: path.clone() },
        _ => {
            sink.emit(monochromatic_file_enforcer::diagnostic::Diagnostic::new("not-implemented", "this command is not built yet").to_event("error"));
            return EXIT_USAGE;
        }
    };
    let (root, config_path) = match resolve_repository(&cli.config) {
        Ok(pair) => pair,
        Err(error) => {
            sink.emit(error.to_event("error"));
            return EXIT_USAGE;
        }
    };
    let request = RunRequest { root, config_path, scope: ConfigScope::Repository, mode };
    return run(&request, &RuleRegistry::with_defaults(), sink);
}
