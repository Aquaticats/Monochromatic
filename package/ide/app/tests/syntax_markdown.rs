//! Markdown and MDX files are painted with the `markup` roles,
//!  fenced code in its own language,
//!  and a second channel (weight or italic face) beside the color.

/// Shared assertions over the application engine.
mod syntax_support;

/// Recognition,
/// role,
/// and visible text assertion that lists every painted interval on failure.
use syntax_support::{assert_reads_as, painted};

/// A Markdown file with every construct that has a role.
const SAMPLE: &str = "---\ntitle: Sample\n---\n\n# Heading one\n\n## Heading two\n\nPlain *emphasis* and _also_ and **strong** and ~~struck~~ and `code`.\nA [link text](https://example.com) and a [label][ref].\n\n> quoted words\n\n- bullet\n1. numbered\n\n[ref]: https://example.com/ref\n";

/// Headings are painted as headings,
///  marker included.
#[test]
fn atx_headings_are_painted_as_headings() {
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.heading",
        "# Heading one",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.heading",
        "## Heading two",
    );
}

/// Strong text and emphasis have their own roles.
#[test]
fn strong_text_and_emphasis_are_painted() {
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.bold",
        "strong",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.italic",
        "emphasis",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.italic",
        "also",
    );
}

/// Struck text and inline code have their own roles.
#[test]
fn struck_text_and_inline_code_are_painted() {
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.strikethrough",
        "struck",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.raw.inline",
        "code",
    );
}

/// Link text,
///  labels,
///  and destinations are painted apart.
#[test]
fn links_are_painted() {
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.link",
        "link text",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.link",
        "label",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.link.url",
        "https://example.com",
    );
}

/// List markers and quotes are painted.
#[test]
fn list_markers_and_quotes_are_painted() {
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.list",
        "- ",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.list",
        "1. ",
    );
    assert_reads_as(
        "/project/README.md",
        SAMPLE,
        "markdown",
        "markup.quote",
        "quoted words",
    );
}

/// The YAML frontmatter keeps its own language.
#[test]
fn frontmatter_is_painted_as_yaml() {
    assert_reads_as("/project/README.md", SAMPLE, "markdown", "string", "Sample");
}

/// MDX is read as Markdown,
///  so the same roles apply and its HTML blocks are painted as HTML.
#[test]
fn mdx_is_painted_like_markdown() {
    let source = "# Title\n\nSome **strong** and `code`.\n\n<Chart data={[1]} />\n";
    assert_reads_as(
        "/project/page.mdx",
        source,
        "markdown",
        "markup.heading",
        "# Title",
    );
    assert_reads_as(
        "/project/page.mdx",
        source,
        "markdown",
        "markup.bold",
        "strong",
    );
    assert_reads_as(
        "/project/page.mdx",
        source,
        "markdown",
        "markup.raw.inline",
        "code",
    );
    let mut tagged = false;
    for (role, text) in painted("/project/page.mdx", source) {
        if role == "tag" && text == "Chart" {
            tagged = true;
        }
    }
    assert!(
        tagged,
        "a JSX element on its own lines is painted by the HTML grammar"
    );
}

/// Fences in bundled languages paint through injection in that language.
#[test]
fn fenced_code_is_painted_in_its_own_language() {
    let cases = [
        ("rust", "fn main() {}", "keyword", "fn"),
        ("js", "const cat = 1;", "keyword", "const"),
        ("json", "{ \"key\": true }", "constant", "true"),
        ("sh", "echo hi && ls", "function", "echo"),
        ("toml", "[package]\nname = \"cat\"", "string", "\"cat\""),
    ];
    for (language, code, role, fragment) in cases {
        let source = format!("# Fence\n\n```{language}\n{code}\n```\n");
        assert_reads_as("/project/fences.md", &source, "markdown", role, fragment);
    }
}

/// A fenced block's text is not given a block color of its own,
///  which would show through every token its language leaves unpainted.
#[test]
fn fenced_code_has_no_block_color() {
    let source = "```rust\nlet cat = 1;\n```\n";
    let all = painted("/project/fences.md", source);
    let mut block_colored = false;
    for (role, _) in all.iter() {
        if role.starts_with("markup") {
            block_colored = true;
        }
    }
    assert!(
        !block_colored,
        "a fenced block is painted with a block color; painted intervals: {all:?}"
    );
}
