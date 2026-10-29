//! What:
//!  The deep processor module at the native orchestration seam.
//! Why:
//!  Configuration callers see virtual paths,
//!  but never duplicate source maps or prefix encoding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // extract(host) -> virtuals; virtual.checkRust(settings); virtual.projectDiagnostic(finding).
//! ```

/// Import diagnostics,
///  grouped source edits and the immutable processor model.
use crate::diagnostic::Diagnostic;
/// Fix groups stay atomic across every extraction layer.
use crate::edits::Fix;
/// Hide implementation records behind VirtualSource's small interface.
use crate::processors_model::{Guard, Mapping};
/// Internal registration is deliberately not a stable public library contract.
pub use crate::processors_model::{ProcessorError, ProcessorLanguage};
/// Reuse typed selected Rust settings rather than another rule registry.
use crate::rust_rule_settings::RustRuleSettings;
/// Shared immutable owned snapshots use Arc instead of duplicating hosts per virtual file.
use std::sync::Arc;

/// Build a direct-parent map whose payload lines are filled by one native extractor.
pub(crate) fn child(
    parent: &Arc<Mapping>,
    filename: String,
    language: ProcessorLanguage,
    guard: Guard,
    anchor: usize,
) -> Mapping {
    // String owns UTF-8 data unlike &str, so a result can outlive the source parser.
    return Mapping {
        filename,
        text: String::new(),
        language,
        lines: Vec::new(),
        parent: Some(Arc::clone(parent)),
        guard,
        anchor,
    };
}

/// One extracted configuration input with immutable original-host mapping.
#[derive(Clone, Debug)]
pub struct VirtualSource {
    /// Prepared Rust or exact authored Markdown behind a read-only interface.
    pub(crate) mapping: Arc<Mapping>,
}

/// Read and use a virtual file without learning internal prefix or position machinery.
impl VirtualSource {
    /// Logical path for ordinary configuration matching,
    ///  never host display output.
    pub fn filename(&self) -> &str {
        return self.mapping.filename.as_str();
    }

    /// Prepared parse bytes;
    ///  synthetic bytes are handled by check_rust and projection.
    pub fn source(&self) -> &str {
        return self.mapping.text.as_str();
    }

    /// Rust or Markdown parser choice;
    ///  Rustdoc is Markdown,
    ///  not MDX.
    pub fn language(&self) -> ProcessorLanguage {
        return self.mapping.language;
    }

    /// Whether Markdown should use Rustdoc-specific rule behavior such as unlabeled Rust fences.
    pub fn is_rustdoc(&self) -> bool {
        return matches!(self.mapping.guard, Guard::Docs { .. });
    }

    /// Map a whole atomic fix,
    ///  or refuse it without returning a partial edit group.
    pub fn project_fix(&self, fix: &Fix) -> Result<Fix, ProcessorError> {
        return crate::processors_projection::project(&self.mapping, fix);
    }

    /// Map all authored labels and any grouped fix;
    ///  synthetic-only findings are absent.
    pub fn project_diagnostic(
        &self,
        mut finding: Diagnostic,
    ) -> Result<Option<Diagnostic>, ProcessorError> {
        if finding.labels.is_empty() {
            return Err(self
                .mapping
                .error("Processor diagnostic supplies no source labels to map."));
        }
        let mut labels: Vec<crate::diagnostic::Label> = Vec::new();
        for label in &finding.labels {
            let span = &label.span;
            let Some(end) = span.offset.checked_add(span.length) else {
                return Err(self
                    .mapping
                    .error("Processor diagnostic has an overflowing byte range."));
            };
            if end > self.source().len()
                || !self.source().is_char_boundary(span.offset)
                || !self.source().is_char_boundary(end)
            {
                return Err(self
                    .mapping
                    .error("Processor diagnostic has an invalid UTF-8 byte range."));
            }
            if let Some((start, finish)) =
                crate::processors_spans::host_range(&self.mapping, span.offset, end)
            {
                labels.push(crate::diagnostic::Label {
                    span: crate::processors_spans::host_span(
                        self.mapping.root(),
                        start,
                        finish - start,
                    ),
                });
            }
        }
        if labels.is_empty() {
            if !finding.processing_failure {
                return Ok(None);
            }
            // An inability to check must not disappear merely because its failure arose in scaffolding.
            labels.push(crate::diagnostic::Label {
                span: crate::processors_spans::host_span(
                    self.mapping.root(),
                    crate::processors_spans::anchor(&self.mapping),
                    0,
                ),
            });
        }
        finding.labels = labels;
        finding.filename = self.mapping.root().filename.clone();
        if let Some(fix) = &finding.fix {
            finding.fix = Some(self.project_fix(fix)?);
        }
        return Ok(Some(finding));
    }

