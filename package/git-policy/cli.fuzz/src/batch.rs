//! What:
//!  `git cat-file --batch` replies built from fuzz bytes,
//!  and the invariants of reading one.
//! Why:
//!  Reply content is file content,
//!  so it can be any bytes,
//!  including text shaped
//!      like a reply.
//!  Reading must accept exactly Git's canonical reply,
//!  attribute it
//!      only to the object requested,
//!  refuse every cut-short form,
//!  and never be
//!      steered by bytes after the reply.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkBatchReply(...requestAndStream(data)); checkGeneratedReply(data);
//! ```

/// Import the reply reader under test,
///  the ceiling a real check uses,
///  and the values it produces.
use git_policy_cli::candidate_batch::{
    BatchReply, CONTENT_CEILING, MAX_BATCH_HEADER_BYTES, ObjectKind, read_batch_reply_within,
};
use git_policy_cli::candidate_error::{CandidateError, CandidateFailure};
use git_policy_cli::candidate_object::{ObjectId, parse_object_id};

/// What:
///  What a generated stream must be read as.
///       `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   The generator knows what it built,
///  so the reader's answer is compared with an
///       expectation computed without the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Expectation = { reply: BatchReply } | { failure: CandidateFailure };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expectation {
    /// The stream is one canonical reply.
    Reply(BatchReply),
    /// The stream must be refused with this cause.
    Failure(CandidateFailure),
}

/// What:
///  One generated case:
///  the request line,
///  the reply stream,
///  and what reading must yield.
///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
/// Why:
///   Requests and replies are bytes;
///  nothing here is decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedReply = { request: Buffer; stream: Buffer; expectation: Expectation };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedReply {
    /// The request line that was "sent",
    ///  without its line feed.
    pub request: Vec<u8>,
    /// The bytes the reader is given.
    pub stream: Vec<u8>,
    /// The content ceiling the reader must be given,
    ///  which the too-large shapes shorten.
    pub ceiling: usize,
    /// What reading the stream must produce.
    pub expectation: Expectation,
}

/// What:
///  Split raw fuzz bytes into a request and a stream at the first line feed.
///       The result is a tuple of two borrowed views into `data`.
/// Why:
///   A request line can never contain a line feed,
///  so the first one is a natural
///       boundary and every other byte reaches the reader unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requestAndStream(data: Uint8Array): [Uint8Array, Uint8Array];
/// ```
pub fn request_and_stream(data: &[u8]) -> (&[u8], &[u8]) {
    // `.iter().position(..)` is `Some(index)` of the first line feed, or `None`.
    match data.iter().position(is_line_feed) {
        Some(index) => return (&data[..index], &data[index + 1..]),
        None => return (data, &[]),
    }
}

/// Named predicate for the first line feed;
///  `&u8` borrows one byte.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Git's word for an object kind.
fn kind_word(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Blob => return "blob",
        ObjectKind::Tree => return "tree",
        ObjectKind::Commit => return "commit",
        ObjectKind::Tag => return "tag",
    }
}

/// What:
///  Git's canonical bytes for a found reply:
///  header,
///  content,
///  closing line feed.
/// Why:
///   Whatever the reader accepts must be exactly these bytes,
///  so acceptance is
///       checked by rendering the result and comparing,
///  not by parsing a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderFound(object: string, kind: ObjectKind, bytes: Buffer): Buffer;
/// ```
pub fn render_found(object: &str, kind: ObjectKind, bytes: &[u8]) -> Vec<u8> {
    let mut rendered: Vec<u8> =
        format!("{object} {} {}\n", kind_word(kind), bytes.len()).into_bytes();
    rendered.extend_from_slice(bytes);
    rendered.push(b'\n');
    return rendered;
}

