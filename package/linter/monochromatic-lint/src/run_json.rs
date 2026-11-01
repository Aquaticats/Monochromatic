//! What:
//!  Render a configuration value as strict,
//!  indented JSON.
//! Why:
//!  `--print-config` output is read by people and by tools such as `jq`.
//!  The repository's
//! JSONC emitter writes trailing commas and comments,
//!  which JSONC accepts and strict JSON parsers
//! reject;
//!  this writer keeps the same data and drops both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // JSON.stringify(value, undefined, 2)
//! ```

/// Import the JSONC value model and its string-literal encoder.
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue, encode_quoted};

/// What:
///  Append two spaces per nesting level.
/// Why:
///  Indentation makes the effective configuration readable without changing its data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indent(output: string[], depth: number): void;
/// ```
fn indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("  ");
    }
}

/// What:
///  Append one value and everything inside it.
/// Why:
///  Recursion follows the value's own nesting.
///  Printed configuration is bounded:
///  an envelope,
/// a rules record,
///  one settings object per rule,
///  and at most one array of strings inside it.
/// Text is re-encoded from its decoded units and numbers keep their exact source token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function write(value: JsoncValue, depth: number, output: string[]): void;
/// ```
fn write(value: &JsoncValue, depth: usize, output: &mut String) {
    match &value.kind {
        JsoncKind::Text { units, .. } => output.push_str(encode_quoted(units).as_str()),
        JsoncKind::Number { raw, .. } => output.push_str(raw.as_str()),
        JsoncKind::Boolean { value: flag } => {
            output.push_str(if *flag { "true" } else { "false" });
        }
        JsoncKind::Null => output.push_str("null"),
        JsoncKind::Array { elements } => {
            if elements.is_empty() {
                output.push_str("[]");
                return;
            }
            output.push_str("[\n");
            for (index, element) in elements.iter().enumerate() {
                indent(output, depth + 1);
                write(element, depth + 1, output);
                // A comma follows every element except the last; strict JSON has no trailing comma.
                if index + 1 < elements.len() {
                    output.push(',');
                }
                output.push('\n');
            }
            indent(output, depth);
            output.push(']');
        }
        JsoncKind::Record { entries } => {
            if entries.is_empty() {
                output.push_str("{}");
                return;
            }
            output.push_str("{\n");
            for (index, entry) in entries.iter().enumerate() {
                indent(output, depth + 1);
                output.push_str(encode_quoted(&entry.key.units).as_str());
                output.push_str(": ");
                write(&entry.value, depth + 1, output);
                if index + 1 < entries.len() {
                    output.push(',');
                }
                output.push('\n');
            }
            indent(output, depth);
            output.push('}');
        }
    }
}

/// What:
///  Render a value as a complete JSON document ending in one newline.
/// Why:
///  Callers print the result as-is;
///  the final newline keeps terminals and line tools tidy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function strictJson(value: JsoncValue): string { return JSON.stringify(value, undefined, 2) + '\n'; }
/// ```
pub fn strict_json(value: &JsoncValue) -> String {
    let mut output: String = String::new();
    write(value, 0, &mut output);
    output.push('\n');
    return output;
}

/// Shape,
///  escaping and strictness controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_json_tests.rs"]
mod tests;
