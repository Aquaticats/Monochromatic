//! What:
//!  Adversarial boundary tests:
//! quotes,
//! entities,
//! CDATA,
//! comments holding markup,
//! attribute escapes,
//!  newlines,
//! BOM,
//! and very large inputs.
//! Why:
//!  Text crossing a syntax boundary must obey the destination grammar,
//! and unbounded input must be handled in
//!  one linear pass without stack growth.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('boundaries', () => { /* quotes, entities, CDATA, comments, BOM, depth */ });
//! ```

use crate::coding::{decode_attribute_value, decode_text, encode_attribute_value, encode_text};
use crate::{parse_xml, XmlDocument, XmlElementId};

/// Parse a source the test knows is valid.
fn parsed(source: &str) -> XmlDocument {
    return parse_xml(source).expect("valid XML fixture");
}

/// Check each quote style carries the other quote and the markup characters that are legal inside values.
#[test]
fn quotes_nest_inside_each_other() {
    let document: XmlDocument = parsed("<a x=\"it's\" y='say \"hi\"' z=\"/>\" w='>'/>");
    assert_eq!(document.attribute(document.root(), "x"), Some("it's"));
    assert_eq!(document.attribute(document.root(), "y"), Some("say \"hi\""));
    assert_eq!(document.attribute(document.root(), "z"), Some("/>"));
    assert_eq!(document.attribute(document.root(), "w"), Some(">"));
}

/// Check every predefined entity and numeric reference decodes,
/// and numeric forms keep real control whitespace.
#[test]
fn references_decode_to_characters() {
    let value: String = decode_attribute_value("&quot;&amp;&lt;&gt;&apos;&#10;&#13;&#9;&#x41;").expect("valid references");
    assert_eq!(value, "\"&<>'\n\r\tA");
    assert_eq!(decode_text("&#x1F600;&#128512;").expect("valid references"), "\u{1F600}\u{1F600}");
    for bad in ["&foo;", "&;", "&#;", "&#x;", "&#xZZ;", "&#0;", "&#1;", "&#xD800;", "&#x110000;", "&#+65;", "&#99999999999;", "&AMP;", "&#X41;"] {
        assert!(decode_attribute_value(bad).is_err(), "{bad} must be rejected");
    }
    assert_eq!(decode_attribute_value("ab&amp").expect_err("unterminated").offset, 2);
    assert_eq!(decode_attribute_value("&#0000000000000000000000000000000000000000065;").expect_err("too long").offset, 0);
}

/// Check literal whitespace in a value is normalized to spaces but references are not.
#[test]
fn attribute_whitespace_is_normalized() {
    let document: XmlDocument = parsed("<a x=\"a\tb\nc\r\nd\re&#10;f\"/>");
    assert_eq!(document.attribute(document.root(), "x"), Some("a b c d e\nf"));
    assert_eq!(decode_text("a\r\nb\rc\n").expect("text"), "a\nb\nc\n");
}

/// Check CDATA content and comment bodies holding markup never produce elements or errors.
#[test]
fn comments_and_cdata_hide_markup() {
    let source: &str = "<a><!-- <entry key=\"x\"> </entry> --><![CDATA[</a><entry key=\"y\"> --> ]]><b/></a>";
    let document: XmlDocument = parsed(source);
    assert!(document.descendants_named(document.root(), "entry").is_empty());
    assert_eq!(document.descendants_named(document.root(), "b").len(), 1);
    assert_eq!(document.text(document.root()), Some("</a><entry key=\"y\"> --> "));
    let edited: XmlDocument = document
        .set_attribute(document.descendants_named(document.root(), "b")[0], "k", "v")
        .expect("edit applies");
    assert_eq!(edited.source(), source.replace("<b/>", "<b k=\"v\"/>"));
    parsed("<a><!----></a>");
    parsed("<a><![CDATA[]]></a>");
    assert!(parse_xml("<a><!---></a>").is_err());
    assert!(parse_xml("<a><![CDATA[ ]] ></a>").is_err());
}

