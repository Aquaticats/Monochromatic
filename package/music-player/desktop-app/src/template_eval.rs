//! Shows a template's text for one track,
//!  or the mistakes that refuse it.
//!
//! A template that does not parse is not evaluated,
//!  so its mistakes are not
//! multiplied by follow-on errors.
//!  Every argument of a call is evaluated before
//! the call's own checks run,
//!  so a mistake in any argument is reported,
//!  except
//! that an unknown function or a wrong argument count stops before any argument is read.

/// What:
///  imports the number reader and the text functions that `tf` and `tc` call.
/// Why:
///  Each function keeps its rule in its own module.
use crate::template_format::{as_number, capitalize_first_unit, format_seconds, keep_code_points, unknown_format_letter};
/// What:
///  imports the function table,
///  the field and result types,
///  and the track trait.
/// Why:
///  Evaluation reads each of them.
use crate::template_model::{
    FunctionName, TemplateError, TemplateField, TemplateFunction, TemplateResult, TemplateTrack, template_function,
};
/// What:
///  imports the formula tree types and the template splitter.
/// Why:
///  Evaluation walks the parsed parts in order.
use crate::template_parse::{CallNode, FormulaNode, ParsedTemplate, TemplatePart, parse_template};

/// What:
///  `struct TemplateEvaluation` evaluates formula trees for one track and one allowed field set,
///  collecting every mistake.
/// Why:
///  The track,
///  the allowed fields,
///  and the mistake list travel together through every nested call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Evaluation { constructor(track, allowed, errors) {} }
/// ```
struct TemplateEvaluation<'a> {
    /// The track whose field text the template reads.
    track: &'a dyn TemplateTrack,
    /// The fields the template is allowed to read.
    allowed: &'a [TemplateField],
    /// Every mistake found so far,
    ///  in the order found.
    errors: Vec<TemplateError>,
}