/// What:
///  Read one reply from an in-memory stream at the content ceiling the caller chose,
///  and say how many bytes were consumed.
///       `usize` is the unsigned integer every length uses (siblings `u32`,
///  `u64`,
///  `i32`,
///  `i64`);
///  it is
///       the type the subject's own ceiling and lengths take.
/// Why:
///   A generated reply cannot hold the 32 MiB the real ceiling needs,
///  so the control shortens the
///       ceiling itself,
///  which is the seam the subject exposes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readWithin(request: Uint8Array, stream: Uint8Array, ceiling: number): [Result, number];
/// ```
fn read_within(
    request: &[u8],
    stream: &[u8],
    ceiling: usize,
) -> (Result<BatchReply, CandidateError>, usize) {
    // `mut` lets the reader advance the view; `remaining` is what is left after the reply.
    let mut remaining: &[u8] = stream;
    // The reader's answer, from the subject's own function at the caller's ceiling.
    let result: Result<BatchReply, CandidateError> =
        read_batch_reply_within(&mut remaining, request, ceiling);
    // `stream.len() - remaining.len()` is how many bytes the reader took.
    return (result, stream.len() - remaining.len());
}

/// What:
///  Assert that every sampled proper prefix of an accepted reply is refused as cut short.
/// Why:
///   A reader that accepted a prefix would hand out less content than Git stored.
///       All prefixes are checked for short replies and an even sample for long ones,
///       which keeps one execution linear in the input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPrefixesRefused(request: Buffer, accepted: Buffer, ceiling: number): void;
/// ```
fn check_prefixes_refused(request: &[u8], accepted: &[u8], ceiling: usize) {
    let step: usize = accepted.len() / 64 + 1;
    // `(0..accepted.len()).step_by(step)` visits prefix lengths 0, step, 2 * step, ...
    for length in (0..accepted.len()).step_by(step) {
        let (result, _consumed) = read_within(request, &accepted[..length], ceiling);
        let expected: CandidateFailure = if length == 0 {
            CandidateFailure::ReaderEnded
        } else {
            CandidateFailure::ReplyTruncated
        };
        match result {
            Ok(reply) => panic!("a {length}-byte prefix was accepted as {reply:?}"),
            Err(error) => assert_eq!(error.failure, expected, "prefix of {length} bytes"),
        }
    }
    // The longest proper prefix lacks only the closing line feed.
    if let Some(last) = accepted.len().checked_sub(1) {
        let (result, _consumed) = read_within(request, &accepted[..last], ceiling);
        assert!(
            result.is_err(),
            "the reply without its last byte was accepted"
        );
    }
}

