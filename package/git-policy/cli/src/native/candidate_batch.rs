//! What:
//!  Read one `git cat-file --batch` reply from a byte stream,
//!  with no process involved.
//! Why:
//!  Object bytes are untrusted input that may contain anything,
//!  including text that
//!      looks like a reply header.
//!  The reply is therefore framed only by the header's
//!      declared size,
//!  never by searching content,
//!  and every malformed shape is a
//!      typed failure instead of a guess.
//!
//! Git 2.56.0 (`Documentation/git-cat-file.adoc`,
//!  "BATCH OUTPUT") prints,
//!  per request line:
//! `<oid> SP <type> SP <size> LF <contents> LF`,
//!  or `<request> SP missing LF`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const header = await readLine(stream); const body = await readExactly(stream, size);
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Import the validated object name and its parser.
use super::candidate_object::{ObjectId, parse_object_id};
/// What:
///  `BufRead` is the trait (interface) of buffered byte sources:
///  it adds
///       `.read_until(..)`.
///  `Read` adds `.take(..)`,
///  `.read_to_end(..)` and `.read_exact(..)`.
/// Why:
///   The same function reads a real child's pipe and an in-memory test or fuzz buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Readable } from 'node:stream';
/// ```
use std::io::{BufRead, Read};

/// What:
///  The longest reply header accepted,
///  in bytes,
///  excluding its line feed.
///       `usize` is the unsigned integer every length uses (siblings `u32`,
///  `u64`).
/// Why:
///   A real header is at most 64 name digits,
///  a type word and a decimal size
///       (under 100 bytes).
///  Without a bound,
///  a stream that never sends a line feed
///       would be buffered without limit.
///  `usize` matches `.len()`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_BATCH_HEADER_BYTES = 128;
/// ```
pub const MAX_BATCH_HEADER_BYTES: usize = 128;

/// What:
///  The four object types Git stores.
/// Why:
///   A candidate's bytes must come from a blob,
///  and `HEAD` must resolve to a commit;
///       the caller checks the type instead of assuming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ObjectKind = 'blob' | 'tree' | 'commit' | 'tag';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    /// File content,
    ///  or a symbolic link's target.
    Blob,
    /// A directory listing.
    Tree,
    /// A commit.
    Commit,
    /// An annotated tag.
    Tag,
}

/// What:
///  What Git answered for one request line.
///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
/// Why:
///   File content is arbitrary bytes and must be kept exactly;
///  a missing object is
///       an ordinary answer the caller decides about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BatchReply = { kind: 'found'; object: ObjectId; type: ObjectKind; bytes: Buffer } | { kind: 'missing' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchReply {
    /// The object exists;
    ///  these are its exact bytes.
    Found {
        /// The full object name Git resolved the request to.
        object: ObjectId,
        /// The stored object type.
        kind: ObjectKind,
        /// Exactly the declared number of content bytes.
        bytes: Vec<u8>,
    },
    /// Git could not resolve the request to an object.
    Missing,
}

/// What:
///  Build a failure whose message names the reply defect.
///       `format!` builds owned text;
///  `.as_str()` lends it to the constructor.
/// Why:
///   Every defect means the stream can no longer be trusted,
///  and the message says
///       so without quoting reply bytes,
///  which may be file content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replyFailure(failure: CandidateFailure, detail: string): CandidateError;
/// ```
fn reply_failure(failure: CandidateFailure, detail: &str) -> CandidateError {
    return CandidateError::new(
        failure,
        format!(
            "cli-git could not read a Git object: git cat-file --batch {detail}. \
             No candidate content was accepted from this reply."
        )
        .as_str(),
    );
}

