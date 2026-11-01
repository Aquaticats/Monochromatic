//! What:
//!  Compose declaration,
//!  generic-instantiation and inference-hole checks.
//! Why:
//!  Syntax alone cannot distinguish a generic call from a nongeneric method with the same name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Check all annotation surfaces against the same registered semantic parse.
//! ```

/// Import owned findings and selected rule severity.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the exact syntax-presence checks.
use crate::rust_explicit_declarations::check_declaration_annotations;
/// Import generic checks driven by actual callee/path resolution.
use crate::rust_explicit_generics::check_generic_arguments;
/// Import resolved placeholder validation.
use crate::rust_explicit_inference::check_inferred_types;
/// Import source positions retained from the semantic parse.
use crate::rust_source::RustSource;
/// Import the database-backed semantic view.
use ra_ap_hir::Semantics;
/// Import the database interface borrowed by the semantic view.
use ra_ap_hir_ty::db::HirDatabase;

/// What:
///  Enforce the complete policy inside an attached semantic database scope.
/// Why:
///  The context must come from that scope's registered syntax root,
///  not a separate parse.
/// Processing-failure findings remain distinct from ordinary violations;
///  no type or capture fixes are guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkExplicitTypes(semantics, context, severity): Diagnostic[];
/// ```
pub fn check_explicit_types(
    semantics: &Semantics<'_, dyn HirDatabase>,
    context: &RustSource,
    severity: Severity,
) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = check_declaration_annotations(context, severity);
    findings.extend(check_generic_arguments(semantics, context, severity));
    findings.extend(check_inferred_types(semantics, context, severity));
    return findings;
}
