//! What:
//!  Linear scanning of the small XML constructs the parser drives:
//! names,
//! comments,
//! processing instructions,
//!  CDATA sections and start tags with their attributes.
//! Why:
//!  Each construct is scanned once from its opening delimiter to its closing delimiter with `str::find`,
//! so
//!  malformed input costs one pass and produces a positioned error instead of backtracking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Scanner { constructor(public source: string, public offset = 0) {} scanName(): Span; scanComment(): Span }
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
/// Import the character classes.
/// Why:
/// Names,
/// whitespace and legal characters follow the XML grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isNameChar, isNameStartChar, isXmlChar, isXmlSpace } from './chars';
/// ```
use crate::chars::{is_name_char, is_name_start_char, is_xml_char, is_xml_space};
/// What:
/// Import the attribute decoder.
/// Why:
/// Values are validated and decoded as they are scanned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { decodeAttributeValue } from './coding';
/// ```
use crate::coding::decode_attribute_value;
/// What:
/// Import the span and attribute record types.
/// Why:
/// The scanner reports where each construct sits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { AttributeData, XmlSpan } from './document';
/// ```
use crate::document::{AttributeData, XmlSpan};
/// What:
/// Import the parse failure type.
/// Why:
/// Every rejection is positioned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { XmlParseError } from './error';
/// ```
use crate::error::XmlParseError;

/// What:
///  A start tag the scanner has fully read.
/// Why:
///  The parser needs the name,
/// the attributes and whether the tag closes itself to decide whether to open a scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartTag = { name: Span; attributes: AttributeData[]; selfClosing: boolean; end: number };
/// ```
pub(crate) struct StartTag {
    /// Span of the element name.
    pub name: XmlSpan,
    /// Attributes in source order.
    pub attributes: Vec<AttributeData>,
    /// True for `<a />`.
    pub self_closing: bool,
    /// Offset one past the closing `>`.
    pub end: usize,
}

/// What:
///  A cursor over borrowed source text.
/// Why:
///  The scanner never copies the source;
/// it only moves an offset and slices what the parser keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Scanner { source: string; offset: number }
/// ```
pub(crate) struct Scanner<'a> {
    /// Borrowed UTF-8 source (`&str`,
    /// not an owned `String`,
    /// so nothing is copied).
    pub source: &'a str,
    /// Offset of the next unread byte.
    pub offset: usize,
}

/// What:
/// Add the scanning methods to the cursor.
/// Why:
/// Each construct is read by one method from its opener.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Scanner { scanName(): Span; scanComment(): Span; scanStartTag(): StartTag }
/// ```
impl<'a> Scanner<'a> {
    /// What:
    /// Build a positioned rejection at `offset`.
    /// Why:
    /// One call site shape for every scanner error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fail(offset: number, message: string): XmlParseError;
    /// ```
    pub fn fail(&self, offset: usize, message: &str) -> XmlParseError {
        return XmlParseError::at(self.source, offset, message);
    }

    /// What:
    /// Report whether the unread text starts with `prefix`.
    /// Why:
    /// Constructs are told apart by their opener.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// startsWith(prefix: string): boolean { return this.source.startsWith(prefix, this.offset); }
    /// ```
    pub fn starts_with(&self, prefix: &str) -> bool {
        return self.source[self.offset..].starts_with(prefix);
    }

    /// What:
    /// Return the byte at the cursor plus `ahead`,
    /// or `None` past the end.
    /// Why:
    /// Lookahead without slicing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// peek(ahead = 0): number | undefined;
    /// ```
    pub fn peek(&self, ahead: usize) -> Option<u8> {
        return self.source.as_bytes().get(self.offset + ahead).copied();
    }

    /// What:
    /// Advance past XML whitespace and return how many bytes were skipped.
    /// Why:
    /// Tags allow whitespace between parts,
    /// and some places require at least one byte of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// skipSpaces(): number;
    /// ```
    pub fn skip_spaces(&mut self) -> usize {
        let start: usize = self.offset;
        while let Some(byte) = self.peek(0) {
            if !is_xml_space(byte) {
                break;
            }
            self.offset += 1;
        }
        return self.offset - start;
    }

    /// What:
    /// Read an XML Name at the cursor and return its span.
    /// Why:
    /// Element and attribute names must be exact Names so they can be compared and later written back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanName(what: string): Span;
    /// ```
    pub fn scan_name(&mut self, what: &str) -> Result<XmlSpan, XmlParseError> {
        let start: usize = self.offset;
        let mut end: usize = start;
        for (index, character) in self.source[start..].char_indices() {
            let allowed: bool = if index == 0 { is_name_start_char(character) } else { is_name_char(character) };
            if !allowed {
                break;
            }
            end = start + index + character.len_utf8();
        }
        if end == start {
            return Err(self.fail(start, &format!("expected {what} here")));
        }
        self.offset = end;
        return Ok(XmlSpan { start, end });
    }

    /// What:
    /// Check that every character in `start..end` is a legal XML character.
    /// Why:
    /// Comments,
    /// CDATA and processing instructions are otherwise opaque,
    /// but control bytes still make a
    ///  document malformed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// checkChars(start: number, end: number): void;
    /// ```
    fn check_characters(&self, start: usize, end: usize) -> Result<(), XmlParseError> {
        for (index, character) in self.source[start..end].char_indices() {
            if !is_xml_char(character) {
                let message: String = format!("U+{:04X} is not a character XML 1.0 allows", u32::from(character));
                return Err(self.fail(start + index, &message));
            }
        }
        return Ok(());
    }

