//! What:
//!  Existing behavior controls for the initial native Markdown rules.
//! Why:
//!  Diagnostics and localized edits must preserve content,
//!  including Unicode and MDX boundaries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('ported Markdown rules', () => { /* report and fix fixtures */ });
//! ```

/// Import production rule entry points and the actual grouped-fix applier.
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::apply_fixes;
use crate::markdown_code::fenced_code_language;
use crate::markdown_headings::{heading_increment, no_emphasis_as_heading, single_h1};
use crate::markdown_links::{link_image_style, no_bare_urls};
use crate::markdown_source::MarkdownSource;

/// Parse one exact source through the native adapter.
fn document(source: &str, mdx: bool) -> MarkdownSource {
    return MarkdownSource::new(
        String::from(if mdx { "input.mdx" } else { "input.md" }),
        String::from(source),
        mdx,
    )
    .expect("fixture parses");
}

/// Apply only the findings' advertised fixes,
///  using the same atomic implementation as production.
fn fixed(source: &str, findings: &[Diagnostic]) -> String {
    let mut fixes = Vec::new();
    for finding in findings {
        if let Some(fix) = &finding.fix {
            fixes.push(fix.clone());
        }
    }
    return apply_fixes(source, fixes.as_slice())
        .expect("fixes apply")
        .source;
}

/// Heading baselines and decreases are allowed;
///  only upward skips are findings.
#[test]
fn heading_increment_matches_existing_messages_and_positions() {
    let source = document("### First\n\n# Reset\n\n### Skipped\n\n## Back\n", false);
    let findings = heading_increment(&source, Severity::Warn);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].message,
        "Heading level jumps from 1 to 3; increment by one."
    );
    assert_eq!(findings[0].labels[0].span.line, 5);
    assert_eq!(findings[0].severity, Severity::Warn);
    assert!(heading_increment(&document("plain\n", false), Severity::Error).is_empty());
}

/// Every legal one-step increase and repeated depth remains clean,
///  including deeper starting levels.
#[test]
fn heading_increment_accepts_adjacent_depths_and_equal_siblings() {
    for source in [
        "# One\n\n## Two\n\n### Three\n\n#### Four\n\n##### Five\n\n###### Six\n",
        "### Three\n\n#### Four\n\n#### Peer\n",
        "# One\n\n# Peer\n",
    ] {
        let context: MarkdownSource = document(source, false);
        assert!(
            heading_increment(&context, Severity::Error).is_empty(),
            "{source}"
        );
    }
}

/// Frontmatter title values do not consume the single allowed h1.
#[test]
fn single_h1_ignores_frontmatter_title() {
    let source = document("---\ntitle: metadata\n---\n# One\n\n# Two\n", false);
    let findings = single_h1(&source, Severity::Error);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].labels[0].span.line, 6);
    assert_eq!(
        findings[0].message,
        "Multiple top-level headings; a document should have a single h1."
    );
}

/// Sentence punctuation,
///  mixed paragraphs and emphasized list labels stay valid.
#[test]
fn emphasis_heading_exceptions_remain_exact() {
    let findings = no_emphasis_as_heading(
        &document("**Heading**\n\n*Another*\n", false),
        Severity::Error,
    );
    assert_eq!(findings.len(), 2);
    for source in [
        "**A sentence.**\n",
        "*Sentence？*\n",
        "- **Label**\n",
        "before **word** after\n",
        "> - *Label*\n",
    ] {
        assert!(
            no_emphasis_as_heading(&document(source, false), Severity::Error).is_empty(),
            "{source}"
        );
    }
}

/// A Markdown bare URL/email gets an autolink;
///  scheme-less www and explicit links are left alone.
#[test]
fn bare_markdown_links_keep_exact_written_bytes() {
    let source = "🚀 https://example.com/a\n\nname@example.com\n\nwww.example.com\n\n<https://already.example>\n";
    let context = document(source, false);
    let findings = no_bare_urls(&context, Severity::Error);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].labels[0].span.column, 4);
    assert_eq!(
        fixed(source, &findings),
        "🚀 <https://example.com/a>\n\n<name@example.com>\n\nwww.example.com\n\n<https://already.example>\n"
    );
}

