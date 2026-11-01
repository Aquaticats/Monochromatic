//! What:
//!  The variables a closure captures,
//!  as rustc's capture analysis settled them.
//! Why:
//!  A closure that captures nothing can always be a named function,
//!  so the first half of the lint criterion is a question about this list.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function capturedNames(closure: ClosureExpr): string[];
//! ```

/// Import the local definition identifier type of a closure.
use rustc_hir::def_id::LocalDefId;
/// Import the lint context that gives access to type-check results.
use rustc_lint::LateContext;

/// What:
///  Names of the variables the closure captures,
///  in first-capture order,
///  without repeats.
///  A variable used only through `let _ = n`,
///  or only in a way edition 2021 precise capture ignores,
///  is not listed.
/// Why:
///  `tcx.upvars_mentioned` lists every variable the closure's text mentions,
///  which includes `let _ = n`;
///  the minimal captures are what the closure really owns or borrows.
/// Gotcha:
///  Before edition 2021 the whole variable is captured even by `let _ = n`,
///  so the list differs by edition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capturedNames(closure: ClosureExpr): string[] {
///   const names: string[] = [];
///   for (const capture of minimalCaptures(closure)) if (!names.includes(capture.variable)) names.push(capture.variable);
///   return names;
/// }
/// ```
pub fn captured_names(cx: &LateContext<'_>, closure: LocalDefId) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for captured in cx.typeck_results().closure_min_captures_flattened(closure) {
        let name: String = captured.var_ident.name.to_string();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    return names;
}