/// What:
///  Assert the invariants of reading one reply to `request` from `stream` at `ceiling`.
/// Why:
///   Whatever the bytes are:
///  reading is repeatable;
///  an accepted reply is exactly
///       Git's canonical bytes for the returned value and names the object requested;
///       bytes after it change nothing;
///  no proper prefix of it is accepted;
///  and a
///       refusal is one of the four reply failures,
///  "ended" only for an empty stream.
///  A reply above the ceiling keeps no content,
///  so its framing is
///       what is checked instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkBatchReplyWithin(request: Uint8Array, stream: Uint8Array, ceiling: number): void;
/// ```
pub fn check_batch_reply_within(request: &[u8], stream: &[u8], ceiling: usize) {
    // The reader's answer and how many bytes of the stream it took.
    let (result, consumed) = read_within(request, stream, ceiling);
    // The same read again, which must agree with the first.
    let (again, consumed_again) = read_within(request, stream, ceiling);
    assert_eq!(again, result, "reading is not repeatable");
    assert_eq!(
        consumed_again, consumed,
        "reading consumed a different amount"
    );
    assert!(
        consumed <= stream.len(),
        "more bytes were consumed than exist"
    );
    match &result {
        Ok(reply) => {
            let accepted: &[u8] = &stream[..consumed];
            match reply {
                BatchReply::Missing => {
                    let mut canonical: Vec<u8> = request.to_vec();
                    canonical.extend_from_slice(b" missing\n");
                    assert_eq!(
                        accepted,
                        canonical.as_slice(),
                        "a non-canonical missing notice was accepted"
                    );
                }
                BatchReply::Found {
                    object,
                    kind,
                    bytes,
                } => {
                    assert_eq!(
                        accepted,
                        render_found(object.as_str(), *kind, bytes.as_slice()).as_slice(),
                        "a non-canonical object reply was accepted"
                    );
                    if let Some(requested) = parse_object_id(request) {
                        assert_eq!(&requested, object, "another object's bytes were accepted");
                    }
                }
                BatchReply::TooLarge {
                    object,
                    kind,
                    size,
                } => {
                    // What: Git's header line for the object, ending with its line feed.
                    // Why:  The content is thrown away, so the header is what can be compared with what was read.
                    let header: String = format!("{} {} {size}\n", object.as_str(), kind_word(*kind));
                    assert!(
                        *size > ceiling,
                        "a reply at or below the ceiling was skipped: {size} <= {ceiling}"
                    );
                    assert_eq!(
                        accepted.len(),
                        header.len() + *size + 1,
                        "a skipped reply kept a different number of bytes"
                    );
                    assert_eq!(
                        &accepted[..header.len()],
                        header.as_bytes(),
                        "a skipped reply was accepted with a different header"
                    );
                    assert_eq!(
                        accepted.last(),
                        Some(&b'\n'),
                        "a skipped reply was accepted without its terminator"
                    );
                    if let Some(requested) = parse_object_id(request) {
                        assert_eq!(&requested, object, "another object's bytes were skipped");
                    }
                }
            }
            // Bytes after the reply are not part of it.
            let (alone, consumed_alone) = read_within(request, accepted, ceiling);
            assert_eq!(alone, result, "trailing bytes changed the reply");
            assert_eq!(
                consumed_alone, consumed,
                "trailing bytes changed what was consumed"
            );
            check_prefixes_refused(request, accepted, ceiling);
        }
        Err(error) => {
            assert!(
                error.failure == CandidateFailure::ReaderEnded
                    || error.failure == CandidateFailure::ReplyMalformed
                    || error.failure == CandidateFailure::ReplyTruncated
                    || error.failure == CandidateFailure::ReplyMismatched,
                "unexpected failure {:?}",
                error.failure
            );
            assert_eq!(
                error.failure == CandidateFailure::ReaderEnded,
                stream.is_empty(),
                "an in-memory stream only ends when it is empty"
            );
            assert!(
                error
                    .message
                    .starts_with("monochromatic-git-policy-cli could not read a Git object: git cat-file --batch "),
                "{error}"
            );
        }
    }
}

/// What:
///  Assert the invariants of reading one reply at the ceiling a real check uses.
/// Why:
///   The fuzz target reads raw bytes exactly as the wrapper does,
///  so the ceiling must be the
///       subject's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkBatchReply(request: Uint8Array, stream: Uint8Array): void;
/// ```
pub fn check_batch_reply(request: &[u8], stream: &[u8]) {
    // `CONTENT_CEILING` is the ceiling a real check reads at.
    check_batch_reply_within(request, stream, CONTENT_CEILING);
}

/// Lowercase hexadecimal digits,
///  indexed by value.
/// This table stays here rather than moving to the wrapper's one encoder
/// (`git-policy/cli/src/native/foundation/hexadecimal_of_bytes.rs`):
///  `git-policy/cli.fuzz` is a workspace of its own,
///  and these digits are not a byte spelled as hexadecimal but a digit chosen by a byte's position
///  (issue 1672 records the owner's answer that the copy stays closed here).
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// What:
///  Build a complete object name from fuzz bytes:
///  40 digits,
///  or 64 when `long`.
/// Why:
///   Generated cases need names the reader accepts,
///  in both hash formats.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedName(data: Uint8Array, long: boolean): string;
/// ```
fn generated_name(data: &[u8], long: bool) -> String {
    let length: usize = if long { 64 } else { 40 };
    let mut name: String = String::new();
    for position in 0..length {
        // `data.get(position % ...)` reads a byte if any exist; an empty input uses the position itself.
        let byte: u8 = match data.get(position % data.len().max(1)) {
            Some(value) => *value,
            None => 0,
        };
        let digit: usize = (usize::from(byte) + position) % 16;
        name.push(char::from(HEX_DIGITS[digit]));
    }
    return name;
}

