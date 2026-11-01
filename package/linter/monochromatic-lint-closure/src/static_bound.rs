//! What:
//!  Whether a callee's declared parameter requires its argument to be `'static`.
//! Why:
//!  A callee that requires `'static` keeps or runs the callable past the call,
//!  so only a closure that owns state can serve it;
//!  this is the callee half of the lint criterion.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parameterRequiresStatic(callee: DefId, index: number, boxed: boolean): boolean;
//! ```

/// Import the definition identifier type of the callee.
use rustc_hir::def_id::DefId;
/// Import the lint context that gives access to compiler queries.
use rustc_lint::LateContext;
/// Import clause kinds (the compiler's name for where-clauses and bounds) and type kinds.
use rustc_middle::ty::{ClauseKind, Ty, TyKind};

/// What:
///  True when the declared parameter at `index` of `callee` must outlive `'static`.
///  `index` counts a method receiver as parameter zero;
///  `boxed` means the argument is wrapped in `Box::new(..)`.
/// Why:
///  The bound sits in three places:
///  a generic parameter's outlives clause (including `impl Trait` arguments,
///  impl and trait parents,
///  and supertraits),
///  or the lifetime of a `dyn Trait` object inside a `Box` parameter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parameterRequiresStatic(callee: DefId, index: number, boxed: boolean): boolean {
///   const declared = signatureOf(callee).inputs[index];
///   if (declared === undefined) return false;
///   return typeRequiresStatic(boxed ? unwrapBoxes(declared) : declared);
/// }
/// ```
pub fn parameter_requires_static(
    cx: &LateContext<'_>,
    callee: DefId,
    index: usize,
    boxed: bool,
) -> bool {
    // What: fn_sig is a query returning the declared signature wrapped in two binders (generic and late-bound
    // lifetimes); skip_binder twice reads the types without instantiating them.
    // Why: Only the declared parameter types matter, not any particular call's substitutions.
    let signature = cx.tcx.fn_sig(callee).skip_binder().skip_binder();
    let Some(declared) = signature.inputs().get(index) else {
        return false;
    };
    let mut parameter: Ty<'_> = *declared;
    // What: A `Box<T>` parameter receiving `Box::new(closure)` is judged by the `T` inside; a generic parameter
    // receiving the box (`F: 'static` with `F = Box<closure>`) is judged as it stands.
    // Why: Both shapes of "callee keeps a boxed callable" reach the same `'static` requirement.
    if boxed {
        while let Some(inner) = parameter.boxed_ty() {
            parameter = inner;
        }
    }
    return type_requires_static(cx, callee, parameter);
}

/// What:
///  True when `parameter` is a generic type with a `'static` outlives clause in `callee`'s environment,
///  or a `dyn Trait` object whose lifetime bound is `'static`.
/// Why:
///  These are the two ways a declared parameter type forces `'static` without naming a concrete type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function typeRequiresStatic(callee: DefId, parameter: Type): boolean;
/// ```
fn type_requires_static<'tcx>(cx: &LateContext<'tcx>, callee: DefId, parameter: Ty<'tcx>) -> bool {
    match parameter.kind() {
        TyKind::Dynamic(_, region) => return region.is_static(),
        TyKind::Param(_) => return environment_requires_static(cx, callee, parameter),
        _ => return false,
    }
}

/// What:
///  Scan the where-clauses of `callee` for `parameter: 'static`.
/// Why:
///  `param_env` already merges the impl and trait parents' clauses and elaborates supertraits,
///  so `F: Task` with `trait Task: 'static` yields `F: 'static` without a separate elaboration step
///  (measured:
///  removing an explicit elaboration pass changed no fixture result).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function environmentRequiresStatic(callee: DefId, parameter: Type): boolean {
///   for (const clause of environmentOf(callee)) {
///     if (clause.kind === 'outlives' && clause.type === parameter && clause.region === 'static') return true;
///   }
///   return false;
/// }
/// ```
fn environment_requires_static<'tcx>(
    cx: &LateContext<'tcx>,
    callee: DefId,
    parameter: Ty<'tcx>,
) -> bool {
    let environment = cx.tcx.param_env(callee);
    for clause in environment.caller_bounds() {
        let ClauseKind::TypeOutlives(outlives) = clause.kind().skip_binder() else {
            continue;
        };
        if outlives.0 == parameter && outlives.1.is_static() {
            return true;
        }
    }
    return false;
}
