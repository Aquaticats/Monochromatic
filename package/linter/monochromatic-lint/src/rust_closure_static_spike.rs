//! What:
//!  THROWAWAY SPIKE. Classify closures as ALLOW or REPORT under the proposed criterion:
//!  the closure captures something AND is a direct argument whose callee parameter carries a `'static` bound.
//! Why:
//!  Decide whether `ra_ap_hir` 0.0.336 can answer the question through the linter's own semantic session.
//!  Gated by an environment variable or a thread-local switch, so the shipped rule behavior is unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // classify(closure): captured_items(closure).length > 0 && calleeParamHasStaticBound(directCall(closure))
//! ```

/// Import findings.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the registered semantic parse.
use crate::rust_source::RustSource;
/// Import hir entry points.
use ra_ap_hir::{AsAssocItem, CallableKind, Function, GenericDef, Semantics, Trait};
/// Import the predicate store and raw type model used for the bound check.
use ra_ap_hir_ty::GenericPredicates;
use ra_ap_hir_ty::db::HirDatabase;
use ra_ap_hir_ty::next_solver::{
    CallableIdWrapper, Clause, ClauseKind, PredicateKind, RegionKind, SolverDefId, Ty, TyKind,
};
/// Import syntax access.
use ra_ap_syntax::ast;
use ra_ap_syntax::{AstNode, SyntaxNode};
use std::cell::Cell;

thread_local! {
    /// Test switch, because tests cannot set environment variables safely.
    static FORCE: Cell<bool> = const { Cell::new(false) };
}

/// Turn the experiment on for the current thread.
pub fn force(on: bool) {
    FORCE.with(|cell| cell.set(on));
}

/// True when the experiment should run.
pub fn enabled() -> bool {
    return FORCE.with(Cell::get)
        || std::env::var_os("MONOCHROMATIC_LINT_SPIKE_CLOSURE_STATIC").is_some();
}

/// The verdict and its evidence trail for one closure.
pub struct Verdict {
    /// True when the proposed criterion lets the closure pass.
    pub allow: bool,
    /// Calls and facts used to reach the verdict.
    pub trace: Vec<String>,
}

/// Read the clause kind without the `IntoKind` trait, which hir_ty imports privately.
fn clause_kind<'db>(clause: &Clause<'db>) -> Option<ClauseKind<'db>> {
    return match clause.as_predicate().inner().internee.skip_binder() {
        PredicateKind::Clause(kind) => Some(kind),
        _ => None,
    };
}

/// Read the type kind without the `IntoKind` trait, which hir_ty imports privately.
fn ty_kind<'a, 'db>(ty: &'a Ty<'db>) -> &'a TyKind<'db> {
    return &ty.inner().internee;
}

