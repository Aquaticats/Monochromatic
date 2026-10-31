//! What:
//!  Read-only questions about a parsed document: attributes, child and descendant lookup by name, text.
//! Why:
//!  A rule that edits `<option name="x" value="...">` entries first has to find them; these queries return
//!  handles and borrowed text, never copies of whole blocks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! doc.attribute(id, 'value'); doc.childrenNamed(id, 'option'); doc.findChild(id, 'option', 'name', 'x');
//! ```

use crate::document::{AttributeData, ElementData, XmlDocument, XmlElementId};

impl XmlDocument {
    /// What: Return the decoded value of the named attribute, or `None` when absent or the handle is unknown.
    /// Why: Entities are resolved, so `&quot;` reads as `"` and a caller can parse embedded JSON directly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// attribute(id: XmlElementId, name: string): string | undefined;
    /// ```
    pub fn attribute(&self, id: XmlElementId, name: &str) -> Option<&str> {
        let data: &ElementData = self.data(id)?;
        for attribute in &data.attributes {
            if &self.source[attribute.name.start..attribute.name.end] == name {
                return Some(&attribute.decoded);
            }
        }
        return None;
    }

    /// What: Return every attribute as a `(name, decoded value)` pair in source order.
    /// Why: Callers that copy a block's options need all of them without knowing the names ahead of time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// attributes(id: XmlElementId): Array<[string, string]>;
    /// ```
    pub fn attributes(&self, id: XmlElementId) -> Vec<(&str, &str)> {
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        if let Some(data) = self.data(id) {
            for attribute in &data.attributes {
                let name: &str = &self.source[attribute.name.start..attribute.name.end];
                pairs.push((name, &attribute.decoded));
            }
        }
        return pairs;
    }

    /// What: Return the child elements with the given tag name.
    /// Why: `entry` elements sit directly under `map`, so a direct-child lookup avoids matching nested lookalikes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// childrenNamed(id: XmlElementId, name: string): XmlElementId[];
    /// ```
    pub fn children_named(&self, id: XmlElementId, name: &str) -> Vec<XmlElementId> {
        let mut found: Vec<XmlElementId> = Vec::new();
        for child in self.children(id) {
            if self.name(*child) == Some(name) {
                found.push(*child);
            }
        }
        return found;
    }

    /// What: Return every descendant (not the element itself) with the given tag name, in document order.
    /// Why: Ids are numbered in pre-order, so a subtree is a contiguous id range and one linear scan finds them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// descendantsNamed(id: XmlElementId, name: string): XmlElementId[];
    /// ```
    pub fn descendants_named(&self, id: XmlElementId, name: &str) -> Vec<XmlElementId> {
        let mut found: Vec<XmlElementId> = Vec::new();
        if let Some(data) = self.data(id) {
            for index in (id.0 + 1)..=data.subtree_end {
                let candidate: XmlElementId = XmlElementId(index);
                if self.name(candidate) == Some(name) {
                    found.push(candidate);
                }
            }
        }
        return found;
    }

    /// What: Return the first direct child named `element_name` whose `attribute_name` attribute decodes to
    ///  `attribute_value`.
    /// Why: This is the `<option name="configurationContent" value="..." />` lookup in one call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// findChild(id, elementName, attributeName, attributeValue): XmlElementId | undefined;
    /// ```
    pub fn find_child(
        &self,
        id: XmlElementId,
        element_name: &str,
        attribute_name: &str,
        attribute_value: &str,
    ) -> Option<XmlElementId> {
        for child in self.children_named(id, element_name) {
            if self.attribute(child, attribute_name) == Some(attribute_value) {
                return Some(child);
            }
        }
        return None;
    }

    /// What: Return the element's direct character data and CDATA, decoded and concatenated.
    /// Why: A caller rewriting an element's text reads it first; whitespace between child elements is included.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// text(id: XmlElementId): string | undefined;
    /// ```
    pub fn text(&self, id: XmlElementId) -> Option<&str> {
        let data: &ElementData = self.data(id)?;
        return Some(&data.text);
    }

    /// What: Return the recorded attribute (spans, quote character, decoded value) with the given name.
    /// Why: Edits splice the value span and reuse the quote style, so they need more than the decoded text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// attributeData(id: XmlElementId, name: string): AttributeData | undefined;
    /// ```
    pub(crate) fn attribute_data(&self, id: XmlElementId, name: &str) -> Option<&AttributeData> {
        let data: &ElementData = self.data(id)?;
        for attribute in &data.attributes {
            if &self.source[attribute.name.start..attribute.name.end] == name {
                return Some(attribute);
            }
        }
        return None;
    }
}
