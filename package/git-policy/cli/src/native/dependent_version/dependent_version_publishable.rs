//! What: Read the publishable package names from the generated registry configuration
//!       (`package/config/pnpr/config.yaml`).
//! Why: Only published packages receive a dependent bump. The generator writes each name as
//!      a single-quoted YAML list item on its own line under `packages:`, so a line scan
//!      reads exactly what it wrote, as the incumbent does (`publishable-names.ts:38-81`).
//!      Lines are trimmed with JavaScript's `String.prototype.trim` set of characters.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // readPublishableNames(configText);
//! ```

/// What: The key line that opens the package list.
/// Why:  The first line that trims to exactly this starts the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PACKAGES_KEY = 'packages:';
/// ```
const PACKAGES_KEY: &str = "packages:";

/// What: The start of one single-quoted list item.
/// Why:  An item is `- '<name>'`; the name is what lies between the quotes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ITEM_PREFIX = "- '";
/// ```
const ITEM_PREFIX: &str = "- '";

/// What: Whether a character is one `String.prototype.trim` removes: ECMAScript
///       `WhiteSpace` and `LineTerminator`. `matches!` tests a value against patterns;
///       `..=` is an inclusive range.
/// Why:  Rust's `str::trim` differs: it keeps U+FEFF and removes U+0085.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTrimmed = (c: string) => c.trim() === '';
/// ```
fn is_js_whitespace(character: char) -> bool {
    return matches!(
        character,
        '\u{9}'..='\u{d}'
            | ' '
            | '\u{a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    );
}

/// What: The list item's name, or nothing when the line is not an item.
///       `.strip_prefix`/`.strip_suffix` remove a known start or end.
/// Why:  The first line that is not a non-empty quoted item ends the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isItem = trimmed.startsWith("- '") && trimmed.endsWith("'") && trimmed.length > 4;
/// ```
fn item_name(trimmed: &str) -> Option<&str> {
    return trimmed
        .strip_prefix(ITEM_PREFIX)
        .and_then(|rest: &str| return rest.strip_suffix('\''))
        .filter(|name: &&str| return !name.is_empty());
}

/// What: The names in the `packages:` list, in configuration order; nothing when the list
///       is absent. `.split('\n')` keeps a carriage return on each line, which the trim
///       removes.
/// Why:  The generator's output is the publish set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPublishableNames(configText: string): string[];
/// ```
pub fn read_publishable_names(config_text: &str) -> Vec<String> {
    let mut lines = config_text
        .split('\n')
        .map(|line: &str| return line.trim_matches(is_js_whitespace));
    // `.any(...)` consumes lines up to and including the key line.
    if !lines.any(|line: &str| return line == PACKAGES_KEY) {
        return Vec::new();
    }
    return lines.map_while(item_name).map(String::from).collect();
}

/// Publishable-name reading, including every case of the incumbent's unit tests.
#[cfg(test)]
#[path = "dependent_version_publishable_tests.rs"]
mod tests;
