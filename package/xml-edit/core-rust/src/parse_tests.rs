//! What:
//!  Parser and query tests on JetBrains-shaped documents and on the accepted and rejected grammar.
//! Why:
//!  Compilation alone does not show that spans,
//! decoding and well-formedness checks agree with the XML grammar.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('XML parser', () => { /* shapes, queries, rejection */ });
//! ```

use crate::fixtures::{LANGUAGE_XML, USER_DEFINED_XML};
use crate::{parse_xml, XmlDocument, XmlElementId};

/// Parse a source that the test knows is valid.
fn parsed(source: &str) -> XmlDocument {
    return parse_xml(source).expect("valid XML fixture");
}

/// Parse a source and return the rejection message and byte offset.
fn rejected(source: &str) -> (String, usize) {
    let error = parse_xml(source).expect_err("source must be rejected");
    return (error.message, error.offset);
}

/// Check the global settings shape answers the consumer's lookups.
#[test]
fn language_settings_shape_is_queryable() {
    let document: XmlDocument = parsed(LANGUAGE_XML);
    let maps: Vec<XmlElementId> = document.descendants_named(document.root(), "map");
    assert_eq!(maps.len(), 1);
    let entries: Vec<XmlElementId> = document.children_named(maps[0], "entry");
    assert_eq!(entries.len(), 1);
    assert_eq!(document.attribute(entries[0], "key"), Some("srv-1"));
    let settings = document.descendants_named(entries[0], "LanguageServerDefinitionSettings")[0];
    let option = document.find_child(settings, "option", "name", "configurationContent").expect("option exists");
    assert_eq!(document.attribute(option, "value"), Some("{\"harper-ls.linters.Keep\": true}"));
    assert_eq!(document.find_child(settings, "option", "name", "missing"), None);
    assert_eq!(document.name(option), Some("option"));
    assert_eq!(document.parent(option), Some(settings));
    assert_eq!(document.children(settings).len(), 4);
    assert_eq!(document.attributes(option).len(), 2);
}

/// Check the user-defined servers shape and the element source slice.
#[test]
fn user_defined_shape_exposes_element_source() {
    let document: XmlDocument = parsed(USER_DEFINED_XML);
    let entry: XmlElementId = document.descendants_named(document.root(), "entry")[0];
    let source: &str = document.element_source(entry).expect("entry source");
    assert!(source.starts_with("<entry key=\"srv-1\">"));
    assert!(source.ends_with("</entry>"));
    assert_eq!(document.span(entry).expect("span").end - document.span(entry).expect("span").start, source.len());
    assert_eq!(document.element_count(), 11);
}

/// Check unknown handles answer with empty results instead of panicking.
#[test]
fn unknown_handles_are_harmless() {
    let document: XmlDocument = parsed("<a/>");
    let stranger: XmlElementId = XmlElementId(99);
    assert_eq!(document.name(stranger), None);
    assert_eq!(document.parent(stranger), None);
    assert!(document.children(stranger).is_empty());
    assert_eq!(document.attribute(stranger, "x"), None);
    assert!(document.attributes(stranger).is_empty());
    assert!(document.descendants_named(stranger, "a").is_empty());
    assert_eq!(document.text(stranger), None);
    assert_eq!(document.span(stranger), None);
}

/// Check text,
/// CDATA and entities decode,
/// and comments stay out of the text.
#[test]
fn text_and_cdata_decode() {
    let document: XmlDocument = parsed("<a>x &amp; y<![CDATA[ <b/> ]]><!-- c -->&#65;\r\nz</a>");
    assert_eq!(document.text(document.root()), Some("x & y <b/> A\nz"));
    assert!(document.children(document.root()).is_empty());
}

/// Check the prolog accepts a declaration,
/// comments and processing instructions,
/// and keeps the BOM flag.
#[test]
fn prolog_and_epilog_are_accepted() {
    let source: &str = "\u{FEFF}<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<!-- c --><?pi data?>\n<a/>\n<!-- end -->\n";
    let document: XmlDocument = parsed(source);
    assert!(document.has_bom());
    assert_eq!(document.source(), source);
    assert!(!parsed("<a/>").has_bom());
    parsed("<?xml version='1.0'?><a/>");
    parsed("<?xml-stylesheet href=\"x\"?><a/>");
}

/// Check nested elements,
/// namespaced names and both quote styles parse.
#[test]
fn names_and_quotes_parse() {
    let document: XmlDocument = parsed("<ns:a x:y='1' z=\"it's\"><b-c.d _e=\"\u{e9}\"/></ns:a>");
    assert_eq!(document.name(document.root()), Some("ns:a"));
    assert_eq!(document.attribute(document.root(), "x:y"), Some("1"));
    assert_eq!(document.attribute(document.root(), "z"), Some("it's"));
    assert_eq!(document.descendants_named(document.root(), "b-c.d").len(), 1);
}

