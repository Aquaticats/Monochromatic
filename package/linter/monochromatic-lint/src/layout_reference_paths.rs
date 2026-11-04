//! What:
//!  Read the module paths a file's code names,
//!  such as `crate::a::b::Item`,
//!  `super::x`,
//!  `self::y` and the leaves of `use crate::{a, b::{c, d}}`.
//! Why:
//!  The layout rules need to know which modules a file uses,
//!  and a path is the only thing that names one.
//!  The reader works on the parser's tokens,
//!  not on text,
//!  so comments,
//!  string literals and character literals can never be mistaken for code,
//!  and the same reader serves ordinary code and the inside of macro calls,
//!  which the parser leaves as loose tokens.
//!  A path that starts with a plain name counts too:
//!  the caller keeps it only when the name is a module declared next to the code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Token = { kind: TokenKind; text: string; offset: number };
//! type RawReference = { segments: string[]; glob: boolean; reexport: boolean; offset: number };
//! function readReferences(tokens: Token[], publicUses: Range[]): RawReference[];
//! ```

/// What:
///  The kinds of token the path reader tells apart.
/// Why:
///  Everything else is `Other`,
///  because the reader only cares about names,
///  `::`,
///  braces,
///  commas,
///  `*`,
///  `as` and the few tokens that stop a path from starting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TokenKind = 'crate' | 'super' | 'self' | 'name' | '::' | ':' | '{' | '}' | ',' | '*' | 'as' | '$' | '.' | 'in' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    /// The `crate` keyword.
    Crate,
    /// The `super` keyword.
    Super,
    /// The `self` keyword.
    SelfKeyword,
    /// A name,
    ///  raw (`r#type`) or plain.
    Identifier,
    /// The `::` separator.
    PathSeparator,
    /// A single `:`.
    ///  Inside a macro call the parser keeps `::` as two of these side by side,
    ///  and the collector joins them into a separator.
    Colon,
    /// An opening brace.
    OpenBrace,
    /// A closing brace.
    CloseBrace,
    /// A comma.
    Comma,
    /// The `*` of a glob.
    Star,
    /// The `as` keyword of a rename.
    As,
    /// The `$` of a macro variable such as `$crate`.
    Dollar,
    /// The `.` of a field or method access.
    Dot,
    /// The `in` keyword,
    ///  as in `pub(in crate::a)`.
    InKeyword,
    /// Any other token.
    Other,
}

/// What:
///  One token of code,
///  with its place in the file.
/// Why:
///  Findings name a file and line,
///  and a line comes from the byte offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Token = { kind: TokenKind; text: string; offset: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// Which kind of token this is.
    pub kind: TokenKind,
    /// The name with a raw prefix (`r#`) removed,
    ///  or the token text.
    pub text: String,
    /// Byte offset of the token in its file.
    pub offset: usize,
}

/// What:
///  One path the code names,
///  as written.
/// Why:
///  A group such as `crate::{a, b::c}` becomes one reference per leaf,
///  each with the full path of its leaf.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawReference = { segments: string[]; glob: boolean; reexport: boolean; offset: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawReference {
    /// The names in order,
    ///  such as `["crate", "a", "b"]`.
    pub segments: Vec<String>,
    /// True when the path ends in `::*`.
    pub glob: bool,
    /// True when the path sits inside a `pub use` (any visibility),
    ///  which makes the names it brings in visible to other modules.
    pub reexport: bool,
    /// Byte offset of the first token of the whole path.
    pub offset: usize,
}

/// What:
///  The result of reading one path or group:
///  its leaves and the index of the first token after it.
/// Why:
///  The caller continues scanning after the path,
///  so a path inside a group is not read twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Read = { leaves: RawReference[]; next: number };
/// ```
struct Read {
    /// The leaves read.
    leaves: Vec<RawReference>,
    /// Index of the first token not consumed.
    next: usize,
}

/// What:
///  Make a leaf reference from a path prefix.
/// Why:
///  Every leaf records where its whole path began.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leaf(segments: string[], glob: boolean, offset: number): RawReference;
/// ```
fn leaf(segments: &[String], glob: bool, offset: usize) -> RawReference {
    return RawReference {
        segments: segments.to_vec(),
        glob,
        reexport: false,
        offset,
    };
}

/// What:
///  The kind of the token at an index,
///  or `Other` past the end.
/// Why:
///  A missing token behaves like one that continues nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindAt(tokens: Token[], index: number): TokenKind;
/// ```
fn kind_at(tokens: &[Token], index: usize) -> TokenKind {
    return match tokens.get(index) {
        Some(token) => token.kind,
        None => TokenKind::Other,
    };
}

/// What:
///  Whether a token can be one segment of a path.
/// Why:
///  Names and the three path keywords can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSegment(kind: TokenKind): boolean;
/// ```
fn is_segment(kind: TokenKind) -> bool {
    return matches!(
        kind,
        TokenKind::Identifier | TokenKind::Crate | TokenKind::Super | TokenKind::SelfKeyword
    );
}

/// What:
///  Whether a path starts at this token.
/// Why:
///  A path starts at a segment followed by `::` and not continued from an earlier `::`,
///  a macro variable (`$crate`),
///  a field or method access (`.name::<T>`) or a visibility restriction (`pub(in crate::a)`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startsPath(tokens: Token[], index: number): boolean;
/// ```
fn starts_path(tokens: &[Token], index: usize) -> bool {
    if !is_segment(kind_at(tokens, index)) {
        return false;
    }
    if kind_at(tokens, index + 1) != TokenKind::PathSeparator {
        return false;
    }
    if index == 0 {
        return true;
    }
    return !matches!(
        kind_at(tokens, index - 1),
        TokenKind::PathSeparator | TokenKind::Dollar | TokenKind::Dot | TokenKind::InKeyword
    );
}

