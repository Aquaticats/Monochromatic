// What:     Property checks and a structured generator for the rules-list grammar
//           (`forbidden_strings::parse_rules_list`).
// Why:      The `fuzz_rules_list` target and the unit tests share one oracle, so a property the fuzzer
//           enforces is also executed, with positive and negative controls, by `cargo nextest run --lib`.
//
// In TS you'd write (pseudocode):
// ```ts
// export function checkRulesListValue(value: string): void { /* compare parseRulesList with a reference */ }
// ```

use forbidden_strings::{parse_rules_list, RulesEntry, RulesListError};

// Pieces a generated list is assembled from: the separator, the optional marker, doubled markers, empty entries,
// lookalike delimiters, whitespace, percent-escapes (which carry no meaning here), non-ASCII commas, and paths.
const TOKENS: &[&str] = &[
    ",", "?", "??", ",,", ",?", "a", "b.txt", "x/y.txt", " ", "%2C", "%3F", "\\", "\n", "\u{ff0c}", "..", "a?b", "-",
    "C:/rules", ";", "\u{fe0f}",
];

// Builds a rules-list value from token indices, so the fuzzer reaches every grammar branch with short inputs.
pub fn generated_rules_list(data: &[u8]) -> String {
    let mut value = String::new();
    for byte in data.iter().take(48) {
        value.push_str(TOKENS[usize::from(*byte) % TOKENS.len()]);
    }
    return value
}

// An independent reference parse: scans characters once instead of splitting, so it shares no code path with
// the implementation under test. Positions are 1-based like the implementation's.
pub fn reference_parse(value: &str) -> Result<Vec<(String, bool)>, RulesListError> {
    if value.is_empty() {
        return Err(RulesListError::Empty);
    }
    let mut segments: Vec<String> = vec![String::new()];
    for character in value.chars() {
        if character == ',' {
            segments.push(String::new());
        } else if let Some(last) = segments.last_mut() {
            last.push(character);
        }
    }
    let mut parsed: Vec<(String, bool)> = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        let position = index + 1;
        let (name, optional) = match segment.strip_suffix('?') {
            Some(without) => (without.to_string(), true),
            None => (segment.clone(), false),
        };
        if name.is_empty() {
            return Err(RulesListError::EmptyEntry { position });
        }
        if let Some(first) = parsed.iter().position(|(earlier, _)| return *earlier == name) {
            return Err(RulesListError::DuplicatePath { first: first + 1, position });
        }
        parsed.push((name, optional));
    }
    return Ok(parsed)
}

// Renders entries back to a list value, the inverse of a successful parse.
pub fn render(entries: &[RulesEntry]) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut part = entry.path.to_str().expect("a parsed path comes from UTF-8 text").to_string();
        if entry.optional {
            part.push('?');
        }
        parts.push(part);
    }
    return parts.join(",")
}

// Asserts every grammar property for one value; panics on any violation.
pub fn check_rules_list_value(value: &str) {
    let actual = parse_rules_list(value);
    let reference = reference_parse(value);
    match (&actual, &reference) {
        (Ok(entries), Ok(expected)) => {
            let seen: Vec<(String, bool)> = entries
                .iter()
                .map(|entry| return (entry.path.to_str().expect("UTF-8 path").to_string(), entry.optional))
                .collect();
            assert_eq!(&seen, expected, "entries differ from the reference for {value:?}");
            // The parse is lossless: rendering the entries gives the value back byte for byte.
            assert_eq!(render(entries), value, "render did not invert the parse for {value:?}");
            assert_eq!(entries.len(), value.matches(',').count() + 1, "{value:?}");
        }
        (Err(found), Err(expected)) => assert_eq!(found, expected, "refusal differs for {value:?}"),
        _ => panic!("parse and reference disagree on acceptance for {value:?}: {actual:?} against {reference:?}"),
    }
    // A value with no comma and no trailing marker is one required entry equal to the whole value.
    if !value.is_empty() && !value.contains(',') && !value.ends_with('?') {
        let entries = actual.expect("a plain path parses");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path.to_str(), Some(value));
        assert!(!entries[0].optional);
    }
}

// Registers the oracle's controls (sidecar, lint-exempt).
#[cfg(test)]
#[path = "rules_list_tests.rs"]
mod tests;
