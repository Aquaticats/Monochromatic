//! What:
//!  Lint or fix one source whose bytes the caller already holds,
//!  as if it lived at a given filename.
//! Why:
//!  The executable's standard-input mode and an embedding caller (`run_embedded`) must give one answer,
//!  so both call this function after obtaining the bytes and routing the findings their own way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lintSourceBytes(store, filename, bytes, fix, debug) -> { findings, notes, result }
//! ```

/// Import planning,
///  contained processing and the setup error.
use crate::{
    config_lookup::CONFIG_NAME,
    diagnostic::Diagnostic,
    run_file::SourceOutcome,
    run_finish::engine,
    run_lfs::LfsRepos,
    run_modes::SetupError,
    run_paths::language_of,
    run_plan::{ConfigStore, Planned},
    run_workers::contained_source,
    rust_file_engine::RustFileEngine,
};
/// Import native paths.
use std::path::Path;

/// What:
///  What linting one source produced,
///  before any routing to streams.
/// Why:
///  The executable renders it as source plus JSON Lines;
///  an embedding caller keeps the findings typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceRun = { findings: Diagnostic[]; notes: string[]; result: string };
/// ```
#[derive(Debug)]
pub(crate) struct SourceRun {
    /// Findings for the final source,
    ///  sorted by position.
    pub findings: Vec<Diagnostic>,
    /// Debug explanations for this source.
    pub notes: Vec<String>,
    /// The source after every accepted fix;
    ///  the input unchanged when nothing was fixed or the fix was refused.
    pub result: String,
}

/// What:
///  Refuse a working directory that is not absolute.
/// Why:
///  Configuration patterns and repository discovery resolve against it,
///  so a relative one would silently depend on the process's own directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requireAbsoluteDirectory(cwd: Path): void; // throws SetupError
/// ```
pub(crate) fn require_absolute_directory(cwd: &Path) -> Result<(), SetupError> {
    if !cwd.is_absolute() {
        return Err(SetupError {
            message: String::from("The working directory must be an absolute path."),
        });
    }
    return Ok(());
}

/// What:
///  Refuse a filename the linter has no language for.
/// Why:
///  Standard input has no extension to inspect by itself,
///  and a name the linter cannot classify would otherwise pass unlinted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requireSupportedName(filename: Path): void; // throws SetupError
/// ```
pub(crate) fn require_supported_name(filename: &Path) -> Result<(), SetupError> {
    if language_of(filename).is_none() {
        return Err(SetupError {
            message: format!(
                "Standard input filename {} must end in .rs, .md or .mdx.",
                filename.display()
            ),
        });
    }
    return Ok(());
}

/// What:
///  Decode the bytes as UTF-8 text,
///  keeping a leading byte-order mark as the character it is.
/// Why:
///  The refusal names the first invalid offset,
///  and a mark stays part of the source so positions count it as the linter always has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeSource(bytes: Uint8Array): string; // throws SetupError
/// ```
fn decode_source(bytes: Vec<u8>) -> Result<String, SetupError> {
    match String::from_utf8(bytes) {
        Ok(text) => return Ok(text),
        Err(error) => {
            return Err(SetupError {
                message: format!(
                    "Standard input is not valid UTF-8 (first invalid byte at offset {}).",
                    error.utf8_error().valid_up_to()
                ),
            });
        }
    }
}

/// What:
///  Lint one source as if it lived at `filename`,
///  fixing it when `fix` is set.
/// Why:
///  The caller owns the bytes it commits:
///  no file is read for content or written.
///  A panic inside a rule becomes a processing finding,
///  never an unwind into the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lintSourceBytes(store, filename, bytes, fix, debug): SourceRun; // throws SetupError
/// ```
pub(crate) fn lint_source_bytes(
    store: &mut ConfigStore,
    filename: &Path,
    bytes: Vec<u8>,
    fix: bool,
    debug: bool,
) -> Result<SourceRun, SetupError> {
    let source: String = decode_source(bytes)?;
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
            let mut semantic: RustFileEngine = engine(debug);
            let outcome: SourceOutcome =
                contained_source(&plan, source.as_str(), fix, &lfs, Some(&mut semantic));
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
    return Ok(SourceRun {
        findings,
        notes,
        result,
    });
}
