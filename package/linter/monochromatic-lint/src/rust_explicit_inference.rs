//! What:
//!  Reject written inference holes when the inferred type can be named.
//! Why:
//!  'let value:
//!  _' and 'parse::<_>()' do not meet the policy merely by containing annotation punctuation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Resolve each placeholder and permit only a positively identified unnameable leaf type.
//! ```

/// Import the owned diagnostic and source interfaces.
use crate::diagnostic::{Diagnostic, Severity};
/// Resolve ambiguous generic placeholder slots against their real parameter kinds.
use crate::rust_inferred_constants::is_inferred_const_argument;
/// Import the source whose syntax root belongs to the semantic database.
use crate::rust_source::RustSource;
/// Import separate finding and unavailable-information constructors.
use crate::rust_type_diagnostic::{resolution_failure, type_finding};
/// Import semantic type families rather than parsing human-readable type displays.
use ra_ap_hir::{Callable, CallableKind, Semantics, Type};
/// Import the database interface kept alive throughout semantic queries.
use ra_ap_hir_ty::db::HirDatabase;
/// Import syntax kinds and typed inference nodes.
use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, ast};

/// What:
///  Recognize unnameable leaf types while retaining explicit outer structures.
/// Why:
///  A reference/container around a function item should be written as '&_'/'Container<_>',
///  not replaced wholesale by '_'.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isUnnameableLeaf(database, type): boolean;
/// ```
fn is_unnameable_leaf(database: &dyn HirDatabase, ty: &Type<'_>) -> bool {
    // is_fn checks the actual type, unlike as_callable, which also follows references and callable trait bounds.
    if ty.is_fn() {
        let Some(callable): Option<Callable<'_>> = ty.as_callable(database) else {
            return false;
        };
        match callable.kind() {
            CallableKind::Function(_)
            | CallableKind::TupleStruct(_)
            | CallableKind::TupleEnumVariant(_) => return true,
            // Function pointers have a writable fn(...) -> ... type and are not exempt.
            CallableKind::FnPtr | CallableKind::FnImpl(_) | CallableKind::Closure(_) => {
                return false;
            }
        }
    }
    // Closures have unnameable types; a separate closure check, not this rule, decides whether their expressions are acceptable.
    if ty.as_closure().is_some() {
        return true;
    }
    // The inspected backend includes opaque impl-Trait values, implicit impl-Trait parameters and async-block coroutines.
    return ty.as_impl_traits(database).is_some();
}

/// Distinguish constant inference from discard expressions,
///  which have no annotation slot.
fn is_constant_placeholder(node: &SyntaxNode) -> bool {
    if node.kind() != SyntaxKind::UNDERSCORE_EXPR {
        return false;
    }
    if let Some(parent) = node.parent() {
        return parent.kind() == SyntaxKind::CONST_ARG;
    }
    return false;
}

/// Check every explicit type placeholder and reject inferred constant placeholders as values.
pub(crate) fn check_inferred_types(
    semantics: &Semantics<'_, dyn HirDatabase>,
    context: &RustSource,
    severity: Severity,
) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for node in context.syntax().descendants() {
        if is_constant_placeholder(&node)
            || (node.kind() == SyntaxKind::INFER_TYPE
                && is_inferred_const_argument(semantics, &node))
        {
            findings.push(type_finding(
                context, &node, severity,
                "Replace inferred '_' with an explicit constant value.",
                "Write the array length or generic constant argument instead of asking Rust to infer it.",
            ));
            continue;
        }
        if node.kind() != SyntaxKind::INFER_TYPE {
            continue;
        }
        // The kind check guarantees this cast; keep the original registered node for its source span.
        let syntax_type: ast::Type =
            ast::Type::cast(node.clone()).expect("inference kind is a type node");
        let Some(inferred): Option<Type<'_>> = semantics.resolve_type(&syntax_type) else {
            findings.push(resolution_failure(
                context, &node,
                "Cannot establish whether this '_' denotes an unnameable Rust type. Write an explicit type or constant value, or provide complete resolution context.",
            ));
            continue;
        };
        if inferred.contains_unknown() {
            findings.push(resolution_failure(
                context, &node,
                "The inferred Rust type still contains unresolved information; this '_' cannot be verified as an unnameable-type exception.",
            ));
            continue;
        }
        if is_unnameable_leaf(semantics.db, &inferred) {
            continue;
        }
        findings.push(type_finding(
            context, &node, severity,
            "Replace inferred '_' with an explicit Rust type.",
            "Name the type at this position. Keep known outer reference/container types explicit; reserve '_' for an unnameable function item, opaque value or anonymous-function type.",
        ));
    }
    return findings;
}
