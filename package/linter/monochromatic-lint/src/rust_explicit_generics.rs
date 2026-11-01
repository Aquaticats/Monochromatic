//! What:
//!  Require generic argument spelling using resolved Rust declarations.
//! Why:
//!  Receiver type,
//!  aliases and imported function items determine the requirement,
//!  not a method-name list.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Resolve each instantiation, compare required and supplied arguments, and report unresolved coverage.
//! ```

/// Import owned findings and the retained source context.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the resolved argument-owner mapping,
///  including enum and function-trait spellings.
use crate::rust_generic_arguments::{
    GenericSite, is_instantiation_path, path_site, required_arguments, supplied_arguments,
};
/// Import exact source coordinates.
use crate::rust_source::RustSource;
/// Import ordinary findings and explicit semantic-processing failures.
use crate::rust_type_diagnostic::{resolution_failure, type_finding};
/// Import the native resolver and its typed declaration results.
use ra_ap_hir::{Function, GenericDef, PathResolution, Semantics};
/// Import the database interface borrowed during a query.
use ra_ap_hir_ty::db::HirDatabase;
/// Import the accessor implemented by method-call syntax.
use ra_ap_syntax::ast::HasGenericArgs;
/// Import typed syntax and generic-argument accessors.
use ra_ap_syntax::{AstNode, ast};

/// Report absent arguments on genuinely generic methods while leaving nongeneric methods unchanged.
fn check_method(
    semantics: &Semantics<'_, dyn HirDatabase>,
    context: &RustSource,
    call: ast::MethodCallExpr,
    severity: Severity,
) -> Option<Diagnostic> {
    let Some(function): Option<Function> = semantics.resolve_method_call(&call) else {
        return Some(resolution_failure(
            context,
            call.syntax(),
            "Cannot resolve this Rust method call to verify its generic arguments.",
        ));
    };
    let required: usize = required_arguments(semantics.db, GenericDef::Function(function));
    let provided: usize = supplied_arguments(call.generic_arg_list());
    if provided >= required {
        return None;
    }
    let message: String = format!(
        "Generic method requires {required} explicit type or constant arguments; found {provided}."
    );
    return Some(type_finding(
        context,
        call.syntax(),
        severity,
        message.as_str(),
        "Write the required arguments after the method name, using '::<...>'. Inferred '_' slots are checked separately.",
    ));
}

/// Check expression/type paths,
///  including generic function items passed as named callbacks.
fn check_path(
    semantics: &Semantics<'_, dyn HirDatabase>,
    context: &RustSource,
    path: ast::Path,
    severity: Severity,
) -> Option<Diagnostic> {
    if !is_instantiation_path(&path) {
        return None;
    }
    let Some(resolution): Option<PathResolution> = semantics.resolve_path(&path) else {
        return Some(resolution_failure(
            context,
            path.syntax(),
            "Cannot resolve this Rust path to verify its generic arguments.",
        ));
    };
    // Locals, Self, and generic parameters already name values/types fixed by their declarations.
    let PathResolution::Def(item): PathResolution = resolution else {
        return None;
    };
    // Absence here means a known nongeneric/paired qualifier site, not a failed resolution.
    let generic_site: GenericSite = path_site(semantics, &path, item)?;
    let required: usize = required_arguments(semantics.db, generic_site.definition);
    if generic_site.provided >= required {
        return None;
    }
    let provided: usize = generic_site.provided;
    let message: String = format!(
        "Generic path requires {required} explicit type or constant arguments; found {provided}."
    );
    return Some(type_finding(
        context,
        path.syntax(),
        severity,
        message.as_str(),
        "Write the required generic arguments on this type or function path. An enum constructor may put them on the enum/alias or variant.",
    ));
}

/// What:
///  Check each authored instantiation against its actual declaration in the current semantic scope.
/// Why:
///  Unresolved syntax produces a processing failure rather than being counted as a verified nongeneric call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGenericArguments(semantics, context, severity): Diagnostic[];
/// ```
pub(crate) fn check_generic_arguments(
    semantics: &Semantics<'_, dyn HirDatabase>,
    context: &RustSource,
    severity: Severity,
) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for node in context.syntax().descendants() {
        // Clone the shared node handle so both typed classifications retain the same registered root.
        if let Some(call) = ast::MethodCallExpr::cast(node.clone()) {
            if let Some(diagnostic) = check_method(semantics, context, call, severity) {
                findings.push(diagnostic);
            }
            continue;
        }
        let Some(path): Option<ast::Path> = ast::Path::cast(node) else {
            continue;
        };
        if let Some(diagnostic) = check_path(semantics, context, path, severity) {
            findings.push(diagnostic);
        }
    }
    return findings;
}
