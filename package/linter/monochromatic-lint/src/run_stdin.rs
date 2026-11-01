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

/// Import the command grammar,
///  planning,
///  contained processing and the shared finish step.
use crate::{
    cli_options::CliOptions,
    config_lookup::CONFIG_NAME,
    diagnostic::Diagnostic,
    run_file::SourceOutcome,
    run_finish::{engine, finish},
    run_lfs::LfsRepos,
    run_modes::SetupError,
    run_output::RunOutput,
    run_paths::language_of,
    run_plan::{ConfigStore, Planned},
    run_workers::contained_source,
    rust_file_engine::RustFileEngine,
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
    if language_of(filename).is_none() {
        return Err(SetupError {
            message: format!(
                "Standard input filename {} must end in .rs, .md or .mdx.",
                filename.display()
            ),
        });
    }
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    if let Err(error) = stdin.read_to_end(&mut bytes) {
        return Err(SetupError {
            message: format!("Cannot read standard input: {error}."),
        });
    }
    let source: String = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return Err(SetupError {
                message: format!(
                    "Standard input is not valid UTF-8 (first invalid byte at offset {}).",
                    error.utf8_error().valid_up_to()
                ),
            });
        }
    };
    let planned: Planned = match store.plan(filename) {
        Ok(value) => value,
        Err(error) => return Err(SetupError::from_display(&error)),
    };
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    let mut notes: Vec<String> = Vec::<String>::new();
    let mut result: String = source.clone();
    match planned {
        Planned::Lint { plan } => {
            let lfs: LfsRepos = LfsRepos::new();
            let mut semantic: RustFileEngine = engine(options.debug);
            let outcome: SourceOutcome = contained_source(
                &plan,
                source.as_str(),
                options.fix,
                &lfs,
                Some(&mut semantic),
            );
            findings = outcome.findings;
            notes = outcome.notes;
            if let Some(fixed) = outcome.fixed {
                result = fixed;
            }
        }
        Planned::Ignored => notes.push(format!("{}: ignored by configuration", filename.display())),
        Planned::NoConfiguration => {
            return Err(SetupError {
                message: format!(
                    "No {} governs standard input filename {}. Create one with --init or pass --config.",
                    CONFIG_NAME,
                    filename.display()
                ),
            });
        }
        Planned::Unsupported => {}
    }
    if options.fix {
        return finish(
            options,
            findings.as_slice(),
            Some(result.as_str()),
            notes.as_slice(),
        );
    }
    return finish(options, findings.as_slice(), None, notes.as_slice());
}
