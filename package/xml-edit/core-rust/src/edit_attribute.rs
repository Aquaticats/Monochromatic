//! What:
//!  Attribute edits:
//! set (replace or add) and remove,
//! each as a splice into the original source.
//! Why:
//!  Only the attribute's own bytes change,
//! so comments,
//! indentation,
//! quote style and attribute order elsewhere
//!  are preserved,
//! and the result is re-parsed before it is returned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! doc.setAttribute(id, 'value', '{"a": 1}'); doc.removeAttribute(id, 'obsolete');
//! ```

/// What:
/// Import the name and whitespace tests.
/// Why:
/// New attribute names are validated,
/// removal trims whitespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isXmlName, isXmlSpace } from './chars';
/// ```
use crate::chars::{is_xml_name, is_xml_space};
/// What:
/// Import the attribute encoder.
/// Why:
/// Values are escaped for the quote style in use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { encodeAttributeValue } from './coding';
/// ```
use crate::coding::encode_attribute_value;
/// What:
/// Import the document model types.
/// Why:
/// Edits read recorded spans and extend the document type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { AttributeData, ElementData, XmlDocument, XmlElementId } from './document';
/// ```
use crate::document::{AttributeData, ElementData, XmlDocument, XmlElementId};
/// What:
/// Import the edit failure type.
/// Why:
/// Every refusal is one of its variants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { XmlEditError } from './error';
/// ```
use crate::error::XmlEditError;
/// What:
/// Import the parser.
/// Why:
/// Every edit result is re-parsed before it is returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseXml } from './parse';
/// ```
use crate::parse::parse_xml;

/// What:
/// Add the attribute edits to the document.
/// Why:
/// They are operations on one parsed document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlDocument { setAttribute(...): XmlDocument; removeAttribute(...): XmlDocument }
/// ```
impl XmlDocument {
    /// What:
    ///  Build the document that results from replacing `start..end` of the source with `replacement`.
    /// Why:
    ///  Every edit funnels through here,
    /// so the result is always re-parsed:
    /// an edit can never hand back
    ///  text this crate itself would reject,
    /// and the original document stays untouched.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private splice(start: number, end: number, replacement: string): XmlDocument;
    /// ```
    pub(crate) fn splice(&self, start: usize, end: usize, replacement: &str) -> Result<XmlDocument, XmlEditError> {
        let mut edited: String = String::with_capacity(self.source.len() + replacement.len());
        edited.push_str(&self.source[..start]);
        edited.push_str(replacement);
        edited.push_str(&self.source[end..]);
        return match parse_xml(&edited) {
            Ok(document) => Ok(document),
            Err(error) => Err(XmlEditError::RejectedResult(error)),
        };
    }

    /// What:
    ///  Offset just after the last attribute (or the tag name when there is none):
    /// where a new attribute goes
    ///  and where a self-closing tag's `/>` region begins.
    /// Why:
    ///  Inserting here keeps any whitespace before `>` or `/>` exactly as the author wrote it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private attributeInsertionPoint(data: ElementData): number;
    /// ```
    pub(crate) fn attribute_insertion_point(data: &ElementData) -> usize {
        if let Some(last) = data.attributes.last() {
            return last.value.end + 1;
        }
        return data.name.end;
    }

    /// What:
    ///  Set an attribute to `value`:
    /// replace the existing value in place,
    /// or append a new attribute.
    /// Why:
    ///  The value is escaped for the quote style already in use (new attributes use double quotes),
    /// so
    ///  `"`,
    /// `&`,
    /// `<`,
    /// tabs and line breaks round-trip through [`XmlDocument::attribute`].
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setAttribute(id: XmlElementId, name: string, value: string): XmlDocument;
    /// ```
    pub fn set_attribute(&self, id: XmlElementId, name: &str, value: &str) -> Result<XmlDocument, XmlEditError> {
        let Some(data) = self.data(id) else {
            return Err(XmlEditError::UnknownElement);
        };
        if !is_xml_name(name) {
            return Err(XmlEditError::InvalidName { name: name.to_string() });
        }
        if let Some(existing) = self.attribute_data(id, name) {
            let encoded: String = encode_attribute_value(value, existing.quote)?;
            return self.splice(existing.value.start, existing.value.end, &encoded);
        }
        let encoded: String = encode_attribute_value(value, '"')?;
        let point: usize = XmlDocument::attribute_insertion_point(data);
        return self.splice(point, point, &format!(" {name}=\"{encoded}\""));
    }

    /// What:
    ///  Remove an attribute together with the whitespace that separated it from the previous token.
    /// Why:
    ///  Leaving that whitespace would strand a double space or an empty indented line inside the tag.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// removeAttribute(id: XmlElementId, name: string): XmlDocument;
    /// ```
    pub fn remove_attribute(&self, id: XmlElementId, name: &str) -> Result<XmlDocument, XmlEditError> {
        if self.data(id).is_none() {
            return Err(XmlEditError::UnknownElement);
        }
        let Some(existing) = self.attribute_data(id, name) else {
            return Err(XmlEditError::AttributeNotFound { name: name.to_string() });
        };
        let attribute: &AttributeData = existing;
        let bytes: &[u8] = self.source.as_bytes();
        let mut start: usize = attribute.name.start;
        while start > 0 && is_xml_space(bytes[start - 1]) {
            start -= 1;
        }
        return self.splice(start, attribute.value.end + 1, "");
    }
}
