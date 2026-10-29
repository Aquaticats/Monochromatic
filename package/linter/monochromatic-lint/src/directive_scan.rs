//! What:
//!  Find the comments that may carry directives,
//! from a real parse of the host file.
//! Why:
//!  A directive looks like text,
//! but text inside a string,
//! a code fence,
//! an inline code span or a doc comment
//!  is not a comment the tool reads.
//!  Rust comments are the parser's non-doc comment tokens;
//!  Markdown comments are the parser's HTML nodes.
//!  Neither language is scanned as text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scanComments(language, filename, source): { sites: CommentSite[]; span(offset, length): Span } | undefined;
//! ```

/// Import the position type that findings use.
use crate::diagnostic::Span;
/// Import the Markdown parse with its UTF-16-column positions and node kinds.
use crate::markdown_source::MarkdownSource;
/// Import the language tag that selects the parser.
use crate::run_paths::Language;
/// Import the Rust parse with its byte-column positions.
use crate::rust_source::RustSource;
/// Import the syntax kind and typed comment view used to skip doc comments.
use ra_ap_syntax::{AstToken, NodeOrToken, SyntaxKind, ast};
/// Import the Markdown node kind that holds an HTML comment.
use satteri_ast::mdast::MdastNodeType;

/// What:
///  One candidate comment's place in the host source.
/// Why:
///  The directive text is the content between the comment's delimiters;
///  findings about the directive underline the whole comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommentSite = { start: number; end: number; contentStart: number; contentEnd: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommentSite {
    /// Host byte offset of the opening delimiter.
    pub start: usize,
    /// Host byte offset after the closing delimiter,
    ///  or after the last byte of a line comment.
    pub end: usize,
    /// Host byte offset of the first byte between the delimiters.
    pub content_start: usize,
    /// Host byte offset after the last byte between the delimiters.
    pub content_end: usize,
}

/// What:
///  The parse that gives a comment list its line and column conventions.
/// Why:
///  Rust findings use LF lines and byte columns,
///  Markdown findings use CR,
/// LF or CRLF lines and UTF-16 columns,
///  and a directive finding must use the host's own convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Locator = RustSource | MarkdownSource;
/// ```
enum Locator {
    /// A Rust host.
    Rust(
        /// The Rust parse.
        RustSource,
    ),
    /// A Markdown host.
    Markdown(
        /// The Markdown parse,
        ///  boxed because it is much larger than the Rust one.
        Box<MarkdownSource>,
    ),
}

/// What:
///  The candidate comments of one host file plus the means to place offsets in it.
/// Why:
///  Callers never see which parser produced the sites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScannedComments = { sites: CommentSite[]; span(offset: number, length: number): Span };
/// ```
pub struct ScannedComments {
    /// Candidate comments in document order.
    pub sites: Vec<CommentSite>,
    /// The parse whose conventions position the file.
    locator: Locator,
}

/// What:
///  Place host byte ranges.
/// Why:
///  The line of a finding and the line of a comment must come from one index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanned.span(offset, length)
/// ```
impl ScannedComments {
    /// Resolve a host byte range in the host language's line and column units.
    pub fn span(&self, offset: usize, length: usize) -> Span {
        match &self.locator {
            Locator::Rust(source) => return source.span(offset, length),
            Locator::Markdown(document) => return document.span(offset, length),
        }
    }
}

/// What:
///  Locate a plain Rust comment's content between its delimiters.
/// Why:
///  An unterminated block comment swallows the rest of the file and is a syntax error,
///  so it is not a directive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rustContent(text: string): [number, number] | undefined;
/// ```
fn rust_content(text: &str) -> Option<(usize, usize)> {
    if text.starts_with("//") {
        return Some((2, text.len()));
    }
    if text.len() >= 4 && text.starts_with("/*") && text.ends_with("*/") {
        return Some((2, text.len() - 2));
    }
    return None;
}

/// What:
///  Collect the non-doc comment tokens of a Rust file.
/// Why:
///  Doc comments are Markdown for rustdoc,
///  not directives,
///  and the parser's token kinds exclude every string and character literal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanRust(filename: string, source: string): ScannedComments;
/// ```
fn scan_rust(filename: &str, source: &str) -> ScannedComments {
    let parsed: RustSource = RustSource::new(String::from(filename), String::from(source));
    let mut sites: Vec<CommentSite> = Vec::<CommentSite>::new();
    for element in parsed.syntax().descendants_with_tokens() {
        let NodeOrToken::Token(token) = element else {
            continue;
        };
        if token.kind() != SyntaxKind::COMMENT {
            continue;
        }
        let Some(comment): Option<ast::Comment> = ast::Comment::cast(token) else {
            continue;
        };
        if comment.is_doc() {
            continue;
        }
        let Some((open, close)): Option<(usize, usize)> = rust_content(comment.text()) else {
            continue;
        };
        let start: usize = usize::from(comment.syntax().text_range().start());
        sites.push(CommentSite {
            start,
            end: start + comment.text().len(),
            content_start: start + open,
            content_end: start + close,
        });
    }
    return ScannedComments {
        sites,
        locator: Locator::Rust(parsed),
    };
}

/// What:
///  Collect the HTML comments that begin an HTML node of a Markdown file.
/// Why:
///  Fenced code,
/// indented code and code spans are not HTML nodes,
///  so a directive shown as an example is never read.
///  Only a comment that starts its node counts,
///  as in `@eslint/markdown`;
///  a comment inside a larger HTML block is raw HTML.
///  `None` means the document did not parse,
///  and the caller's own processing failure already says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanMarkdown(filename: string, source: string): ScannedComments | undefined;
/// ```
fn scan_markdown(filename: &str, source: &str) -> Option<ScannedComments> {
    let document: MarkdownSource =
        MarkdownSource::new(String::from(filename), String::from(source), false).ok()?;
    let mut sites: Vec<CommentSite> = Vec::<CommentSite>::new();
    for id in document.visible_nodes() {
        if document.kind(*id) != MdastNodeType::Html {
            continue;
        }
        let (node_start, _node_end): (usize, usize) = document.offsets(*id);
        let text: &str = document.slice(*id);
        let indented: &str = text.trim_start_matches([' ', '\t']);
        let Some(inside): Option<&str> = indented.strip_prefix("<!--") else {
            continue;
        };
        let Some(close): Option<usize> = inside.find("-->") else {
            continue;
        };
        let start: usize = node_start + (text.len() - indented.len());
        sites.push(CommentSite {
            start,
            end: start + 4 + close + 3,
            content_start: start + 4,
            content_end: start + 4 + close,
        });
    }
    return Some(ScannedComments {
        sites,
        locator: Locator::Markdown(Box::new(document)),
    });
}

/// What:
///  Parse the host and list its candidate comments.
/// Why:
///  MDX has no directive form:
///  its HTML comments are syntax errors and its expression comments are not read.
///  Other languages return `None` too,
///  which the caller treats as no directives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanComments(language, filename, source): ScannedComments | undefined;
/// ```
pub fn scan_comments(language: Language, filename: &str, source: &str) -> Option<ScannedComments> {
    match language {
        Language::Rust => return Some(scan_rust(filename, source)),
        Language::Markdown => return scan_markdown(filename, source),
        Language::Mdx => return None,
        // The `shell/` rules are unsuppressible, so a workflow has no directive form either.
        Language::Yaml => return None,
    }
}

/// Scanning controls stay outside release artifacts.
#[cfg(test)]
#[path = "directive_scan_tests.rs"]
mod tests;