/// What:
///  Read the comma-separated entries of a `{ ... }` group below a prefix.
/// Why:
///  Each entry continues the prefix and may itself be a group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readGroup(tokens: Token[], open: number, prefix: string[], offset: number): Read;
/// ```
fn read_group(tokens: &[Token], open: usize, prefix: &[String], offset: usize) -> Read {
    let mut leaves: Vec<RawReference> = Vec::<RawReference>::new();
    let mut index: usize = open + 1;
    loop {
        match kind_at(tokens, index) {
            TokenKind::CloseBrace => {
                return Read {
                    leaves,
                    next: index + 1,
                };
            }
            TokenKind::Comma => index += 1,
            _ if index >= tokens.len() => {
                return Read {
                    leaves,
                    next: index,
                };
            }
            _ => {
                let entry: Read = read_entry(tokens, index, prefix, offset);
                leaves.extend(entry.leaves);
                // An entry always consumes a token, so the loop cannot stand still.
                index = entry.next.max(index + 1);
            }
        }
    }
}

/// What:
///  Read one entry of a group:
///  a path continuing the prefix,
///  `self`,
///  `*` or a nested group,
///  and skip a following `as` rename.
/// Why:
///  `use a::{self as b}` names module `a`,
///  and the rename `b` must not be read as a module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readEntry(tokens: Token[], start: number, prefix: string[], offset: number): Read;
/// ```
fn read_entry(tokens: &[Token], start: usize, prefix: &[String], offset: usize) -> Read {
    let first: TokenKind = kind_at(tokens, start);
    if first == TokenKind::OpenBrace {
        return read_group(tokens, start, prefix, offset);
    }
    if first == TokenKind::Star {
        return Read {
            leaves: vec![leaf(prefix, true, offset)],
            next: start + 1,
        };
    }
    if !is_segment(first) {
        // A stray token, such as the `as` of an earlier rename: step over it.
        return Read {
            leaves: Vec::<RawReference>::new(),
            next: start + 1,
        };
    }
    let mut current: Vec<String> = prefix.to_vec();
    let mut index: usize = start;
    loop {
        let kind: TokenKind = kind_at(tokens, index);
        // `self` alone in a group names the prefix module itself.
        let alone: bool =
            kind == TokenKind::SelfKeyword && kind_at(tokens, index + 1) != TokenKind::PathSeparator;
        if !alone {
            current.push(tokens[index].text.clone());
        }
        index += 1;
        if kind_at(tokens, index) != TokenKind::PathSeparator {
            break;
        }
        index += 1;
        let following: TokenKind = kind_at(tokens, index);
        if following == TokenKind::OpenBrace {
            return read_group(tokens, index, &current, offset);
        }
        if following == TokenKind::Star {
            return Read {
                leaves: vec![leaf(&current, true, offset)],
                next: index + 1,
            };
        }
        if !is_segment(following) {
            break;
        }
    }
    // Skip `as name`, whose name is a new binding and not a path.
    if kind_at(tokens, index) == TokenKind::As {
        index += 2;
    }
    return Read {
        leaves: vec![leaf(&current, false, offset)],
        next: index,
    };
}

/// What:
///  Read the path that starts at a token,
///  with the leaves of a group or glob it ends in.
/// Why:
///  The path stops at anything that is not `::` followed by a segment,
///  a group or `*`;
///  a turbofish (`::<T>`) ends it without adding a segment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPath(tokens: Token[], start: number): Read;
/// ```
fn read_path(tokens: &[Token], start: usize) -> Read {
    let offset: usize = tokens[start].offset;
    let mut segments: Vec<String> = vec![tokens[start].text.clone()];
    let mut index: usize = start + 1;
    while kind_at(tokens, index) == TokenKind::PathSeparator {
        index += 1;
        let kind: TokenKind = kind_at(tokens, index);
        if kind == TokenKind::OpenBrace {
            return read_group(tokens, index, &segments, offset);
        }
        if kind == TokenKind::Star {
            return Read {
                leaves: vec![leaf(&segments, true, offset)],
                next: index + 1,
            };
        }
        if !is_segment(kind) {
            break;
        }
        segments.push(tokens[index].text.clone());
        index += 1;
    }
    return Read {
        leaves: vec![leaf(&segments, false, offset)],
        next: index,
    };
}

/// What:
///  Every path the tokens name,
///  in source order.
/// Why:
///  One linear pass:
///  each path is read once and the scan resumes after it.
///  A reference whose first token lies inside one of the given byte ranges is marked as a re-export.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReferences(tokens: Token[], publicUses: Range[]): RawReference[];
/// ```
pub fn read_references(tokens: &[Token], public_uses: &[(usize, usize)]) -> Vec<RawReference> {
    let mut found: Vec<RawReference> = Vec::<RawReference>::new();
    let mut index: usize = 0;
    while index < tokens.len() {
        if !starts_path(tokens, index) {
            index += 1;
            continue;
        }
        let read: Read = read_path(tokens, index);
        for mut reference in read.leaves {
            reference.reexport = public_uses
                .iter()
                .any(|range| return reference.offset >= range.0 && reference.offset < range.1);
            found.push(reference);
        }
        index = read.next.max(index + 1);
    }
    return found;
}

/// Controls of the path reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_reference_paths_tests.rs"]
mod tests;
