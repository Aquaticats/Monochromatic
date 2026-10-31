//! What:
//!  The parsed document model: the source text plus an arena of elements that remember where every tag,
//!  name and attribute value sits in that text.
//! Why:
//!  Edits splice replacement text into those remembered byte ranges, so everything outside the edited range
//!  (comments, indentation, line endings, quote style, attribute order) is untouched by construction.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Span = { start: number; end: number };
//! class XmlDocument { source: string; elements: ElementData[] }
//! ```

/// What:
///  A half-open byte range `start..end` into the document source.
/// Why:
///  UTF-8 byte offsets slice a Rust `&str` directly and are what diagnostics report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type XmlSpan = { start: number; end: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XmlSpan {
    /// Offset of the first byte.
    pub start: usize,
    /// Offset one past the last byte.
    pub end: usize,
}

/// What:
///  A handle to one element of one parsed document, numbered in document order (the root is the first).
/// Why:
///  Handles are plain numbers, so callers hold no borrows into the document and an edit (which returns a
///  new document) never leaves a dangling reference; a handle is only meaningful for the document that issued it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type XmlElementId = number & { readonly brand: 'XmlElementId' };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct XmlElementId(pub(crate) usize);

/// What:
///  One attribute as recorded by the parser.
/// Why:
///  The name and value spans let an edit replace only the value, and the decoded value is kept so reads
///  need no second decoding pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeData = { name: Span; value: Span; quote: string; decoded: string };
/// ```
#[derive(Clone, Debug)]
pub(crate) struct AttributeData {
    /// Span of the attribute name.
    pub name: XmlSpan,
    /// Span of the raw value between the quotes.
    pub value: XmlSpan,
    /// Quote character the source used, `"` or `'`.
    pub quote: char,
    /// Value with references resolved and whitespace normalized.
    pub decoded: String,
}

/// What:
///  One element as recorded by the parser.
/// Why:
///  Tag spans give edits their splice points, and the content flags tell insertions whether a layout is safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ElementData = { name: Span; startTag: Span; endTag?: Span; parent?: number; children: number[] };
/// ```
#[derive(Clone, Debug)]
pub(crate) struct ElementData {
    /// Span of the element name inside the start tag.
    pub name: XmlSpan,
    /// Span from `<` through the closing `>` of the start tag (through `/>` for an empty-element tag).
    pub start_tag: XmlSpan,
    /// Span of the end tag, absent for an empty-element tag such as `<a />`.
    pub end_tag: Option<XmlSpan>,
    /// Parent element, absent for the root.
    pub parent: Option<XmlElementId>,
    /// Child elements in document order.
    pub children: Vec<XmlElementId>,
    /// Attributes in source order.
    pub attributes: Vec<AttributeData>,
    /// Index of the last element in this element's subtree; subtrees are contiguous because ids are pre-order.
    pub subtree_end: usize,
    /// Decoded direct character data and CDATA, whitespace between children included.
    pub text: String,
    /// True when a direct text or CDATA child holds a non-whitespace character.
    pub has_text: bool,
    /// True when a direct comment or processing instruction child exists.
    pub has_other_markup: bool,
    /// End offset of the last direct non-whitespace child node, absent for an element with no content.
    pub last_content_end: Option<usize>,
}

/// What:
///  A well-formed XML document with queryable element spans.
/// Why:
///  Parsing is the only way to build one, so every document in memory has passed the well-formedness checks,
///  and every edit re-parses its result before handing it back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlDocument { readonly source: string; root(): XmlElementId; /* queries and edits */ }
/// ```
#[derive(Clone, Debug)]
pub struct XmlDocument {
    /// The full source text, byte-for-byte as parsed (a leading BOM included).
    pub(crate) source: String,
    /// Element arena in document order.
    pub(crate) elements: Vec<ElementData>,
    /// Whether the source starts with U+FEFF.
    pub(crate) has_bom: bool,
}

impl XmlDocument {
    /// What: Return the exact source text. Why: A caller writes this back to disk after edits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get source(): string;
    /// ```
    pub fn source(&self) -> &str {
        return &self.source;
    }

    /// What: Report whether the source starts with a byte order mark. Why: Edits keep it; callers may want to know.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get hasBom(): boolean;
    /// ```
    pub fn has_bom(&self) -> bool {
        return self.has_bom;
    }

    /// What: Return the root element. Why: Every query starts from a handle, and a document always has a root.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// root(): XmlElementId { return 0; }
    /// ```
    pub fn root(&self) -> XmlElementId {
        return XmlElementId(0);
    }

    /// What: Return how many elements the document holds. Why: Handles run from zero to this count minus one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get elementCount(): number;
    /// ```
    pub fn element_count(&self) -> usize {
        return self.elements.len();
    }

    /// What: Look up an element's recorded data, or `None` for a handle this document did not issue.
    /// Why: Every public method funnels handle checks through here instead of indexing directly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private data(id: XmlElementId): ElementData | undefined { return this.elements[id]; }
    /// ```
    pub(crate) fn data(&self, id: XmlElementId) -> Option<&ElementData> {
        return self.elements.get(id.0);
    }

    /// What: Return the element's tag name, such as `option`. Why: Callers select elements by name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// name(id: XmlElementId): string | undefined;
    /// ```
    pub fn name(&self, id: XmlElementId) -> Option<&str> {
        let data: &ElementData = self.data(id)?;
        return Some(&self.source[data.name.start..data.name.end]);
    }

    /// What: Return the parent element, or `None` for the root or an unknown handle.
    /// Why: Callers walk upward to find the entry that holds an option.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parent(id: XmlElementId): XmlElementId | undefined;
    /// ```
    pub fn parent(&self, id: XmlElementId) -> Option<XmlElementId> {
        let data: &ElementData = self.data(id)?;
        return data.parent;
    }

    /// What: Return the child elements in document order; empty for an unknown handle.
    /// Why: Callers walk downward from `map` to `entry` to `option`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// children(id: XmlElementId): XmlElementId[];
    /// ```
    pub fn children(&self, id: XmlElementId) -> &[XmlElementId] {
        if let Some(data) = self.data(id) {
            return &data.children;
        }
        return &[];
    }

    /// What: Return the byte range of the whole element, start tag through end tag.
    /// Why: Callers can show or copy the exact original text of a managed block.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// span(id: XmlElementId): XmlSpan | undefined;
    /// ```
    pub fn span(&self, id: XmlElementId) -> Option<XmlSpan> {
        let data: &ElementData = self.data(id)?;
        let end: usize = if let Some(end_tag) = data.end_tag { end_tag.end } else { data.start_tag.end };
        return Some(XmlSpan { start: data.start_tag.start, end });
    }

    /// What: Return the original text of the whole element. Why: See [`XmlDocument::span`]; this slices it for you.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// elementSource(id: XmlElementId): string | undefined;
    /// ```
    pub fn element_source(&self, id: XmlElementId) -> Option<&str> {
        let span: XmlSpan = self.span(id)?;
        return Some(&self.source[span.start..span.end]);
    }
}
