//! What:
//!  Element-level edits: replace an element, append a child, remove an element, and set an element's text.
//! Why:
//!  A rule that manages whole `<entry>` blocks needs these beside the attribute edits; each one splices a
//!  rendered fragment (or nothing) into exactly the span it owns and re-parses the result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! doc.appendChild(map, entry); doc.replaceElement(id, spec); doc.removeElement(id); doc.setText(id, 'x');
//! ```

use crate::coding::encode_text;
use crate::document::{XmlDocument, XmlElementId};
use crate::error::XmlEditError;
use crate::layout::{blank_tail_end, detect_eol, detect_indent_unit, line_indent, line_start, starts_own_line};
use crate::render::{render_element, Layout, XmlElementSpec};

impl XmlDocument {
    /// What:
    ///  Replace a whole element, comments inside it included, with a freshly rendered description.
    /// Why:
    ///  This is the incumbent whole-entry behavior; prefer attribute edits when comments inside must survive.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// replaceElement(id: XmlElementId, spec: XmlElementSpec): XmlDocument;
    /// ```
    pub fn replace_element(&self, id: XmlElementId, spec: &XmlElementSpec) -> Result<XmlDocument, XmlEditError> {
        let Some(span) = self.span(id) else {
            return Err(XmlEditError::UnknownElement);
        };
        let unit: String = detect_indent_unit(&self.source);
        let layout: Layout<'_> = Layout {
            eol: detect_eol(&self.source),
            unit: &unit,
            base_indent: line_indent(&self.source, span.start),
        };
        let rendered: String = render_element(spec, &layout)?;
        return self.splice(span.start, span.end, &rendered);
    }

    /// What:
    ///  Append a rendered child after the parent's last content, on its own line when the parent is laid out
    ///  over several lines.
    /// Why:
    ///  This is "insert before the last `</map>`": the new child copies its siblings' indentation and the
    ///  document's line ending, and an empty or self-closing parent is expanded to hold it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// appendChild(parent: XmlElementId, spec: XmlElementSpec): XmlDocument;
    /// ```
    pub fn append_child(&self, parent: XmlElementId, spec: &XmlElementSpec) -> Result<XmlDocument, XmlEditError> {
        let Some(data) = self.data(parent) else {
            return Err(XmlEditError::UnknownElement);
        };
        if data.has_text {
            let element: String = self.source[data.name.start..data.name.end].to_string();
            return Err(XmlEditError::MixedContent { element });
        }
        let parent_indent: &str = line_indent(&self.source, data.start_tag.start);
        let unit: String = detect_indent_unit(&self.source);
        let eol: &str = detect_eol(&self.source);
        let mut child_indent: String = format!("{parent_indent}{unit}");
        if let Some(last) = data.children.last() {
            let last_start: usize = self.elements[last.0].start_tag.start;
            if starts_own_line(&self.source, last_start) {
                child_indent = line_indent(&self.source, last_start).to_string();
            }
        }
        let layout: Layout<'_> = Layout { eol, unit: &unit, base_indent: &child_indent };
        let rendered: String = render_element(spec, &layout)?;
        let Some(end_tag) = data.end_tag else {
            let name: &str = &self.source[data.name.start..data.name.end];
            let point: usize = XmlDocument::attribute_insertion_point(data);
            let opened: String = format!(">{eol}{child_indent}{rendered}{eol}{parent_indent}</{name}>");
            return self.splice(point, data.start_tag.end, &opened);
        };
        let Some(content_end) = data.last_content_end else {
            let block: String = format!("{eol}{child_indent}{rendered}{eol}{parent_indent}");
            return self.splice(data.start_tag.end, end_tag.start, &block);
        };
        let tail: &str = &self.source[content_end..end_tag.start];
        if tail.contains('\n') || tail.contains('\r') {
            return self.splice(content_end, content_end, &format!("{eol}{child_indent}{rendered}"));
        }
        return self.splice(content_end, content_end, &rendered);
    }

    /// What:
    ///  Remove an element; when it owns its lines, its indentation and line break go with it.
    /// Why:
    ///  Leaving a blank indented line behind would make every removal visible as stray whitespace in a diff.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// removeElement(id: XmlElementId): XmlDocument;
    /// ```
    pub fn remove_element(&self, id: XmlElementId) -> Result<XmlDocument, XmlEditError> {
        let Some(span) = self.span(id) else {
            return Err(XmlEditError::UnknownElement);
        };
        if id.0 == 0 {
            return Err(XmlEditError::CannotRemoveRoot);
        }
        if starts_own_line(&self.source, span.start)
            && let Some(tail_end) = blank_tail_end(&self.source, span.end)
        {
            return self.splice(line_start(&self.source, span.start), tail_end, "");
        }
        return self.splice(span.start, span.end, "");
    }

    /// What:
    ///  Replace an element's character data with `text`, escaped.
    /// Why:
    ///  Text-only elements such as `<name>x</name>` are the other value-bearing shape besides attributes.
    ///  Elements with children, comments or instructions are refused because replacing their content would
    ///  silently drop them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setText(id: XmlElementId, text: string): XmlDocument;
    /// ```
    pub fn set_text(&self, id: XmlElementId, text: &str) -> Result<XmlDocument, XmlEditError> {
        let Some(data) = self.data(id) else {
            return Err(XmlEditError::UnknownElement);
        };
        let name: &str = &self.source[data.name.start..data.name.end];
        if !data.children.is_empty() || data.has_other_markup {
            return Err(XmlEditError::MixedContent { element: name.to_string() });
        }
        let encoded: String = encode_text(text)?;
        let Some(end_tag) = data.end_tag else {
            let point: usize = XmlDocument::attribute_insertion_point(data);
            return self.splice(point, data.start_tag.end, &format!(">{encoded}</{name}>"));
        };
        return self.splice(data.start_tag.end, end_tag.start, &encoded);
    }
}
