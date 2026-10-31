//! What:
//!  Evaluate the config's value expressions against the virtual view.
//! Why:
//!  `{"file": ...}` reads through the view,
//! so a rule sees what earlier rules planned;
//! `{"var": ...}` reads a fan-out
//!  variable;
//! `{"concat": [...]}` joins text.
//! Nothing else exists,
//! so every input is visible in the config.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function evaluate(view: View, expr: ValueExpr, vars: Record<string, string>): Evaluated
//! ```

/// Imports `is_repo_relative` from `crate::config_decode`.
use crate::config_decode::is_repo_relative;
/// Imports `ValueExpr` from `crate::config_parts`.
use crate::config_parts::ValueExpr;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;

/// What:
///  The variables a fan-out rule binds for its value expressions.
/// Why:
///  A per-manifest rule binds `dir` (the crate directory) and `manifest` (the manifest path).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Vars = Record<string, string>;
/// ```
pub type Vars = BTreeMap<String, String>;

/// What:
///  A computed value:
/// text,
/// boolean,
/// integer or a list of those.
/// Why:
///  TOML edits need typed values;
/// everything else needs text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Evaluated = string | boolean | number | Evaluated[];
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Evaluated {
    /// Text.
    Text(
        /// The wrapped value.
        String,
    ),
    /// Boolean.
    Bool(
        /// The wrapped value.
        bool,
    ),
    /// Integer.
    Integer(
        /// The wrapped value.
        i64,
    ),
    /// List.
    List(
        /// The wrapped value.
        Vec<Evaluated>,
    ),
}

/// What:
///  Evaluate one expression.
/// Why:
///  The single interpreter for the config's value language.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function evaluate(view: View, expr: ValueExpr, vars: Vars): Evaluated
/// ```
pub fn evaluate(view: &mut View, expr: &ValueExpr, vars: &Vars) -> Result<Evaluated, Diagnostic> {
    match expr {
        ValueExpr::Text(text) => return Ok(Evaluated::Text(text.clone())),
        ValueExpr::Bool(flag) => return Ok(Evaluated::Bool(*flag)),
        ValueExpr::Integer(number) => return Ok(Evaluated::Integer(*number)),
        ValueExpr::List(items) => {
            let mut values: Vec<Evaluated> = Vec::new();
            for item in items {
                values.push(evaluate(view, item, vars)?);
            }
            return Ok(Evaluated::List(values));
        }
        ValueExpr::Var { var } => match vars.get(var) {
            Some(value) => return Ok(Evaluated::Text(value.clone())),
            None => {
                let message = format!("the variable {var:?} is not bound here; this rule binds {:?}", vars.keys());
                return Err(Diagnostic::new("variable-unbound", &message));
            }
        },
        ValueExpr::Concat { concat } => {
            let mut joined = String::new();
            for part in concat {
                joined.push_str(&evaluate_text(view, part, vars)?);
            }
            return Ok(Evaluated::Text(joined));
        }
        ValueExpr::File { file } => {
            let path = evaluate_text(view, file, vars)?;
            if !is_repo_relative(&path) {
                let message = format!("{path:?} is not a relative path inside the worktree");
                return Err(Diagnostic::new("path-invalid", &message));
            }
            match view.read_text(&path)? {
                Some(text) => return Ok(Evaluated::Text(text)),
                None => {
                    let message = format!("the input file {path} does not exist");
                    return Err(Diagnostic::new("input-missing", &message).with_path(&path));
                }
            }
        }
    }
}

/// What:
///  Evaluate an expression that must produce text.
/// Why:
///  Paths,
/// file parts and block text are all text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function evaluateText(view: View, expr: ValueExpr, vars: Vars): string
/// ```
pub fn evaluate_text(view: &mut View, expr: &ValueExpr, vars: &Vars) -> Result<String, Diagnostic> {
    match evaluate(view, expr, vars)? {
        Evaluated::Text(text) => return Ok(text),
        other => {
            let message = format!("expected text but the expression produced {other:?}");
            return Err(Diagnostic::new("value-type", &message));
        }
    }
}
