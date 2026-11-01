//! What:
//!  Findings and explicit coverage failures for the Rust type-annotation rule.
//! Why:
//!  An unresolved call is not proof that no generic arguments are needed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Keep style findings separate from failures to establish the semantic facts.
//! ```

/// Import the shared finding,
///  severity and byte-span model.
use crate::diagnostic::{Diagnostic, Severity, Span};
/// Import the exact input bytes and their indexed positions.
use crate::rust_source::RustSource;
/// Import the syntax node and byte-range types supplied by the shared parser.
use ra_ap_syntax::{SyntaxNode, TextRange};

/// Build a report-only type-policy finding from authored syntax.
pub(crate) fn type_finding(
    context: &RustSource,
    node: &SyntaxNode,
    severity: Severity,
    message: &str,
    help: &str,
) -> Diagnostic {
    // TextRange stores bytes; usize matches source indexes rather than fixed-width integer siblings.
    let range: TextRange = node.text_range();
    let offset: usize = usize::from(range.start());
    let length: usize = usize::from(range.len());
    let span: Span = context.span(offset, length);
    // Own strings because findings can outlive the parser and its input context.
    let mut diagnostic: Diagnostic = Diagnostic::new(
        "rust/require-explicit-types",
        severity,
        String::from(message),
        context.filename.clone(),
        span,
    );
    // Some marks present guidance rather than an absent optional field.
    diagnostic.help = Some(String::from(help));
    return diagnostic;
}

/// What:
///  Record a failure to obtain type information,
///  not a successful or exempt check.
/// Why:
///  The driver must fail processing even when ordinary type-policy findings were configured as warnings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolutionFailure(context, node, message): Diagnostic;
/// ```
pub(crate) fn resolution_failure(
    context: &RustSource,
    node: &SyntaxNode,
    message: &str,
) -> Diagnostic {
    let mut diagnostic: Diagnostic = type_finding(
        context,
        node,
        Severity::Error,
        message,
        "This input was not verified. Check that its Cargo target, dependencies, features and generated sources are available and that the code resolves. For an intentional standalone snippet, configure this semantic rule off. If the project type-checks, report the resolver gap rather than treating this result as a pass.",
    );
    diagnostic.code = String::from("core/rust-type-resolution");
    diagnostic.processing_failure = true;
    return diagnostic;
}
