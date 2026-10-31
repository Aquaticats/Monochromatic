//! What:
//!  Character classes from the XML 1.0 (fifth edition) grammar:
//! legal characters,
//! name characters,
//! whitespace.
//! Why:
//!  Well-formedness checking and name validation for new elements both need the same exact classes,
//!  so they live in one small module with no other logic.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function isXmlChar(character: string): boolean;
//! export function isXmlName(text: string): boolean;
//! ```

/// What:
///  True when `character` may appear anywhere in an XML 1.0 document (the `Char` production).
/// Why:
///  Control characters other than tab,
/// line feed and carriage return,
/// and U+FFFE and U+FFFF,
/// make a document
///  malformed even inside a comment,
/// and cannot be written as a character reference either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isXmlChar(character: string): boolean;
/// ```
pub fn is_xml_char(character: char) -> bool {
    // What: `u32::from(character)` converts a `char` into its numeric code point (sibling `as u32` casts).
    // Why: The grammar is stated as code point ranges.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const codePoint = character.codePointAt(0)!;
    // ```
    let code: u32 = u32::from(character);
    if code == 0x9 || code == 0xA || code == 0xD {
        return true;
    }
    return (0x20..=0xD7FF).contains(&code) || (0xE000..=0xFFFD).contains(&code) || (0x10000..=0x10FFFF).contains(&code);
}

/// What:
///  True for the four whitespace bytes XML allows between markup:
/// space,
/// tab,
/// line feed,
/// carriage return.
/// Why:
///  XML whitespace is narrower than Unicode whitespace,
/// so `char::is_whitespace` would accept malformed input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isXmlSpace(byte: number): boolean { return [0x20, 0x9, 0xA, 0xD].includes(byte); }
/// ```
pub fn is_xml_space(byte: u8) -> bool {
    return byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r';
}

/// What:
///  True when `character` may start an XML name (the `NameStartChar` production).
/// Why:
///  Element and attribute names must start with a letter-like character,
/// underscore or colon.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNameStartChar(character: string): boolean;
/// ```
pub fn is_name_start_char(character: char) -> bool {
    let code: u32 = u32::from(character);
    if character == ':' || character == '_' || character.is_ascii_alphabetic() {
        return true;
    }
    return (0xC0..=0xD6).contains(&code)
        || (0xD8..=0xF6).contains(&code)
        || (0xF8..=0x2FF).contains(&code)
        || (0x370..=0x37D).contains(&code)
        || (0x37F..=0x1FFF).contains(&code)
        || (0x200C..=0x200D).contains(&code)
        || (0x2070..=0x218F).contains(&code)
        || (0x2C00..=0x2FEF).contains(&code)
        || (0x3001..=0xD7FF).contains(&code)
        || (0xF900..=0xFDCF).contains(&code)
        || (0xFDF0..=0xFFFD).contains(&code)
        || (0x10000..=0xEFFFF).contains(&code);
}

/// What:
///  True when `character` may continue an XML name (the `NameChar` production).
/// Why:
///  Names also allow digits,
/// hyphen,
/// period and a few combining marks after the first character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNameChar(character: string): boolean;
/// ```
pub fn is_name_char(character: char) -> bool {
    if is_name_start_char(character) {
        return true;
    }
    let code: u32 = u32::from(character);
    if character == '-' || character == '.' || character.is_ascii_digit() || code == 0xB7 {
        return true;
    }
    return (0x300..=0x36F).contains(&code) || (0x203F..=0x2040).contains(&code);
}

/// What:
///  True when `text` is a non-empty XML Name.
/// Why:
///  Element and attribute names a caller supplies are written into markup,
/// so they must be proven names
///  before they can break out of a tag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isXmlName(text: string): boolean;
/// ```
pub fn is_xml_name(text: &str) -> bool {
    // What: `chars()` yields the string's Unicode scalar values one at a time (a `char` iterator).
    // Why: Names are defined per scalar, and the first scalar follows a different rule than the rest.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const characters = [...text];
    // ```
    let mut characters: std::str::Chars<'_> = text.chars();
    // What: `if let Some(first) = ...` runs the block only when the iterator produced a value.
    // Why: An empty string has no first character and is not a name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = characters.shift(); if (first === undefined) return false;
    // ```
    if let Some(first) = characters.next() {
        if !is_name_start_char(first) {
            return false;
        }
        for character in characters {
            if !is_name_char(character) {
                return false;
            }
        }
        return true;
    }
    return false;
}
