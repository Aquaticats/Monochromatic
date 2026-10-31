//! What:
//!  The iterative XML parser: prolog, one element tree built with an explicit stack, then the epilog.
//! Why:
//!  A recursive-descent parser would use call-stack depth proportional to element nesting; an explicit
//!  `Vec` of open elements keeps the stack constant and the whole parse one linear pass over the source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function parseXml(source: string): XmlDocument;
//! ```

use crate::coding::decode_text;
use crate::document::{ElementData, XmlDocument, XmlElementId, XmlSpan};
use crate::chars::is_xml_space;
use crate::error::XmlParseError;
use crate::scan::{Scanner, StartTag};

/// What: Record that a non-element child node ended at `end` inside the element at the top of the stack.
/// Why: Insertions place new children after the last real content, so each parser branch reports where it stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteContent(elements: ElementData[], parent: number, end: number): void;
/// ```
fn note_content(elements: &mut [ElementData], parent: usize, end: usize) {
    elements[parent].last_content_end = Some(end);
}

/// What: Skip the prolog (BOM, declaration, comments, processing instructions) and return the root's offset.
/// Why: Before the root only those constructs and whitespace are well-formed; a DOCTYPE is out of scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseProlog(scanner: Scanner): number;
/// ```
fn parse_prolog(scanner: &mut Scanner<'_>) -> Result<(), XmlParseError> {
    if scanner.at_declaration() {
        scanner.scan_declaration()?;
    }
    loop {
        scanner.skip_spaces();
        if scanner.starts_with("<!--") {
            scanner.scan_comment()?;
        } else if scanner.starts_with("<?") {
            scanner.scan_processing_instruction()?;
        } else if scanner.starts_with("<!") {
            return Err(scanner.fail(scanner.offset, "DOCTYPE and other markup declarations are outside the supported XML grammar"));
        } else if scanner.peek(0) == Some(b'<') {
            return Ok(());
        } else if scanner.peek(0).is_none() {
            return Err(scanner.fail(scanner.offset, "the document has no root element"));
        } else {
            return Err(scanner.fail(scanner.offset, "text is not allowed before the root element"));
        }
    }
}

/// What: Check that only whitespace, comments and processing instructions follow the root element.
/// Why: A second root element or stray text after the root makes the document malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseEpilog(scanner: Scanner): void;
/// ```
fn parse_epilog(scanner: &mut Scanner<'_>) -> Result<(), XmlParseError> {
    loop {
        scanner.skip_spaces();
        if scanner.peek(0).is_none() {
            return Ok(());
        } else if scanner.starts_with("<!--") {
            scanner.scan_comment()?;
        } else if scanner.starts_with("<?") {
            scanner.scan_processing_instruction()?;
        } else {
            return Err(scanner.fail(scanner.offset, "only comments and whitespace may follow the root element"));
        }
    }
}

/// What: Append a freshly scanned start tag to the arena and link it to its parent.
/// Why: Ids are the arena indices, so pushing in start-tag order makes every subtree a contiguous id range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openElement(elements: ElementData[], stack: number[], tag: StartTag, start: number): number;
/// ```
fn open_element(elements: &mut Vec<ElementData>, stack: &[usize], tag: StartTag, start: usize) -> usize {
    let id: usize = elements.len();
    let mut parent: Option<XmlElementId> = None;
    if let Some(index) = stack.last() {
        parent = Some(XmlElementId(*index));
    }
    elements.push(ElementData {
        name: tag.name,
        start_tag: XmlSpan { start, end: tag.end },
        end_tag: None,
        parent,
        children: Vec::new(),
        attributes: tag.attributes,
        subtree_end: id,
        text: String::new(),
        has_text: false,
        has_other_markup: false,
        last_content_end: None,
    });
    if let Some(parent_id) = parent {
        elements[parent_id.0].children.push(XmlElementId(id));
    }
    return id;
}

