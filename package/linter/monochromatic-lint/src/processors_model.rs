//! What:
//!  Immutable processor snapshots and their internal mapping records.
//! Why:
//!  Callers never reconstruct comment prefixes or synthetic positions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Mapping = { text: string; parent?: Mapping; lines: MappedLine[] };
//! ```

/// Import the host diagnostic and owned,
///  shared snapshot handle.
use crate::diagnostic::{Diagnostic, Severity};
/// What:
///  Arc shares immutable owned records,
///  unlike Rc (single-thread only) or Box (one owner).
/// Why:
///  Sibling virtual files reuse their host snapshot without copying host bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Immutable objects can share the same parent reference.
/// ```
use std::sync::Arc;

/// Language determines parsing and the original host's column convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessorLanguage {
    /// Native Rust,
    ///  including prepared doctests.
    Rust,
    /// Markdown,
    ///  including authored Rustdoc.
    Markdown,
    /// Markdown with MDX discovery enabled,
    ///  never executed.
    Mdx,
    /// A GitHub Actions workflow or composite action,
    ///  read only for the `run:` scripts of its steps.
    Yaml,
    /// One step's script,
    ///  extracted from a workflow.
    Shell,
}

/// What:
///  A typed refusal to extract,
///  map or project unsupported source.
/// Why:
///  Inability to check is not an ordinary rule violation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProcessorError extends Error { offset: number }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessorError {
    /// Affected original-host filename.
    pub filename: String,
    /// Original-host byte position.
    pub offset: usize,
    /// Explanation of the refused operation.
    pub message: String,
}

/// Render the affected host and operation for application error handling.
impl std::fmt::Display for ProcessorError {
    /// Borrow the formatter only for this write.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            formatter,
            "{} at byte {}: {}",
            self.filename, self.offset, self.message
        );
    }
}

/// Allow orchestration to propagate typed processor failures.
impl std::error::Error for ProcessorError {}

/// One copied physical line;
///  synthetic text deliberately has no record.
#[derive(Clone, Debug)]
pub(crate) struct MappedLine {
    /// Virtual half-open byte range,
    ///  including its original newline.
    pub start: usize,
    /// Virtual exclusive byte end.
    pub end: usize,
    /// Parent byte address of the first copied byte.
    pub parent_start: usize,
    /// Parent start including removable container prefix.
    pub envelope: usize,
    /// Container spelling repeated after inserted newlines.
    pub prefix: String,
}

/// Native re-extraction operation that proves a projected rewrite kept its container.
#[derive(Clone, Debug)]
pub(crate) enum Guard {
    /// Physical host,
    ///  with no extraction.
    Root,
    /// Fence marker's original byte address.
    Fence {
        /// Byte address of the fence marker in the physical host.
        marker: usize,
    },
    /// Physical line start of an authored comment run,
    ///  or block marker address.
    Docs {
        /// Byte address of the first comment line or block marker of the run.
        anchor: usize,
    },
    /// Hidden-line stripping and synthetic-main preparation.
    Prepared,
    /// A workflow step's script;
    ///  steps carry no fixes, so nothing is re-extracted.
    Step,
}

/// Exact child snapshot with its immutable direct parent and verification operation.
#[derive(Clone, Debug)]
pub(crate) struct Mapping {
    /// Logical configuration path,
    ///  not the display host filename.
    pub filename: String,
    /// Exact copied/prepared virtual bytes.
    pub text: String,
    /// Original source language used for host columns.
    pub language: ProcessorLanguage,
    /// Authored direct-parent positions.
    pub lines: Vec<MappedLine>,
    /// Absence marks the original physical host.
    pub parent: Option<Arc<Mapping>>,
    /// Extraction identity used after projecting a grouped fix.
    pub guard: Guard,
    /// Position for empty snippets or extraction failures.
    pub anchor: usize,
}

/// Shared original-host operations for every immutable map layer.
impl Mapping {
    /// Walk the bounded parent chain to the original input.
    pub(crate) fn root(&self) -> &Mapping {
        // What: Borrow a snapshot read-only; &Mapping is a view, not an owned copy.
        // Why: Diagnostics need the same immutable original host for every nested input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let current = this; while (current.parent) current = current.parent;
        // ```
        let mut current: &Mapping = self;
        // Some extracts the present optional parent; None ends the walk.
        while let Some(parent) = &current.parent {
            current = parent;
        }
        return current;
    }

    /// Carry an unsupported operation to its original host anchor.
    pub(crate) fn error(&self, message: &str) -> ProcessorError {
        // Delegate original-host addressing to the shared diagnostic mapper.
        let offset: usize = crate::processors_spans::anchor(self);
        return ProcessorError {
            // Copy the host path because the error outlives this borrowed snapshot.
            filename: self.root().filename.clone(),
            offset,
            message: String::from(message),
        };
    }

    /// Carry a native extraction error at its actual direct-source position to the original host.
    pub(crate) fn error_at(&self, offset: usize, message: &str) -> ProcessorError {
        let mut error: ProcessorError = self.error(message);
        if let Some((mapped, _)) = crate::processors_spans::host_range(self, offset, offset) {
            error.offset = mapped;
        }
        return error;
    }

    /// Produce the explicit processing-failure record expected by the fixing/output engines.
    pub(crate) fn failure(&self, message: &str) -> Diagnostic {
        // Map the processor's authored anchor, never synthetic main text.
        let error: ProcessorError = self.error(message);
        let span = crate::processors_spans::host_span(self.root(), error.offset, 0);
        let mut finding: Diagnostic = Diagnostic::new(
            "core/processing-failure",
            Severity::Error,
            error.message,
            error.filename,
            span,
        );
        finding.processing_failure = true;
        return finding;
    }
}