/// A different complete name of the same length:
///  the first digit is changed.
fn other_name(name: &str) -> String {
    let replacement: char = if name.starts_with('0') { '1' } else { '0' };
    let mut other: String = String::from(replacement);
    other.push_str(&name[1..]);
    return other;
}

/// The validated form of a generated name.
fn object_of(name: &str) -> ObjectId {
    match parse_object_id(name.as_bytes()) {
        Some(object) => return object,
        None => panic!("the generator built an invalid object name: {name}"),
    }
}

/// What:
///  Build exactly `size` content bytes from the fuzz input's own bytes,
///  repeating them when the input is shorter.
/// Why:
///   A skipped reply must carry every byte its header declares,
///  or the reader reports a
///       truncated reply instead of skipping the object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fillerContent(content: Uint8Array, size: number): Buffer;
/// ```
fn filler_content(content: &[u8], size: usize) -> Vec<u8> {
    // An owned byte list of the length the header will declare.
    let mut filled: Vec<u8> = Vec::with_capacity(size);
    // `0..size` visits every position the header declares.
    for position in 0..size {
        // `content.get(..)` repeats the input's bytes, or yields nothing for an empty input.
        let byte: u8 = match content.get(position % content.len().max(1)) {
            Some(value) => *value,
            None => 0,
        };
        filled.push(byte);
    }
    return filled;
}

/// What:
///  Build a reply that declares `size` bytes and really holds `held` of them,
///  with or without the
///       line feed that ends it.
/// Why:
///   A skipped reply is framed by its header and its declared size,
///  so a control can build the
///       exact number of bytes the header claims without holding a 32 MiB object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replyDeclaring(name: string, kind: ObjectKind, size: number, held: Buffer, terminated: boolean): Buffer;
/// ```
fn reply_declaring(
    name: &str,
    kind: ObjectKind,
    size: usize,
    held: &[u8],
    terminated: bool,
) -> Vec<u8> {
    // The header Git prints: the object name, the kind word and the declared size.
    let mut stream: Vec<u8> = format!("{name} {} {size}\n", kind_word(kind)).into_bytes();
    // The content bytes the stream really holds.
    stream.extend_from_slice(held);
    // A truncated shape leaves the closing line feed out.
    if terminated {
        stream.push(b'\n');
    }
    return stream;
}

/// What:
///  The first shape whose reply is above a shortened ceiling,
///  so its content is consumed and not kept.
/// Why:
///   The shape alone says which ceiling the generated case must be read at,
///  and the fuzz input
///       cannot hold the 32 MiB the real ceiling needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIRST_SKIPPED_SHAPE = 14;
/// ```
const FIRST_SKIPPED_SHAPE: u8 = 14;

/// The number of case shapes `generated_reply` builds:
///  the earlier shapes and five above a shortened ceiling.
pub const GENERATED_SHAPES: u8 = FIRST_SKIPPED_SHAPE + 5;

/// Header texts Git never prints;
///  `{name}` is replaced by the requested object name.
const MALFORMED_HEADERS: &[&str] = &[
    "{name} ambiguous",
    "{name} blob",
    "{name} blob +3",
    "{name} blob 03",
    "{name} blob 3 extra",
    "{name}  blob 3",
    "{name} file 3",
    "{name} blob 99999999999999999999999999",
    "missing",
    "",
];