    /// What:
    /// Skip a comment at the cursor (`<!-- ... -->`) and return its span.
    /// Why:
    /// The body is opaque,
    /// so markup-looking text inside it is never seen as an element;
    /// `--` inside is rejected.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanComment(): Span;
    /// ```
    pub fn scan_comment(&mut self) -> Result<XmlSpan, XmlParseError> {
        let start: usize = self.offset;
        let body_start: usize = start + 4;
        let Some(length) = self.source[body_start..].find("-->") else {
            return Err(self.fail(start, "this comment is never closed with -->"));
        };
        let body_end: usize = body_start + length;
        if let Some(inside) = self.source[body_start..body_end].find("--") {
            return Err(self.fail(body_start + inside, "-- is not allowed inside a comment"));
        }
        if body_end > body_start && self.source.as_bytes()[body_end - 1] == b'-' {
            return Err(self.fail(body_end - 1, "a comment must not end with --->"));
        }
        self.check_characters(body_start, body_end)?;
        self.offset = body_end + 3;
        return Ok(XmlSpan { start, end: self.offset });
    }

    /// What:
    /// Skip a processing instruction at the cursor (`<?target ... ?>`) and return its span.
    /// Why:
    /// The XML declaration is only legal at the very start,
    /// so a target spelled `xml` is rejected here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanProcessingInstruction(): Span;
    /// ```
    pub fn scan_processing_instruction(&mut self) -> Result<XmlSpan, XmlParseError> {
        let start: usize = self.offset;
        self.offset += 2;
        let target: XmlSpan = self.scan_name("a processing instruction target")?;
        if self.source[target.start..target.end].eq_ignore_ascii_case("xml") {
            return Err(self.fail(start, "an XML declaration is only allowed at the very start of the document"));
        }
        let Some(length) = self.source[self.offset..].find("?>") else {
            return Err(self.fail(start, "this processing instruction is never closed with ?>"));
        };
        let body_end: usize = self.offset + length;
        self.check_characters(self.offset, body_end)?;
        self.offset = body_end + 2;
        return Ok(XmlSpan { start, end: self.offset });
    }

    /// What:
    /// Skip a CDATA section at the cursor and return its content span (without the delimiters).
    /// Why:
    /// The content is literal text,
    /// so `<` and `&` inside it are not markup.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanCdata(): { whole: Span; content: Span };
    /// ```
    pub fn scan_cdata(&mut self) -> Result<(XmlSpan, XmlSpan), XmlParseError> {
        let start: usize = self.offset;
        let content_start: usize = start + 9;
        let Some(length) = self.source[content_start..].find("]]>") else {
            return Err(self.fail(start, "this CDATA section is never closed with ]]>"));
        };
        let content_end: usize = content_start + length;
        self.check_characters(content_start, content_end)?;
        self.offset = content_end + 3;
        return Ok((XmlSpan { start, end: self.offset }, XmlSpan { start: content_start, end: content_end }));
    }

    /// What:
    /// Read one quoted attribute value at the cursor and return its attribute record.
    /// Why:
    /// The record keeps the value span (inside the quotes) so an edit can replace just the value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanAttribute(): AttributeData;
    /// ```
    fn scan_attribute(&mut self) -> Result<AttributeData, XmlParseError> {
        let name: XmlSpan = self.scan_name("an attribute name")?;
        self.skip_spaces();
        if self.peek(0) != Some(b'=') {
            return Err(self.fail(self.offset, "expected = after the attribute name"));
        }
        self.offset += 1;
        self.skip_spaces();
        let quote_byte: u8 = self.peek(0).unwrap_or(0);
        if quote_byte != b'"' && quote_byte != b'\'' {
            return Err(self.fail(self.offset, "an attribute value must start with a quote"));
        }
        let quote: char = char::from(quote_byte);
        let value_start: usize = self.offset + 1;
        let Some(length) = self.source[value_start..].find(quote) else {
            return Err(self.fail(self.offset, "this attribute value is never closed with its quote"));
        };
        let value: XmlSpan = XmlSpan { start: value_start, end: value_start + length };
        let decoded: String = match decode_attribute_value(&self.source[value.start..value.end]) {
            Ok(text) => text,
            Err(error) => return Err(self.fail(value.start + error.offset, &error.message)),
        };
        self.offset = value.end + 1;
        return Ok(AttributeData { name, value, quote, decoded });
    }

    /// What:
    /// Read a whole start tag at the cursor (`<name a="1" b='2'>` or `<name />`).
    /// Why:
    /// Attributes must be separated by whitespace and unique,
    /// which only the tag as a whole can check.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanStartTag(): StartTag;
    /// ```
    pub fn scan_start_tag(&mut self) -> Result<StartTag, XmlParseError> {
        self.offset += 1;
        let name: XmlSpan = self.scan_name("an element name after <")?;
        let mut attributes: Vec<AttributeData> = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();
        loop {
            let gap: usize = self.skip_spaces();
            let Some(byte) = self.peek(0) else {
                return Err(self.fail(name.start - 1, "this start tag is never closed with >"));
            };
            if byte == b'>' || byte == b'/' {
                break;
            }
            if gap == 0 {
                return Err(self.fail(self.offset, "attributes must be separated by whitespace"));
            }
            let attribute: AttributeData = self.scan_attribute()?;
            let attribute_name: &str = &self.source[attribute.name.start..attribute.name.end];
            if !seen.insert(attribute_name) {
                let message: String = format!("attribute {attribute_name:?} appears twice in this tag");
                return Err(self.fail(attribute.name.start, &message));
            }
            attributes.push(attribute);
        }
        let self_closing: bool = self.peek(0) == Some(b'/');
        if self_closing {
            if self.peek(1) != Some(b'>') {
                return Err(self.fail(self.offset, "expected > right after / in an empty-element tag"));
            }
            self.offset += 1;
        }
        self.offset += 1;
        return Ok(StartTag { name, attributes, self_closing, end: self.offset });
    }
}
