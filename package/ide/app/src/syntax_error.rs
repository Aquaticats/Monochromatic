//! Translate parser failures into source-reading diagnostics without leaking internal terms.

/// The upstream enum implements Display but not std::error::Error.
use helix_core::syntax::HighlighterError;
/// Paths identify the affected input in the user-facing diagnostic.
use std::path::Path;

/// Match each parser failure to its actual recovery paths,
///  retaining raw evidence in logs.
pub(crate) fn parser_failure(
    path: &Path,
    language: &str,
    error: HighlighterError,
) -> anyhow::Error {
    tracing::warn!(path = %path.display(), language, %error, "source syntax parse failed");
    // What: match handles every variant of the upstream failure enum.
    // Why: A timeout must not prescribe the missing-asset remedy, and new variants require review.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (error.kind) { case 'timeout': return timeoutDiagnostic; /* ... */ }
    // ```
    let explanation = match error {
        HighlighterError::Timeout => {
            "Parsing exceeded its deadline. Source remains readable without coloring; reopen the file or restart the application to retry."
        }
        HighlighterError::ExceededMaximumSize => {
            "The file exceeds the parser's size limit. Source remains readable without coloring; use a smaller source file for highlighting."
        }
        HighlighterError::NoRootConfig => {
            "The bundled parser or highlighting rules could not be loaded. Replace the executable with a fresh copy of the application, or build it again from source, then restart the application."
        }
        HighlighterError::IncompatibleGrammar(_, _) => {
            "The bundled parser data is incompatible with this application. Replace the executable with a fresh copy of the application, or build it again from source, then restart the application."
        }
        HighlighterError::InvalidRanges => {
            "The parser rejected source ranges. Source remains readable without coloring; reopen the file or restart to retry, and report a reproducible input if it persists."
        }
        HighlighterError::Unknown => {
            "The parser failed without a specific cause. Source remains readable without coloring; reopen the file or restart to retry, and report a reproducible input if it persists."
        }
    };
    return anyhow::anyhow!(
        "Cannot highlight {} as {language}. {explanation}",
        path.display()
    );
}
