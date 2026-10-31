//! What:
//!  The emitter:
//! a small element description type and an iterative renderer that writes it as indented XML text.
//! Why:
//!  New elements are described as data (names,
//! attribute values,
//! child elements or text) and rendered here,
//!  so callers never concatenate markup and cannot break out of a tag;
//! every name and value is checked and escaped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type XmlElementSpec = { name: string; attributes: [string, string][]; children: (XmlElementSpec | string)[] };
//! function renderElement(spec: XmlElementSpec, layout: Layout): string;
//! ```

/// What:
/// Import the standard hash set (like a TS `Set<string>`).
/// Why:
/// Duplicate attribute names are caught in one pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seen = new Set<string>();
/// ```
use std::collections::HashSet;

/// What:
/// Import the name test.
/// Why:
/// Caller-supplied names are proven to be XML Names before being written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isXmlName } from './chars';
/// ```
use crate::chars::is_xml_name;
/// What:
/// Import both encoders.
/// Why:
/// Attribute values and text are escaped for their destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { encodeAttributeValue, encodeText } from './coding';
/// ```
use crate::coding::{encode_attribute_value, encode_text};
/// What:
/// Import the edit failure type.
/// Why:
/// Rendering refuses invalid descriptions with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { XmlEditError } from './error';
/// ```
use crate::error::XmlEditError;

/// What:
///  One child of a described element:
/// another element,
/// or a run of text.
/// Why:
///  An element holds either child elements or text,
/// never a mix,
/// so the renderer can lay it out unambiguously.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type XmlNodeSpec = { element: XmlElementSpec } | { text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XmlNodeSpec {
    /// A nested element.
    Element(
        /// The nested element description.
        XmlElementSpec,
    ),
    /// Character data,
    /// escaped when rendered.
    Text(
        /// The unescaped text.
        String,
    ),
}

/// What:
///  A description of an element to write:
/// tag name,
/// attributes in order,
/// and children.
/// Why:
///  The decision record asks for an emitter beside the parser;
/// this is its input,
/// validated at render time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlElementSpec { name: string; attributes: [string, string][]; children: XmlNodeSpec[] }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlElementSpec {
    /// Tag name;
    /// must be an XML Name.
    pub name: String,
    /// Attribute name and unescaped value pairs,
    /// written in this order.
    pub attributes: Vec<(String, String)>,
    /// Children;
    /// an empty list renders as a self-closing tag.
    pub children: Vec<XmlNodeSpec>,
}

/// What:
/// Add the builder methods to the element description.
/// Why:
/// Chained calls read like the markup they produce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlElementSpec { static named(name: string): XmlElementSpec; withAttribute(...): XmlElementSpec }
/// ```
impl XmlElementSpec {
    /// What:
    /// Start a description with a tag name and nothing else.
    /// Why:
    /// Builder methods then add the rest.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static named(name: string): XmlElementSpec;
    /// ```
    pub fn named(name: &str) -> XmlElementSpec {
        return XmlElementSpec { name: name.to_string(), attributes: Vec::new(), children: Vec::new() };
    }

    /// What:
    /// Return this description with one more attribute.
    /// Why:
    /// Chaining reads like the markup it produces.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withAttribute(name: string, value: string): XmlElementSpec;
    /// ```
    pub fn with_attribute(mut self, name: &str, value: &str) -> XmlElementSpec {
        self.attributes.push((name.to_string(), value.to_string()));
        return self;
    }

    /// What:
    /// Return this description with one more child element.
    /// Why:
    /// Nested options build up in one expression.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withChild(child: XmlElementSpec): XmlElementSpec;
    /// ```
    pub fn with_child(mut self, child: XmlElementSpec) -> XmlElementSpec {
        self.children.push(XmlNodeSpec::Element(child));
        return self;
    }

