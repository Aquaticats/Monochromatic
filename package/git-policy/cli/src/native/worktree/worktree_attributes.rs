//! What:
//!  Git's attribute-file lexer and pattern matcher over raw bytes,
//! as far as a refusal needs them:
//! which lines of an
//!  attributes file set an attribute that can change a file's bytes on the way into the index,
//! and whether the pattern of
//!  such a line may apply to a path.
//! Why:
//!  Paths and patterns are byte strings to Git,
//! and `attr.c` splits a line on the four blank bytes space,
//! tab,
//! carriage
//!  return and line feed only,
//! ends a line's text at a NUL,
//! and skips a UTF-8 byte order mark on the first line of a worktree
//!  file.
//! Any other reading of the same bytes (Unicode whitespace,
//! lossy decoding,
//! Unicode case folding) can miss a line Git
//!  applies,
//! and a miss accepts a file whose bytes Git would convert.
//! Every spelling the matcher does not model answers that
//!  the line may apply,
//! so the error is always a refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = parseAttributeLines(bytes); // Buffer -> { subject: Buffer; isMacro: boolean; tokens: Buffer[] }[]
//! mayApply(pattern, base, path); // true unless the pattern provably misses the path
//! ```

/// The attributes that change a file's bytes on the way into the index,
/// besides the line-ending ones.
pub const CONVERTING_ATTRIBUTES: &[&[u8]] = &[b"filter", b"ident", b"working-tree-encoding"];

/// The four bytes `attr.c` treats as blank.
const BLANK: &[u8] = b" \t\r\n";

/// The marker of a macro definition.
const MACRO_PREFIX: &[u8] = b"[attr]";

/// The most cells of the matcher's table that one comparison may fill;
/// a larger comparison answers that the pattern may apply.
pub const MATCH_WORK_LIMIT: usize = 1 << 22;

/// The UTF-8 byte order mark `read_attr_from_file` removes from the first line of a worktree file.
const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];

/// One line of an attributes file:
/// a pattern with its attribute tokens,
/// or a macro definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeLine {
    /// The pattern,
    /// or the macro name when `is_macro`.
    pub subject: Vec<u8>,
    /// Whether the line defines a macro (`[attr]name ...`).
    pub is_macro: bool,
    /// Whether the line starts with a quote;
    /// its pattern,
    /// its token boundary and its macro marker are not modelled,
    /// so a
    ///  caller treats the line as applying to every path below its directory.
    pub quoted: bool,
    /// The attribute tokens as written (`filter=lfs`,
    /// `-text`,
    /// `ident`).
    pub tokens: Vec<Vec<u8>>,
}

/// What:
///  Whether a byte separates the pieces of an attribute line:
/// a space or a tab.
/// Why:
///   Pieces are the words between blanks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// BLANK.includes(byte)
/// ```
fn is_blank(byte: &u8) -> bool {
    return BLANK.contains(byte);
}

/// What:
///  Whether a byte is not a blank.
/// Why:
///   The first byte that is not a blank starts the line's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// !BLANK.includes(byte)
/// ```
fn is_not_blank(byte: &u8) -> bool {
    return !BLANK.contains(byte);
}

/// What:
///  Whether a piece split out of a line holds any byte.
/// Why:
///   Runs of blanks leave empty pieces between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// piece.length > 0
/// ```
fn is_nonempty_piece(piece: &&[u8]) -> bool {
    return !piece.is_empty();
}

/// What:
///  Whether a byte is a line feed.
/// Why:
///   The file is read line by line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0x0a
/// ```
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether a byte is NUL.
/// Why:
///   Git reads a line only up to a NUL.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether a byte is a slash.
/// Why:
///   Paths and patterns are split on their slashes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0x2f
/// ```
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Whether a byte is an equals sign.
/// Why:
///   An attribute token is its name,
/// then `=` and a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0x3d
/// ```
fn is_equals_sign(byte: &u8) -> bool {
    return *byte == b'=';
}

/// What:
///  Whether a byte is neither `-` nor `!`.
/// Why:
///   Those two marks in front of an attribute name unset or reset it,
/// and are not part of the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte !== 0x2d && byte !== 0x21
/// ```
fn is_not_negation_mark(byte: &u8) -> bool {
    return *byte != b'-' && *byte != b'!';
}

/// What:
///  Whether a byte is a backslash or an opening bracket.
/// Why:
///   A pattern with either is not compared by the cheap rules,
/// so the file may apply to any path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0x5c || byte === 0x5b
/// ```
fn is_backslash_or_bracket(byte: &u8) -> bool {
    return *byte == b'\\' || *byte == b'[';
}

/// What:
///  Whether two bytes are `**`.
/// Why:
///   A double star crosses directories,
/// so the cheap rules cannot decide such a pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pair === '**'
/// ```
fn is_double_star(pair: &[u8]) -> bool {
    return pair == b"**";
}

/// The pieces of `line` separated by blank bytes.
fn blank_separated(line: &[u8]) -> Vec<Vec<u8>> {
    return line.split(is_blank).filter(is_nonempty_piece).map(<[u8]>::to_vec).collect();
}

