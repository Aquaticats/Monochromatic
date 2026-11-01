//! What:
//!  Controls for finding candidate comments with a real parse.
//! Why:
//!  A directive must be read only from a comment the parser says is one,
//!  so text inside strings,
//!  doc comments,
//!  code fences and code spans must never become a candidate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('scanComments', () => { /* rust tokens, markdown html nodes, mdx, offsets */ });
//! ```

/// Import the scanner under test and the language tag.
use super::{CommentSite, scan_comments};
use crate::run_paths::Language;

/// The comment texts between delimiters that the scanner found.
fn contents(language: Language, source: &str) -> Vec<String> {
    let scanned = scan_comments(language, "fixture", source).expect("the language has directives");
    let mut found: Vec<String> = Vec::<String>::new();
    for site in &scanned.sites {
        found.push(String::from(&source[site.content_start..site.content_end]));
    }
    return found;
}

/// Plain line and block comments are candidates with exact delimiter-free content ranges,
///  including the empty block comment,
///  and a line comment keeps the carriage return of a CRLF ending in its content.
#[test]
fn rust_plain_comments_are_candidates_with_exact_ranges() {
    let source: &str = "// line\r\n/* block */ fn f() {} /**/ // tail\n";
    let scanned = scan_comments(Language::Rust, "f.rs", source).expect("rust");
    assert_eq!(
        scanned.sites,
        [
            CommentSite {
                start: 0,
                end: 8,
                content_start: 2,
                content_end: 8
            },
            CommentSite {
                start: 9,
                end: 20,
                content_start: 11,
                content_end: 18
            },
            CommentSite {
                start: 31,
                end: 35,
                content_start: 33,
                content_end: 33
            },
            CommentSite {
                start: 36,
                end: 43,
                content_start: 38,
                content_end: 43
            },
        ]
    );
}

/// Every doc comment form is rustdoc prose,
///  while four slashes and three stars are plain comments.
#[test]
fn rust_doc_comments_are_never_candidates() {
    let source: &str = "/// outer\n//! inner\n/** outer block */\n/*! inner block */\n//// four\n/*** three */\nfn f() {}\n";
    assert_eq!(
        contents(Language::Rust, source),
        [String::from("// four"), String::from("** three ")]
    );
}

/// Text in strings,
///  raw strings,
///  byte strings and character literals is not a comment.
#[test]
fn rust_literals_do_not_hold_candidates() {
    let source: &str = concat!(
        "fn f() {\n",
        "    let a = \"// monochromatic-lint-disable -- r\";\n",
        "    let b = r#\"/* monochromatic-lint-disable -- r */\"#;\n",
        "    let c = b\"// monochromatic-lint-disable -- r\";\n",
        "    let d = '/';\n",
        "    let e = \"\\\"// still inside\";\n",
        "}\n",
    );
    assert_eq!(contents(Language::Rust, source), Vec::<String>::new());
}

/// Nested block comments are one candidate,
///  and an unterminated block comment is not a candidate.
#[test]
fn rust_nested_and_unterminated_block_comments() {
    assert_eq!(
        contents(Language::Rust, "/* a /* b */ c */ fn f() {}"),
        [String::from(" a /* b */ c ")]
    );
    assert_eq!(
        contents(
            Language::Rust,
            "fn f() {} /* monochromatic-lint-disable -- r"
        ),
        Vec::<String>::new()
    );
    // A block comment that is only an opener cannot be closed by the slash of its own opener.
    assert_eq!(contents(Language::Rust, "/*/"), Vec::<String>::new());
}

/// A file with syntax errors still yields its comments,
///  because the parser recovers around them.
#[test]
fn rust_comments_survive_syntax_errors() {
    assert_eq!(
        contents(Language::Rust, "fn {{{ // kept\n"),
        [String::from(" kept")]
    );
}

/// An HTML comment that starts an HTML node is a candidate,
///  in a block or inline position,
///  with leading spaces and trailing text allowed.
#[test]
fn markdown_html_comments_are_candidates() {
    let source: &str = "<!-- block -->\n\ntext <!-- inline --> more\n\n   <!-- indented --> tail\n";
    assert_eq!(
        contents(Language::Markdown, source),
        [
            String::from(" block "),
            String::from(" inline "),
            String::from(" indented ")
        ]
    );
    let scanned = scan_comments(Language::Markdown, "f.md", source).expect("markdown");
    assert_eq!(
        scanned.sites[2],
        CommentSite {
            start: 46,
            end: 63,
            content_start: 50,
            content_end: 60
        }
    );
}

/// Multi-line comments are candidates,
///  and a comment that never closes is not.
#[test]
fn markdown_multi_line_and_unclosed_comments() {
    assert_eq!(
        contents(
            Language::Markdown,
            "<!--\nmonochromatic-lint-disable -- r\n-->\n"
        ),
        [String::from("\nmonochromatic-lint-disable -- r\n")]
    );
    assert_eq!(
        contents(Language::Markdown, "<!-- never closed\n\ntext\n"),
        Vec::<String>::new()
    );
}

/// Code fences,
///  indented code,
///  inline code and raw HTML around a comment are not candidates.
#[test]
fn markdown_code_and_raw_html_do_not_hold_candidates() {
    let source: &str = concat!(
        "```\n<!-- monochromatic-lint-disable -- r -->\n```\n\n",
        "    <!-- monochromatic-lint-disable -- r -->\n\n",
        "inline `<!-- monochromatic-lint-disable -- r -->` code\n\n",
        "<div>\n<!-- monochromatic-lint-disable -- r -->\n</div>\n\n",
        "[link]: <https://example.com> \"<!-- monochromatic-lint-disable -- r -->\"\n",
    );
    assert_eq!(contents(Language::Markdown, source), Vec::<String>::new());
}

/// A byte order mark shifts nothing:
///  offsets are into the original source.
#[test]
fn markdown_offsets_include_a_byte_order_mark() {
    let source: &str = "\u{feff}<!-- x -->\n";
    let scanned = scan_comments(Language::Markdown, "f.md", source).expect("markdown");
    assert_eq!(
        scanned.sites,
        [CommentSite {
            start: 3,
            end: 13,
            content_start: 7,
            content_end: 10
        }]
    );
}

/// MDX has no directive form.
#[test]
fn mdx_has_no_candidates() {
    assert!(
        scan_comments(
            Language::Mdx,
            "f.mdx",
            "{/* monochromatic-lint-disable -- r */}\n"
        )
        .is_none()
    );
}

/// Positions follow the host language:
///  byte columns for Rust and UTF-16 columns for Markdown,
///  with Markdown lines ending at CR,
///  LF or CRLF.
#[test]
fn spans_use_the_host_languages_units() {
    let rust: &str = "let s = \"é🚀\"; // c\n";
    let rust_scanned = scan_comments(Language::Rust, "f.rs", rust).expect("rust");
    let site: CommentSite = rust_scanned.sites[0];
    let rust_span = rust_scanned.span(site.start, site.end - site.start);
    // The string holds a two-byte and a four-byte character before the comment.
    assert_eq!((rust_span.line, rust_span.column), (1, 19));
    let markdown: &str = "a\r\nb\rc 🚀 <!-- x -->\n";
    let markdown_scanned = scan_comments(Language::Markdown, "f.md", markdown).expect("markdown");
    let first: CommentSite = markdown_scanned.sites[0];
    let markdown_span = markdown_scanned.span(first.start, first.end - first.start);
    // The rocket is four bytes and two UTF-16 units, so the column is two fewer than the byte count.
    assert_eq!((markdown_span.line, markdown_span.column), (3, 6));
}
