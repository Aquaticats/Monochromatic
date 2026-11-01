//! What:
//!  Identify constant placeholders parsed in the ambiguous generic-argument type slot.
//! Why:
//!  Rust's syntax tree alone cannot tell whether '::<_>' supplies a type or a const parameter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Match the written argument's position to the resolved declaration's explicit parameter list.
//! ```

/// Import resolved argument ownership,
///  including aliases and enum variants.
use crate::rust_generic_arguments::{GenericSite, generic_definition, path_site};
/// Import real declarations instead of identifier spelling.
use ra_ap_hir::{GenericDef, GenericParam, ModuleDef, PathResolution, Semantics};
/// Import the query database interface.
use ra_ap_hir_ty::db::HirDatabase;
/// Import syntax identities and argument kinds.
use ra_ap_syntax::{AstNode, SyntaxNode, ast};

/// Find which resolved declaration supplies the argument-list grammar.
fn argument_owner(
    semantics: &Semantics<'_, dyn HirDatabase>,
    list: &ast::GenericArgList,
) -> Option<GenericDef> {
    let parent: SyntaxNode = list.syntax().parent()?;
    if let Some(call) = ast::MethodCallExpr::cast(parent.clone()) {
        return Some(GenericDef::Function(semantics.resolve_method_call(&call)?));
    }
    let segment: ast::PathSegment = ast::PathSegment::cast(parent)?;
    let path: ast::Path = ast::Path::cast(segment.syntax().parent()?)?;
    let PathResolution::Def(item): PathResolution = semantics.resolve_path(&path)? else {
        return None;
    };
    if let ModuleDef::EnumVariant(_) = item {
        let site: GenericSite = path_site(semantics, &path, item)?;
        return Some(site.definition);
    }
    // Do not suppress enum qualifiers here: their own written argument slots still have a declaration.
    return generic_definition(item);
}

/// Find the zero-based type/const position without counting lifetimes or associated-type constraints.
fn argument_index(list: &ast::GenericArgList, target: &SyntaxNode) -> Option<usize> {
    let mut index: usize = 0;
    for argument in list.generic_args() {
        if argument.syntax() == target {
            return Some(index);
        }
        match argument {
            ast::GenericArg::TypeArg(_) | ast::GenericArg::ConstArg(_) => index += 1,
            ast::GenericArg::LifetimeArg(_) | ast::GenericArg::AssocTypeArg(_) => {}
        }
    }
    return None;
}

/// Prove a direct generic placeholder belongs to a const parameter before requesting type information for it.
pub(crate) fn is_inferred_const_argument(
    semantics: &Semantics<'_, dyn HirDatabase>,
    node: &SyntaxNode,
) -> bool {
    let Some(argument): Option<SyntaxNode> = node.parent() else {
        return false;
    };
    // Nested type holes such as Vec<_> do not stand for the enclosing argument itself.
    if ast::TypeArg::cast(argument.clone()).is_none() {
        return false;
    }
    let Some(parent): Option<SyntaxNode> = argument.parent() else {
        return false;
    };
    let Some(list): Option<ast::GenericArgList> = ast::GenericArgList::cast(parent) else {
        return false;
    };
    let Some(wanted): Option<usize> = argument_index(&list, &argument) else {
        return false;
    };
    let Some(owner): Option<GenericDef> = argument_owner(semantics, &list) else {
        return false;
    };
    let mut index: usize = 0;
    for parameter in owner.params(semantics.db) {
        if let GenericParam::LifetimeParam(_) = parameter {
            continue;
        }
        if let GenericParam::TypeParam(ty) = parameter
            && ty.is_implicit(semantics.db)
        {
            continue;
        }
        if index == wanted {
            return matches!(parameter, GenericParam::ConstParam(_));
        }
        index += 1;
    }
    return false;
}

/// Slot-position controls need a loaded Cargo project and stay outside release code.
#[cfg(test)]
#[path = "rust_inferred_constants_tests.rs"]
mod tests;
