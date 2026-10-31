//! What:
//!  Element edit tests: append, replace, remove and set text, on JetBrains-shaped and on awkward layouts.
//! Why:
//!  Inserted text must match its neighbours' indentation and line endings, and refusals must leave the
//!  original untouched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('element edits', () => { /* append, replace, remove, setText */ });
//! ```

use crate::fixtures::{LANGUAGE_XML, USER_DEFINED_XML};
use crate::{parse_xml, XmlDocument, XmlEditError, XmlElementId, XmlElementSpec};

/// Parse a source the test knows is valid.
fn parsed(source: &str) -> XmlDocument {
    return parse_xml(source).expect("valid XML fixture");
}

/// Build the scoped entry the consumer appends: entry, value, settings, one option.
fn scoped_entry() -> XmlElementSpec {
    let option: XmlElementSpec = XmlElementSpec::named("option")
        .with_attribute("name", "serverName")
        .with_attribute("value", "Harper \"scoped\"");
    let settings: XmlElementSpec = XmlElementSpec::named("Settings").with_child(option);
    let value: XmlElementSpec = XmlElementSpec::named("value").with_child(settings);
    return XmlElementSpec::named("entry").with_attribute("key", "srv-2").with_child(value);
}

/// Check appending to a multi-line map copies the sibling indentation and keeps the closing tag.
#[test]
fn append_child_matches_sibling_layout() {
    let document: XmlDocument = parsed(LANGUAGE_XML);
    let map: XmlElementId = document.descendants_named(document.root(), "map")[0];
    let edited: XmlDocument = document.append_child(map, &scoped_entry()).expect("edit applies");
    let expected: String = LANGUAGE_XML.replace(
        "        </entry>\n      </map>",
        "        </entry>\n        <entry key=\"srv-2\">\n          <value>\n            <Settings>\n              <option name=\"serverName\" value=\"Harper &quot;scoped&quot;\" />\n            </Settings>\n          </value>\n        </entry>\n      </map>",
    );
    assert_eq!(edited.source(), expected);
    let found: XmlElementId = edited.descendants_named(edited.root(), "entry")[1];
    assert_eq!(edited.attribute(found, "key"), Some("srv-2"));
    assert_eq!(parsed(USER_DEFINED_XML).descendants_named(document.root(), "entry").len(), 1);
}

/// Check appending expands self-closing, empty and inline parents.
#[test]
fn append_child_expands_empty_parents() {
    let leaf: XmlElementSpec = XmlElementSpec::named("b").with_attribute("k", "v");
    let closed: XmlDocument = parsed("<root>\n  <map />\n</root>");
    let map: XmlElementId = closed.descendants_named(closed.root(), "map")[0];
    assert_eq!(
        closed.append_child(map, &leaf).expect("edit applies").source(),
        "<root>\n  <map>\n    <b k=\"v\" />\n  </map>\n</root>"
    );
    let empty: XmlDocument = parsed("<root>\n  <map></map>\n</root>");
    let map: XmlElementId = empty.descendants_named(empty.root(), "map")[0];
    assert_eq!(
        empty.append_child(map, &leaf).expect("edit applies").source(),
        "<root>\n  <map>\n    <b k=\"v\" />\n  </map>\n</root>"
    );
    let inline: XmlDocument = parsed("<a><c/></a>");
    assert_eq!(inline.append_child(inline.root(), &leaf).expect("edit applies").source(), "<a><c/><b k=\"v\" /></a>");
}

/// Check CRLF and tab layouts are copied, and a trailing comment does not take the new child's place.
#[test]
fn append_child_copies_line_endings_and_tabs() {
    let leaf: XmlElementSpec = XmlElementSpec::named("b");
    let document: XmlDocument = parsed("<a>\r\n\t<c/>\r\n</a>");
    assert_eq!(document.append_child(document.root(), &leaf).expect("edit applies").source(), "<a>\r\n\t<c/>\r\n\t<b />\r\n</a>");
    let commented: XmlDocument = parsed("<a>\n  <c/>\n  <!-- end -->\n</a>");
    assert_eq!(
        commented.append_child(commented.root(), &leaf).expect("edit applies").source(),
        "<a>\n  <c/>\n  <!-- end -->\n  <b />\n</a>"
    );
    let only_comment: XmlDocument = parsed("<a>\n  <!-- c -->\n</a>");
    assert_eq!(
        only_comment.append_child(only_comment.root(), &leaf).expect("edit applies").source(),
        "<a>\n  <!-- c -->\n  <b />\n</a>"
    );
}

