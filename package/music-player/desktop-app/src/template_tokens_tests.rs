// Unit tests for `template_tokens.rs`, pulled in by `#[cfg(test)] #[path = "template_tokens_tests.rs"] mod tests;`.
//
// The positions checked here are the UTF-16 offsets the reference reports, so the
// expected numbers count an astral character as two units.

use super::*;

// What: `fn utf16(text: &str) -> Vec<u16>` converts test text to the units the tokenizer reads.
// Why: The tokenizer takes UTF-16 units, so each test converts its literal once here.
//
// In TS you'd write (pseudocode):
// ```ts
// const units = [...text].map(c => c.charCodeAt(0));
// ```
fn utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().collect();
}

// What: `fn spans(tokens: &[Token]) -> Vec<(TokenKind, String, usize, usize)>` lists each token's kind, value, and span.
// Why: One comparison checks every field of each token at once.
//
// In TS you'd write (pseudocode):
// ```ts
// const spans = tokens.map(t => [t.kind, t.value, t.start, t.end]);
// ```
fn spans(tokens: &[Token]) -> Vec<(TokenKind, String, usize, usize)> {
    return tokens.iter().map(|token| return (token.kind, token.value.clone(), token.start, token.end)).collect();
}

// What: `fn tokenize_splits_a_call_into_tokens_with_offsets(...)` checks a call's tokens and their spans.
// Why: Help needs the bracket and word offsets, so each token's span is pinned.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(tokenize("mi(title)", 0, errors).map(t => t.start)).toEqual([0, 2, 3, 8]);
// ```
#[test]
fn tokenize_splits_a_call_into_tokens_with_offsets() {
    let mut errors = Vec::new();
    let tokens = tokenize(&utf16("mi(title)"), 0, &mut errors);
    assert_eq!(
        spans(&tokens),
        vec![
            (TokenKind::Word, String::from("mi"), 0, 2),
            (TokenKind::Operator, String::from("("), 2, 3),
            (TokenKind::Word, String::from("title"), 3, 8),
            (TokenKind::Operator, String::from(")"), 8, 9),
        ]
    );
    assert!(errors.is_empty());
}

// What: `fn tokenize_shifts_offsets_by_the_formula_offset(...)` checks that a formula's offset moves every token.
// Why: A formula starts after its dollar sign, so its tokens report positions in the whole template.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(tokenize("mi", 10, errors)[0].start).toBe(10);
// ```
#[test]
fn tokenize_shifts_offsets_by_the_formula_offset() {
    let mut errors = Vec::new();
    let tokens = tokenize(&utf16("mi(x)"), 10, &mut errors);
    assert_eq!(tokens[0].start, 10);
    assert_eq!(tokens[1].start, 12);
    assert_eq!(tokens[3].end, 15);
}

// What: `fn tokenize_reads_quoted_text_without_its_marks(...)` checks text tokens and their spans.
// Why: Quoted text is kept as written and its span includes both marks.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(tokenize('"title"')).toEqual([{ kind: "text", value: "title", start: 0, end: 7 }]);
// ```
#[test]
fn tokenize_reads_quoted_text_without_its_marks() {
    let mut errors = Vec::new();
    let tokens = tokenize(&utf16("\"m min\""), 0, &mut errors);
    assert_eq!(spans(&tokens), vec![(TokenKind::Text, String::from("m min"), 0, 7)]);
    assert!(errors.is_empty());
}

// What: `fn tokenize_reports_an_unclosed_quotation_mark(...)` checks the mistake and the text it leaves behind.
// Why: The rest of the formula after an open quotation mark is text, and the mistake is placed at the mark.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(errors).toEqual([{ subject: "text", message: "a quotation mark is not closed", at: 0 }]);
// ```
#[test]
fn tokenize_reports_an_unclosed_quotation_mark() {
    let mut errors = Vec::new();
    let tokens = tokenize(&utf16("\"abc"), 0, &mut errors);
    assert_eq!(spans(&tokens), vec![(TokenKind::Text, String::from("abc"), 0, 4)]);
    assert_eq!(
        errors,
        vec![TemplateError {
            subject: String::from("text"),
            message: String::from("a quotation mark is not closed"),
            at: 0,
        }]
    );
}