/// What:
///  the evaluator methods,
///  one per node kind and function.
/// Why:
///  They share the track,
///  the allowed fields,
///  and the mistake list.
impl TemplateEvaluation<'_> {
    /// What:
    ///  `fn evaluate(&mut self, node: &FormulaNode) -> String` returns the text one node yields.
    /// Why:
    ///  Every node,
    ///  including each argument,
    ///  yields text,
    ///  so one function covers the whole tree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function evaluate(node: FormulaNode): string { ... }
    /// ```
    fn evaluate(&mut self, node: &FormulaNode) -> String {
        return match node {
            FormulaNode::Text(value) | FormulaNode::Word(value) => value.clone(),
            FormulaNode::Join(operands) => self.evaluate_join(operands),
            FormulaNode::Call(call) => self.evaluate_call(call),
        };
    }

    /// What:
    ///  `fn evaluate_join(&mut self, operands: &[FormulaNode]) -> String` concatenates the operands in order.
    /// Why:
    ///  Operands are evaluated left to right,
    ///  so mistakes come out in template order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = operands.map(evaluate).join("");
    /// ```
    fn evaluate_join(&mut self, operands: &[FormulaNode]) -> String {
        let mut output = String::new();
        for operand in operands {
            output.push_str(&self.evaluate(operand));
        }
        return output;
    }

    /// What:
    ///  `fn evaluate_call(&mut self, call: &CallNode) -> String` checks the name and arity,
    ///  then evaluates the call.
    /// Why:
    ///  Unknown names and wrong argument counts are refused before any argument is read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function evaluateCall(node: CallNode): string { ... }
    /// ```
    fn evaluate_call(&mut self, call: &CallNode) -> String {
        let Some(definition) = template_function(&call.name) else {
            self.errors.push(TemplateError {
                subject: call.name.clone(),
                message: String::from("unknown function"),
                at: call.start,
            });
            return String::new();
        };
        if call.parameters.len() < definition.minimum || call.parameters.len() > definition.maximum {
            self.errors.push(TemplateError {
                subject: call.name.clone(),
                message: arity_message(&definition),
                at: call.start,
            });
            return String::new();
        }
        let mut values = Vec::new();
        for parameter in &call.parameters {
            values.push(self.evaluate(parameter));
        }
        return match definition.name {
            FunctionName::Mi => self.evaluate_mi(call, &values),
            FunctionName::Tf => self.evaluate_tf(call, &values),
            FunctionName::Tc => self.evaluate_tc(call, &values),
        };
    }

    /// What:
    ///  `fn evaluate_mi(&mut self, call: &CallNode, values: &[String]) -> String` reads one field the template may use.
    /// Why:
    ///  A field outside the template's allowed list is refused,
    ///  even when the track has a value for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function evaluateMi(node: CallNode, values: string[]): string { ... }
    /// ```
    fn evaluate_mi(&mut self, call: &CallNode, values: &[String]) -> String {
        let mode = values.first().map_or("", String::as_str);
        if !self.allowed.iter().any(|field| return field.mode == mode) {
            self.errors.push(TemplateError {
                subject: String::from("mi"),
                message: format!("unknown field {mode}"),
                at: call.start,
            });
            return String::new();
        }
        return self.track.field_text(mode).unwrap_or_default();
    }

    /// What:
    ///  `fn evaluate_tf(&mut self, call: &CallNode, values: &[String]) -> String` formats seconds with the given format or the default `m:ss`.
    /// Why:
    ///  Missing seconds show nothing,
    ///  but text that is not a number is a mistake,
    ///  and so is a format with a letter that is not `h`, `m` or `s` outside apostrophes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function evaluateTf(node: CallNode, values: string[]): string { ... }
    /// ```
    fn evaluate_tf(&mut self, call: &CallNode, values: &[String]) -> String {
        let raw = values.first().map_or("", String::as_str);
        let format = values.get(1).map_or("m:ss", String::as_str);
        // The format is checked even when the track has no duration, so a typo in it never hides behind an empty field.
        if let Some(letter) = unknown_format_letter(format) {
            self.errors.push(TemplateError {
                subject: String::from("tf"),
                message: format!("unknown format letter {letter}"),
                at: call.start,
            });
            return String::new();
        }
        if raw.is_empty() {
            return String::new();
        }
        let Some(seconds) = as_number(raw) else {
            self.errors.push(TemplateError {
                subject: String::from("tf"),
                message: format!("needs a number of seconds, not {raw}"),
                at: call.start,
            });
            return String::new();
        };
        return format_seconds(seconds, format);
    }

    /// What:
    ///  `fn evaluate_tc(&mut self, call: &CallNode, values: &[String]) -> String` converts text by mode.
    /// Why:
    ///  Each mode has a fixed rule,
    ///  and an unknown mode is named as a mistake.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function evaluateTc(node: CallNode, values: string[]): string { ... }
    /// ```
    fn evaluate_tc(&mut self, call: &CallNode, values: &[String]) -> String {
        let mode = values.first().map_or("", String::as_str);
        let text = values.get(1).map_or("", String::as_str);
        return match mode {
            "low" => text.to_lowercase(),
            "up" => text.to_uppercase(),
            "cap" => capitalize_first_unit(text),
            "cut" => self.evaluate_cut(call, text, values.get(2)),
            _ => {
                self.errors.push(TemplateError {
                    subject: String::from("tc"),
                    message: format!("unknown mode {mode}"),
                    at: call.start,
                });
                String::new()
            }
        };
    }

    /// What:
    ///  `fn evaluate_cut(&mut self, call: &CallNode, text: &str, raw_length: Option<&String>) -> String` keeps the first characters a length allows.
    /// Why:
    ///  A cut needs a length that is a number and not negative,
    ///  and anything else is a mistake.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const length = values.length === 3 ? asNumber(values[2]) : undefined;
    /// ```
    fn evaluate_cut(&mut self, call: &CallNode, text: &str, raw_length: Option<&String>) -> String {
        let length = raw_length.and_then(|raw| return as_number(raw));
        return match length {
            Some(count) if count >= 0.0 => keep_code_points(text, count.floor()),
            _ => {
                self.errors.push(TemplateError {
                    subject: String::from("tc"),
                    message: String::from("cut needs a length, as in tc(cut, text, 10)"),
                    at: call.start,
                });
                String::new()
            }
        };
    }

    /// What:
    ///  `fn evaluate_parts(&mut self, parts: &[TemplatePart]) -> String` shows the template's pieces in order.
    /// Why:
    ///  Literal text is always shown,
    ///  and each formula contributes the text it yields.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const output = parts.map(part => part.kind === "literal" ? part.value : evaluate(part.tree)).join("");
    /// ```
    fn evaluate_parts(&mut self, parts: &[TemplatePart]) -> String {
        let mut output = String::new();
        for part in parts {
            match part {
                TemplatePart::Literal(value) => output.push_str(value),
                TemplatePart::Formula(formula) => output.push_str(&self.evaluate(&formula.tree)),
            }
        }
        return output;
    }
}