/// Check append refusals and emitter validation.
#[test]
fn append_child_refuses_bad_requests() {
    let document: XmlDocument = parsed("<a>text</a>");
    let leaf: XmlElementSpec = XmlElementSpec::named("b");
    assert_eq!(
        document.append_child(document.root(), &leaf),
        Err(XmlEditError::MixedContent { element: "a".to_string() })
    );
    assert_eq!(document.append_child(XmlElementId(3), &leaf), Err(XmlEditError::UnknownElement));
    let empty: XmlDocument = parsed("<a/>");
    for (spec, name) in [(XmlElementSpec::named("1x"), "1x"), (XmlElementSpec::named("b").with_attribute("a b", "1"), "a b")] {
        assert_eq!(empty.append_child(empty.root(), &spec), Err(XmlEditError::InvalidName { name: name.to_string() }));
    }
    let twice: XmlElementSpec = XmlElementSpec::named("b").with_attribute("k", "1").with_attribute("k", "2");
    assert_eq!(empty.append_child(empty.root(), &twice), Err(XmlEditError::DuplicateAttribute { name: "k".to_string() }));
    let mixed: XmlElementSpec = XmlElementSpec::named("b").with_text("t").with_child(XmlElementSpec::named("c"));
    assert_eq!(empty.append_child(empty.root(), &mixed), Err(XmlEditError::MixedContent { element: "b".to_string() }));
    let bad_text: XmlElementSpec = XmlElementSpec::named("b").with_text("\u{1}");
    assert!(matches!(empty.append_child(empty.root(), &bad_text), Err(XmlEditError::InvalidCharacter { .. })));
    let text_leaf: XmlElementSpec = XmlElementSpec::named("b").with_text("a < b & c\r");
    assert_eq!(
        empty.append_child(empty.root(), &text_leaf).expect("edit applies").source(),
        "<a>\n  <b>a &lt; b &amp; c&#13;</b>\n</a>"
    );
}

/// Check replacing an element re-indents its description under the old element's line.
#[test]
fn replace_element_rebuilds_the_block() {
    let document: XmlDocument = parsed(LANGUAGE_XML);
    let entry: XmlElementId = document.descendants_named(document.root(), "entry")[0];
    let edited: XmlDocument = document.replace_element(entry, &scoped_entry()).expect("edit applies");
    assert!(edited.source().contains("        <entry key=\"srv-2\">\n          <value>"));
    assert!(edited.source().ends_with("      </map>\n    </state>\n  </component>\n</application>"));
    assert_eq!(edited.descendants_named(edited.root(), "entry").len(), 1);
    assert_eq!(document.replace_element(XmlElementId(99), &scoped_entry()), Err(XmlEditError::UnknownElement));
}

/// Check removal takes whole lines when it can and only the span otherwise, and never the root.
#[test]
fn remove_element_cleans_up_its_lines() {
    let document: XmlDocument = parsed("<a>\n  <b/>\n  <c/>\n</a>");
    let b: XmlElementId = document.children(document.root())[0];
    assert_eq!(document.remove_element(b).expect("edit applies").source(), "<a>\n  <c/>\n</a>");
    let crlf: XmlDocument = parsed("<a>\r\n  <b/>\r\n  <c/>\r\n</a>");
    assert_eq!(crlf.remove_element(crlf.children(crlf.root())[1]).expect("edit applies").source(), "<a>\r\n  <b/>\r\n</a>");
    let shared: XmlDocument = parsed("<a><b/> <c/></a>");
    assert_eq!(shared.remove_element(shared.children(shared.root())[0]).expect("edit applies").source(), "<a> <c/></a>");
    assert_eq!(document.remove_element(document.root()), Err(XmlEditError::CannotRemoveRoot));
    assert_eq!(document.remove_element(XmlElementId(9)), Err(XmlEditError::UnknownElement));
}

/// Check text edits on leaf, self-closing and CDATA-holding elements, and their refusals.
#[test]
fn set_text_rewrites_leaf_content() {
    let leaf: XmlDocument = parsed("<a x=\"1\">old</a>");
    assert_eq!(leaf.set_text(leaf.root(), "n<&>").expect("edit applies").source(), "<a x=\"1\">n&lt;&amp;&gt;</a>");
    let closed: XmlDocument = parsed("<a x=\"1\" />");
    assert_eq!(closed.set_text(closed.root(), "t").expect("edit applies").source(), "<a x=\"1\">t</a>");
    let cdata: XmlDocument = parsed("<a><![CDATA[<raw>]]></a>");
    assert_eq!(cdata.text(cdata.root()), Some("<raw>"));
    assert_eq!(cdata.set_text(cdata.root(), "x").expect("edit applies").source(), "<a>x</a>");
    let nested: XmlDocument = parsed("<a><b/></a>");
    assert_eq!(nested.set_text(nested.root(), "x"), Err(XmlEditError::MixedContent { element: "a".to_string() }));
    let commented: XmlDocument = parsed("<a><!-- c --></a>");
    assert!(commented.set_text(commented.root(), "x").is_err());
    assert_eq!(leaf.set_text(XmlElementId(4), "x"), Err(XmlEditError::UnknownElement));
    assert!(matches!(leaf.set_text(leaf.root(), "\u{0}"), Err(XmlEditError::InvalidCharacter { .. })));
}