/// What:
///  Build one case from fuzz bytes:
///  the first byte picks the shape,
///  the second
///       the object kind and hash format,
///  the rest is the content.
/// Why:
///   Raw bytes almost never spell a valid header,
///  so the generator builds valid
///       replies and each specific corruption directly,
///  with the outcome it must have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedReply(data: Uint8Array): GeneratedReply;
/// ```
pub fn generated_reply(data: &[u8]) -> GeneratedReply {
    let shape: u8 = data.first().copied().unwrap_or(0) % GENERATED_SHAPES;
    let selector: u8 = data.get(1).copied().unwrap_or(0);
    let content: &[u8] = data.get(2..).unwrap_or(&[]);
    let kind: ObjectKind = [
        ObjectKind::Blob,
        ObjectKind::Tree,
        ObjectKind::Commit,
        ObjectKind::Tag,
    ][usize::from(selector % 4)];
    let name: String = generated_name(content, selector >= 128);
    let found: Expectation = Expectation::Reply(BatchReply::Found {
        object: object_of(name.as_str()),
        kind,
        bytes: content.to_vec(),
    });
    let canonical: Vec<u8> = render_found(name.as_str(), kind, content);
    let header: String = format!("{name} {} {}", kind_word(kind), content.len());
    // Each arm returns the request, the stream and the expectation of one shape.
    let (request, stream, expectation): (Vec<u8>, Vec<u8>, Expectation) = match shape {
        // A canonical reply to a request by name, alone and followed by further bytes.
        0 => (name.clone().into_bytes(), canonical, found),
        1 => {
            let mut followed: Vec<u8> = canonical;
            followed.extend_from_slice(content);
            (name.clone().into_bytes(), followed, found)
        }
        // A canonical reply to a symbolic request, which accepts any object.
        2 => (b"HEAD".to_vec(), canonical, found),
        // Missing notices for a name and for a symbolic request.
        3 => (
            name.clone().into_bytes(),
            format!("{name} missing\n").into_bytes(),
            Expectation::Reply(BatchReply::Missing),
        ),
        4 => (
            b"HEAD".to_vec(),
            b"HEAD missing\n".to_vec(),
            Expectation::Reply(BatchReply::Missing),
        ),
        // A canonical reply for a different object than the one requested.
        5 => (
            other_name(name.as_str()).into_bytes(),
            canonical,
            Expectation::Failure(CandidateFailure::ReplyMismatched),
        ),
        // The declared size exceeds the content by one or more: the stream ends early.
        6 => {
            let declared: usize = content.len() + 1 + usize::from(selector % 3);
            let mut short: Vec<u8> =
                format!("{name} {} {declared}\n", kind_word(kind)).into_bytes();
            short.extend_from_slice(content);
            short.push(b'\n');
            (
                name.clone().into_bytes(),
                short,
                Expectation::Failure(CandidateFailure::ReplyTruncated),
            )
        }
        // The declared size is one less than the content, whose last byte is not a line feed.
        7 => {
            let mut longer: Vec<u8> = content.to_vec();
            longer.push(b'x');
            let mut oversized: Vec<u8> =
                format!("{name} {} {}\n", kind_word(kind), longer.len() - 1).into_bytes();
            oversized.extend_from_slice(longer.as_slice());
            oversized.push(b'\n');
            (
                name.clone().into_bytes(),
                oversized,
                Expectation::Failure(CandidateFailure::ReplyMalformed),
            )
        }
        // The stream ends inside the header, or after the header, or after the content.
        8 => (
            name.clone().into_bytes(),
            header.as_bytes()[..header.len() - usize::from(selector % 8)].to_vec(),
            Expectation::Failure(CandidateFailure::ReplyTruncated),
        ),
        9 => (
            name.clone().into_bytes(),
            canonical[..canonical.len() - 1].to_vec(),
            Expectation::Failure(CandidateFailure::ReplyTruncated),
        ),
        // More bytes than any header and no line feed.
        10 => (
            name.clone().into_bytes(),
            vec![b'h'; MAX_BATCH_HEADER_BYTES + 1 + usize::from(selector)],
            Expectation::Failure(CandidateFailure::ReplyMalformed),
        ),
        // A header Git never prints.
        11 => {
            let template: &str = MALFORMED_HEADERS[usize::from(selector) % MALFORMED_HEADERS.len()];
            let mut unprintable: Vec<u8> = template.replace("{name}", name.as_str()).into_bytes();
            unprintable.push(b'\n');
            unprintable.extend_from_slice(content);
            (
                name.clone().into_bytes(),
                unprintable,
                Expectation::Failure(CandidateFailure::ReplyMalformed),
            )
        }
        // A missing notice that echoes a different request.
        12 => (
            name.clone().into_bytes(),
            format!("{} missing\n", other_name(name.as_str())).into_bytes(),
            Expectation::Failure(CandidateFailure::ReplyMalformed),
        ),
        // A reply whose declared size is above a shortened ceiling: the content is consumed and not kept.
        14 => {
            let size: usize = content.len() + 1 + usize::from(selector % 3);
            let held: Vec<u8> = filler_content(content, size);
            (
                name.clone().into_bytes(),
                reply_declaring(name.as_str(), kind, size, held.as_slice(), true),
                Expectation::Reply(BatchReply::TooLarge {
                    object: object_of(name.as_str()),
                    kind,
                    size,
                }),
            )
        }
        // The same skipped reply followed by further bytes.
        15 => {
            let size: usize = content.len() + 1 + usize::from(selector % 3);
            let held: Vec<u8> = filler_content(content, size);
            let mut followed: Vec<u8> =
                reply_declaring(name.as_str(), kind, size, held.as_slice(), true);
            followed.extend_from_slice(content);
            (
                name.clone().into_bytes(),
                followed,
                Expectation::Reply(BatchReply::TooLarge {
                    object: object_of(name.as_str()),
                    kind,
                    size,
                }),
            )
        }
        // A skipped reply whose content ends before the declared size.
        16 => {
            let size: usize = content.len() + 2;
            let held: Vec<u8> = filler_content(content, size - 1);
            (
                name.clone().into_bytes(),
                reply_declaring(name.as_str(), kind, size, held.as_slice(), true),
                Expectation::Failure(CandidateFailure::ReplyTruncated),
            )
        }
        // A skipped reply whose content is complete but whose terminator is missing.
        17 => {
            let size: usize = content.len() + 1;
            let held: Vec<u8> = filler_content(content, size);
            (
                name.clone().into_bytes(),
                reply_declaring(name.as_str(), kind, size, held.as_slice(), false),
                Expectation::Failure(CandidateFailure::ReplyTruncated),
            )
        }
        // A skipped reply for a different object than the one requested.
        18 => {
            let size: usize = content.len() + 1;
            let held: Vec<u8> = filler_content(content, size);
            (
                other_name(name.as_str()).into_bytes(),
                reply_declaring(name.as_str(), kind, size, held.as_slice(), true),
                Expectation::Failure(CandidateFailure::ReplyMismatched),
            )
        }
        // Nothing at all: the process ended.
        _ => (
            name.clone().into_bytes(),
            Vec::new(),
            Expectation::Failure(CandidateFailure::ReaderEnded),
        ),
    };
    // What: A skipped shape is read at the ceiling just below its declared size; every other shape at the real one.
    // Why:  The generated case carries the ceiling its stream was framed for, so no control holds a 32 MiB object.
    let ceiling: usize = if shape >= FIRST_SKIPPED_SHAPE {
        content.len()
    } else {
        CONTENT_CEILING
    };
    // What: The one generated case: its request, stream, ceiling and the outcome reading must produce.
    // Why:  The caller reads at exactly this ceiling and compares with exactly this expectation.
    return GeneratedReply {
        request,
        stream,
        ceiling,
        expectation,
    };
}

/// What:
///  Build a case from fuzz bytes,
///  require the reader to produce its expectation,
///  and run the general invariants on it.
/// Why:
///   The general invariants cannot tell a wrongly refused valid reply from a
///       correctly refused one;
///  the generator's expectation can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedReply(data: Uint8Array): void;
/// ```
pub fn check_generated_reply(data: &[u8]) {
    let case: GeneratedReply = generated_reply(data);
    // The reader's answer at the ceiling the case was framed for.
    let (result, _consumed) =
        read_within(case.request.as_slice(), case.stream.as_slice(), case.ceiling);
    let observed: Expectation = match result {
        Ok(reply) => Expectation::Reply(reply),
        Err(error) => Expectation::Failure(error.failure),
    };
    assert_eq!(
        observed, case.expectation,
        "generated case was read differently"
    );
    check_batch_reply_within(case.request.as_slice(), case.stream.as_slice(), case.ceiling);
}

/// Generator controls stay out of the fuzz targets.
#[cfg(test)]
#[path = "batch_tests.rs"]
mod tests;
