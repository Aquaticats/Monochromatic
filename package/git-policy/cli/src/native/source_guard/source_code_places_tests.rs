//! What:
//!  Which bytes of a Rust source file are code,
//!  and which sit in a comment or a text literal.
//! Why:
//!  A control that reads source text for a call must not read the call out of a comment or out of a text a control plants,
//!  and the crate's own checks are the files that quote the calls they search for.
//!  Reading the place once,
//! rather than per rule,
//! keeps every rule on the same answer.
//! Gotcha:
//!  This is a reader of the two shapes that matter,
//! not a Rust parser:
//!  a character literal holding a quotation mark (`'"'`) is code and is not read as a text literal's start,
//!  while a lifetime (`'a`) is left alone.
//!  A raw text (`r#"..."#`) is followed to its own closing hashes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function isCode(text: string): boolean[] // one entry per byte
//! ```

/// Where a byte sits in a Rust source file,
/// as far as a text scan cares.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Place {
    /// Code,
    /// which no comment and no text literal covers.
    Code,
    /// A `//` comment,
    /// which runs to the end of its line.
    LineComment,
    /// A `/* */` comment,
    /// which runs to its closing `*` `/`.
    BlockComment,
    /// A `"..."` text literal,
    /// whose `\` escapes the byte after it.
    Text,
    /// A `r#"..."#` raw text,
    /// which ends at a `"` followed by its own number of `#`.
    RawText,
}

/// What:
///  `fn starts_a_raw_text(bytes: &[u8], index: usize) -> Option<usize>` answers the number of `#` characters when a raw
///  text opens at `index`,
/// and `None` when it does not.
///  `&[u8]` is a borrowed slice of bytes (siblings:
/// `Vec<u8>`,
/// the owned buffer;
/// `&str`,
/// which promises UTF-8).
///  `Option<usize>` is "a count or nothing" (sibling:
/// `usize`,
/// which would need a second value to say "none").
/// Why:
///  `r#"..."#` is the one Rust text shape a plain `"` reader gets wrong,
///  and the count of hashes is what ends it.
fn starts_a_raw_text(
    bytes: &[u8],
    index: usize,
) -> Option<usize> {
    if bytes.get(index) != Some(&b'r') {
        return None;
    }
    let mut hashes: usize = 0;
    while bytes.get(index + 1 + hashes) == Some(&b'#') {
        hashes += 1;
    }
    if bytes.get(index + 1 + hashes) != Some(&b'"') {
        return None;
    }
    return Some(hashes);
}

/// What:
///  `fn is_code(text: &str) -> Vec<bool>` answers,
/// for every byte of `text`,
/// whether that byte belongs to code.
///  `Vec<bool>` is a growable list of yes/no values (siblings:
/// `&[bool]`,
/// a borrowed view;
/// `[bool; N]`,
/// a fixed array).
/// Why:
///  A start or a signal a control only names in a comment or a planted text is no start and no signal,
///  and every scan of this module asks the same question,
///  so it is answered once,
/// here.
///  `Vec<bool>` and not a borrowed slice because the answer outlives this function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCode(text: string): boolean[] {
///   const code = new Array(text.length).fill(true);
///   // walk the text, marking every byte of a comment or a text literal false
/// }
/// ```
pub(super) fn is_code(text: &str) -> Vec<bool> {
    let bytes: &[u8] = text.as_bytes();
    let mut code: Vec<bool> = vec![true; bytes.len()];
    let mut index: usize = 0;
    let mut place: Place = Place::Code;
    // How many `#` the raw text that is open must close with.
    let mut raw_hashes: usize = 0;
    while index < bytes.len() {
        let byte: u8 = bytes[index];
        if place == Place::LineComment {
            code[index] = false;
            if byte == b'\n' {
                place = Place::Code;
            }
            index += 1;
            continue;
        }
        if place == Place::BlockComment {
            code[index] = false;
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                code[index + 1] = false;
                place = Place::Code;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if place == Place::Text {
            code[index] = false;
            // A backslash escapes the byte after it, so `\"` neither ends the text nor starts a new shape.
            if byte == b'\\' {
                if index + 1 < bytes.len() {
                    code[index + 1] = false;
                }
                index += 2;
                continue;
            }
            if byte == b'"' {
                place = Place::Code;
            }
            index += 1;
            continue;
        }
        if place == Place::RawText {
            code[index] = false;
            if byte == b'"' {
                let mut closing: usize = 0;
                while closing < raw_hashes && bytes.get(index + 1 + closing) == Some(&b'#') {
                    code[index + 1 + closing] = false;
                    closing += 1;
                }
                if closing == raw_hashes {
                    place = Place::Code;
                    index += raw_hashes + 1;
                    continue;
                }
            }
            index += 1;
            continue;
        }
        // From here the byte is code unless it opens a comment or a text literal.
        if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            code[index] = false;
            place = Place::LineComment;
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            code[index] = false;
            place = Place::BlockComment;
            index += 1;
            continue;
        }
        // A quotation mark just after a single quote is a character literal that holds one, so it opens no text.
        if byte == b'"' && (index == 0 || bytes[index - 1] != b'\'') {
            code[index] = false;
            place = Place::Text;
            index += 1;
            continue;
        }
        if byte == b'r' && starts_a_raw_text(bytes, index).is_some() {
            let hashes: usize = starts_a_raw_text(bytes, index).unwrap_or(0);
            // The opening `r`, its hashes and its `"` are all part of the text, so none of them is code.
            let after_opening: usize = index + hashes + 2;
            code[index..after_opening].fill(false);
            raw_hashes = hashes;
            place = Place::RawText;
            index = after_opening;
            continue;
        }
        index += 1;
    }
    return code;
}
