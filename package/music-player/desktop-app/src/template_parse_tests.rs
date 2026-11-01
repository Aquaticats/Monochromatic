// Unit tests for `template_parse.rs`, pulled in by `#[cfg(test)] #[path = "template_parse_tests.rs"] mod tests;`.
//
// The parse-time mistakes checked here are the lines the reference refuses a template with
// before any evaluation happens.

use super::*;
use crate::template_model::error_lines;

// What: `fn parse_lines(text: &str) -> Vec<String>` returns the parse-time mistake lines of a template.
// Why: Each refusal case compares its exact lines, with the same wording the reference gives.
//
// In TS you'd write (pseudocode):
// ```ts
// const lines = errorLines(parseTemplate(text).errors);
// ```
fn parse_lines(text: &str) -> Vec<String> {
    return error_lines(&parse_template(text).errors);
}

// What: `fn parse_keeps_where_each_formula_sits(...)` checks the start, end, and closed flag of each piece.
// Why: Help and error positions depend on these UTF-16 offsets, so they are checked directly.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(parseTemplate("a$mi(ext)$b$mi(").parts.map(describe)).toEqual([...]);
// ```
#[test]
fn parse_keeps_where_each_formula_sits() {
    let parsed = parse_template("a$mi(ext)$b$mi(");
    let described: Vec<String> = parsed
        .parts
        .iter()
        .map(|part| {
            return match part {
                TemplatePart::Literal(value) => format!("literal:{value}"),
                TemplatePart::Formula(formula) => format!(
                    "formula:{}-{}{}",
                    formula.start,
                    formula.end,
                    if formula.closed { "" } else { " open" }
                ),
            };
        })
        .collect();
    assert_eq!(described, vec!["literal:a", "formula:1-10", "literal:b", "formula:11-15 open"]);
    assert!(parsed.errors.iter().any(|error| return error.message.contains("no closing")));
}

// What: `fn parse_reports_an_unclosed_dollar_sign(...)` checks the mistake for a formula with no closing dollar sign.
// Why: A formula left open is reported, and its text is never shown as literal text.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$mi(title)")).toEqual(["formula: the $ at character 1 has no closing $"]);
// ```
#[test]
fn parse_reports_an_unclosed_dollar_sign() {
    assert_eq!(parse_lines("$mi(title)"), vec!["formula: the $ at character 1 has no closing $"]);
    assert_eq!(
        parse_lines("a $mi(title) - text"),
        vec!["formula: the $ at character 3 has no closing $", "formula: unexpected -"]
    );
}

// What: `fn parse_reports_missing_and_misplaced_values(...)` checks empty formulas, leading operators, and missing separators.
// Why: Each mistake names what is missing or misplaced, and the order of the lines follows the template.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$+$")).toEqual(["formula: a value is missing before +", "formula: a value is missing"]);
// ```
#[test]
fn parse_reports_missing_and_misplaced_values() {
    assert_eq!(parse_lines("$$"), vec!["formula: a value is missing"]);
    assert_eq!(
        parse_lines("$+$"),
        vec!["formula: a value is missing before +", "formula: a value is missing"]
    );
    assert_eq!(parse_lines("$mi(title) mi(ext)$"), vec!["formula: unexpected mi"]);
    assert_eq!(
        parse_lines("$mi(,)$"),
        vec!["formula: a value is missing before ,", "formula: a value is missing before )"]
    );
    assert_eq!(parse_lines("$mi(len) = 275$"), vec!["formula: unexpected ="]);
    assert_eq!(parse_lines("$mi(len) > 1$"), vec!["formula: unexpected >"]);
    assert_eq!(parse_lines("$mi(peak) != \"\"$"), vec!["formula: unexpected !="]);
}

// What: `fn parse_reports_brackets_and_quotation_marks(...)` checks a missing closing bracket and an unclosed quotation mark.
// Why: Both mistakes name the call they sit in, and the quotation mark's mistake comes after the bracket's in template order.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines('$mi("title)$')).toEqual(["mi: a closing bracket is missing", "text: a quotation mark is not closed"]);
// ```
#[test]
fn parse_reports_brackets_and_quotation_marks() {
    assert_eq!(parse_lines("$mi(title$"), vec!["mi: a closing bracket is missing"]);
    assert_eq!(
        parse_lines("$mi(\"title)$"),
        vec!["mi: a closing bracket is missing", "text: a quotation mark is not closed"]
    );
}

// What: `fn parse_stops_reading_calls_past_the_nesting_bound(...)` checks the mistake for thirteen nested calls.
// Why: Past the bound the parser stops reading, and the one mistake covers the whole formula.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$" + "tc(up, ".repeat(13) + "x" + ")".repeat(13) + "$")).toEqual(["tc: calls are nested too deeply"]);
// ```
#[test]
fn parse_stops_reading_calls_past_the_nesting_bound() {
    let deep = format!("${}x{}$", "tc(up, ".repeat(13), ")".repeat(13));
    assert_eq!(parse_lines(&deep), vec!["tc: calls are nested too deeply"]);
    let deepest = format!("${}x{}$", "tc(up, ".repeat(12), ")".repeat(12));
    assert!(parse_lines(&deepest).is_empty());
}

// What: `fn parse_reports_unknown_words_as_tokens_are_not_checked(...)` checks that a call's name and text are read, not judged.
// Why: The parser reads any word as a call name; unknown names are reported later by evaluation.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(lines("$xx(1)$")).toEqual([]);
// ```
#[test]
fn parse_reports_unknown_words_as_tokens_are_not_checked() {
    assert!(parse_lines("$xx(1)$ and $mi(nope)$").is_empty());
    assert!(parse_lines("$\"<script>\" + mi(ext)$").is_empty());
}

// What: `fn parse_joins_keep_operands_in_order(...)` checks that a long join is one flat list of operands in order.
// Why: A flat list keeps the tree from nesting once per `+`, so a long join cannot exhaust the stack.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(parseTemplate('$"a" + "b" + mi(ext)$').parts[0].tree).toEqual(join(["a", "b", ...]));
// ```
#[test]
fn parse_joins_keep_operands_in_order() {
    let parsed = parse_template("$\"a\" + \"b\" + mi(ext)$");
    let operands = match parsed.parts.first() {
        Some(TemplatePart::Formula(formula)) => match &formula.tree {
            FormulaNode::Join(items) => items.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    assert_eq!(operands.len(), 3);
    assert_eq!(operands[0], FormulaNode::Text(String::from("a")));
    assert_eq!(operands[1], FormulaNode::Text(String::from("b")));
    assert!(matches!(&operands[2], FormulaNode::Call(call) if call.name == "mi"));
}
