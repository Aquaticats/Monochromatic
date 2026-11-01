//! The typing help:
//!  the innermost call around a caret,
//!  and the argument the caret is in.
//!
//! The search walks the formula trees with a work stack,
//!  so its depth is set by the
//! call nesting bound and its breadth by the template's own length.

/// What:
///  imports the function table and the field helpers.
/// Why:
///  Help reads the same table as evaluation.
use crate::template_model::{FunctionName, TemplateField, field_modes, template_function};
/// What:
///  imports the formula tree,
///  the parts,
///  and the caret rule.
/// Why:
///  Help walks the same tree that evaluation walks.
use crate::template_parse::{CallNode, FormulaNode, TemplatePart, formula_holds_caret, parse_template};

/// What:
///  `pub struct TemplateHelp` is the signature and argument the caret is inside.
/// Why:
///  The editor shows the call's shape and highlights the argument the caret is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplateHelp = { name: string; signature: string; parameter: string; description: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateHelp {
    /// The function name as written.
    pub name: String,
    /// The call shape,
    ///  such as `tf(seconds, [format])`.
    pub signature: &'static str,
    /// The argument name the caret is in.
    pub parameter: &'static str,
    /// The argument's help sentence for this template.
    pub description: String,
}

/// What:
///  `fn formula_tree_at(part: &TemplatePart, caret: usize) -> Option<&FormulaNode>` returns the tree of a formula part that holds the caret.
/// Why:
///  The work stack starts from exactly the trees of the formulas around the caret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending = parts.filter(p => p.kind === "formula" && inside(p)).map(p => p.tree);
/// ```
fn formula_tree_at(part: &TemplatePart, caret: usize) -> Option<&FormulaNode> {
    return match part {
        TemplatePart::Formula(formula) if formula_holds_caret(formula, caret) => Some(&formula.tree),
        _ => None,
    };
}

/// What:
///  `fn call_contains(call: &CallNode, caret: usize) -> bool` tells whether the caret lies inside a call's brackets.
/// Why:
///  A call with no closing bracket counts as open to the end of its formula.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inside = caret >= node.open && (node.close === undefined || caret <= node.close);
/// ```
fn call_contains(call: &CallNode, caret: usize) -> bool {
    return caret >= call.open && call.close.is_none_or(|close| return caret <= close);
}

/// What:
///  `fn innermost_call(parts: &[TemplatePart], caret: usize) -> Option<&CallNode>` finds the call around the caret that starts last.
/// Why:
///  Nested calls start after their parents,
///  so the one that starts last is the innermost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function innermostCall(parts: TemplatePart[], caret: number): CallNode | undefined { ... }
/// ```
fn innermost_call(parts: &[TemplatePart], caret: usize) -> Option<&CallNode> {
    let mut pending: Vec<&FormulaNode> = parts.iter().filter_map(|part| return formula_tree_at(part, caret)).collect();
    let mut found: Option<&CallNode> = None;
    while let Some(node) = pending.pop() {
        match node {
            FormulaNode::Join(operands) => pending.extend(operands.iter()),
            FormulaNode::Call(call) => {
                pending.extend(call.parameters.iter());
                if call_contains(call, caret) && found.is_none_or(|current| return call.open > current.open) {
                    found = Some(call);
                }
            }
            FormulaNode::Text(_) | FormulaNode::Word(_) => {}
        }
    }
    return found;
}

/// What:
///  `fn help_for_call(call: &CallNode, caret: usize, fields: &[TemplateField]) -> Option<TemplateHelp>` builds the help for one call and the argument the caret is in.
/// Why:
///  The argument is the number of commas before the caret,
///  capped at the function's last argument,
///  and `mi` lists this template's fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function helpForCall(call: CallNode, caret: number, fields: Field[]): Help | undefined { ... }
/// ```
fn help_for_call(call: &CallNode, caret: usize, fields: &[TemplateField]) -> Option<TemplateHelp> {
    let definition = template_function(&call.name)?;
    let commas_before = call.separators.iter().filter(|separator| return **separator < caret).count();
    let index = commas_before.min(definition.parameters.len().saturating_sub(1));
    let parameter = definition.parameters.get(index)?;
    let description = if definition.name == FunctionName::Mi {
        format!("Field: one of {}.", field_modes(fields))
    } else {
        parameter.description.clone()
    };
    return Some(TemplateHelp {
        name: call.name.clone(),
        signature: definition.signature,
        parameter: parameter.name,
        description,
    });
}

/// What:
///  `pub fn help_at(text: &str, caret: usize, fields: &[TemplateField]) -> Option<TemplateHelp>` finds the innermost call around the caret and the argument under it.
/// Why:
///  A caret outside every call has no signature to show,
///  and an unknown call name has no help either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function helpAt(text: string, caret: number, fields = trackFields): Help | undefined { ... }
/// ```
pub fn help_at(text: &str, caret: usize, fields: &[TemplateField]) -> Option<TemplateHelp> {
    let parsed = parse_template(text);
    let innermost = innermost_call(&parsed.parts, caret)?;
    return help_for_call(innermost, caret, fields);
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_help_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_help_tests.rs"]
mod tests;