/// What:
///  Read the header line,
///  without its line feed,
///  refusing an overlong one.
///       `&mut dyn BufRead` lends any buffered source for reading;
///  `dyn` means the
///       concrete type is decided at run time,
///  like a TS interface parameter.
///       `Result<T, E>` is "a value or a failure".
/// Why:
///   The header is the only delimiter-searched part of a reply,
///  so it is the only
///       part that needs a length bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readHeaderLine(stream: Readable): Promise<Buffer>;
/// ```
fn read_header_line(stream: &mut dyn BufRead) -> Result<Vec<u8>, CandidateError> {
    // `Vec::new()` is an empty growable byte list; `mut` lets the read append to it.
    let mut line: Vec<u8> = Vec::new();
    // What: `.take(n)` stops after `n` bytes; `as u64` widens the length losslessly.
    //       `.read_until(b'\n', &mut line)` appends bytes up to and including a line feed.
    // Why:  One byte past the bound is enough to tell "too long" from "ended early".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const line = await readUntil(stream, '\n', MAX_BATCH_HEADER_BYTES + 1);
    // ```
    let limit: u64 = MAX_BATCH_HEADER_BYTES as u64 + 1;
    // What: `match` on the read's `Result`: `Ok(_)` ignores the byte count, `Err(error)`
    //       binds the input/output failure.
    // Why:  A failed read means the process or pipe is gone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { await read(); } catch (error) { throw readerEnded(error); }
    // ```
    match stream.take(limit).read_until(b'\n', &mut line) {
        Ok(_) => {}
        Err(error) => {
            // `Err(...)` is the failure variant.
            return Err(reply_failure(
                CandidateFailure::ReaderEnded,
                format!("could not be read ({error})").as_str(),
            ));
        }
    }
    if line.is_empty() {
        return Err(reply_failure(
            CandidateFailure::ReaderEnded,
            "ended before answering a request",
        ));
    }
    // What: `.last()` is `Option<&u8>`, the final byte if any; `Some(&b'\n')` matches
    //       only a line feed.
    // Why:  A complete header ends with exactly one line feed. Without one, either the
    //       bound was reached (more bytes than any header) or the stream ended early.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (line.at(-1) !== 0x0a) throw ...;
    // ```
    if line.last() != Some(&b'\n') {
        if line.len() > MAX_BATCH_HEADER_BYTES {
            return Err(reply_failure(
                CandidateFailure::ReplyMalformed,
                "sent a header longer than any object header",
            ));
        }
        return Err(reply_failure(
            CandidateFailure::ReplyTruncated,
            "ended in the middle of a header",
        ));
    }
    // `.pop()` removes the line feed, which is not part of the header.
    line.pop();
    // `Ok(...)` is the success variant.
    return Ok(line);
}

/// What:
///  Parse a decimal object size.
///  `Option<usize>` is "a size or nothing".
/// Why:
///   The size decides how many content bytes are consumed,
///  so only Git's canonical
///       form is accepted:
///  digits only,
///  no sign,
///  no leading zero,
///  and no overflow.
///       The standard parser does the arithmetic,
///  so no hand-written digit loop can
///       be off by one;
///  it alone would also accept a leading `+`,
///  which the digit
///       check refuses first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSize(digits: string): number | undefined;
/// ```
fn parse_size(digits: &[u8]) -> Option<usize> {
    if digits.len() > 1 && digits[0] == b'0' {
        // `None` is the "absent" variant.
        return None;
    }
    // `.iter().all(u8::is_ascii_digit)` asks the standard digit test about every byte.
    if !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    // What: `std::str::from_utf8(..)` views the bytes as text; `.ok()?` returns `None`
    //       when that fails. `.parse::<usize>()` reads a number; `.ok()` turns a failure
    //       (empty text, or a value too large for `usize`) into `None`.
    // Why:  An empty field and an overflowing one are refused by the same parser that
    //       accepts every canonical size.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const size = Number(text); return Number.isSafeInteger(size) ? size : undefined;
    // ```
    let text: &str = std::str::from_utf8(digits).ok()?;
    return text.parse::<usize>().ok();
}

/// Map Git's object type word to a kind;
///  any other word is not a header this layer accepts.
fn object_kind(word: &[u8]) -> Option<ObjectKind> {
    if word == b"blob" {
        return Some(ObjectKind::Blob);
    }
    if word == b"tree" {
        return Some(ObjectKind::Tree);
    }
    if word == b"commit" {
        return Some(ObjectKind::Commit);
    }
    if word == b"tag" {
        return Some(ObjectKind::Tag);
    }
    return None;
}

/// Named predicate for splitting a header into fields;
///  `&u8` borrows one byte.
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What:
///  Parse `<oid> SP <type> SP <size>`.
///  The result is a tuple,
///  three values returned together.
/// Why:
///   Exactly three fields in Git's canonical spelling is the only shape whose size
///       this layer will trust to frame the content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseFoundHeader(header: Buffer): [ObjectId, ObjectKind, number] | undefined;
/// ```
fn parse_found_header(header: &[u8]) -> Option<(ObjectId, ObjectKind, usize)> {
    // What: `.split(is_space)` cuts at every space; `.collect()` gathers the borrowed pieces.
    // Why:  Two consecutive spaces produce an empty field, which no parser below accepts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fields = header.toString('latin1').split(' ');
    // ```
    let fields: Vec<&[u8]> = header.split(is_space).collect();
    if fields.len() != 3 {
        return None;
    }
    let object: ObjectId = parse_object_id(fields[0])?;
    let kind: ObjectKind = object_kind(fields[1])?;
    let size: usize = parse_size(fields[2])?;
    return Some((object, kind, size));
}