    /// What:
    /// Return this description with one more run of text.
    /// Why:
    /// Leaf elements such as `<name>text</name>`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withText(text: string): XmlElementSpec;
    /// ```
    pub fn with_text(mut self, text: &str) -> XmlElementSpec {
        self.children.push(XmlNodeSpec::Text(text.to_string()));
        return self;
    }
}

/// What:
///  Line ending,
/// indentation step and the indentation of the line the element's first line starts on.
/// Why:
///  Rendering copies the host document's conventions so inserted text blends in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Layout = { eol: string; unit: string; baseIndent: string };
/// ```
pub(crate) struct Layout<'a> {
    /// Line ending written between lines.
    pub eol: &'a str,
    /// One indentation step.
    pub unit: &'a str,
    /// Indentation of the element's own first line (not written before the first line).
    pub base_indent: &'a str,
}

/// What:
/// Write a start tag (and,
/// for leaves,
/// the whole element) and report whether children still need a scope.
/// Why:
/// The three layouts (empty,
/// text-only,
/// element children) share name and attribute handling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeOpening(spec: XmlElementSpec, output: string[]): boolean;
/// ```
fn write_opening(spec: &XmlElementSpec, output: &mut String) -> Result<bool, XmlEditError> {
    if !is_xml_name(&spec.name) {
        return Err(XmlEditError::InvalidName { name: spec.name.clone() });
    }
    output.push('<');
    output.push_str(&spec.name);
    let mut seen: HashSet<&str> = HashSet::new();
    for (name, value) in &spec.attributes {
        if !is_xml_name(name) {
            return Err(XmlEditError::InvalidName { name: name.clone() });
        }
        if !seen.insert(name) {
            return Err(XmlEditError::DuplicateAttribute { name: name.clone() });
        }
        output.push(' ');
        output.push_str(name);
        output.push_str("=\"");
        output.push_str(&encode_attribute_value(value, '"')?);
        output.push('"');
    }
    if spec.children.is_empty() {
        output.push_str(" />");
        return Ok(false);
    }
    let mut texts: usize = 0;
    for child in &spec.children {
        if let XmlNodeSpec::Text(_) = child {
            texts += 1;
        }
    }
    if texts == 0 {
        output.push('>');
        return Ok(true);
    }
    if texts != spec.children.len() {
        return Err(XmlEditError::MixedContent { element: spec.name.clone() });
    }
    output.push('>');
    for child in &spec.children {
        if let XmlNodeSpec::Text(text) = child {
            output.push_str(&encode_text(text)?);
        }
    }
    output.push_str("</");
    output.push_str(&spec.name);
    output.push('>');
    return Ok(false);
}

/// What:
///  Render a described element as indented XML text without a trailing line break.
/// Why:
///  An explicit stack of `(element, next child)` frames keeps the call stack flat for any nesting depth,
///  and the output is one forward pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderElement(spec: XmlElementSpec, layout: Layout): string;
/// ```
pub(crate) fn render_element(spec: &XmlElementSpec, layout: &Layout<'_>) -> Result<String, XmlEditError> {
    let mut output: String = String::new();
    let mut stack: Vec<(&XmlElementSpec, usize)> = Vec::new();
    if write_opening(spec, &mut output)? {
        stack.push((spec, 0));
    }
    while let Some(frame) = stack.last_mut() {
        let parent: &XmlElementSpec = frame.0;
        if frame.1 < parent.children.len() {
            let child: &XmlNodeSpec = &parent.children[frame.1];
            frame.1 += 1;
            if let XmlNodeSpec::Element(element) = child {
                output.push_str(layout.eol);
                output.push_str(layout.base_indent);
                output.push_str(&layout.unit.repeat(stack.len()));
                if write_opening(element, &mut output)? {
                    stack.push((element, 0));
                }
            }
        } else {
            stack.pop();
            output.push_str(layout.eol);
            output.push_str(layout.base_indent);
            output.push_str(&layout.unit.repeat(stack.len()));
            output.push_str("</");
            output.push_str(&parent.name);
            output.push('>');
        }
    }
    return Ok(output);
}
