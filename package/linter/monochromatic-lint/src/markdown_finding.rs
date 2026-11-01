//! What:
//!  Shared construction for node-anchored Markdown findings.
//! Why:
//!  Rule ports share the same byte coordinates,
//!  optional single-edit fix,
//!  and wire defaults.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // diagnose({ node, ruleId, message, fix })
//! ```

/// Import the common finding and edit models and the native parse interface.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit and fix types that every finding constructor attaches to a diagnostic.
use crate::edits::{Edit, Fix};
/// Import the parse error and source model that structure failures are built from.
use crate::markdown_source::{MarkdownError, MarkdownSource};
/// What:
///  Import the shared constructor of `core/processing-failure` findings.
/// Why:
///  A rule that cannot finish must report incomplete processing the same way the engine does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { processingFailure } from './run-failure';
/// ```
use crate::run_failure::processing_failure;

/// What:
///  Build an owned diagnostic from a rule-visible node.
/// Why:
///  The parser arena can be released after all findings and edits have been collected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finding(context, id, rule, severity, message, edit): Diagnostic;
/// ```
pub(crate) fn finding(
    context: &MarkdownSource,
    id: u32,
    rule: &str,
    severity: Severity,
    message: String,
    edit: Option<Edit>,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(
        rule,
        severity,
        message,
        context.filename.clone(),
        context.node_span(id),
    );
    if let Some(source_edit) = edit {
        diagnostic.fix = Some(Fix {
            edits: vec![source_edit],
        });
    }
    return diagnostic;
}

/// What:
///  Turn a failed ancestor or descendant walk into a processing-failure finding that names the rule which stopped.
/// Why:
///  A rule that cannot read the document's structure has not checked the file.
///  A processing failure
/// makes the run exit with status 2 and stops the fixer,
///  where an empty or partial list would look clean.
/// The zero-width position is the node whose walk failed,
///  in this document's own coordinates;
/// for a virtual document the processor maps it to the host like any other finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function structureFailure(context: MarkdownSource, rule: string, error: MarkdownError): Diagnostic;
/// ```
pub(crate) fn structure_failure(
    context: &MarkdownSource,
    rule: &str,
    error: MarkdownError,
) -> Diagnostic {
    return processing_failure(
        context.filename.as_str(),
        context.span(error.offset, 0),
        format!("{rule} could not check this file: {}", error.message),
    );
}
