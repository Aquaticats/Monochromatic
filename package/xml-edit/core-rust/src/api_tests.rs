//! What:
//!  Public-surface tests for character classes,
//! layout probes,
//! element descriptions and error text.
//! Why:
//!  Every exported item needs coverage before the crate can be called complete,
//! including the refusals
//!  and the safety net that re-parses each edit result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('api surface', () => { /* chars, layout, spec builders, errors */ });
//! ```

use crate::chars::{is_name_char, is_name_start_char, is_xml_char, is_xml_name, is_xml_space};
use crate::layout::{blank_tail_end, detect_eol, detect_indent_unit, line_indent, line_start, starts_own_line};
use crate::{parse_xml, XmlEditError, XmlElementSpec, XmlNodeSpec, XmlParseError};

/// Check the character classes at their boundaries.
#[test]
fn character_classes_follow_the_grammar() {
    for allowed in ['\t', '\n', '\r', ' ', '\u{D7FF}', '\u{E000}', '\u{FFFD}', '\u{10000}', '\u{10FFFF}'] {
        assert!(is_xml_char(allowed), "{allowed:?}");
    }
    for refused in ['\u{0}', '\u{8}', '\u{B}', '\u{C}', '\u{E}', '\u{1F}', '\u{FFFE}', '\u{FFFF}'] {
        assert!(!is_xml_char(refused), "{refused:?}");
    }
    assert!(is_xml_space(b' ') && is_xml_space(b'\t') && is_xml_space(b'\r') && is_xml_space(b'\n'));
    assert!(!is_xml_space(0xA0) && !is_xml_space(b'x'));
    for start in [':', '_', 'a', 'Z', '\u{C0}', '\u{37F}', '\u{200C}', '\u{2070}', '\u{2C00}', '\u{3001}', '\u{F900}', '\u{FDF0}', '\u{10000}'] {
        assert!(is_name_start_char(start), "{start:?}");
    }
    for other in ['-', '.', '0', '\u{B7}', '\u{300}', '\u{203F}'] {
        assert!(!is_name_start_char(other) && is_name_char(other), "{other:?}");
    }
    for refused in [' ', '<', '=', '\u{D7}', '\u{F7}', '\u{2000}', '\u{E000}'] {
        assert!(!is_name_start_char(refused) && !is_name_char(refused), "{refused:?}");
    }
    for name in ["a", "_", "a:b", "x-y.z9", "\u{e9}t\u{e9}"] {
        assert!(is_xml_name(name), "{name}");
    }
    for name in ["", "1a", "-a", "a b", "a<b", "a=", ".x"] {
        assert!(!is_xml_name(name), "{name}");
    }
}

/// Check the layout probes on LF,
/// CRLF,
/// bare CR,
/// tabs and missing indentation.
#[test]
fn layout_probes_read_the_source() {
    assert_eq!(detect_eol("a\nb"), "\n");
    assert_eq!(detect_eol("a\r\nb\n"), "\r\n");
    assert_eq!(detect_eol("a\rb"), "\r");
    assert_eq!(detect_eol("none"), "\n");
    let source: &str = "<a>\n\t<b/>\n  <c/>\n</a>";
    assert_eq!(line_start(source, 6), 4);
    assert_eq!(line_start(source, 0), 0);
    assert_eq!(line_indent(source, 6), "\t");
    assert_eq!(line_indent(source, 0), "");
    assert!(starts_own_line(source, 5));
    assert!(!starts_own_line(source, 9));
    assert_eq!(detect_indent_unit(source), "\t");
    assert_eq!(detect_indent_unit("<a>\r\n    <b/>\r\n</a>"), "    ");
    assert_eq!(detect_indent_unit("<a><b/></a>"), "  ");
    assert_eq!(detect_indent_unit("<a>\n  text\n</a>"), "  ");
    assert_eq!(blank_tail_end("<b/>  \nrest", 4), Some(7));
    assert_eq!(blank_tail_end("<b/>\r\nrest", 4), Some(6));
    assert_eq!(blank_tail_end("<b/>\rrest", 4), Some(5));
    assert_eq!(blank_tail_end("<b/>", 4), Some(4));
    assert_eq!(blank_tail_end("<b/> <c/>", 4), None);
}

/// Check the element description builders fill the public fields.
#[test]
fn element_specs_build_up() {
    let spec: XmlElementSpec = XmlElementSpec::named("a")
        .with_attribute("k", "v")
        .with_child(XmlElementSpec::named("b"))
        .with_text("t");
    assert_eq!(spec.name, "a");
    assert_eq!(spec.attributes, vec![("k".to_string(), "v".to_string())]);
    assert_eq!(spec.children[0], XmlNodeSpec::Element(XmlElementSpec::named("b")));
    assert_eq!(spec.children[1], XmlNodeSpec::Text("t".to_string()));
}

/// Check every edit refusal prints a sentence naming its input.
#[test]
fn edit_errors_print_their_cause() {
    let inner: XmlParseError = parse_xml("<a>").expect_err("unclosed");
    let cases: [(XmlEditError, &str); 8] = [
        (XmlEditError::UnknownElement, "does not belong"),
        (XmlEditError::AttributeNotFound { name: "k".to_string() }, "\"k\""),
        (XmlEditError::InvalidName { name: "1x".to_string() }, "\"1x\""),
        (XmlEditError::DuplicateAttribute { name: "k".to_string() }, "more than once"),
        (XmlEditError::InvalidCharacter { context: "text content".to_string(), character: '\u{1}' }, "U+0001"),
        (XmlEditError::MixedContent { element: "a".to_string() }, "\"a\""),
        (XmlEditError::CannotRemoveRoot, "no root"),
        (XmlEditError::RejectedResult(inner), "invalid XML"),
    ];
    for (error, needle) in cases {
        assert!(error.to_string().contains(needle), "{error}");
        let boxed: Box<dyn std::error::Error> = Box::new(error);
        assert!(!boxed.to_string().is_empty());
    }
    let parse: Box<dyn std::error::Error> = Box::new(XmlParseError::at("abc", 1, "m"));
    assert_eq!(parse.to_string(), "m (line 1, column 2, byte offset 1)");
}

/// Check the safety net:
/// a splice that would break well-formedness is refused and the original is unchanged.
#[test]
fn splice_refuses_to_return_malformed_xml() {
    let document = parse_xml("<a x=\"1\"/>").expect("valid");
    let refused = document.splice(0, 0, "<<").expect_err("malformed result");
    assert!(matches!(refused, XmlEditError::RejectedResult(_)));
    assert_eq!(document.source(), "<a x=\"1\"/>");
    let same = document.splice(0, 0, "").expect("no-op splice");
    assert_eq!(same, document);
}
