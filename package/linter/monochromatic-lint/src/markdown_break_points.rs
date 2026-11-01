//! What:
//!  Byte offsets of missing prose line breaks,
//!  with token and abbreviation exclusions.
//! Why:
//!  Declining ambiguous breaks preserves content;
//!  inserted breaks must never split names or numbers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Scan prose once, checking only bounded abbreviation suffixes and the following whitespace run.
//! ```

/// Accepted abbreviation catalog from the incumbent,
///  not a language-dependent sentence heuristic.
const ABBREVIATIONS: &[&str] = &[
    "e.g.", "i.e.", "etc.", "vs.", "cf.", "al.", "dr.", "mr.", "mrs.", "ms.", "st.", "no.", "fig.",
    "eq.", "approx.", "a.m.", "p.m.", "u.s.", "u.k.", "ph.d.",
];

/// Recognize an abbreviation ending at this period without lowercasing and shifting source offsets.
fn abbreviation_at(source: &str, end: usize) -> bool {
    for abbreviation in ABBREVIATIONS {
        // Checked subtraction and slicing reject short or non-character-aligned candidates without guessing offsets.
        let Some(start): Option<usize> = end.checked_sub(abbreviation.len()) else {
            continue;
        };
        let Some(candidate): Option<&str> = source.get(start..end) else {
            continue;
        };
        if !candidate.eq_ignore_ascii_case(abbreviation) {
            continue;
        }
        // The incumbent tests the final lowercased character before the match for an ASCII letter.
        if let Some(previous) = source[..start].chars().next_back()
            && let Some(lower) = previous.to_lowercase().last()
            && lower.is_ascii_lowercase()
        {
            continue;
        }
        return true;
    }
    return false;
}

/// Closing prose delimiters move the break past the completed quote or bracket.
fn closing(character: char) -> bool {
    return ['"', '\'', ')', ']', '}', '”', '’', '»', '›'].contains(&character);
}

/// Only written word boundaries permit breaks;
///  a glued letter/digit/delimiter keeps the token intact.
fn separator(character: Option<char>) -> bool {
    let Some(value) = character else {
        return true;
    };
    // A newline already satisfies the rule, so only horizontal spacing can lead to an inserted break.
    return value == ' ' || value == '\t';
}

/// Find content before any existing line ending across the node tail and its following paragraph source.
fn needs_break(slice_tail: &str, trailing: &str, paragraph_tail: bool) -> bool {
    // Borrow both stretches; neither creates a copied whitespace lookahead or a fixed-size cutoff.
    for text in [slice_tail, trailing] {
        for byte in text.bytes() {
            if byte == b' ' || byte == b'\t' {
                continue;
            }
            if byte == b'\n' || byte == b'\r' {
                return false;
            }
            return true;
        }
    }
    // Only paragraph-final punctuation is exempt after both stretches are exhausted.
    return !paragraph_tail;
}

/// What:
///  Locate insertion offsets within one authored text-node slice.
/// Why:
///  Byte addressing survives Unicode before abbreviations and punctuation;
///  checks never rebuild the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function breakOffsets(slice, trailingSource, paragraphTail): number[];
/// ```
pub(crate) fn break_offsets(slice: &str, trailing: &str, paragraph_tail: bool) -> Vec<usize> {
    // Own a variable-length result list; usize indexes source bytes rather than UTF-16 columns.
    let mut offsets: Vec<usize> = Vec::<usize>::new();
    for (index, character) in slice.char_indices() {
        if ![',', '.', ';', ':', '?', '!'].contains(&character) {
            continue;
        }
        // All break points are ASCII, so their next byte is also a character boundary.
        // The standard trim passes every closing delimiter by calling the named test; no offset is stepped by hand.
        let tail: &str = slice[index + 1..].trim_start_matches(closing);
        let after: usize = slice.len() - tail.len();
        // The next written character may lie past this text node, so the paragraph's following source continues the tail.
        let following: Option<char> = tail.chars().chain(trailing.chars()).next();
        if !separator(following) || !needs_break(tail, trailing, paragraph_tail) {
            continue;
        }
        if character == '.' {
            // A preceding dot marks the final dot of an ellipsis; abbreviation patterns have bounded length.
            // A node that begins with its break point has an empty prefix, which simply ends with nothing.
            if slice[..index].ends_with('.') {
                continue;
            }
            if abbreviation_at(slice, index + 1) {
                continue;
            }
        }
        offsets.push(after);
    }
    return offsets;
}

/// Lexical controls remain separate from the AST-driven rule.
#[cfg(test)]
#[path = "markdown_break_points_tests.rs"]
mod tests;
