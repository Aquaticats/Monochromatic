//! Splits one formula's text into tokens in a single left-to-right pass.
//!
//! The input is UTF-16 code units,
//!  so every token's `start` and `end` is the
//! UTF-16 offset in the whole template,
//!  as the reference reports them.
//!  Every
//! boundary falls on an ASCII delimiter,
//!  so the text of each token converts back
//! from UTF-16 without splitting a surrogate pair.

/// What:
///  `use crate::template_model::TemplateError;` imports the mistake record.
/// Why:
///  Tokenizer mistakes use the same record as every other template mistake.
use crate::template_model::TemplateError;

/// What:
///  `const QUOTE: u16` is the quotation mark that opens and closes a text token.
/// Why:
///  Quoted text runs to the matching mark and is stored without them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quote = '"';
/// ```
const QUOTE: u16 = b'"' as u16;

/// What:
///  `const OPERATOR_UNITS: [u16; 4]` lists the characters that always stand alone:
///  `+`,
///  `(`,
///  `)`,
///  and `,`.
/// Why:
///  A bare word stops at these,
///  so a call and a join are always separate tokens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const operators = ["+", "(", ")", ","];
/// ```
const OPERATOR_UNITS: [u16; 4] = [b'+' as u16, b'(' as u16, b')' as u16, b',' as u16];

/// What:
///  `pub enum TokenKind` names the three kinds of formula token.
/// Why:
///  The parser branches on text,
///  operator,
///  and word tokens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TokenKind = "text" | "operator" | "word";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A quoted text token,
    ///  stored without its quotation marks.
    Text,
    /// A single operator character:
    ///  `+`,
    ///  `(`,
    ///  `)`,
    ///  or `,`.
    Operator,
    /// A bare word that runs to the next break character.
    Word,
}

/// What:
///  `pub struct Token` records one token with its absolute UTF-16 start and end offsets.
/// Why:
///  Errors and call positions point back into the whole template,
///  so offsets are absolute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Token = { kind: TokenKind; value: string; start: number; end: number };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Which kind of token this is.
    pub kind: TokenKind,
    /// Token text,
    ///  without quotation marks for a text token.
    pub value: String,
    /// Absolute UTF-16 offset of the first unit.
    pub start: usize,
    /// Absolute UTF-16 offset just past the last unit.
    pub end: usize,
}

/// What:
///  `pub fn is_space_unit(unit: u16) -> bool` tells whether a unit is one of the three spaces the tokenizer skips.
/// Why:
///  Only space,
///  tab,
///  and line feed are skipped,
///  so a carriage return stays inside a word,
///  as in the reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const skipped = (c: string) => c === " " || c === "\t" || c === "\n";
/// ```
pub fn is_space_unit(unit: u16) -> bool {
    return unit == b' ' as u16 || unit == b'\t' as u16 || unit == b'\n' as u16;
}

/// What:
///  `pub fn is_word_break(unit: u16) -> bool` tells whether a unit ends a bare word.
/// Why:
///  A word runs to the next space,
///  quotation mark,
///  or operator character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isWordBreak = (c: string) => " \t\n\"".includes(c) || "+(),".includes(c);
/// ```
pub fn is_word_break(unit: u16) -> bool {
    return is_space_unit(unit) || unit == QUOTE || OPERATOR_UNITS.contains(&unit);
}

/// What:
///  `pub fn find_unit(units: &[u16], from: usize, target: u16) -> Option<usize>` finds the next occurrence of a unit at or after `from`.
/// Why:
///  The template and formula readers search for dollar signs and quotation marks by offset,
///  as `indexOf` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = text.indexOf(target, from);
/// ```
pub fn find_unit(units: &[u16], from: usize, target: u16) -> Option<usize> {
    let found = units.get(from..)?.iter().position(|unit| return *unit == target)?;
    return Some(from + found);
}

/// What:
///  `pub fn slice_units(units: &[u16], start: usize, end: usize) -> String` converts a span of UTF-16 units to text.
/// Why:
///  Every span the engine cuts ends at an ASCII delimiter,
///  so the conversion keeps every character whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const piece = text.slice(start, end);
/// ```
pub fn slice_units(units: &[u16], start: usize, end: usize) -> String {
    return String::from_utf16_lossy(&units[start..end]);
}

/// What:
///  `pub fn tokenize(units: &[u16], offset: usize, errors: &mut Vec<TemplateError>) -> Vec<Token>` splits one formula's units into tokens.
/// Why:
///  Formulas are short,
///  and one pass keeps each mistake's position exact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tokenize(text: string, offset: number, errors: TemplateError[]): Token[] { ... }
/// ```
pub fn tokenize(units: &[u16], offset: usize, errors: &mut Vec<TemplateError>) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if is_space_unit(unit) {
            index += 1;
            continue;
        }
        if unit == QUOTE {
            let Some(close) = find_unit(units, index + 1, QUOTE) else {
                errors.push(TemplateError {
                    subject: String::from("text"),
                    message: String::from("a quotation mark is not closed"),
                    at: offset + index,
                });
                tokens.push(Token {
                    kind: TokenKind::Text,
                    value: slice_units(units, index + 1, units.len()),
                    start: offset + index,
                    end: offset + units.len(),
                });
                break;
            };
            tokens.push(Token {
                kind: TokenKind::Text,
                value: slice_units(units, index + 1, close),
                start: offset + index,
                end: offset + close + 1,
            });
            index = close + 1;
            continue;
        }
        if OPERATOR_UNITS.contains(&unit) {
            tokens.push(Token {
                kind: TokenKind::Operator,
                value: slice_units(units, index, index + 1),
                start: offset + index,
                end: offset + index + 1,
            });
            index += 1;
            continue;
        }
        let mut end = index;
        while end < units.len() && !is_word_break(units[end]) {
            end += 1;
        }
        tokens.push(Token {
            kind: TokenKind::Word,
            value: slice_units(units, index, end),
            start: offset + index,
            end: offset + end,
        });
        index = end;
    }
    return tokens;
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_tokens_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_tokens_tests.rs"]
mod tests;
