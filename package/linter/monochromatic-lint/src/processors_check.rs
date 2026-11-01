//! What:
//!  Syntax checks over virtual Rust without synthetic-main side effects.
//! Why:
//!  Scaffolding affects parsing,
//!  never the file's code-line budget or authored documentation policy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Count authored tokens, inspect prepared syntax, map findings, refuse unknown semantic context.
//! ```

/// Original host diagnostics are the only returned findings.
use crate::diagnostic::Diagnostic;
/// Import caller-visible processor input and typed selections.
use crate::processors::{ProcessorError, ProcessorLanguage, VirtualSource};
/// Rule settings remain the configuration module's responsibility.
use crate::rust_rule_settings::RustRuleSettings;
/// Existing code-line and documentation checks remain the policy owners.
use crate::rust_rules::{check_max_lines, check_rustdoc};
/// Reuse the current native parse and rules,
///  not copied policy implementations.
use crate::rust_source::RustSource;

/// Check prepared syntax and authored line counts,
///  returning only original-host records.
pub(crate) fn check(
    input: &VirtualSource,
    settings: RustRuleSettings,
) -> Result<Vec<Diagnostic>, ProcessorError> {
    if input.language() != ProcessorLanguage::Rust {
        return Err(input
            .mapping
            .error("Rust checking was requested for a virtual Markdown input."));
    }
    if settings.explicit_types.is_some() {
        return Ok(vec![input.mapping.failure(
            "Full rust/require-explicit-types checking cannot resolve this virtual Rust input without an adequate registered Cargo semantic context. Configure that rule off for this virtual path. Checking virtual inputs with that rule requires executable integration that registers their semantic context; syntax-only inference is not used.",
        )]);
    }
    let parsed: RustSource =
        RustSource::new(String::from(input.filename()), String::from(input.source()));
    let mut findings: Vec<Diagnostic> = Vec::new();
    if let Some(severity) = settings.rustdoc {
        findings.extend(check_rustdoc(&parsed, severity));
    }
    let mut output: Vec<Diagnostic> = Vec::new();
    for mut finding in findings {
        // The file requirement still exists when its first token is generated main.
        if finding.code == "rust/require-rustdoc" && finding.message == "Missing rustdoc on file." {
            if let Some(first) = input.mapping.lines.first() {
                finding.labels[0].span = parsed.span(first.start, 0);
            } else {
                // An empty snippet still owes opening //! docs; use the authored fence anchor.
                finding.filename = input.mapping.root().filename.clone();
                finding.labels[0].span = crate::processors_spans::host_span(
                    input.mapping.root(),
                    crate::processors_spans::anchor(&input.mapping),
                    0,
                );
                output.push(finding);
                continue;
            }
        }
        if let Some(mapped) = input.project_diagnostic(finding)? {
            output.push(mapped);
        }
    }
    if let Some(limit) = settings.max_lines {
        // Count hidden-stripped authored source, never the generated main/brace lines.
        let authored = input
            .mapping
            .parent
            .as_ref()
            .expect("virtual Rust always has a preparation parent");
        let raw: RustSource = RustSource::new(authored.filename.clone(), authored.text.clone());
        let virtual_authored: VirtualSource = VirtualSource {
            mapping: std::sync::Arc::clone(authored),
        };
        for finding in check_max_lines(&raw, limit.max, limit.severity) {
            if let Some(mapped) = virtual_authored.project_diagnostic(finding)? {
                output.push(mapped);
            }
        }
    }
    return Ok(output);
}