/// What:
///  `fn arity_message(definition: &TemplateFunction) -> String` writes the wrong-count mistake.
/// Why:
///  The message names the allowed count and the call shape,
///  so the user can fix the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = `takes ${range} ${unit}, as in ${signature}`;
/// ```
fn arity_message(definition: &TemplateFunction) -> String {
    let range = if definition.minimum == definition.maximum {
        definition.minimum.to_string()
    } else {
        format!("{} or {}", definition.minimum, definition.maximum)
    };
    let unit = if definition.maximum == 1 { "value" } else { "values" };
    return format!("takes {range} {unit}, as in {}", definition.signature);
}

/// What:
///  `pub fn evaluate_template(text: &str, track: &dyn TemplateTrack, fields: &[TemplateField]) -> TemplateResult` shows a template's text for one track,
///  or the mistakes that refuse it.
/// Why:
///  The shown text and the mistakes are two outcomes that never overlap,
///  and `fields` decides which fields the template may read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function evaluateTemplate(text: string, track: TemplateTrack, fields = trackFields): TemplateResult { ... }
/// ```
pub fn evaluate_template(text: &str, track: &dyn TemplateTrack, fields: &[TemplateField]) -> TemplateResult {
    return evaluate_parsed_template(&parse_template(text), track, fields);
}

/// What:
///  `pub fn evaluate_parsed_template(parsed: &ParsedTemplate, track: &dyn TemplateTrack, fields: &[TemplateField]) -> TemplateResult`
///  shows an already parsed template's text for one track, or the mistakes that refuse it.
/// Why:
///  A list of rows shows one template for every track, and parsing it again for each row cost about 2.7 microseconds a row,
///  most of a 30,000-row page's projection; `evaluate_template` parses and calls this, so both give one result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function evaluateParsedTemplate(parsed: ParsedTemplate, track: TemplateTrack, fields = trackFields): TemplateResult { ... }
/// ```
pub fn evaluate_parsed_template(parsed: &ParsedTemplate, track: &dyn TemplateTrack, fields: &[TemplateField]) -> TemplateResult {
    if !parsed.errors.is_empty() {
        return TemplateResult::Refused(parsed.errors.clone());
    }
    let mut evaluation = TemplateEvaluation {
        track,
        allowed: fields,
        errors: Vec::new(),
    };
    let output = evaluation.evaluate_parts(&parsed.parts);
    if evaluation.errors.is_empty() {
        return TemplateResult::Shown(output);
    }
    return TemplateResult::Refused(evaluation.errors);
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_eval_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_eval_tests.rs"]
mod tests;

/// What:
///  `mod mistake_tests;` pulls in the mistake group from `template_eval_mistake_tests.rs`.
/// Why:
///  A second file keeps each test file under the code-line budget.
#[cfg(test)]
#[path = "template_eval_mistake_tests.rs"]
mod mistake_tests;
