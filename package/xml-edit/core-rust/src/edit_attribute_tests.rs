//! What:
//!  Attribute edit tests, including the byte-for-byte preservation of everything outside the edited value.
//! Why:
//!  Preservation is the crate's reason to exist, so each test compares whole documents, not just the edit site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('attribute edits', () => { /* set, add, remove, escape, refuse */ });
//! ```

use crate::fixtures::LANGUAGE_XML;
use crate::{parse_xml, XmlDocument, XmlEditError, XmlElementId};

/// Parse a source the test knows is valid.
fn parsed(source: &str) -> XmlDocument {
    return parse_xml(source).expect("valid XML fixture");
}

/// Find the `configurationContent` option of the fixture.
fn content_option(document: &XmlDocument) -> XmlElementId {
    let settings = document.descendants_named(document.root(), "LanguageServerDefinitionSettings")[0];
    return document.find_child(settings, "option", "name", "configurationContent").expect("option");
}

/// Check replacing a value changes only that value's bytes and leaves the original document usable.
#[test]
fn set_attribute_replaces_only_the_value_bytes() {
    let document: XmlDocument = parsed(LANGUAGE_XML);
    let edited: XmlDocument = document
        .set_attribute(content_option(&document), "value", "{\"harper-ls.linters.Keep\": false}")
        .expect("edit applies");
    let expected: String = LANGUAGE_XML.replace(
        "{&quot;harper-ls.linters.Keep&quot;: true}",
        "{&quot;harper-ls.linters.Keep&quot;: false}",
    );
    assert_eq!(edited.source(), expected);
    assert_eq!(document.source(), LANGUAGE_XML);
    assert_eq!(edited.attribute(content_option(&edited), "value"), Some("{\"harper-ls.linters.Keep\": false}"));
}

/// Check awkward characters survive an edit and a re-read.
#[test]
fn set_attribute_round_trips_awkward_text() {
    let document: XmlDocument = parsed("<a x=\"1\"/>");
    for text in ["a\"b", "a'b", "a&b", "a<b>c", "line\nbreak", "tab\there", "cr\r\nlf", "&amp;", "\u{e9}\u{1F600}", ""] {
        let edited: XmlDocument = document.set_attribute(document.root(), "x", text).expect("edit applies");
        assert_eq!(edited.attribute(edited.root(), "x"), Some(text), "text {text:?} via {}", edited.source());
    }
    let edited = document.set_attribute(document.root(), "x", "\"&<>\n\r\t").expect("edit applies");
    assert_eq!(edited.source(), "<a x=\"&quot;&amp;&lt;&gt;&#10;&#13;&#9;\"/>");
}

/// Check single-quoted attributes keep their quote and escape it.
#[test]
fn set_attribute_keeps_the_quote_style() {
    let document: XmlDocument = parsed("<a x='1' y=\"2\"/>");
    let edited: XmlDocument = document.set_attribute(document.root(), "x", "it's \"fine\"").expect("edit applies");
    assert_eq!(edited.source(), "<a x='it&apos;s \"fine\"' y=\"2\"/>");
    assert_eq!(edited.attribute(edited.root(), "x"), Some("it's \"fine\""));
}

/// Check a missing attribute is appended after the last one, before the closing delimiter, with spacing kept.
#[test]
fn set_attribute_appends_a_new_attribute() {
    let spaced: XmlDocument = parsed("<a x=\"1\" />");
    let edited: XmlDocument = spaced.set_attribute(spaced.root(), "y", "2").expect("edit applies");
    assert_eq!(edited.source(), "<a x=\"1\" y=\"2\" />");
    let bare: XmlDocument = parsed("<a>t</a>");
    assert_eq!(bare.set_attribute(bare.root(), "y", "2").expect("edit applies").source(), "<a y=\"2\">t</a>");
    let multi: XmlDocument = parsed("<a\n  x=\"1\"\n>t</a>");
    let source: String = multi.set_attribute(multi.root(), "y", "2").expect("edit applies").source().to_string();
    assert_eq!(source, "<a\n  x=\"1\" y=\"2\"\n>t</a>");
}

/// Check removal takes the separating whitespace and nothing else.
#[test]
fn remove_attribute_takes_its_leading_whitespace() {
    let document: XmlDocument = parsed("<a\n  x=\"1\"\n  y=\"2\" z='3'/>");
    let without_x: XmlDocument = document.remove_attribute(document.root(), "x").expect("edit applies");
    assert_eq!(without_x.source(), "<a\n  y=\"2\" z='3'/>");
    let without_z: XmlDocument = without_x.remove_attribute(without_x.root(), "z").expect("edit applies");
    assert_eq!(without_z.source(), "<a\n  y=\"2\"/>");
    let alone: XmlDocument = parsed("<a x=\"1\" />");
    assert_eq!(alone.remove_attribute(alone.root(), "x").expect("edit applies").source(), "<a />");
}

/// Check edits outside the target keep comments, line endings, BOM and sibling text.
#[test]
fn edits_preserve_everything_else() {
    let source: &str = "\u{FEFF}<!-- head -->\r\n<a>\r\n  <!-- keep <b x=\"9\"/> -->\r\n  <b x=\"1\"/>\r\n  <c y=\"2\"/>\r\n</a>\r\n<!-- tail -->";
    let document: XmlDocument = parsed(source);
    let target: XmlElementId = document.descendants_named(document.root(), "b")[0];
    let edited: XmlDocument = document.set_attribute(target, "x", "changed").expect("edit applies");
    assert_eq!(edited.source(), source.replace("x=\"1\"", "x=\"changed\""));
    assert!(edited.has_bom());
}

/// Check every refusal path of the attribute edits.
#[test]
fn attribute_edits_refuse_bad_requests() {
    let document: XmlDocument = parsed("<a x=\"1\"/>");
    let root: XmlElementId = document.root();
    let stranger: XmlElementId = XmlElementId(5);
    assert_eq!(document.set_attribute(stranger, "x", "1"), Err(XmlEditError::UnknownElement));
    assert_eq!(document.remove_attribute(stranger, "x"), Err(XmlEditError::UnknownElement));
    assert_eq!(
        document.set_attribute(root, "1bad", "1"),
        Err(XmlEditError::InvalidName { name: "1bad".to_string() })
    );
    assert_eq!(
        document.set_attribute(root, "x y", "1"),
        Err(XmlEditError::InvalidName { name: "x y".to_string() })
    );
    assert_eq!(
        document.remove_attribute(root, "nope"),
        Err(XmlEditError::AttributeNotFound { name: "nope".to_string() })
    );
    let refused = document.set_attribute(root, "x", "bell\u{7}").expect_err("control character");
    assert_eq!(refused, XmlEditError::InvalidCharacter { context: "attribute value".to_string(), character: '\u{7}' });
    assert!(refused.to_string().contains("U+0007"));
}