/// What:
///  The lines of an attributes file's bytes that set something:
/// patterns with tokens,
/// and macro definitions.
/// Why:
///   A comment or blank line applies nothing.
/// A line ends at its first NUL,
/// as a C string does,
/// and the lines after it are
///  still read,
/// which can only make more lines count.
/// A quoted line is only flagged,
/// because its true pattern,
/// its token
///  boundary and its macro marker depend on C-style unquoting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAttributeLines(bytes: Buffer, worktreeFile: boolean): AttributeLine[];
/// ```
pub fn parse_attribute_lines(bytes: &[u8]) -> Vec<AttributeLine> {
    let mut lines: Vec<AttributeLine> = Vec::new();
    for (number, raw) in bytes.split(is_newline).enumerate() {
        let without_mark: &[u8] = if number == 0 { raw.strip_prefix(BYTE_ORDER_MARK).unwrap_or(raw) } else { raw };
        let text: &[u8] = without_mark.split(is_nul).next().unwrap_or(&[]);
        let start: usize = text.iter().position(is_not_blank).unwrap_or(text.len());
        let line: &[u8] = &text[start..];
        if line.is_empty() || line[0] == b'#' {
            continue;
        }
        if line[0] == b'"' {
            // A quoted line is never read for its tokens or its macro marker: the caller refuses on the quote itself.
            lines.push(AttributeLine { subject: line.to_vec(), is_macro: false, quoted: true, tokens: Vec::new() });
            continue;
        }
        let mut pieces: Vec<Vec<u8>> = blank_separated(line);
        let name: Vec<u8> = pieces.remove(0);
        if name.len() > MACRO_PREFIX.len() && name.starts_with(MACRO_PREFIX) {
            lines.push(AttributeLine { subject: name[MACRO_PREFIX.len()..].to_vec(), is_macro: true, quoted: false, tokens: pieces });
        } else {
            lines.push(AttributeLine { subject: name, is_macro: false, quoted: false, tokens: pieces });
        }
    }
    return lines;
}

/// The attribute a token sets,
/// unsets or leaves unspecified,
/// without the sign and the value.
pub fn attribute_name(token: &[u8]) -> &[u8] {
    let start: usize = token.iter().position(is_not_negation_mark).unwrap_or(token.len());
    let rest: &[u8] = &token[start..];
    return rest.split(is_equals_sign).next().unwrap_or(&[]);
}

/// What:
///  Whether `text` matches the shell pattern `pattern`,
/// byte for byte with ASCII case folded,
/// where `*` and `?` stop at a
///  slash.
/// Why:
///   Git's wildmatch compares bytes and folds ASCII only;
/// folding more than Git does can only make a match more likely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wildcard(pattern: Buffer, text: Buffer): boolean;
/// ```
pub fn wildcard(pattern: &[u8], text: &[u8]) -> bool {
    // `reach[j]` says the pattern consumed so far can match the first `j` bytes of the text; two rows are reused.
    let mut reach: Vec<bool> = vec![false; text.len() + 1];
    let mut next: Vec<bool> = vec![false; text.len() + 1];
    reach[0] = true;
    for &symbol in pattern {
        next.fill(false);
        if symbol == b'*' {
            next[0] = reach[0];
            for at in 1..=text.len() {
                next[at] = reach[at] || (next[at - 1] && text[at - 1] != b'/');
            }
        } else {
            for at in 1..=text.len() {
                let same: bool = if symbol == b'?' { text[at - 1] != b'/' } else { symbol.eq_ignore_ascii_case(&text[at - 1]) };
                next[at] = reach[at - 1] && same;
            }
        }
        std::mem::swap(&mut reach, &mut next);
        if !reach.contains(&true) {
            return false;
        }
    }
    return reach[text.len()];
}

/// What:
///  Whether the pattern of an attributes line in the directory `base` may apply to the file `path`;
/// a spelling the matcher
///  does not model (a quote,
/// an escape,
/// a bracket class,
/// `**`,
/// a trailing slash),
/// or whose comparison would exceed the work
///  budget,
/// answers yes.
/// Why:
///   Answering yes refuses a file that Git might not have matched;
/// answering no for a file it matches would accept one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayApply(pattern: Buffer, base: Buffer, path: Buffer): boolean;
/// ```
pub fn may_apply(pattern: &[u8], base: &[u8], path: &[u8]) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if pattern[0] == b'"' || pattern.iter().any(is_backslash_or_bracket) || pattern.windows(2).any(is_double_star) || pattern.ends_with(b"/") {
        return true;
    }
    let relative: &[u8] = if base.is_empty() {
        path
    } else {
        let below_base: Option<&[u8]> = match path.strip_prefix(base) {
            Some(rest) => rest.strip_prefix(b"/"),
            None => None,
        };
        match below_base {
            Some(rest) => rest,
            None => return false,
        }
    };
    let compared: &[u8] = if pattern.contains(&b'/') { relative } else { relative.rsplit(is_slash).next().unwrap_or(&[]) };
    // The table has one cell per pattern byte and text byte; past the budget the comparison is not made and the answer is yes.
    if pattern.len().saturating_mul(compared.len() + 1) > MATCH_WORK_LIMIT {
        return true;
    }
    if pattern.contains(&b'/') {
        return wildcard(pattern.strip_prefix(b"/").unwrap_or(pattern), compared);
    }
    return wildcard(pattern, compared);
}

#[cfg(test)]
#[path = "worktree_attributes_tests.rs"]
mod tests;