/// Does `ty` carry a `'static` bound, and by which route?
fn static_route(
    database: &dyn HirDatabase,
    clauses: &[Clause<'_>],
    ty: Ty<'_>,
    depth: usize,
    trace: &mut Vec<String>,
) -> bool {
    if depth > 4 {
        return false;
    }
    match ty_kind(&ty) {
        TyKind::Param(_) => {
            for clause in clauses {
                if let Some(ClauseKind::TypeOutlives(outlives)) = clause_kind(clause) {
                    if outlives.0 == ty && matches!(outlives.1.inner(), RegionKind::ReStatic) {
                        trace.push(String::from("GenericPredicates::query_all -> ClauseKind::TypeOutlives(param, ReStatic)"));
                        return true;
                    }
                }
            }
            // Supertrait elaboration: `F: Task` with `trait Task: 'static`.
            for clause in clauses {
                if let Some(ClauseKind::Trait(trait_predicate)) = clause_kind(clause) {
                    if trait_predicate.self_ty() == ty {
                        let trait_: Trait = Trait::from(trait_predicate.def_id().0);
                        let Ok(trait_id) = GenericDef::Trait(trait_).try_into() else { continue };
                        let own = GenericPredicates::query_all(database, trait_id);
                        for supertrait_clause in own.iter_identity() {
                            if let Some(ClauseKind::TypeOutlives(outlives)) = clause_kind(&supertrait_clause.skip_norm_wip()) {
                                if matches!(outlives.1.inner(), RegionKind::ReStatic)
                                    && matches!(ty_kind(&outlives.0), TyKind::Param(_))
                                {
                                    trace.push(String::from("supertrait elaboration: trait predicates hold Self: 'static"));
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            return false;
        }
        TyKind::Dynamic(_, region) => {
            let is_static: bool = matches!(region.inner(), RegionKind::ReStatic);
            trace.push(format!("TyKind::Dynamic region is ReStatic = {is_static}"));
            return is_static;
        }
        TyKind::Adt(_, arguments) => {
            for argument in arguments.iter() {
                if let Some(inner) = argument.ty() {
                    if static_route(database, clauses, inner, depth + 1, trace) {
                        return true;
                    }
                }
            }
            return false;
        }
        _ => return false,
    }
}

/// Ask whether input `index` of `function` carries a `'static` bound.
fn input_is_static(database: &dyn HirDatabase, function: Function, index: usize, trace: &mut Vec<String>) -> bool {
    let Ok(function_id) = function.try_into() else {
        trace.push(String::from("Function -> FunctionId failed (builtin derive method)"));
        return false;
    };
    let Ok(wrapper) = CallableIdWrapper::try_from(SolverDefId::FunctionId(function_id)) else {
        trace.push(String::from("SolverDefId -> CallableIdWrapper failed"));
        return false;
    };
    trace.push(String::from("db.callable_item_signature(CallableIdWrapper(..).0).skip_binder().skip_binder().inputs()"));
    let signature = database.callable_item_signature(wrapper.0);
    let inputs = signature.skip_binder().skip_binder().inputs().to_vec();
    let Some(&input) = inputs.get(index) else {
        trace.push(format!("no input at index {index} (arity {})", inputs.len()));
        return false;
    };
    let Ok(generic_id) = GenericDef::Function(function).try_into() else {
        trace.push(String::from("GenericDef::Function -> GenericDefId failed"));
        return false;
    };
    let clauses: Vec<_> = GenericPredicates::query_all(database, generic_id).iter_identity().map(|clause| clause.skip_norm_wip()).collect();
    trace.push(format!("GenericPredicates::query_all(Function) -> {} clauses; input[{index}] kind = {:?}", clauses.len(), ty_kind(&input)));
    return static_route(database, &clauses, input, 0, trace);
}

/// Is `function` the associated function `Box::new`?
fn is_box_new(semantics: &Semantics<'_, dyn HirDatabase>, function: Function) -> bool {
    let database: &dyn HirDatabase = semantics.db;
    if function.name(database).as_str() != "new" {
        return false;
    }
    let Some(item) = function.as_assoc_item(database) else { return false };
    let ra_ap_hir::AssocItemContainer::Impl(implementation) = item.container(database) else {
        return false;
    };
    let Some(adt) = implementation.self_ty(database).as_adt() else { return false };
    return adt.name(database).as_str() == "Box";
}

/// Resolve the declared callee of a call expression and the index of the argument in the declared inputs.
fn callee_of(
    semantics: &Semantics<'_, dyn HirDatabase>,
    call: &SyntaxNode,
    argument_position: usize,
    trace: &mut Vec<String>,
) -> Option<(Function, usize)> {
    if let Some(method) = ast::MethodCallExpr::cast(call.clone()) {
        let function: Function = semantics.resolve_method_call(&method)?;
        trace.push(String::from("Semantics::resolve_method_call -> Function; declared input index = position + 1 (self)"));
        return Some((function, argument_position + 1));
    }
    let direct = ast::CallExpr::cast(call.clone())?;
    let callee = direct.expr()?;
    let info = semantics.type_of_expr(&callee)?;
    let callable = info.original.as_callable(semantics.db)?;
    match callable.kind() {
        CallableKind::Function(function) => {
            trace.push(String::from("Semantics::type_of_expr(callee).original.as_callable().kind() -> CallableKind::Function; index = position"));
            return Some((function, argument_position));
        }
        CallableKind::FnImpl(_) | CallableKind::FnPtr | CallableKind::Closure(_) => {
            trace.push(String::from("callee is a value of fn-trait/fn-pointer/closure type: no declared Function"));
            return None;
        }
        CallableKind::TupleStruct(_) | CallableKind::TupleEnumVariant(_) => {
            trace.push(String::from("callee is a tuple constructor: no bound"));
            return None;
        }
    }
}

/// Find the call whose argument list holds `node` directly.
fn enclosing_call(node: &SyntaxNode) -> Option<(SyntaxNode, usize)> {
    let list = ast::ArgList::cast(node.parent()?)?;
    let position = list.args().position(|argument| argument.syntax() == node)?;
    let call = list.syntax().parent()?;
    return Some((call, position));
}

/// Classify one closure.
pub fn classify(semantics: &Semantics<'_, dyn HirDatabase>, closure: &ast::ClosureExpr) -> Verdict {
    let mut trace: Vec<String> = Vec::new();
    let database: &dyn HirDatabase = semantics.db;
    let expression = ast::Expr::ClosureExpr(closure.clone());
    let Some(info) = semantics.type_of_expr(&expression) else {
        trace.push(String::from("type_of_expr(closure) = None"));
        return Verdict { allow: false, trace };
    };
    let Some(hir_closure) = info.original.as_closure() else {
        trace.push(String::from("Type::as_closure() = None"));
        return Verdict { allow: false, trace };
    };
    let captures: usize = hir_closure.captured_items(database).len();
    trace.push(format!("Semantics::type_of_expr(closure).original.as_closure().captured_items() = {captures}"));
    if captures == 0 {
        return Verdict { allow: false, trace };
    }
    let mut node: SyntaxNode = closure.syntax().clone();
    let Some((mut call, mut position)) = enclosing_call(&node) else {
        trace.push(String::from("closure is not a direct call argument"));
        return Verdict { allow: false, trace };
    };
    // Look through `Box::new(closure)` so a `Box<dyn FnMut() + 'static>` parameter can be reached.
    for _ in 0..2 {
        let Some((function, _)) = callee_of(semantics, &call, position, &mut Vec::new()) else { break };
        if !is_box_new(semantics, function) {
            break;
        }
        trace.push(String::from("callee is Box::new: looking through to the outer call (Function::as_assoc_item, container Impl self_ty Adt name)"));
        node = call.clone();
        let Some((outer, outer_position)) = enclosing_call(&node) else {
            trace.push(String::from("Box::new(..) is not a direct call argument"));
            return Verdict { allow: false, trace };
        };
        call = outer;
        position = outer_position;
    }
    let Some((function, index)) = callee_of(semantics, &call, position, &mut trace) else {
        trace.push(String::from("callee did not resolve to a declared Function"));
        return Verdict { allow: false, trace };
    };
    trace.push(format!("callee = {}", function.name(database).as_str()));
    let allow: bool = input_is_static(database, function, index, &mut trace);
    return Verdict { allow, trace };
}

/// Run the classification over every closure the existing syntax rule sees, plus one macro expansion level.
pub fn classify_closures(semantics: &Semantics<'_, dyn HirDatabase>, context: &RustSource) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::new();
    for node in context.syntax().descendants() {
        if let Some(closure) = ast::ClosureExpr::cast(node.clone()) {
            let range = closure.syntax().text_range();
            let verdict: Verdict = classify(semantics, &closure);
            let word: &str = if verdict.allow { "ALLOW" } else { "REPORT" };
            let message: String = format!("closure@{} {word} :: {}", usize::from(range.start()), verdict.trace.join(" ; "));
            findings.push(Diagnostic::new(
                "spike/closure-static",
                Severity::Warn,
                message,
                context.filename.clone(),
                context.span(usize::from(range.start()), usize::from(range.len())),
            ));
        }
        if let Some(call) = ast::MacroCall::cast(node) {
            let Some(expanded) = semantics.expand_macro_call(&call) else {
                findings.push(Diagnostic::new(
                    "spike/closure-static",
                    Severity::Warn,
                    String::from("macro-call unexpanded :: Semantics::expand_macro_call = None"),
                    context.filename.clone(),
                    context.span(usize::from(call.syntax().text_range().start()), usize::from(call.syntax().text_range().len())),
                ));
                continue;
            };
            for inner in expanded.value.descendants() {
                if let Some(closure) = ast::ClosureExpr::cast(inner) {
                    let verdict: Verdict = classify(semantics, &closure);
                    let word: &str = if verdict.allow { "ALLOW" } else { "REPORT" };
                    let message: String = format!("macro-closure@{} {word} :: {}", usize::from(call.syntax().text_range().start()), verdict.trace.join(" ; "));
                    findings.push(Diagnostic::new(
                        "spike/closure-static",
                        Severity::Warn,
                        message,
                        context.filename.clone(),
                        context.span(usize::from(call.syntax().text_range().start()), usize::from(call.syntax().text_range().len())),
                    ));
                }
            }
        }
    }
    return findings;
}

/// Spike controls.
#[cfg(test)]
#[path = "rust_closure_static_spike_tests.rs"]
mod tests;
