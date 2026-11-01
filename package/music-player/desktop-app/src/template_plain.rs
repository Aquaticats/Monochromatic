//! Plain sentences for the template editor's mistake lines.
//!
//! The parser and the evaluator record each mistake as a short developer note (`mi: unknown field nope`), and the tests
//! pin those notes. The editor shows something else: one sentence per problem, with the name of the call and what to
//! do. This module is that translation, so the engine keeps its stable notes and the page keeps its wording.

/// What:
///  imports the mistake record.
/// Why:
///  Each sentence is written from one recorded mistake.
use crate::template_model::TemplateError;
/// What:
///  imports the template splitter and its part type.
/// Why:
///  A half-written formula records several follow-on mistakes, and grouping them needs each formula's span.
use crate::template_parse::{TemplatePart, parse_template};

/// What:
///  `pub fn plain_lines(text: &str, errors: &[TemplateError]) -> Vec<String>` writes one plain sentence per problem,
///  in template order.
/// Why:
///  A formula that is not finished, such as `$mi(`, makes the parser record three mistakes that are one problem, so
///  the editor keeps the first mistake of each formula that does not parse. Formulas that parse but misuse a call
///  keep every mistake, because each is its own fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plainLines = (text: string, errors: TemplateError[]) => collapse(sortBy(errors, "at"), formulaSpans(text)).map(sentence);
/// ```
pub fn plain_lines(text: &str, errors: &[TemplateError]) -> Vec<String> {
    let mut ordered: Vec<&TemplateError> = errors.iter().collect();
    ordered.sort_by_key(|error| return error.at);
    let parsed = parse_template(text);
    if parsed.errors.is_empty() {
        return ordered.iter().map(|error| return plain_sentence(error)).collect();
    }
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for part in &parsed.parts {
        if let TemplatePart::Formula(formula) = part {
            spans.push((formula.start, formula.end));
        }
    }
    let mut seen: Vec<usize> = Vec::new();
    let mut lines = Vec::new();
    for error in ordered {
        let owner = spans.iter().position(|(start, end)| return *start <= error.at && error.at <= *end);
        if let Some(index) = owner {
            if seen.contains(&index) {
                continue;
            }
            seen.push(index);
        }
        lines.push(plain_sentence(error));
    }
    return lines;
}

/// What:
///  `fn plain_sentence(error: &TemplateError) -> String` writes one mistake as a sentence that names the call and says what to do.
/// Why:
///  The recorded note says what the engine found; the user needs to know what to change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = (e: TemplateError) => table[kindOf(e)](e);
/// ```
fn plain_sentence(error: &TemplateError) -> String {
    let subject = error.subject.as_str();
    let message = error.message.as_str();
    if let Some(field) = message.strip_prefix("unknown field ") {
        return format!("{subject} has no field called {field}. Pick a name from the Fields list.");
    }
    if let Some(letter) = message.strip_prefix("unknown format letter ") {
        return format!(
            "{subject} does not know the format letter {letter}. Use h, m or s, and put any other words between apostrophes."
        );
    }
    if let Some(mode) = message.strip_prefix("unknown mode ") {
        return format!("{subject} has no mode called {mode}. Use low, up, cap or cut.");
    }
    if let Some(given) = message.strip_prefix("needs a number of seconds, not ") {
        return format!("{subject} needs a number of seconds, not {given}. Use mi(len) for the duration.");
    }
    if let Some(shape) = message.strip_prefix("takes ") {
        return format!("{subject} takes {shape}.");
    }
    if let Some(token) = message.strip_prefix("unexpected ") {
        return format!("Unexpected {token} in a formula. Join values with + or put each one in its own pair of $ signs.");
    }
    if let Some(next) = message.strip_prefix("a value is missing before ") {
        return format!("A value is missing before {next} in a formula. Put a field such as mi(title), or some text, there.");
    }
    if let Some(position) = message.strip_prefix("the $ at character ") {
        let number = position.split(' ').next().unwrap_or(position);
        return format!("The $ at character {number} has no closing $. Add a $ where the formula ends.");
    }
    return match message {
        "unknown function" => format!("{subject} is not a function. Use mi, tf or tc."),
        "a closing bracket is missing" => format!("{subject} is missing its closing bracket. Add a ) after its last value."),
        "calls are nested too deeply" => format!("{subject} sits inside too many calls. Take some of the calls out."),
        "a value is missing" => String::from("A formula is missing a value. Put a field such as mi(title), or some text, in it."),
        "a quotation mark is not closed" => String::from("A quotation mark is not closed. Add a closing quotation mark."),
        "cut needs a length, as in tc(cut, text, 10)" => String::from("tc needs a length to cut to, as in tc(cut, text, 10)."),
        other => format!("{subject}: {other}."),
    };
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_plain_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_plain_tests.rs"]
mod tests;
