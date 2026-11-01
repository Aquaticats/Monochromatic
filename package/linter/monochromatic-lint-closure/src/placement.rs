//! What:
//!  Where a closure expression sits relative to the call that receives it.
//! Why:
//!  The lint accepts a closure only when it is a direct argument whose declared parameter must be `'static`,
//!  so this module answers the placement half of the criterion and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Placement = 'kept-by-callee' | 'not-kept-by-callee' | 'let-initializer' | 'unresolved-callee' | 'other';
//! function classifyPlacement(closure: Expr): Placement;
//! ```

/// Import the shared test for a declared `'static` parameter.
use crate::static_bound::parameter_requires_static;
/// Import the compiler's definition kind to separate functions from constructors.
use rustc_hir::def::DefKind;
/// Import the definition identifier type of the callee.
use rustc_hir::def_id::DefId;
/// Import the compiler's high-level IR node types:
/// expressions,
/// call kinds,
/// nodes.
use rustc_hir::{Expr, ExprKind, Node};
/// Import the lint context that gives access to type information.
use rustc_lint::LateContext;
/// Import the compiler's type kind enum to inspect what a callee expression is.
use rustc_middle::ty::TyKind;
/// Import the interned symbol table for the standard library's diagnostic item names.
use rustc_span::sym;

/// What:
///  The placement categories of a closure expression.
///  `#[derive(Clone, Copy, Debug, Eq, PartialEq)]` lets values be copied and compared like TS string-literal unions.
/// Why:
///  The lint message names exactly which placement failed,
///  so each category is its own variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placement = 'kept-by-callee' | 'not-kept-by-callee' | 'let-initializer' | 'unresolved-callee' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    /// Direct call argument whose declared parameter requires `'static`.
    KeptByCallee,
    /// Direct call argument whose declared parameter has no `'static` requirement.
    NotKeptByCallee,
    /// Initializer of a `let` statement.
    LetInitializer,
    /// Direct call argument,
    /// but the callee is not a declared function (a value,
    /// a function pointer,
    /// a constructor).
    UnresolvedCallee,
    /// Any other position:
    /// returned,
    /// stored in a field,
    /// wrapped in a type other than `Box`,
    /// a method receiver.
    Other,
}

/// What:
///  Where one call-like expression sends an argument:
///  a declared function and parameter index,
///  or nothing usable.
/// Why:
///  Separates "which parameter is it" from "does that parameter need `'static`".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slot = { kind: 'declared'; callee: DefId; index: number } | { kind: 'unresolved' } | { kind: 'not-argument' };
/// ```
enum Slot {
    /// The declared function and the zero-based parameter index,
    /// counting a method receiver as parameter zero.
    Declared {
        /// The callee function or method.
        callee: DefId,
        /// The zero-based parameter index.
        index: usize,
    },
    /// The expression is an argument of a call whose callee has no declared parameter list.
    Unresolved,
    /// The expression is not an argument of a call.
    NotArgument,
}

/// What:
///  Decide the placement of `closure`,
///  looking through one `Box::new(..)` call.
/// Why:
///  `Box::new(closure)` is how a closure reaches a `Box<dyn FnMut() + 'static>` parameter,
///  so the box is transparent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyPlacement(closure: Expr): Placement;
/// ```
pub fn classify_placement<'tcx>(cx: &LateContext<'tcx>, closure: &'tcx Expr<'tcx>) -> Placement {
    let (subject, boxed): (&'tcx Expr<'tcx>, bool) = look_through_box(cx, closure);
    let parent: Node<'tcx> = cx.tcx.parent_hir_node(subject.hir_id);
    if let Node::LetStmt(local) = parent {
        if let Some(init) = local.init
            && init.hir_id == subject.hir_id
        {
            return Placement::LetInitializer;
        }
        return Placement::Other;
    }
    let Node::Expr(parent_expr) = parent else {
        return Placement::Other;
    };
    return match declared_slot(cx, parent_expr, subject) {
        Slot::Declared { callee, index } => {
            if parameter_requires_static(cx, callee, index, boxed) {
                Placement::KeptByCallee
            } else {
                Placement::NotKeptByCallee
            }
        }
        Slot::Unresolved => Placement::UnresolvedCallee,
        Slot::NotArgument => Placement::Other,
    };
}