/// Check encoding mirrors the incumbent set and every encoded text decodes back unchanged.
#[test]
fn encoders_round_trip_through_the_decoders() {
    assert_eq!(encode_attribute_value("\"&<>\n\r\t'", '"').expect("encodes"), "&quot;&amp;&lt;&gt;&#10;&#13;&#9;'");
    assert_eq!(encode_attribute_value("'\"", '\'').expect("encodes"), "&apos;\"");
    assert_eq!(encode_text("a<b>&\r\n").expect("encodes"), "a&lt;b&gt;&amp;&#13;\n");
    for text in ["", "plain", "]]>", "a]]>b", "\u{FEFF}bom", "\u{10FFFF}", "\u{7F}", "\u{85}"] {
        let attribute: String = encode_attribute_value(text, '"').expect("encodes");
        assert_eq!(decode_attribute_value(&attribute).expect("decodes"), text);
        let content: String = encode_text(text).expect("encodes");
        assert_eq!(decode_text(&content).expect("decodes"), text);
    }
    for bad in ['\u{0}', '\u{8}', '\u{B}', '\u{1F}', '\u{FFFE}', '\u{FFFF}'] {
        assert!(encode_attribute_value(&bad.to_string(), '"').is_err());
        assert!(encode_text(&bad.to_string()).is_err());
    }
}

/// Check a BOM is accepted only at the start and survives byte-for-byte.
#[test]
fn bom_is_preserved_and_confined_to_the_start() {
    let document: XmlDocument = parsed("\u{FEFF}<a x=\"1\"/>");
    let edited: XmlDocument = document.set_attribute(document.root(), "x", "2").expect("edit applies");
    assert_eq!(edited.source(), "\u{FEFF}<a x=\"2\"/>");
    assert!(parse_xml(" \u{FEFF}<a/>").is_err());
    assert!(parse_xml("\u{FEFF}\u{FEFF}<a/>").is_err());
}

/// Check line-ending variety inside markup parses and the diagnostic positions count each style once.
#[test]
fn mixed_line_endings_parse() {
    let source: &str = "<a\r\n  x=\"1\"\r  y=\"2\"\n>\r\n<b/>\r</a>";
    let document: XmlDocument = parsed(source);
    assert_eq!(document.attribute(document.root(), "y"), Some("2"));
    let error = parse_xml("<a\r\n\r\n\r<b></c></a>").expect_err("mismatch");
    assert_eq!(error.line, 4);
}

/// Check structure that is deep,
/// wide or attribute-heavy parses in one pass and unclosed input reports its offset.
#[test]
fn large_inputs_are_handled_without_recursion() {
    let depth: usize = 20_000;
    let nested: String = format!("{}{}", "<a>".repeat(depth), "</a>".repeat(depth));
    let document: XmlDocument = parsed(&nested);
    assert_eq!(document.element_count(), depth);
    let deepest: XmlElementId = XmlElementId(depth - 1);
    assert_eq!(document.children(deepest).len(), 0);
    let unclosed: String = format!("{}{}", "<a>".repeat(depth), "</a>".repeat(depth - 1));
    assert_eq!(parse_xml(&unclosed).expect_err("one end tag missing").offset, 0);
    let wide: String = format!("<r>{}</r>", "<c/>".repeat(50_000));
    assert_eq!(parsed(&wide).children(XmlElementId(0)).len(), 50_000);
    let mut attributes: String = String::from("<r");
    for index in 0..20_000 {
        attributes.push_str(&format!(" a{index}=\"{index}\""));
    }
    attributes.push_str("/>");
    assert_eq!(parsed(&attributes).attributes(XmlElementId(0)).len(), 20_000);
    let ampersands: String = format!("<r>{}</r>", "&amp;".repeat(100_000));
    assert_eq!(parsed(&ampersands).text(XmlElementId(0)).map(str::len), Some(100_000));
    let bare: String = format!("<r>{}</r>", "&".repeat(100_000));
    assert_eq!(parse_xml(&bare).expect_err("bare ampersand").offset, 3);
}