/// Check malformed documents are rejected with the offset of the first problem.
#[test]
fn malformed_documents_are_rejected_with_offsets() {
    assert_eq!(rejected("").1, 0);
    assert_eq!(rejected("   ").1, 3);
    assert_eq!(rejected("<a>").1, 0);
    assert_eq!(rejected("<a></b>").1, 3);
    assert_eq!(rejected("<a/><b/>").1, 4);
    assert_eq!(rejected("text<a/>").1, 0);
    assert_eq!(rejected("<a/>text").1, 4);
    assert_eq!(rejected("<a x=\"1\" x=\"2\"/>").1, 9);
    assert_eq!(rejected("<a x=\"1\"y=\"2\"/>").1, 8);
    assert_eq!(rejected("<a x=1/>").1, 5);
    assert_eq!(rejected("<a x/>").1, 4);
    assert_eq!(rejected("<a x=\"1/>").1, 5);
    assert_eq!(rejected("<a ").1, 0);
    assert_eq!(rejected("<a/ >").1, 2);
    assert_eq!(rejected("< a/>").1, 1);
    assert_eq!(rejected("<a></a b>").1, 7);
    assert_eq!(rejected("<1a/>").1, 1);
}

/// Check out-of-scope and illegal constructs are rejected with a naming message.
#[test]
fn out_of_scope_constructs_are_rejected() {
    assert!(rejected("<!DOCTYPE a><a/>").0.contains("DOCTYPE"));
    assert!(rejected("<a><!ELEMENT x></a>").0.contains("outside the supported"));
    assert!(rejected("<?xml version=\"1.1\"?><a/>").0.contains("1.1"));
    assert!(rejected("<?xml version=\"1.0\" encoding=\"UTF-16\"?><a/>").0.contains("UTF-16"));
    assert!(rejected("<?xml version=\"1.0\" standalone=\"maybe\"?><a/>").0.contains("standalone"));
    assert!(rejected("<?xml encoding=\"UTF-8\"?><a/>").0.contains("out-of-order"));
    assert!(rejected("<?xml ?><a/>").0.contains("version"));
    assert!(rejected("<?xml version=\"1.0\"encoding=\"UTF-8\"?><a/>").0.contains("separated"));
    assert!(rejected("<?xml version=\"1.0\"?><?xml version=\"1.0\"?><a/>").0.contains("very start"));
    assert!(rejected("<?xml version 1.0?><a/>").0.contains("expected ="));
    assert!(rejected("<?xml version=1.0?><a/>").0.contains("must start with a quote"));
    assert!(rejected("<?xml version=\"1.0?><a/>").0.contains("never closed"));
    assert!(rejected("<?xml version=\"1.0\" version=\"1.0\"?><a/>").0.contains("out-of-order"));
    assert!(rejected("<a>&foo;</a>").0.contains("&foo;"));
    assert!(rejected("<a>&amp</a>").0.contains("closing ;"));
    assert!(rejected("<a x=\"<\"/>").0.contains("&lt;"));
    assert!(rejected("<a>]]></a>").0.contains("]]>"));
    assert!(rejected("<a>\u{1}</a>").0.contains("U+0001"));
    assert!(rejected("<a><!-- -- --></a>").0.contains("--"));
    assert!(rejected("<a><!-- x ---></a>").0.contains("--->"));
    assert!(rejected("<a><!-- x </a>").0.contains("never closed"));
    assert!(rejected("<a><![CDATA[ x </a>").0.contains("CDATA"));
    assert!(rejected("<a><?pi x </a>").0.contains("processing instruction"));
    assert!(rejected("<a><?xml x?></a>").0.contains("very start"));
    assert!(rejected("<a><!-- \u{1} --></a>").0.contains("U+0001"));
    assert!(rejected("<a>x\u{FFFF}</a>").0.contains("U+FFFF"));
}

/// Check the diagnostic carries line and column and prints them.
#[test]
fn diagnostics_print_line_and_column() {
    let error = parse_xml("<a>\r\n  <b></c>\n</a>").expect_err("mismatch");
    assert_eq!((error.line, error.column), (2, 6));
    assert!(error.to_string().contains("(line 2, column 6, byte offset 10)"));
    let bare_cr = parse_xml("<a>\r<b></c></a>").expect_err("mismatch");
    assert_eq!(bare_cr.line, 2);
}
