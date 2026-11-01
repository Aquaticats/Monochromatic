//! What:
//!  Property checks shared by every fuzz target.
//! Why:
//!  Each check states one promise of `monochromatic-xml-edit` as an assertion,
//!  so a quiet campaign is evidence rather than silence.
//!  The checks take plain strings and counts where they can,
//!  so the unit tests can show each one panics on doctored input.

/// What:
/// Import the crate under test.
/// Why:
/// Invariants drive its public interface only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseXml, XmlDocument, XmlElementId, XmlElementSpec } from 'xml-edit';
/// ```
use monochromatic_xml_edit::{XmlDocument, XmlElementId, XmlElementSpec, parse_xml};

/// What:
/// Assert that `after` equals `before` outside the byte range `start..end` of `before`.
/// Why:
/// This is the preservation promise:
/// an edit may only change bytes inside the element it targets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertOutsideUnchanged(before: string, after: string, start: number, end: number): void;
/// ```
pub fn assert_outside_unchanged(before: &str, after: &str, start: usize, end: usize) {
    assert!(after.starts_with(&before[..start]), "bytes before the edited element changed");
    assert!(after.ends_with(&before[end..]), "bytes after the edited element changed");
}

/// What:
/// Assert that a document re-parses to an identical document and that every element source parses alone.
/// Why:
/// Spans recorded by the parser must stay consistent with the text they index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertSelfConsistent(document: XmlDocument): void;
/// ```
pub fn assert_self_consistent(document: &XmlDocument) {
    let again: XmlDocument = parse_xml(document.source()).expect("an accepted document must re-parse");
    assert_eq!(&again, document, "re-parsing changed the document");
    for id in document.element_ids() {
        let text: &str = document.element_source(id).expect("every issued handle has a span");
        parse_xml(text).expect("an element's own source is a well-formed document");
    }
}

/// What:
/// Assert that a rejection carries a usable position and message.
/// Why:
/// An empty message or an out-of-range position would be undiagnosable at the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertUsableRejection(source: string, error: XmlParseError): void;
/// ```
pub fn assert_usable_rejection(source: &str, offset: usize, line: usize, column: usize, message: &str) {
    assert!(!message.is_empty(), "rejection of {source:?} had no message");
    assert!(offset <= source.len(), "offset {offset} is past the end of {source:?}");
    assert!(line >= 1 && column >= 1, "line {line} column {column} is not one-based");
}

/// What:
/// Parse a candidate and assert either self-consistency or a usable rejection.
/// Why:
/// The contract for arbitrary input is a returned error,
/// never a panic,
/// overflow or hang.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertParseOrReject(candidate: string): XmlDocument | undefined;
/// ```
pub fn assert_parse_or_reject(candidate: &str) -> Option<XmlDocument> {
    return match parse_xml(candidate) {
        Ok(document) => {
            assert_self_consistent(&document);
            Some(document)
        }
        Err(error) => {
            assert_usable_rejection(candidate, error.offset, error.line, error.column, &error.message);
            None
        }
    };
}

/// What:
/// True when `value` holds a character XML 1.0 cannot carry,
/// even as a reference.
/// Why:
/// The only legitimate reason for `set_attribute` to refuse a valid name is such a character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasUnrepresentable(value: string): boolean;
/// ```
pub fn has_unrepresentable(value: &str) -> bool {
    for character in value.chars() {
        let code: u32 = u32::from(character);
        let control: bool = code < 0x20 && character != '\t' && character != '\n' && character != '\r';
        if control || character == '\u{FFFE}' || character == '\u{FFFF}' {
            return true;
        }
    }
    return false;
}

/// What:
/// Set an attribute on one element and assert the edit's promises.
/// Why:
/// Bytes outside the element are unchanged,
/// the value reads back,
/// and a refused value is only ever
/// one XML 1.0 cannot represent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertSetAttribute(document: XmlDocument, id: XmlElementId, name: string, value: string): void;
/// ```
pub fn assert_set_attribute(document: &XmlDocument, id: XmlElementId, name: &str, value: &str) {
    let span = document.span(id).expect("the handle came from this document");
    match document.set_attribute(id, name, value) {
        Ok(edited) => {
            assert_outside_unchanged(document.source(), edited.source(), span.start, span.end);
            assert_eq!(edited.attribute(id, name), Some(value), "value did not read back");
            assert_eq!(edited.element_count(), document.element_count());
            assert_eq!(edited.source().matches("<!--").count(), document.source().matches("<!--").count());
            assert_self_consistent(&edited);
        }
        Err(error) => {
            let text: String = error.to_string();
            assert!(!text.is_empty());
            assert!(has_unrepresentable(value), "a representable value {value:?} was refused: {error}");
        }
    }
}

/// What:
/// Append a child to one element and assert the edit's promises.
/// Why:
/// Prior children keep their handles' attribute values,
/// the new child is last,
/// and a refusal is only a
/// text-bearing parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertAppendChild(document: XmlDocument, parent: XmlElementId): void;
/// ```
pub fn assert_append_child(document: &XmlDocument, parent: XmlElementId) {
    let span = document.span(parent).expect("the handle came from this document");
    let spec: XmlElementSpec = XmlElementSpec::named("fuzz-child").with_attribute("k", "v\"&<>\n");
    match document.append_child(parent, &spec) {
        Ok(edited) => {
            assert_outside_unchanged(document.source(), edited.source(), span.start, span.end);
            assert_eq!(edited.element_count(), document.element_count() + 1);
            let found = edited.children(parent).last().expect("the parent has a last child");
            assert_eq!(edited.name(*found), Some("fuzz-child"));
            assert_eq!(edited.attribute(*found, "k"), Some("v\"&<>\n"));
            assert_self_consistent(&edited);
        }
        Err(error) => {
            assert!(error.to_string().contains("beside child elements"), "unexpected refusal: {error}");
        }
    }
}

/// What:
/// Remove one non-root element and assert the element count drops by exactly its subtree size.
/// Why:
/// Removal must not touch any other element,
/// whatever whitespace cleanup it performs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertRemoveElement(document: XmlDocument, id: XmlElementId): void;
/// ```
pub fn assert_remove_element(document: &XmlDocument, id: XmlElementId) {
    let text: &str = document.element_source(id).expect("the handle came from this document");
    let subtree: usize = parse_xml(text).expect("an element's own source parses").element_count();
    match document.remove_element(id) {
        Ok(edited) => {
            assert_eq!(edited.element_count() + subtree, document.element_count());
            assert_self_consistent(&edited);
        }
        Err(error) => {
            assert_eq!(id, document.root(), "only the root may be refused: {error}");
        }
    }
}
