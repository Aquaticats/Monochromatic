//! What:
//!  Entity and character-reference coding for attribute values and text content,
//! in both directions.
//! Why:
//!  Reading an attribute must give the text the XML processor would give (references resolved,
//! literal tabs
//!  and line breaks turned into spaces),
//! and writing one must produce text that reads back unchanged.
//!  Each direction is one linear pass over the input.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function decodeAttributeValue(raw: string): string;
//! export function encodeAttributeValue(text: string, quote: '"' | "'"): string;
//! ```

/// What:
/// Import the legal-character test.
/// Why:
/// Both directions reject characters XML 1.0 cannot carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isXmlChar } from './chars';
/// ```
use crate::chars::is_xml_char;
/// What:
/// Import the edit failure type.
/// Why:
/// Encoding refuses unrepresentable characters with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { XmlEditError } from './error';
/// ```
use crate::error::XmlEditError;

/// Longest reference body (`#x10FFFF`,
/// with leading zeros) the decoder looks for before giving up.
/// A fixed bound keeps the search for `;` constant work per `&`,
/// so decoding stays linear.
const MAX_REFERENCE_BODY: usize = 32;

/// What:
///  A decoding rejection with the byte offset inside the raw text where it happened.
/// Why:
///  The parser adds the raw text's own position in the document to turn this into a positioned diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CodingError = { offset: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingError {
    /// Zero-based byte offset into the raw text that was decoded.
    pub offset: usize,
    /// Explanation naming the malformed input.
    pub message: String,
}

/// What:
///  Resolve the text between `&` and `;` to a character,
/// for the five predefined entities and numeric references.
/// Why:
///  No DTD is supported,
/// so any other name is an undefined entity and the document is malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveReference(body: string): string | undefined;
/// ```
fn resolve_reference(body: &str) -> Option<char> {
    if body == "lt" {
        // What: `Some('<')` wraps the character in the present variant of `Option` (TS `T | undefined`).
        // Why: The caller needs to tell "resolved" from "not a valid reference".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return '<';
        // ```
        return Some('<');
    }
    if body == "gt" {
        return Some('>');
    }
    if body == "amp" {
        return Some('&');
    }
    if body == "quot" {
        return Some('"');
    }
    if body == "apos" {
        return Some('\'');
    }
    // What: `strip_prefix` returns the text after the prefix when it matches, else `None`.
    // Why: `&#x41;` is hexadecimal and `&#65;` is decimal; only the lowercase `x` is legal XML.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hex = body.startsWith('#x') ? body.slice(2) : undefined;
    // ```
    let (digits, radix): (&str, u32) = if let Some(hex) = body.strip_prefix("#x") {
        (hex, 16)
    } else {
        let decimal: &str = body.strip_prefix('#')?;
        (decimal, 10)
    };
    // What: `u32::from_str_radix` accepts a leading `+`, so the digits are checked first; `.ok()?` turns an
    //  overflow `Err` into `None` and the `?` returns that `None` from this function.
    // Why: A code point above `u32::MAX` or a sign must not slip through as a valid reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!/^[0-9a-fA-F]+$/.test(digits)) return undefined; const code = parseInt(digits, radix);
    // ```
    if digits.is_empty() {
        return None;
    }
    for digit in digits.chars() {
        if !digit.is_digit(radix) {
            return None;
        }
    }
    let code: u32 = u32::from_str_radix(digits, radix).ok()?;
    let character: char = char::from_u32(code)?;
    if !is_xml_char(character) {
        return None;
    }
    return Some(character);
}