    /// Run selected syntax checks and explicitly refuse full semantics without a supplied virtual Cargo context.
    pub fn check_rust(
        &self,
        settings: RustRuleSettings,
    ) -> Result<Vec<Diagnostic>, ProcessorError> {
        return crate::processors_check::check(self, settings);
    }
}

/// Pending extraction work retains its Rust-fence depth and authored Rustdoc mode.
struct Pending {
    /// Raw/hidden Rust for discovery,
    ///  never generated main text.
    mapping: Arc<Mapping>,
    /// Number of embedded Rust fences traversed from the real host.
    depth: usize,
    /// Unlabeled fences are Rust only when this Markdown came from Rustdoc.
    rustdoc: bool,
}

/// Extract all virtual files in deterministic source order;
///  nested Rust fences are capped at two.
/// Root rules remain the caller's responsibility.
///  Re-extract after every changed host snapshot.
pub fn extract(
    filename: String,
    source: String,
    language: ProcessorLanguage,
) -> Result<Vec<VirtualSource>, ProcessorError> {
    let root: Arc<Mapping> = Arc::new(Mapping {
        filename,
        text: source,
        language,
        lines: Vec::new(),
        parent: None,
        guard: Guard::Root,
        anchor: 0,
    });
    let mut pending: Vec<Pending> = vec![Pending {
        mapping: root,
        depth: 0,
        rustdoc: false,
    }];
    let mut output: Vec<VirtualSource> = Vec::new();
    while let Some(work) = pending.pop() {
        let parent: &Arc<Mapping> = &work.mapping;
        if parent.language == ProcessorLanguage::Yaml {
            // A workflow's steps become virtual shell files; they are never extracted again.
            for mapping in crate::processors_workflow::steps(parent)? {
                output.push(VirtualSource {
                    mapping: Arc::new(mapping),
                });
            }
            continue;
        }
        if parent.language == ProcessorLanguage::Rust {
            let children: Vec<Mapping> = crate::processors_docs::docs(parent)?;
            let mut next: Vec<Pending> = Vec::new();
            for mapping in children {
                let shared: Arc<Mapping> = Arc::new(mapping);
                output.push(VirtualSource {
                    mapping: Arc::clone(&shared),
                });
                next.push(Pending {
                    mapping: shared,
                    depth: work.depth,
                    rustdoc: true,
                });
            }
            pending.extend(next.into_iter().rev());
        } else {
            // The cap applies to Rust embeddings, not to authored Markdown prose at the final level.
            if work.depth >= 2 {
                continue;
            }
            let children: Vec<Mapping> = crate::processors_fences::fences(parent, work.rustdoc)?;
            let mut next: Vec<Pending> = Vec::new();
            for mapping in children {
                let raw: Arc<Mapping> = Arc::new(mapping);
                let hidden: Arc<Mapping> = Arc::new(crate::processors_prepare::hidden(&raw));
                let prepared: Arc<Mapping> = Arc::new(crate::processors_prepare::wrap(&hidden));
                output.push(VirtualSource { mapping: prepared });
                next.push(Pending {
                    mapping: hidden,
                    depth: work.depth + 1,
                    rustdoc: false,
                });
            }
            pending.extend(next.into_iter().rev());
        }
    }
    return Ok(output);
}

/// Consumer-facing extraction,
///  projection,
///  native parsing and generated adversarial controls.
#[cfg(test)]
#[path = "processors_tests.rs"]
mod tests;