// What: `fn tokenize_skips_only_space_tab_and_line_feed(...)` checks which white space separates tokens.
// Why: A carriage return is not skipped, so it stays inside a word, as the reference does.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(tokenize("a\rb").map(t => t.value)).toEqual(["a\rb"]);
// ```
#[test]
fn tokenize_skips_only_space_tab_and_line_feed() {
    let mut errors = Vec::new();
    let skipped = tokenize(&utf16("  a \t+\n b"), 0, &mut errors);
    assert_eq!(
        spans(&skipped),
        vec![
            (TokenKind::Word, String::from("a"), 2, 3),
            (TokenKind::Operator, String::from("+"), 5, 6),
            (TokenKind::Word, String::from("b"), 8, 9),
        ]
    );
    let kept = tokenize(&utf16("a\rb"), 0, &mut errors);
    assert_eq!(spans(&kept), vec![(TokenKind::Word, String::from("a\rb"), 0, 3)]);
}

// What: `fn tokenize_counts_astral_characters_as_two_units(...)` checks offsets after a surrogate pair.
// Why: Template offsets are UTF-16 offsets, so a character outside the Basic Multilingual Plane takes two positions.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(tokenize('"\u{10428}" + a')[2].start).toBe(6);
// ```
#[test]
fn tokenize_counts_astral_characters_as_two_units() {
    let mut errors = Vec::new();
    let tokens = tokenize(&utf16("\"\u{10428}\"+a"), 0, &mut errors);
    assert_eq!(tokens[0].end, 4);
    assert_eq!(tokens[1].start, 4);
    assert_eq!(tokens[2].start, 5);
}

// What: `fn tokenize_of_empty_text_is_empty(...)` checks that no text gives no tokens and no mistakes.
// Why: An empty formula is reported by the parser, not by the tokenizer.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(tokenize("")).toEqual([]);
// ```
#[test]
fn tokenize_of_empty_text_is_empty() {
    let mut errors = Vec::new();
    assert!(tokenize(&utf16(""), 0, &mut errors).is_empty());
    assert!(errors.is_empty());
}

// What: `fn find_unit_and_slice_units_keep_offsets(...)` checks the search and span helpers.
// Why: The template reader relies on these helpers for every offset it reports.
//
// In TS you'd write (pseudocode):
// ```ts
// expect("a$b$".indexOf("$", 2)).toBe(3);
// ```
#[test]
fn find_unit_and_slice_units_keep_offsets() {
    let text = utf16("a$b$");
    assert_eq!(find_unit(&text, 0, b'$' as u16), Some(1));
    assert_eq!(find_unit(&text, 2, b'$' as u16), Some(3));
    assert_eq!(find_unit(&text, 4, b'$' as u16), None);
    assert_eq!(find_unit(&text, 99, b'$' as u16), None);
    assert_eq!(slice_units(&text, 1, 3), "$b");
    assert_eq!(slice_units(&text, 4, 4), "");
}

// What: `fn word_break_follows_the_reference_delimiters(...)` checks which units end a bare word.
// Why: A word stops at a space, a quotation mark, or an operator character, and nothing else.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(isWordBreak("+")).toBe(true);
// ```
#[test]
fn word_break_follows_the_reference_delimiters() {
    assert!(is_word_break(b'+' as u16));
    assert!(is_word_break(b',' as u16));
    assert!(is_word_break(b'"' as u16));
    assert!(is_word_break(b' ' as u16));
    assert!(!is_word_break(b'=' as u16));
    assert!(!is_word_break(b'<' as u16));
    assert!(!is_word_break(b':' as u16));
}
