//! What:
//!  Rust lexical and source-position parity controls.
//! Why:
//!  Blank lines,
//!  actual comments and comment-like text inside literals must not share a heuristic.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('Rust source context', () => { /* token and position cases */ });
//! ```

/// Import the production context and exact diagnostic span.
use super::RustSource;
use crate::diagnostic::Span;

/// A supplied syntax root retains its identity and exact bytes instead of being reparsed.
#[test]
fn reuses_registered_syntax_without_changing_source_bytes() {
    let text: &str = "\u{feff}// 🚀\r\nfn main() { let value: u16 = 1; }\r\n";
    let parsed: ra_ap_syntax::Parse<ra_ap_syntax::SourceFile> =
        ra_ap_syntax::SourceFile::parse(text, ra_ap_syntax::Edition::CURRENT);
    let syntax: ra_ap_syntax::SyntaxNode = parsed.syntax_node();
    // Clone the syntax handle, not the source tree; equality below tests the retained node identity.
    let context: RustSource =
        RustSource::from_syntax(String::from("registered.rs"), syntax.clone());
    assert_eq!(context.source, text);
    assert_eq!(context.syntax(), &syntax);
    let separate: RustSource = RustSource::new(String::from("separate.rs"), String::from(text));
    // Positive control: a new parse of the same text is not the registered syntax root.
    assert_ne!(context.syntax(), separate.syntax());
    assert_eq!(context.code_line_count(), separate.code_line_count());
    assert_eq!(context.line_span(2), separate.line_span(2));
}

/// Blank and comment-only lines do not count,
///  while string contents remain code.
#[test]
fn real_lexer_distinguishes_comments_from_string_contents() {
    let source = RustSource::new(
        String::from("input.rs"),
        String::from(
            "// comment\n\n/* block\ncomment */\nfn main() {\n  let text = \"// not a comment\";\n}\n",
        ),
    );
    assert_eq!(source.code_line_count(), 3);
    assert_eq!(source.code_line_at(0), Some(5));
    assert_eq!(source.code_line_at(1), Some(6));
    assert_eq!(source.code_line_at(2), Some(7));
    assert_eq!(source.code_line_at(3), None);
    assert!(!source.syntax().text().is_empty());
    assert_eq!(source.filename, "input.rs");
}

/// Every physical line in a multiline literal counts,
///  including its internally blank line.
#[test]
fn multiline_literals_count_every_touched_line() {
    let source = RustSource::new(
        String::from("input.rs"),
        String::from("const TEXT: &str = r#\"one\n\nthree\"#;\n"),
    );
    assert_eq!(source.code_line_count(), 3);
}

/// Whole-line spans preserve the incumbent CRLF byte behavior and empty final line.
#[test]
fn line_spans_keep_lf_conventions_and_bounds() {
    let source = RustSource::new(String::from("input.rs"), String::from("a\r\nb\n"));
    assert_eq!(source.line_span(0), None);
    assert_eq!(
        source.line_span(1),
        Some(Span {
            offset: 0,
            length: 2,
            line: 1,
            column: 1
        })
    );
    assert_eq!(
        source.line_span(2),
        Some(Span {
            offset: 3,
            length: 1,
            line: 2,
            column: 1
        })
    );
    assert_eq!(
        source.line_span(3),
        Some(Span {
            offset: 5,
            length: 0,
            line: 3,
            column: 1
        })
    );
    assert_eq!(source.line_span(4), None);
}

/// Rust columns remain byte-based and multiline underlines stop at the first LF.
#[test]
fn ranges_preserve_byte_columns_and_clamp_underlines() {
    let source = RustSource::new(String::from("input.rs"), String::from("éx\nnext"));
    assert_eq!(
        source.span(2, 20),
        Span {
            offset: 2,
            length: 1,
            line: 1,
            column: 3
        }
    );
    assert_eq!(
        source.span(4, 4),
        Span {
            offset: 4,
            length: 4,
            line: 2,
            column: 1
        }
    );
    assert_eq!(
        source.span(20, 4),
        Span {
            offset: 20,
            length: 0,
            line: 2,
            column: 17
        }
    );
}

/// A recovered string token ending at EOF must not mark the empty line after its final newline.
#[test]
fn unterminated_literal_does_not_claim_the_empty_final_line() {
    let source = RustSource::new(String::from("broken.rs"), String::from("\"unterminated\n"));
    assert_eq!(source.code_line_count(), 1);
    assert_eq!(source.code_line_at(0), Some(1));
    assert_eq!(source.code_line_at(1), None);
}

/// A final line without LF retains its complete text span.
#[test]
fn final_non_lf_line_keeps_its_last_byte() {
    let source = RustSource::new(String::from("last.rs"), String::from("abc"));
    assert_eq!(
        source.line_span(1),
        Some(Span {
            offset: 0,
            length: 3,
            line: 1,
            column: 1
        })
    );
}

/// Empty and syntactically broken sources still produce queryable recovery trees.
#[test]
fn empty_and_recovery_trees_do_not_become_parse_findings() {
    let empty = RustSource::new(String::from("empty.rs"), String::new());
    assert_eq!(empty.code_line_count(), 0);
    assert_eq!(
        empty.line_span(1),
        Some(Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1
        })
    );
    let broken = RustSource::new(String::from("broken.rs"), String::from("fn broken( {"));
    assert_eq!(broken.code_line_count(), 1);
    assert!(!broken.syntax().text().is_empty());
}