/// What: Parse the element tree starting at the root's `<`, using an explicit stack of open element ids.
/// Why: This is the single pass that checks tag matching, text, comments, CDATA and nesting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseTree(scanner: Scanner): ElementData[];
/// ```
fn parse_tree(scanner: &mut Scanner<'_>) -> Result<Vec<ElementData>, XmlParseError> {
    let mut elements: Vec<ElementData> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    loop {
        let at: usize = scanner.offset;
        let Some(parent) = stack.last().copied() else {
            // No open element: this is the root's start tag.
            let tag: StartTag = scanner.scan_start_tag()?;
            let self_closing: bool = tag.self_closing;
            let id: usize = open_element(&mut elements, &stack, tag, at);
            if self_closing {
                return Ok(elements);
            }
            stack.push(id);
            continue;
        };
        if scanner.peek(0).is_none() {
            let open_at: usize = elements[parent].start_tag.start;
            return Err(scanner.fail(open_at, "this element is never closed by an end tag"));
        }
        if scanner.starts_with("</") {
            scanner.offset += 2;
            let name: XmlSpan = scanner.scan_name("an element name after </")?;
            scanner.skip_spaces();
            if scanner.peek(0) != Some(b'>') {
                return Err(scanner.fail(scanner.offset, "expected > to finish this end tag"));
            }
            scanner.offset += 1;
            let open: XmlSpan = elements[parent].name;
            if scanner.source[name.start..name.end] != scanner.source[open.start..open.end] {
                let message: String = format!(
                    "end tag </{}> does not match the start tag <{}> opened at byte offset {}",
                    &scanner.source[name.start..name.end],
                    &scanner.source[open.start..open.end],
                    open.start - 1
                );
                return Err(scanner.fail(at, &message));
            }
            elements[parent].end_tag = Some(XmlSpan { start: at, end: scanner.offset });
            elements[parent].subtree_end = elements.len() - 1;
            stack.pop();
            if let Some(grandparent) = stack.last().copied() {
                note_content(&mut elements, grandparent, scanner.offset);
            } else {
                return Ok(elements);
            }
        } else if scanner.starts_with("<!--") {
            scanner.scan_comment()?;
            elements[parent].has_other_markup = true;
            note_content(&mut elements, parent, scanner.offset);
        } else if scanner.starts_with("<![CDATA[") {
            let (_, content): (XmlSpan, XmlSpan) = scanner.scan_cdata()?;
            elements[parent].text.push_str(&scanner.source[content.start..content.end]);
            elements[parent].has_text = true;
            note_content(&mut elements, parent, scanner.offset);
        } else if scanner.starts_with("<?") {
            scanner.scan_processing_instruction()?;
            elements[parent].has_other_markup = true;
            note_content(&mut elements, parent, scanner.offset);
        } else if scanner.starts_with("<!") {
            return Err(scanner.fail(at, "this markup declaration is outside the supported XML grammar"));
        } else if scanner.peek(0) == Some(b'<') {
            let tag: StartTag = scanner.scan_start_tag()?;
            let self_closing: bool = tag.self_closing;
            let end: usize = tag.end;
            let id: usize = open_element(&mut elements, &stack, tag, at);
            if self_closing {
                note_content(&mut elements, parent, end);
            } else {
                stack.push(id);
            }
        } else {
            let length: usize = scanner.source[at..].find('<').unwrap_or(scanner.source.len() - at);
            let raw: &str = &scanner.source[at..at + length];
            let decoded: String = match decode_text(raw) {
                Ok(text) => text,
                Err(error) => return Err(scanner.fail(at + error.offset, &error.message)),
            };
            elements[parent].text.push_str(&decoded);
            scanner.offset = at + length;
            let mut has_visible: bool = false;
            for byte in raw.bytes() {
                if !is_xml_space(byte) {
                    has_visible = true;
                    break;
                }
            }
            if has_visible {
                elements[parent].has_text = true;
                note_content(&mut elements, parent, scanner.offset);
            }
        }
    }
}

/// What:
///  Parse XML text into a document, or explain exactly where and why it is malformed or out of scope.
/// Why:
///  This is the only constructor of [`XmlDocument`], so holding one proves the text passed every check:
///  one root, matching tags, unique attributes, resolvable references, legal characters, no DOCTYPE.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseXml(source: string): XmlDocument; // throws XmlParseError
/// ```
pub fn parse_xml(source: &str) -> Result<XmlDocument, XmlParseError> {
    let has_bom: bool = source.starts_with('\u{FEFF}');
    let mut scanner: Scanner<'_> = Scanner { source, offset: if has_bom { 3 } else { 0 } };
    parse_prolog(&mut scanner)?;
    let elements: Vec<ElementData> = parse_tree(&mut scanner)?;
    parse_epilog(&mut scanner)?;
    return Ok(XmlDocument { source: source.to_string(), elements, has_bom });
}
