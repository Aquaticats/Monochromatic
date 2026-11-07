//! What:
//!  Lint or fix one source read from standard input.
//! Why:
//!  A commit pipeline pipes the bytes it is about to commit and owns what happens to them;
//! this mode reads no file for content and writes none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runStdin(options, store, stdin) -> RunOutput
//! ```

/// Import the command grammar and the shared source core.
use crate::{
    cli_options::CliOptions,
    run_finish::finish,
    run_modes::SetupError,
    run_output::RunOutput,
    run_plan::ConfigStore,
    run_source::{SourceRun, lint_source_bytes, require_supported_name},
};
/// Import the reader trait for standard input and borrowed native paths.
use std::{io::Read, path::Path};

/// What:
///  Lint one source read from standard input as if it lived at `--stdin-filename`.
/// Why:
///  The caller owns the bytes it commits:
///  no file is read for content or written.
///  With
/// `--fix` the (possibly unchanged) source goes to standard output and findings to standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runStdin(options, store, stdin): RunOutput;
/// ```
pub(crate) fn run_stdin(
    options: &CliOptions,
    store: &mut ConfigStore,
    stdin: &mut dyn Read,
) -> Result<RunOutput, SetupError> {
    let filename: &Path = options
        .stdin_filename
        .as_deref()
        .expect("the command grammar requires --stdin-filename with --stdin");
    require_supported_name(filename)?;
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    if let Err(error) = stdin.read_to_end(&mut bytes) {
        return Err(SetupError {
            message: format!("Cannot read standard input: {error}."),
        });
    }
    let run: SourceRun = lint_source_bytes(store, filename, bytes, options.fix, options.debug)?;
    if options.fix {
        return finish(
            options,
            run.findings.as_slice(),
            Some(run.result.as_str()),
            run.notes.as_slice(),
        );
    }
    return finish(options, run.findings.as_slice(), None, run.notes.as_slice());
}