/// What:
///  Read exactly `size` content bytes and the line feed that follows them.
/// Why:
///   `.take(size).read_to_end(..)` grows the buffer only as bytes really arrive,
///  so
///       a header that lies about a huge size produces a truncated-reply failure
///       instead of one huge allocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readContent(stream: Readable, size: number): Promise<Buffer>;
/// ```
fn read_content(stream: &mut dyn BufRead, size: usize) -> Result<Vec<u8>, CandidateError> {
    let mut bytes: Vec<u8> = Vec::new();
    // `&mut *stream` re-lends the source, so `stream` stays usable for the terminator below.
    if let Err(error) = (&mut *stream).take(size as u64).read_to_end(&mut bytes) {
        return Err(reply_failure(
            CandidateFailure::ReaderEnded,
            format!("could not be read ({error})").as_str(),
        ));
    }
    if bytes.len() != size {
        return Err(reply_failure(
            CandidateFailure::ReplyTruncated,
            "ended before the declared object size",
        ));
    }
    // What: `[0_u8; 1]` is a one-byte buffer; `.read_exact(..)` fills it or fails.
    // Why:  Git ends every object with one line feed that is not content. Its absence
    //       means the declared size was wrong and the stream is out of step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const terminator = await readExactly(stream, 1);
    // ```
    let mut terminator: [u8; 1] = [0_u8; 1];
    if stream.read_exact(&mut terminator).is_err() {
        return Err(reply_failure(
            CandidateFailure::ReplyTruncated,
            "ended before the line feed that closes an object",
        ));
    }
    if terminator[0] != b'\n' {
        return Err(reply_failure(
            CandidateFailure::ReplyMalformed,
            "sent content longer than the declared object size",
        ));
    }
    return Ok(bytes);
}

/// What:
///  Read one complete reply to `request` from `stream`.
///       `request` is the exact request line that was sent,
///  without its line feed.
/// Why:
///   Git echoes the request in a `missing` reply and prints the resolved name in a
///       found reply.
///  When the request is itself a complete object name,
///  a found reply
///       naming any other object means requests and replies are out of step,
///  and its
///       bytes are refused rather than attributed to the wrong object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readBatchReply(stream: Readable, request: Buffer): Promise<BatchReply>;
/// ```
pub fn read_batch_reply(
    stream: &mut dyn BufRead,
    request: &[u8],
) -> Result<BatchReply, CandidateError> {
    // A trailing `?` returns the failure to our caller, or unwraps the header.
    let header: Vec<u8> = read_header_line(stream)?;
    // What: `.strip_suffix(..)` is `Some(prefix)` when the header ends with ` missing`.
    // Why:  Only the echoed request followed by that word is the missing form; a found
    //       header can never end with it because its last field is a number.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (header.equals(Buffer.concat([request, Buffer.from(' missing')]))) return { kind: 'missing' };
    // ```
    if header.strip_suffix(b" missing") == Some(request) {
        return Ok(BatchReply::Missing);
    }
    // `let Some(..) = .. else { .. }` unwraps the parsed fields or leaves with a failure.
    let Some((object, kind, size)) = parse_found_header(header.as_slice()) else {
        return Err(reply_failure(
            CandidateFailure::ReplyMalformed,
            "sent a header that is neither an object header nor a missing notice",
        ));
    };
    // What: `requested` is the request as an object name, or nothing when it was a word
    //       such as `HEAD`; `.as_ref()` views it as `Option<&ObjectId>` for comparing.
    // Why:  Only a request that is itself a complete name pins which object must answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (requested !== undefined && requested !== object) throw mismatched();
    // ```
    let requested: Option<ObjectId> = parse_object_id(request);
    if requested.is_some() && requested.as_ref() != Some(&object) {
        return Err(reply_failure(
            CandidateFailure::ReplyMismatched,
            "answered with a different object than the one requested",
        ));
    }
    let bytes: Vec<u8> = read_content(stream, size)?;
    return Ok(BatchReply::Found {
        object,
        kind,
        bytes,
    });
}

/// Framing controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_batch_tests.rs"]
mod tests;

/// A byte source that reports its end and later yields more bytes.
#[cfg(test)]
#[path = "candidate_batch_resume_tests.rs"]
mod resume_tests;
