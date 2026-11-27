//! What:
//!  The fields of a commit object's content,
//!  and the subject `%s` prints for its message.
//! Why:
//!  A commit is not a tree:
//!  this file reads the header a commit states
//!  (its tree,
//! its parents in order,
//! its message after the blank line)
//!  and the first paragraph `show --no-patch --format=%s` prints.
//!  A commit with an `encoding` line or a carriage return returns `None`,
//!  because Git would re-encode or cut that text and this reader must not guess.
//!  These stood in `repo_read_tree.rs`,
//!  which re-exports them so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseCommit(bytes, format): CommitFields | undefined;
//! const subject = subjectOf(message);
//! ```

#![cfg(unix)]

use crate::repo_read_layout::ObjectFormat;

/// Whether `byte` is a newline.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// The fields of a commit the readers use.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitFields {
    /// The root tree.
    pub tree: Vec<u8>,
    /// The parents in order.
    pub parents: Vec<Vec<u8>>,
    /// The message:
    /// everything after the blank line that ends the header.
    pub message: Vec<u8>,
}

/// What:
///  The tree,
/// parents and message of a commit object's content;
///  `None` for a header with an `encoding` line (Git would re-encode the message) or a malformed one.
/// Why:
///   `show --format=%s` prints the message converted to UTF-8;
/// only a commit that needs no conversion is read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCommit(bytes: Uint8Array, format): CommitFields | undefined;
/// ```
pub fn parse_commit(
    bytes: &[u8],
    format: ObjectFormat,
) -> Option<CommitFields> {
    let mut blank_line_at: Option<usize> = None;
    for (position, pair) in bytes.windows(2).enumerate() {
        if pair == b"\n\n" {
            blank_line_at = Some(position);
            break;
        }
    }
    let split: usize = blank_line_at?;
    let header: &[u8] = &bytes[..split];
    let mut tree: Option<Vec<u8>> = None;
    let mut parents: Vec<Vec<u8>> = Vec::new();
    for (index, line) in header.split(is_newline).enumerate() {
        if let Some(text) = line.strip_prefix(b"tree ") {
            if index != 0 {
                return None;
            }
            tree = Some(crate::repo_read_object::from_hex(text, format)?);
        } else if let Some(text) = line.strip_prefix(b"parent ") {
            parents.push(crate::repo_read_object::from_hex(text, format)?);
        } else if line.starts_with(b"encoding ") {
            return None;
        }
    }
    return Some(CommitFields {
        tree: tree?,
        parents,
        message: bytes[split + 2..].to_vec(),
    });
}

/// What:
///  The subject `%s` prints for a message:
/// the first paragraph with its lines joined by spaces.
/// Why:
///   Git skips leading blank lines,
/// stops at the first blank line,
/// trims each line's trailing white space and joins
///  with a single space;
/// any message the plain rule does not cover (a carriage return) returns `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subject = message.replace(/^\n+/, '').split(/\n\s*\n/)[0].split('\n').map((l) => l.trimEnd()).join(' ');
/// ```
pub fn subject_of(message: &[u8]) -> Option<Vec<u8>> {
    if message.contains(&b'\r') || message.contains(&0) {
        return None;
    }
    let mut subject: Vec<u8> = Vec::new();
    let mut started: bool = false;
    for line in message.split(is_newline) {
        let trimmed: &[u8] = line.trim_ascii_end();
        let blank: bool = trimmed.is_empty();
        if blank && !started {
            continue;
        }
        if blank {
            break;
        }
        if started {
            subject.push(b' ');
        }
        started = true;
        subject.extend_from_slice(line.trim_ascii_end());
    }
    return Some(subject);
}
