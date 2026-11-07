//! What:
//!  The in-process entry for a caller that would otherwise start
//!  `monochromatic-lint --config=<file> --stdin --stdin-filename=<name> --fix-content`:
//!  one function that takes the configuration text,
//!  the filename and the source bytes,
//!  and returns the fixed source and the findings the fixes left.
//! Why:
//!  A caller that links the library avoids a program start and a temporary configuration file per run,
//!  and cannot be misled by which executable a search path finds.
//!  The function is the executable's standard-input fix mode,
//!  not a copy of it:
//!  it parses the configuration with the same parser as `--config`,
//!  decodes and lints the source with the same `run_source` core as `--stdin`,
//!  and classifies the run with the same `exit_status` as the executable,
//!  so a run the executable would end with status 0 returns `Ok` here,
//!  and every other status returns `Err` carrying that status.
//!  Nothing here starts a program or changes the process's working directory;
//!  the `top_level` argument is the directory the executable would be started in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = fixSource({ topLevel, configuration, name, source }); // { fixed, findings } or throws { exitStatus, explanation }
//! ```

/// Import the configuration parser,
///  the finding model and the shared source core.
use crate::{
    config_error::ConfigError,
    config_lookup::{ConfigurationSource, configuration_from_text},
    diagnostic::{Diagnostic, Severity},
    edits::FixScope,
    run_modes::SetupError,
    run_output::exit_status,
    run_plan::ConfigStore,
    run_source::{
        SourceRun, lint_source_bytes, require_absolute_directory, require_supported_name,
    },
};
/// Import native paths.
use std::path::{Path, PathBuf};

/// The label that names the in-memory configuration in its own parse errors,
///  where the executable names the file.
pub const EMBEDDED_CONFIGURATION_LABEL: &str = "(embedded configuration)";

/// What:
///  Everything one in-process run needs.
///  `'a` names how long the borrowed parts last.
/// Why:
///  The fields are the executable's working directory,
///  its `--config` document,
///  its `--stdin-filename` and its standard input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EmbeddedRequest = { topLevel: string; configuration: string; name: string; source: Uint8Array };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct EmbeddedRequest<'a> {
    /// The absolute directory the executable would run from:
    ///  configuration patterns,
    ///  `.lfsconfig` and linked images resolve against it.
    pub top_level: &'a Path,
    /// The JSONC configuration document,
    ///  used for the source as `--config` would be.
    pub configuration: &'a str,
    /// The name the source is linted as,
    ///  relative to `top_level` or absolute;
    ///  its extension selects the language.
    pub name: &'a Path,
    /// The source bytes the executable would read from standard input.
    pub source: &'a [u8],
}

/// What:
///  A run the executable would have ended with status 0:
///  the source after every accepted fix and the warnings the fixes left.
/// Why:
///  The executable prints the same two things as source and JSON Lines;
///  here they stay typed,
///  so the offsets and positions are the ones it would print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EmbeddedRun = { fixed: string; findings: Diagnostic[] };
/// ```
#[derive(Debug)]
pub struct EmbeddedRun {
    /// The source after every accepted fix;
    ///  equal to the input exactly when nothing changed.
    pub fixed: String,
    /// The findings the fixes did not remove,
    ///  in the executable's order.
    pub findings: Vec<Diagnostic>,
}

/// What:
///  A run the executable would have ended with a non-zero status.
/// Why:
///  The status says which kind it was:
///  1 for an error finding or a warning limit,
///  2 for a setup failure or a processing failure such as a rule panic;
///  the explanation is the linter's own words,
///  without the program prefix and without the file name,
///  which the caller already knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EmbeddedFailure = { exitStatus: number; explanation: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmbeddedFailure {
    /// The status the executable would exit with.
    pub exit_status: u8,
    /// Why the run cannot be used.
    pub explanation: String,
}

/// What:
///  The explanation of a run whose findings include an error or a processing failure.
/// Why:
///  The executable prints those findings as records;
/// the person needs their messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explanationOf(findings: Diagnostic[]): string;
/// ```
fn explanation_of(findings: &[Diagnostic]) -> String {
    let mut messages: Vec<&str> = Vec::new();
    for finding in findings {
        if finding.severity == Severity::Error || finding.processing_failure {
            messages.push(finding.message.as_str());
        }
    }
    if messages.is_empty() {
        return String::from("it printed no explanation");
    }
    return messages.join("; ");
}

/// What:
///  A setup failure as the failure the executable's status 2 reports.
/// Why:
///  The executable prints the setup message on standard error and exits with 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setupFailure = (error: SetupError) => ({ exitStatus: 2, explanation: error.message });
/// ```
fn setup_failure(error: SetupError) -> EmbeddedFailure {
    return EmbeddedFailure {
        exit_status: 2,
        explanation: error.message,
    };
}

/// What:
///  A configuration error as the failure the executable's status 2 reports.
/// Why:
///  The executable prints the error's message on standard error and exits with 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configFailure = (error: ConfigError) => setupFailure(SetupError.from(error));
/// ```
fn config_failure(error: ConfigError) -> EmbeddedFailure {
    return setup_failure(SetupError::from_display(&error));
}

/// What:
///  Fix `request.source` as the executable's `--stdin --fix` mode would,
///  under `request.configuration`,
///  from `request.top_level`.
///  `Result<EmbeddedRun, EmbeddedFailure>` is a status 0 run,
///  or the status and reason of any other.
/// Why:
///  This is the whole contract:
///  same configuration parsing,
///  same fix loop,
///  same rule behavior including byte-order marks and astral characters,
///  same classification of the run.
///  A panic inside a rule is contained by the same code the executable uses and returns as status 2;
///  a hang is not contained here,
///  and a caller that must bound it runs this on a thread of its own with a deadline,
///  on a stack of at least `run_workers::WORKER_STACK_BYTES`,
///  the size the executable gives every thread that lints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixSource(request: EmbeddedRequest): EmbeddedRun; // throws EmbeddedFailure
/// ```
pub fn fix_source(request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    require_absolute_directory(request.top_level).map_err(setup_failure)?;
    let described: ConfigurationSource = configuration_from_text(
        PathBuf::from(EMBEDDED_CONFIGURATION_LABEL),
        request.top_level.to_path_buf(),
        request.configuration,
    )
    .map_err(config_failure)?;
    // The embedded caller names the rules it wants in its own configuration,
    //  so every class of fix those rules attach applies:
    //  the commit path's `markdown/autofix` exists to rewrite LFS image URLs, which is a content fix.
    let mut store: ConfigStore = ConfigStore::from_source(request.top_level, described)
        .map_err(config_failure)?
        .with_fix_scope(FixScope::WhitespaceAndContent);
    require_supported_name(request.name).map_err(setup_failure)?;
    let run: SourceRun = lint_source_bytes(
        &mut store,
        request.name,
        request.source.to_vec(),
        true,
        false,
    )
    .map_err(setup_failure)?;
    let status: u8 = exit_status(run.findings.as_slice(), None);
    if status != 0 {
        return Err(EmbeddedFailure {
            exit_status: status,
            explanation: explanation_of(run.findings.as_slice()),
        });
    }
    return Ok(EmbeddedRun {
        fixed: run.result,
        findings: run.findings,
    });
}

/// In-process contract controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_embedded_tests.rs"]
mod tests;