/// MDX gets inline links rather than JSX-looking angle autolinks.
#[test]
fn bare_mdx_links_remain_parseable_after_fixing() {
    let source = "See https://example.com/path and name@example.com.\n";
    let context = document(source, true);
    let findings = no_bare_urls(&context, Severity::Error);
    let output = fixed(source, &findings);
    assert_eq!(
        output,
        "See [https://example.com/path](<https://example.com/path>) and [name@example.com](<mailto:name@example.com>).\n"
    );
    let reparsed = document(output.as_str(), true);
    assert!(no_bare_urls(&reparsed, Severity::Error).is_empty());
}

/// Shortcut link and image references gain only the missing brackets.
#[test]
fn shortcut_references_become_collapsed_without_changing_definitions() {
    let source = "[label] and ![label]\n\n[label]: /asset\n";
    let context = document(source, false);
    let findings = link_image_style(&context, Severity::Error);
    assert_eq!(findings.len(), 2);
    assert_eq!(
        fixed(source, &findings),
        "[label][] and ![label][]\n\n[label]: /asset\n"
    );
    let complete = document("[label][]\n\n[label]: /asset\n", false);
    assert!(link_image_style(&complete, Severity::Error).is_empty());
}

/// Fence length,
///  indentation,
///  whitespace and newline spelling survive the language insertion.
#[test]
fn fence_language_fixes_preserve_opener_shape() {
    let source = "  ~~~~  \nbody\n  ~~~~\n";
    let context = document(source, false);
    let findings = fenced_code_language(&context, Severity::Error, false);
    assert_eq!(fixed(source, &findings), "  ~~~~text  \nbody\n  ~~~~\n");
    let rustdoc = fenced_code_language(&context, Severity::Error, true);
    assert_eq!(fixed(source, &rustdoc), "  ~~~~rust  \nbody\n  ~~~~\n");
    assert!(
        fenced_code_language(&document("    indented\n", false), Severity::Error, false).is_empty()
    );
    assert!(
        fenced_code_language(
            &document("```rust\nfn f() {}\n```\n", false),
            Severity::Error,
            false
        )
        .is_empty()
    );
}

/// Native fenced spans start at the marker,
///  not at indentation or enclosing container text.
#[test]
fn fence_spans_preserve_native_marker_boundaries() {
    for (source, start, expected) in [
        ("~~~\nx\n~~~\n", 0, "~~~text\nx\n~~~\n"),
        ("  ~~~\nx\n  ~~~\n", 2, "  ~~~text\nx\n  ~~~\n"),
        ("> ~~~\n> x\n> ~~~\n", 2, "> ~~~text\n> x\n> ~~~\n"),
        ("- ~~~\n  x\n  ~~~\n", 2, "- ~~~text\n  x\n  ~~~\n"),
        (
            "🚀\n\n   ~~~\nx\n   ~~~\n",
            9,
            "🚀\n\n   ~~~text\nx\n   ~~~\n",
        ),
    ] {
        let context: MarkdownSource = document(source, false);
        let mut code_nodes: usize = 0;
        for id in context.visible_nodes() {
            if context.kind(*id) != satteri_ast::mdast::MdastNodeType::Code {
                continue;
            }
            code_nodes += 1;
            assert_eq!(context.offsets(*id).0, start);
            assert!(context.slice(*id).starts_with("~~~"));
        }
        assert_eq!(code_nodes, 1);
        let findings: Vec<Diagnostic> = fenced_code_language(&context, Severity::Error, false);
        assert_eq!(fixed(source, &findings), expected);
    }
}

/// Plain-text collection does not import image alt text into heading/emphasis semantics.
#[test]
fn collected_heading_text_matches_the_incumbent_helper() {
    let context = document("# Name ![alt](image.png) `code`\n", false);
    let heading = context
        .visible_nodes()
        .iter()
        .find(|id| return context.kind(**id) == satteri_ast::mdast::MdastNodeType::Heading)
        .expect("heading");
    assert_eq!(
        context.text_content(*heading),
        Ok(String::from("Name  code"))
    );
}
