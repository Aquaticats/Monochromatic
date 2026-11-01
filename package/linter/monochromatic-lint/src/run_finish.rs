//! What:
//!  The shared end of every lint mode:
//!  output controls,
//!  debug notes and the semantic engine.
//! Why:
//!  File mode and standard-input mode differ in where source comes from,
//!  not in how findings
//! are routed,
//!  how debug notes are printed,
//!  or how the Cargo-backed engine is created.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // finish(options, findings, fixedStdin, notes) -> RunOutput; engine(debug) -> RustFileEngine
//! ```

/// Import the command grammar,
///  the finding model,
///  output routing and the semantic engine.
use crate::{
    cli_options::CliOptions,
    diagnostic::Diagnostic,
    run_modes::SetupError,
    run_output::{OutputOptions, RunOutput, run_output},
    rust_file_engine::RustFileEngine,
    rust_workspace::WorkspacePreparation,
};
/// Import the write trait that provides `writeln!` on standard error.
use std::io::Write;

/// What:
///  The prefix of every non-finding line this program writes to standard error.
/// Why:
///  A consumer can separate program messages from JSONL records by this prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PROGRAM = 'monochromatic-lint';
/// ```
pub const PROGRAM: &str = "monochromatic-lint";

/// What:
///  The version of the build that is running,
///  exactly as the package manifest records it.
/// Why:
///  Startup diagnostics append it so the running binary identifies itself in the failure,
///  which a stale resolved binary would otherwise hide;
///  `--version` prints the same value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VERSION = process.env.npm_package_version;
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What:
///  The running binary's name and version as one quotable identity.
/// Why:
///  A diagnostic suffix naming both stands alone when a reader quotes it out of context.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function binaryIdentity(): string { return `${PROGRAM} ${VERSION}`; }
/// ```
pub fn binary_identity() -> String {
    return format!("{PROGRAM} {VERSION}");
}

/// What:
///  Discard semantic-workspace progress messages.
/// Why:
///  The engine takes a plain function pointer;
///  without `--debug` progress is not shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function silentProgress(_message: string): void {}
/// ```
fn silent_progress(_message: String) {}

/// What:
///  Write semantic-workspace progress to standard error as it happens.
/// Why:
///  Loading a Cargo workspace can take a while;
///  `--debug` shows that it is working.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function debugProgress(message: string): void { console.error(`monochromatic-lint: debug: ${message}`); }
/// ```
fn debug_progress(message: String) {
    // A closed standard error has nowhere left to report its own failure, so the result is unused.
    let _unreportable: std::io::Result<()> =
        writeln!(std::io::stderr(), "{PROGRAM}: debug: {message}");
}

/// What:
///  Render collected debug notes as prefixed standard-error lines.
/// Why:
///  Debug information stays off standard output,
///  which carries only JSONL or fixed source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function debugLines(notes: string[]): string;
/// ```
fn debug_lines(notes: &[String]) -> String {
    let mut output: String = String::new();
    for note in notes {
        output.push_str(format!("{PROGRAM}: debug: {note}\n").as_str());
    }
    return output;
}

/// What:
///  Collect the output controls from the command options.
/// Why:
///  Quiet,
///  silent and the warning limit affect display and exit status only,
///  never linting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outputOptions(options: CliOptions): OutputOptions;
/// ```
fn output_options(options: &CliOptions) -> OutputOptions {
    return OutputOptions {
        quiet: options.quiet,
        silent: options.silent,
        max_warnings: options.max_warnings,
    };
}

/// What:
///  Route findings and add debug notes to standard error when requested.
/// Why:
///  Every lint mode ends the same way;
///  only the presence of fixed standard-input source differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(options, findings, fixedStdin, notes): RunOutput;
/// ```
pub(crate) fn finish(
    options: &CliOptions,
    findings: &[Diagnostic],
    fixed_stdin: Option<&str>,
    notes: &[String],
) -> Result<RunOutput, SetupError> {
    let mut output: RunOutput = match run_output(findings, fixed_stdin, output_options(options)) {
        Ok(routed) => routed,
        Err(error) => return Err(SetupError::from_display(&error)),
    };
    if options.debug {
        output.stderr.push_str(debug_lines(notes).as_str());
    }
    return Ok(output);
}

/// What:
///  Create the invocation's semantic engine without opening any workspace.
/// Why:
///  A workspace is loaded only when a selected rule needs it.
///  Source-only preparation never
/// runs build scripts;
///  no command-line option requests generated-source preparation yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function engine(debug: boolean): RustFileEngine;
/// ```
pub(crate) fn engine(debug: bool) -> RustFileEngine {
    if debug {
        return RustFileEngine::new(WorkspacePreparation::SourceOnly, debug_progress);
    }
    return RustFileEngine::new(WorkspacePreparation::SourceOnly, silent_progress);
}