/// What:
///  If `closure` is the only argument of a `Box::new(..)` call,
///  return that call and `true`;
///  else the closure and `false`.
/// Why:
///  The declared parameter type belongs to the box,
///  not to the closure inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookThroughBox(closure: Expr): [Expr, boolean];
/// ```
fn look_through_box<'tcx>(
    cx: &LateContext<'tcx>,
    closure: &'tcx Expr<'tcx>,
) -> (&'tcx Expr<'tcx>, bool) {
    let Node::Expr(parent) = cx.tcx.parent_hir_node(closure.hir_id) else {
        return (closure, false);
    };
    let ExprKind::Call(callee, arguments) = parent.kind else {
        return (closure, false);
    };
    if arguments.len() != 1 || arguments[0].hir_id != closure.hir_id {
        return (closure, false);
    }
    let Some(callee_def) = function_definition(cx, callee) else {
        return (closure, false);
    };
    if cx.tcx.is_diagnostic_item(sym::box_new, callee_def) {
        return (parent, true);
    }
    return (closure, false);
}

/// What:
///  The declared function definition a callee expression names,
///  when its type is a plain function item.
/// Why:
///  Constructors (`Some`,
///  tuple structs),
///  function pointers,
///  closures and generic callee values have no declared parameter list whose bounds can be read,
///  so they yield nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function functionDefinition(callee: Expr): DefId | undefined;
/// ```
fn function_definition<'tcx>(cx: &LateContext<'tcx>, callee: &'tcx Expr<'tcx>) -> Option<DefId> {
    let callee_type = cx.typeck_results().expr_ty(callee);
    let TyKind::FnDef(definition, _) = callee_type.kind() else {
        return None;
    };
    let kind: DefKind = cx.tcx.def_kind(*definition);
    if matches!(kind, DefKind::Fn | DefKind::AssocFn) {
        return Some(*definition);
    }
    return None;
}

/// What:
///  Find which declared parameter of `parent` receives `argument`.
/// Why:
///  A call and a method call count parameters differently:
///  a method call's receiver is parameter zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declaredSlot(parent: Expr, argument: Expr): Slot;
/// ```
fn declared_slot<'tcx>(
    cx: &LateContext<'tcx>,
    parent: &'tcx Expr<'tcx>,
    argument: &'tcx Expr<'tcx>,
) -> Slot {
    match parent.kind {
        ExprKind::Call(callee, arguments) => {
            let Some(index) = argument_index(arguments, argument) else {
                return Slot::NotArgument;
            };
            return match function_definition(cx, callee) {
                Some(definition) => Slot::Declared {
                    callee: definition,
                    index,
                },
                None => Slot::Unresolved,
            };
        }
        ExprKind::MethodCall(_, _, arguments, _) => {
            let Some(index) = argument_index(arguments, argument) else {
                return Slot::NotArgument;
            };
            return match cx.typeck_results().type_dependent_def_id(parent.hir_id) {
                Some(definition) => Slot::Declared {
                    callee: definition,
                    index: index + 1,
                },
                None => Slot::Unresolved,
            };
        }
        _ => return Slot::NotArgument,
    }
}

/// What:
///  The position of `argument` in `arguments`,
///  found by comparing node identifiers.
/// Why:
///  A loop keeps this readable and avoids a closure in the lint's own source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function argumentIndex(args: Expr[], arg: Expr): number | undefined;
/// ```
fn argument_index(arguments: &[Expr<'_>], argument: &Expr<'_>) -> Option<usize> {
    for (index, candidate) in arguments.iter().enumerate() {
        if candidate.hir_id == argument.hir_id {
            return Some(index);
        }
    }
    return None;
}
