//! Loads the real forbidden-strings ruleset and ports each rule for both engines.
//!
//! What:
//!  reads the scanner's embedded baseline constant and the shared appendix,
//! extracts the `/PATTERN/FLAGS` regex rules,
//!  and ports each into an `(ours, bare)`
//! pair.
//!  Why:
//!  a realistic benchmark must run the credential shapes the scanner
//! actually ships,
//!  not toy patterns.

/// Imports the per-rule porter.
use crate::port::port;

/// What:
///      `use forbidden_strings::BUILTIN_RULES;` imports the scanner crate's
///           exported baseline constant:
///  the betterleaks port plus set-algebra
///           demonstrations,
///  embedded in that crate at compile time.
/// Why:
///       The bench replays the exact ruleset the scanner ships;
///  importing the
///           constant replaces an `include_str!` with a fragile `../../../../`
///           path into the repository root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BUILTIN_RULES } from '@monochromatic-dev/forbidden-strings';
/// ```
use forbidden_strings::BUILTIN_RULES;

/// The shared appendix of shortcode-label rules.
const APPEND: &str = include_str!("../../../../forbidden-strings.append.txt");

/// Loads every regex rule,
///  ported into `(ours, bare)` pairs.
///
/// What:
///  walks both rule sources,
///  keeps the `/.../` regex lines,
///  and ports each.
/// Why:
///  the caller compile-filters these to the subset both engines accept.
pub fn load_rules() -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for source in [BUILTIN_RULES, APPEND] {
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(inner) = extract_pattern(trimmed) {
                pairs.push(port(inner));
            }
        }
    }
    return pairs
}

/// Extracts the pattern body of a `/PATTERN/FLAGS` rule line.
///
/// What:
///  returns the text between the opening `/` and the first unescaped `/` that
/// is not inside a class;
///  `None` for non-regex lines.
///  Why:
///  rule bodies contain `/`
/// inside classes and as `\/`,
///  so the closing delimiter must be found carefully.
fn extract_pattern(line: &str) -> Option<&str> {
    let b = line.as_bytes();
    if b.first() != Some(&b'/') {
        return None;
    }
    let mut i = 1;
    let mut in_class = false;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' {
            i += 2;
            continue;
        }
        if in_class {
            if c == b']' {
                in_class = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'[' => in_class = true,
            b'/' => return Some(&line[1..i]),
            _ => {}
        }
        i += 1;
    }
    return None
}
