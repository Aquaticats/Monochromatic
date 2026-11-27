//! What:
//!  The diagnostics a rule's findings become,
//!  the `--debug` note that names the allow-list key of each,
//!  and the failure a list that cannot be read turns into.
//! Why:
//!  A reader writes an allow-list entry by copying the key the rule reports,
//!  and never by guessing its shape,
//!  while a list that cannot be read would let every finding through unseen.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function diagnostics(found: LayoutFinding[], code: string, severity: Severity, cwd: Path, spans: LineSpans): Diagnostic[];
//! function listFor(path: Path | undefined, rule: string, cwd: Path, outcome: LayoutOutcome): AllowList | undefined;
//! function noteKeys(found: LayoutFinding[], rule: string, outcome: LayoutOutcome): void;
//! ```

/// Import the diagnostic shape and its severity,
///  the list reader,
///  the finding shape,
///  the line spans,
///  the words a processing failure is built from,
///  the run's paths,
///  and the outcome the notes and failures join.
use crate::{
    diagnostic::{
        Diagnostic,
        Severity,
    },
    layout_allow_list::{
        AllowList,
        read_allow_list,
    },
    layout_finding::LayoutFinding,
    layout_line_spans::LineSpans,
    layout_step::LayoutOutcome,
    run_failure::{
        file_start,
        processing_failure,
    },
    run_paths::display_name,
};
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Turn the findings of one rule into diagnostics.
/// Why:
///  The rule's code and severity come from the step;
///  the help text travels with the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function diagnostics(found: LayoutFinding[], code: string, severity: Severity, cwd: Path): Diagnostic[];
/// ```
pub(crate) fn diagnostics(
    found: Vec<LayoutFinding>,
    code: &str,
    severity: Severity,
    cwd: &Path,
    spans: &mut LineSpans,
) -> Vec<Diagnostic> {
    let mut result: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for item in found {
        let mut diagnostic: Diagnostic = Diagnostic::new(
            code,
            severity,
            item.message,
            display_name(&item.file, cwd),
            spans.span(&item.file, item.line),
        );
        diagnostic.help = item.help;
        result.push(diagnostic);
    }
    return result;
}

/// What:
///  Read one rule's allow list,
///  or report why the rule cannot run.
/// Why:
///  A list that cannot be read would let every finding through unseen,
///  so the package fails the run instead of being checked without it.
///  The package folder reads every listed path back into the spelling a key writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listFor(path: Path | undefined, base: Path, rule: string, cwd: Path, outcome: LayoutOutcome): AllowList | undefined;
/// ```
pub(crate) fn list_for(
    path: &Option<PathBuf>,
    base: &Path,
    rule: &str,
    cwd: &Path,
    outcome: &mut LayoutOutcome,
) -> Result<Option<AllowList>, ()> {
    let Some(named) = path else {
        return Ok(None);
    };
    match read_allow_list(named, base) {
        Ok(list) => return Ok(Some(list)),
        Err(reason) => {
            outcome.findings.push(processing_failure(
                display_name(named, cwd).as_str(),
                file_start(),
                format!(
                    "{rule} names an allow list that cannot be read, so the rule was not checked: {reason}"
                ),
            ));
            return Err(());
        }
    }
}

/// What:
///  Add one `--debug` note per finding that an allow-list entry could record.
/// Why:
///  A reader writes an entry by copying the key the rule reports,
///  and never by guessing its shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteKeys(found: LayoutFinding[], rule: string, outcome: LayoutOutcome): void;
/// ```
pub(crate) fn note_keys(
    found: &[LayoutFinding],
    rule: &str,
    outcome: &mut LayoutOutcome,
) {
    for item in found {
        if item.key.is_empty() {
            continue;
        }
        outcome
            .notes
            .push(format!("{rule}: allow-list key: {}", item.key));
    }
}
