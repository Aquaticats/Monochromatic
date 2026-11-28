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

/// Import the parsed document,
///  because the position a break lands at is the tree's answer and not a run of characters.
use crate::markdown_prose_context::delimiter_tail;
/// Import the native source interface that owns the parse and its byte offsets.
use crate::markdown_source::MarkdownSource;

/// Accepted abbreviation catalog from the incumbent,
///  not a language-dependent sentence heuristic.
const ABBREVIATIONS: &[&str] = &[
    "e.g.", "i.e.", "etc.", "vs.", "cf.", "al.", "dr.", "mr.", "mrs.", "ms.", "st.", "no.", "fig.",
    "eq.", "approx.", "a.m.", "p.m.", "u.s.", "u.k.", "ph.d.",
];

/// Recognize an abbreviation ending at this period without lowercasing and shifting source offsets.
fn abbreviation_at(
    source: &str,
    end: usize,
) -> bool {
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

/// The punctuation characters this rule breaks after,
///  which is the same list a fragment join must leave ending its line.
fn is_break_point(character: char) -> bool {
    return [',', '.', ';', ':', '?', '!'].contains(&character);
}

/// What:
///  The last break-point character a stretch of prose holds,
///  as its byte offset in the document.
/// Why:
///  The rule breaks after the last break point of a line,
///  so that one character decides whether a fragment the join would move
///  lands where the rule wants a line to end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastBreakPoint(source: string, start: number, end: number): number | undefined;
/// ```
fn last_break_point(
    source: &str,
    start: usize,
    end: usize,
) -> Option<usize> {
    // `Option<usize>` is `Some(offset)` for the last break point seen so far, like `number | undefined`.
    let mut found: Option<usize> = None;
    for (index, character) in source[start..end].char_indices() {
        if is_break_point(character) {
            // `Some(...)` constructs the present variant of `Option`, so the caller can tell "no break point" from byte zero.
            found = Some(start + index);
        }
    }
    return found;
}

/// What:
///  The byte position the rule's own break reaches for the break point at `at`,
///  or nothing when no text node of the paragraph holds that character.
/// Why:
///  The rule moves its break past the closing brackets and quotes its own text node ends at,
///  and past an emphasis, a strong emphasis or a strikethrough that ends there;
///  only the parsed tree says which of those characters closes something,
///  so `*held,*` completes a break past its `*` and `held,*` breaks straight after the `,`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function breakPosition(context: MarkdownSource, paragraph: number, at: number): number | undefined;
/// ```
fn break_position(
    context: &MarkdownSource,
    paragraph: u32,
    at: usize,
) -> Option<usize> {
    // Every text node below the paragraph, where the rule reads break points from.
    let nodes: Vec<u32> = match context.text_nodes(paragraph) {
        Ok(found) => found,
        // A child-index cycle is a defect of this crate; the rule reports the document's structure itself.
        Err(_) => return None,
    };
    // `Option<u32>` is `Some(id)` for the text node that covers the break point and `None` before one is found.
    let mut holder: Option<u32> = None;
    for id in nodes {
        let (start, end): (usize, usize) = context.offsets(id);
        if start <= at && at < end {
            holder = Some(id);
            break;
        }
    }
    // `?` hands the absence to the caller at once: no text node holds a break point there.
    let node: u32 = holder?;
    let (node_start, node_end): (usize, usize) = context.offsets(node);
    // The break point is ASCII, so the byte after it starts a character of its own text node.
    let slice: &str = context.slice(node);
    let tail: &str = slice[at + 1 - node_start..].trim_start_matches(closing);
    let after: usize = node_end - tail.len();
    if after != node_end {
        return Some(after);
    }
    // The closers run to the end of this text node, so the break continues past the delimiters that end here.
    let ancestors: Vec<u32> = match context.ancestors(node) {
        Ok(chain) => chain,
        // A parent-index cycle is a defect of this crate, and the rule reports the document's structure itself.
        Err(_) => return None,
    };
    let tail_node: u32 = delimiter_tail(context, node, ancestors.as_slice());
    return Some(context.offsets(tail_node).1);
}

/// What:
///  Whether the rule's own line break lands at the end of a stretch of prose.
/// Why:
///  A fragment that ends where the rule breaks would be broken off again on the next pass,
///  so a join keeps the author's line there;
///  the answer is the tree's, because only the tree says whether a delimiter closes something:
///  `*held,*` ends with a `*` that closes an emphasis, where `held,*` ends with a star that closes nothing.
///  A delimiter chain that reaches past the stretch keeps the line too,
///  because the space a join writes would then stand inside the delimited text, where it is a separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endsAtBreakPoint(context: MarkdownSource, paragraph: number, start: number, end: number): boolean {
///   const at = lastBreakPoint(context.source, start, end);
///   if (at === undefined) return false;
///   const position = breakPosition(context, paragraph, at);
///   if (position === undefined) return false;
///   return position >= end;
/// }
/// ```
pub(crate) fn ends_at_break_point(
    context: &MarkdownSource,
    paragraph: u32,
    start: usize,
    end: usize,
) -> bool {
    // `Option<usize>` carries the offset of the last break point of the stretch, like `number | undefined`.
    let Some(at) = last_break_point(context.source.as_str(), start, end) else {
        return false;
    };
    let Some(position) = break_position(context, paragraph, at) else {
        return false;
    };
    return position >= end;
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
fn needs_break(
    slice_tail: &str,
    trailing: &str,
    paragraph_tail: bool,
) -> bool {
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
pub(crate) fn break_offsets(
    slice: &str,
    trailing: &str,
    paragraph_tail: bool,
) -> Vec<usize> {
    // Own a variable-length result list; usize indexes source bytes rather than UTF-16 columns.
    let mut offsets: Vec<usize> = Vec::<usize>::new();
    for (index, character) in slice.char_indices() {
        if !is_break_point(character) {
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
