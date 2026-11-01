//! What:
//!  Encoding controls for Markdown cell text moved into HTML/MDX.
//! Why:
//!  Unicode padding,
//!  escape runs,
//!  tag delimiters and braces exercise different syntax boundaries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Compare exact encoded output and preserve every nonreserved authored character.
//! ```

/// Import the production encoder rather than test a copied entity map.
use super::html_table_cell_text;

/// Every reserved HTML character is encoded only after Markdown-sensitive escapes are consumed.
#[test]
fn html_text_encoding_neutralizes_markup_without_stripping_markdown() {
    assert_eq!(
        html_table_cell_text("  a \\| \\<img> & \"x\" ' **bold** `code`  ", false),
        "a | &lt;img&gt; &amp; &quot;x&quot; &#39; **bold** `code`"
    );
    assert_eq!(
        html_table_cell_text("\\&\\>\\\"\\'", false),
        "&amp;&gt;&quot;&#39;"
    );
    assert_eq!(html_table_cell_text("\\\\|", false), "\\|");
    assert_eq!(html_table_cell_text("\\*", false), "\\*");
    assert_eq!(html_table_cell_text("end\\", false), "end\\");
}

/// Authored entity spelling remains literal,
///  matching the incumbent's raw-cell conversion.
#[test]
fn entity_spelling_is_preserved_in_the_html_text() {
    for (source, expected) in [
        ("&amp;", "&amp;amp;"),
        ("&lt;", "&amp;lt;"),
        ("&nbsp;", "&amp;nbsp;"),
        ("&#x26;", "&amp;#x26;"),
        ("&", "&amp;"),
    ] {
        assert_eq!(html_table_cell_text(source, false), expected);
    }
}

/// MDX braces are literal text,
///  never an expression introduced by the conversion.
#[test]
fn mdx_text_cannot_gain_executable_braces() {
    assert_eq!(
        html_table_cell_text("{danger()} \\{x\\}", true),
        "&#123;danger()&#125; \\&#123;x\\&#125;"
    );
    assert_eq!(html_table_cell_text("{literal}", false), "{literal}");
}

/// The exact incumbent trim behavior differs from Rust for BOM and NEXT LINE.
#[test]
fn padding_keeps_the_incumbent_unicode_boundaries() {
    assert_eq!(
        html_table_cell_text("\u{feff}\u{00a0}🚀\u{feff}", false),
        "🚀"
    );
    assert_eq!(html_table_cell_text("\u{0085}", false), "\u{0085}");
    assert_eq!(html_table_cell_text("", false), "");
}