/// What:
///  Decode raw markup text,
/// resolving references and applying the whitespace rules for the given context.
/// Why:
///  Attribute values and text content share every rule except whitespace and `]]>`,
/// so one loop serves both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeContent(raw: string, isAttribute: boolean): string;
/// ```
fn decode_content(raw: &str, is_attribute: bool) -> Result<String, CodingError> {
    let mut output: String = String::with_capacity(raw.len());
    let mut characters = raw.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if !is_xml_char(character) {
            let message: String = format!("U+{:04X} is not a character XML 1.0 allows", u32::from(character));
            return Err(CodingError { offset: index, message });
        }
        if character == '<' {
            return Err(CodingError { offset: index, message: "a literal < must be written as &lt;".to_string() });
        }
        if character == '&' {
            // Look at most MAX_REFERENCE_BODY + 1 bytes ahead for the closing semicolon.
            let ahead_end: usize = std::cmp::min(raw.len(), index + 1 + MAX_REFERENCE_BODY + 1);
            let ahead: &[u8] = &raw.as_bytes()[index + 1..ahead_end];
            let mut semicolon: Option<usize> = None;
            for (position, byte) in ahead.iter().enumerate() {
                if *byte == b';' {
                    semicolon = Some(position);
                    break;
                }
            }
            let Some(length) = semicolon else {
                let message: &str = "a reference starting with & has no closing ; within a reasonable length";
                return Err(CodingError { offset: index, message: message.to_string() });
            };
            let body: &str = &raw[index + 1..index + 1 + length];
            let Some(resolved) = resolve_reference(body) else {
                let message: String = format!("&{body}; is not a predefined entity or a valid character reference");
                return Err(CodingError { offset: index, message });
            };
            output.push(resolved);
            // Skip the body and the semicolon (all ASCII, so byte offsets equal character counts).
            for _ in 0..=length {
                characters.next();
            }
        } else if is_attribute && (character == '\t' || character == '\n') {
            output.push(' ');
        } else if character == '\r' {
            // CRLF is one line break; a bare CR is one too.
            if let Some((_, following)) = characters.peek()
                && *following == '\n'
            {
                characters.next();
            }
            output.push(if is_attribute { ' ' } else { '\n' });
        } else if !is_attribute && character == '>' && raw[..index].ends_with("]]") {
            return Err(CodingError { offset: index - 2, message: "]]> is not allowed in text content".to_string() });
        } else {
            output.push(character);
        }
    }
    return Ok(output);
}

/// What:
///  Decode the raw text between an attribute's quotes into its value.
/// Why:
///  XML 1.0 section 3.3.3:
/// a literal tab,
/// line feed or carriage return in a value becomes one space,
///  while a character reference such as `&#10;` stays the real character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeAttributeValue(raw: string): string;
/// ```
pub fn decode_attribute_value(raw: &str) -> Result<String, CodingError> {
    return decode_content(raw, true);
}

/// What:
///  Decode raw element text into its character data.
/// Why:
///  Text keeps its line breaks,
/// but CRLF and bare CR are normalized to LF as XML 1.0 section 2.11 requires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeText(raw: string): string;
/// ```
pub fn decode_text(raw: &str) -> Result<String, CodingError> {
    return decode_content(raw, false);
}

/// What:
///  Encode text for an attribute written with the given quote character.
/// Why:
///  `&`,
/// `<`,
/// `>` and the quote are escaped,
/// and tab,
/// line feed and carriage return become numeric references
///  because a literal one would read back as a space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeAttributeValue(text: string, quote: '"' | "'"): string;
/// ```
pub fn encode_attribute_value(text: &str, quote: char) -> Result<String, XmlEditError> {
    let mut output: String = String::with_capacity(text.len());
    for character in text.chars() {
        if !is_xml_char(character) {
            let context: String = "attribute value".to_string();
            return Err(XmlEditError::InvalidCharacter { context, character });
        }
        if character == '&' {
            output.push_str("&amp;");
        } else if character == '<' {
            output.push_str("&lt;");
        } else if character == '>' {
            output.push_str("&gt;");
        } else if character == quote {
            output.push_str(if quote == '"' { "&quot;" } else { "&apos;" });
        } else if character == '\n' {
            output.push_str("&#10;");
        } else if character == '\r' {
            output.push_str("&#13;");
        } else if character == '\t' {
            output.push_str("&#9;");
        } else {
            output.push(character);
        }
    }
    return Ok(output);
}

/// What:
///  Encode text for element content.
/// Why:
///  `&`,
/// `<` and `>` are escaped (the last so `]]>` never appears),
/// and a carriage return becomes `&#13;`
///  because a literal one would read back as a line feed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeText(text: string): string;
/// ```
pub fn encode_text(text: &str) -> Result<String, XmlEditError> {
    let mut output: String = String::with_capacity(text.len());
    for character in text.chars() {
        if !is_xml_char(character) {
            let context: String = "text content".to_string();
            return Err(XmlEditError::InvalidCharacter { context, character });
        }
        if character == '&' {
            output.push_str("&amp;");
        } else if character == '<' {
            output.push_str("&lt;");
        } else if character == '>' {
            output.push_str("&gt;");
        } else if character == '\r' {
            output.push_str("&#13;");
        } else {
            output.push(character);
        }
    }
    return Ok(output);
}
