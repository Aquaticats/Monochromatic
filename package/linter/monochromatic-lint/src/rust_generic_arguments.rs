//! What:
//!  Associate written generic arguments with their resolved declaration.
//! Why:
//!  Enum constructors accept arguments on the type or variant,
//!  aliases can bind parameters,
//!  and names alone are insufficient.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Resolve the declaration owning each argument list before checking its required parameters.
//! ```

/// Import resolved declarations rather than spelling-based method catalogues.
use ra_ap_hir::{
    Adt, Crate, DisplayTarget, EnumVariant, GenericDef, GenericParam, Module, ModuleDef,
    PathResolution, Semantics,
};
/// Import the database interface queried inside the existing attachment scope.
use ra_ap_hir_ty::db::HirDatabase;
/// Import the trait implementing generic_arg_list on both methods and path segments.
use ra_ap_syntax::ast::HasGenericArgs;
/// Import typed syntax views and the shared generic-argument accessor.
use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, ast};

/// What:
///  Written arguments paired with the declaration that supplies their meaning.
/// Why:
///  A fixed alias can eliminate an underlying enum's parameters without requiring redundant variant arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GenericSite = { definition: GenericDef; provided: number };
/// ```
pub(crate) struct GenericSite {
    /// Resolved declaration whose explicit parameters can be supplied at this site.
    pub definition: GenericDef,
    /// Argument count from the permitted spelling;
    ///  function-trait parentheses supply one tuple argument.
    pub provided: usize,
}

/// Convert the resolved item kind without inspecting any identifier text.
pub(crate) fn generic_definition(item: ModuleDef) -> Option<GenericDef> {
    // Match extracts each tagged payload, like a switch over a TypeScript discriminated union.
    match item {
        ModuleDef::Function(value) => return Some(GenericDef::Function(value)),
        ModuleDef::Adt(value) => return Some(GenericDef::Adt(value)),
        ModuleDef::Trait(value) => return Some(GenericDef::Trait(value)),
        ModuleDef::TypeAlias(value) => return Some(GenericDef::TypeAlias(value)),
        ModuleDef::Const(value) => return Some(GenericDef::Const(value)),
        ModuleDef::Static(value) => return Some(GenericDef::Static(value)),
        // A variant's enum or alias supplies its parameters; handled at the paired path boundary.
        ModuleDef::EnumVariant(_)
        | ModuleDef::Module(_)
        | ModuleDef::BuiltinType(_)
        | ModuleDef::Macro(_) => return None,
    }
}

/// Count the final segment's explicit arguments,
///  including Rust's Fn(Args) shorthand.
fn written_arguments(path: &ast::Path) -> usize {
    let Some(segment): Option<ast::PathSegment> = path.segment() else {
        return 0;
    };
    if segment.parenthesized_arg_list().is_some() {
        // Fn(u16, u8) spells the single tuple parameter explicitly, as does Fn() for an empty tuple.
        return 1;
    }
    return supplied_arguments(segment.generic_arg_list());
}

/// Pair enum-constructor spellings and count them at the variant rather than at both adjacent names.
fn variant_site(
    semantics: &Semantics<'_, dyn HirDatabase>,
    path: &ast::Path,
    variant: EnumVariant,
) -> GenericSite {
    let fallback: GenericSite = GenericSite {
        definition: GenericDef::Adt(Adt::Enum(variant.parent_enum(semantics.db))),
        provided: written_arguments(path),
    };
    let Some(qualifier): Option<ast::Path> = path.qualifier() else {
        return fallback;
    };
    let Some(PathResolution::Def(item)): Option<PathResolution> =
        semantics.resolve_path(&qualifier)
    else {
        return fallback;
    };
    let Some(owner): Option<GenericDef> = generic_definition(item) else {
        return fallback;
    };
    // The qualifier can be a generic or fixed alias, not necessarily the underlying enum name.
    // Do not add counts: enum and variant lists spell the same parameter group, not separate groups.
    let provided: usize = fallback.provided.max(written_arguments(&qualifier));
    return GenericSite {
        definition: owner,
        provided,
    };
}

/// True when the parent path selects an enum variant,
///  which owns the combined enum/alias argument check.
fn followed_by_variant(semantics: &Semantics<'_, dyn HirDatabase>, path: &ast::Path) -> bool {
    let Some(parent): Option<SyntaxNode> = path.syntax().parent() else {
        return false;
    };
    let Some(parent_path): Option<ast::Path> = ast::Path::cast(parent) else {
        return false;
    };
    if let Some(PathResolution::Def(ModuleDef::EnumVariant(_))) =
        semantics.resolve_path(&parent_path)
    {
        return true;
    }
    return false;
}

/// Return a type/expression site's actual generic declaration,
///  with valid nongeneric items returning absence.
pub(crate) fn path_site(
    semantics: &Semantics<'_, dyn HirDatabase>,
    path: &ast::Path,
    item: ModuleDef,
) -> Option<GenericSite> {
    if let ModuleDef::EnumVariant(variant) = item {
        return Some(variant_site(semantics, path, variant));
    }
    let definition: GenericDef = generic_definition(item)?;
    // A following variant owns the paired enum/alias application, so its qualifier is not checked twice.
    if followed_by_variant(semantics, path) {
        return None;
    }
    return Some(GenericSite {
        definition,
        provided: written_arguments(path),
    });
}

/// Type and expression paths carry instantiations;
///  imports and match patterns do not create this requirement.
pub(crate) fn is_instantiation_path(path: &ast::Path) -> bool {
    let mut current: Option<SyntaxNode> = path.syntax().parent();
    while let Some(parent) = current {
        let kind: SyntaxKind = parent.kind();
        if kind == SyntaxKind::PATH {
            current = parent.parent();
            continue;
        }
        return kind == SyntaxKind::PATH_EXPR
            || kind == SyntaxKind::PATH_TYPE
            || kind == SyntaxKind::RECORD_EXPR;
    }
    return false;
}

/// Distinguish a required type/constant slot from lifetimes,
///  compiler-introduced slots and fixed defaults.
fn is_required(database: &dyn HirDatabase, parameter: GenericParam, target: DisplayTarget) -> bool {
    if let GenericParam::TypeParam(ty) = parameter {
        return !ty.is_implicit(database) && ty.default(database).is_none();
    }
    if let GenericParam::ConstParam(value) = parameter {
        return value.default(database, target).is_none();
    }
    return false;
}

/// Count only explicit required type/constant parameters;
///  lifetimes,
///  implicit impl-Trait parameters and defaults are exempt.
pub(crate) fn required_arguments(database: &dyn HirDatabase, definition: GenericDef) -> usize {
    let module: Module = definition.module(database);
    let krate: Crate = module.krate(database);
    let target: DisplayTarget = krate.to_display_target(database);
    let parameters: Vec<GenericParam> = definition.params(database);
    let mut count: usize = 0;
    for parameter in parameters {
        if is_required(database, parameter, target) {
            count += 1;
        }
    }
    return count;
}

/// Count actual argument slots without treating lifetimes or associated-type constraints as type arguments.
pub(crate) fn supplied_arguments(arguments: Option<ast::GenericArgList>) -> usize {
    let Some(list): Option<ast::GenericArgList> = arguments else {
        return 0;
    };
    let mut count: usize = 0;
    for argument in list.generic_args() {
        match argument {
            ast::GenericArg::TypeArg(_) | ast::GenericArg::ConstArg(_) => count += 1,
            ast::GenericArg::LifetimeArg(_) | ast::GenericArg::AssocTypeArg(_) => {}
        }
    }
    return count;
}
