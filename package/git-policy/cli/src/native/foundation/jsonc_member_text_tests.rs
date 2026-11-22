//! What:
//!  Controls of the one nested-member reader.
//! Why:
//!  Every reader of the handler's configuration tells a document it cannot
//!  parse from an object another part has not written,
//!  from a value of the wrong type,
//!  and a wrong answer would send a person to the wrong file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(textOfMember('{"a":{"b":"c"}}', 'a', 'b')).toEqual({ kind: 'text', text: 'c' });
//! ```

use super::{MemberReading, text_of_member};

/// One document with the object the controls read.
const DOCUMENT: &str = r#"{ "outer": { "inner": "value", "number": 7 }, "other": { "inner": "elsewhere" } }"#;

/// The text at a member inside an object is that member's text.
#[test]
fn a_member_inside_an_object_is_its_text() {
    assert_eq!(text_of_member(DOCUMENT, "outer", "inner"), MemberReading::Text(String::from("value")));
    assert_eq!(text_of_member(DOCUMENT, "other", "inner"), MemberReading::Text(String::from("elsewhere")));
}

/// A document that is not JSONC is its own answer,
/// so a broken file is never reported as an unwritten object.
#[test]
fn a_document_that_does_not_parse_is_not_jsonc() {
    assert_eq!(text_of_member("{ \"outer\": ", "outer", "inner"), MemberReading::DocumentNotJsonc);
    assert_eq!(text_of_member("", "outer", "inner"), MemberReading::DocumentNotJsonc);
}

/// An object or a member that is not there is absent.
#[test]
fn a_missing_object_or_member_is_absent() {
    assert_eq!(text_of_member(DOCUMENT, "absent", "inner"), MemberReading::Absent);
    assert_eq!(text_of_member(DOCUMENT, "outer", "absent"), MemberReading::Absent);
    assert_eq!(text_of_member("[]", "outer", "inner"), MemberReading::Absent);
    let missing: Option<&str> = MemberReading::Absent.text();
    assert!(missing.is_none(), "an absent member has no text");
}

/// A member whose value is not text is not text,
/// which is what a person must be told rather than "absent".
#[test]
fn a_value_of_another_type_is_not_text() {
    assert_eq!(text_of_member(DOCUMENT, "outer", "number"), MemberReading::NotText);
    assert_eq!(text_of_member(r#"{ "outer": { "inner": { "deeper": "x" } } }"#, "outer", "inner"), MemberReading::NotText);
    assert_eq!(text_of_member(r#"{ "outer": "not an object" }"#, "outer", "inner"), MemberReading::Absent);
}

/// A comment inside the document is a comment,
/// because the handler's configuration is JSONC.
#[test]
fn a_comment_is_not_a_value() {
    let commented: &str = "{\n  // the clone parent, written by the setup run\n  \"outer\": { \"inner\": \"value\" },\n}\n";
    assert_eq!(text_of_member(commented, "outer", "inner"), MemberReading::Text(String::from("value")));
}
