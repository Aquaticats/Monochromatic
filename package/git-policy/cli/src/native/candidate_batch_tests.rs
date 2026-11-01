//! What:
//!  Framing controls for `git cat-file --batch` replies read from in-memory streams.
//! Why:
//!  Object content is arbitrary bytes.
//!  These controls prove a reply is framed by its
//!      declared size alone,
//!  that every malformed shape is a typed failure,
//!  and that
//!      exactly one reply is consumed so the next request stays in step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await readBatchReply(Readable.from(bytes), oid)).toEqual({ kind: 'found', ... });
//! ```

/// Import the reader under test and the values it produces.
use super::{BatchReply, MAX_BATCH_HEADER_BYTES, ObjectKind, read_batch_reply};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{ObjectId, parse_object_id};
use std::io::{BufRead, Read};

/// A complete SHA-1 object name used as the request in most controls.
const NAME: &str = "0123456789abcdef0123456789abcdef01234567";

/// A second,
///  different object name.
const OTHER: &str = "89abcdef0123456789abcdef0123456789abcdef";

/// The validated form of `NAME`.
fn name() -> ObjectId {
    return parse_object_id(NAME.as_bytes()).expect("fixture name");
}

/// Read one reply to a `NAME` request and return the failure cause it must produce.
fn failure_of(stream: &[u8]) -> CandidateFailure {
    let mut remaining: &[u8] = stream;
    let error: CandidateError =
        read_batch_reply(&mut remaining, NAME.as_bytes()).expect_err("rejected reply");
    assert!(
        error
            .message
            .starts_with("cli-git could not read a Git object: git cat-file --batch "),
        "{error}"
    );
    return error.failure;
}

/// A byte source whose every read fails,
///  standing in for a pipe that broke.
struct BrokenStream;

/// Reading reports a broken pipe.
impl Read for BrokenStream {
    fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }
}

/// Buffered reading reports the same broken pipe.
impl BufRead for BrokenStream {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }

    fn consume(&mut self, _amount: usize) {}
}

/// A found reply yields exactly the declared bytes,
///  whatever they contain,
///  and consumes nothing after its terminator.
#[test]
fn found_reply_is_framed_by_its_declared_size() {
    // The content holds NUL, a non-UTF-8 byte, line feeds and a forged header for another object.
    let content: Vec<u8> = format!("a\0b\u{0}\n{OTHER} blob 3\nxyz\n").into_bytes();
    let mut content_with_byte: Vec<u8> = content.clone();
    content_with_byte.push(0xff);
    let mut stream: Vec<u8> = format!("{NAME} blob {}\n", content_with_byte.len()).into_bytes();
    stream.extend_from_slice(content_with_byte.as_slice());
    stream.extend_from_slice(b"\nNEXT");
    let mut remaining: &[u8] = stream.as_slice();
    assert_eq!(
        read_batch_reply(&mut remaining, NAME.as_bytes()).expect("found"),
        BatchReply::Found {
            object: name(),
            kind: ObjectKind::Blob,
            bytes: content_with_byte.clone(),
        }
    );
    assert_eq!(remaining, b"NEXT");
    assert!(
        content_with_byte.len() >= 10,
        "a two-digit size is exercised"
    );
}

/// An empty object is a size of zero followed directly by the terminator.
#[test]
fn empty_object_is_found_with_no_bytes() {
    let stream: String = format!("{NAME} blob 0\n\n");
    let mut remaining: &[u8] = stream.as_bytes();
    assert_eq!(
        read_batch_reply(&mut remaining, NAME.as_bytes()).expect("empty"),
        BatchReply::Found {
            object: name(),
            kind: ObjectKind::Blob,
            bytes: Vec::new(),
        }
    );
    assert!(remaining.is_empty());
}

/// Two replies in one stream are read one at a time,
///  each from where the last ended.
#[test]
fn consecutive_replies_stay_in_step() {
    let stream: String = format!("{NAME} blob 2\nhi\n{OTHER} missing\n{OTHER} tree 1\nT\n");
    let mut remaining: &[u8] = stream.as_bytes();
    let first: BatchReply = read_batch_reply(&mut remaining, NAME.as_bytes()).expect("first");
    assert_eq!(
        first,
        BatchReply::Found {
            object: name(),
            kind: ObjectKind::Blob,
            bytes: b"hi".to_vec(),
        }
    );
    assert_eq!(
        read_batch_reply(&mut remaining, OTHER.as_bytes()).expect("second"),
        BatchReply::Missing
    );
    assert_eq!(
        read_batch_reply(&mut remaining, OTHER.as_bytes()).expect("third"),
        BatchReply::Found {
            object: parse_object_id(OTHER.as_bytes()).expect("name"),
            kind: ObjectKind::Tree,
            bytes: b"T".to_vec(),
        }
    );
    assert!(remaining.is_empty());
}

/// Every object type word is recognized,
///  and a symbolic request accepts whichever object it resolved to.
#[test]
fn object_types_and_symbolic_requests_are_reported() {
    for (word, kind) in [
        ("blob", ObjectKind::Blob),
        ("tree", ObjectKind::Tree),
        ("commit", ObjectKind::Commit),
        ("tag", ObjectKind::Tag),
    ] {
        let stream: String = format!("{NAME} {word} 3\nabc\n");
        let mut remaining: &[u8] = stream.as_bytes();
        assert_eq!(
            read_batch_reply(&mut remaining, b"HEAD").expect("found"),
            BatchReply::Found {
                object: name(),
                kind,
                bytes: b"abc".to_vec(),
            }
        );
    }
}

/// The missing form is the echoed request followed by the word;
///  it consumes only its own line.
#[test]
fn missing_reply_echoes_the_request() {
    for request in [NAME, "HEAD", ""] {
        let stream: String = format!("{request} missing\nNEXT");
        let mut remaining: &[u8] = stream.as_bytes();
        assert_eq!(
            read_batch_reply(&mut remaining, request.as_bytes()).expect("missing"),
            BatchReply::Missing
        );
        assert_eq!(remaining, b"NEXT");
    }
}

/// Headers that are neither form are refused:
///  wrong field counts,
///  spellings,
///  sizes,
///  and a missing notice for another request.
#[test]
fn malformed_headers_are_refused() {
    let huge: String = "9".repeat(30);
    for header in [
        format!("{NAME} blob"),
        format!("{NAME} blob 3 extra"),
        format!("{NAME}  blob 3"),
        format!(" {NAME} blob 3"),
        format!("{} blob 3", NAME.to_uppercase()),
        format!("{} blob 3", &NAME[..39]),
        format!("{NAME} BLOB 3"),
        format!("{NAME} blobs 3"),
        format!("{NAME} file 3"),
        format!("{NAME} blob "),
        format!("{NAME} blob +3"),
        format!("{NAME} blob -3"),
        format!("{NAME} blob 3x"),
        format!("{NAME} blob 03"),
        format!("{NAME} blob 00"),
        format!("{NAME} blob 0x3"),
        format!("{NAME} blob {huge}"),
        format!("{NAME} ambiguous"),
        format!("{OTHER} missing"),
        format!("{NAME} missing "),
        String::from("missing"),
        String::from(""),
    ] {
        let stream: String = format!("{header}\nabc\n");
        assert_eq!(
            failure_of(stream.as_bytes()),
            CandidateFailure::ReplyMalformed,
            "{header:?}"
        );
    }
}

/// A found reply naming another object than the complete name requested is refused before its content is read.
#[test]
fn reply_for_another_object_is_refused() {
    let stream: String = format!("{OTHER} blob 3\nabc\n");
    assert_eq!(
        failure_of(stream.as_bytes()),
        CandidateFailure::ReplyMismatched
    );
}

/// A stream that ends anywhere inside a reply is reported as truncated,
///  never as shorter content.
#[test]
fn truncated_replies_are_refused() {
    for stream in [
        format!("{NAME} blob 3"),
        format!("{NAME} bl"),
        format!("{NAME} blob 3\n"),
        format!("{NAME} blob 3\nab"),
        format!("{NAME} blob 3\nabc"),
        format!("{NAME} blob 0\n"),
        // A header may claim far more bytes than exist; nothing near that size is allocated.
        format!("{NAME} blob 999999999999999\nabc\n"),
    ] {
        assert_eq!(
            failure_of(stream.as_bytes()),
            CandidateFailure::ReplyTruncated,
            "{stream:?}"
        );
    }
}

/// Content longer than its declared size is detected at the terminator.
#[test]
fn content_past_the_declared_size_is_refused() {
    let stream: String = format!("{NAME} blob 3\nabcd\n");
    assert_eq!(
        failure_of(stream.as_bytes()),
        CandidateFailure::ReplyMalformed
    );
    let no_feed: String = format!("{NAME} blob 3\nabc\r");
    assert_eq!(
        failure_of(no_feed.as_bytes()),
        CandidateFailure::ReplyMalformed
    );
}

/// A stream that ends before any reply byte,
///  or that cannot be read,
///  means the reader is gone.
#[test]
fn ended_and_unreadable_streams_are_reader_failures() {
    assert_eq!(failure_of(b""), CandidateFailure::ReaderEnded);
    let mut broken: BrokenStream = BrokenStream;
    let error: CandidateError =
        read_batch_reply(&mut broken, NAME.as_bytes()).expect_err("unreadable header");
    assert_eq!(error.failure, CandidateFailure::ReaderEnded);
    assert!(error.message.contains("could not be read"), "{error}");
}

/// A read failure after a valid header is also a reader failure,
///  not truncation.
#[test]
fn unreadable_content_is_a_reader_failure() {
    let header: String = format!("{NAME} blob 3\n");
    // `.chain(..)` reads the header bytes first and then the always-failing source.
    let mut stream: std::io::BufReader<std::io::Chain<&[u8], BrokenStream>> =
        std::io::BufReader::new(header.as_bytes().chain(BrokenStream));
    let error: CandidateError =
        read_batch_reply(&mut stream, NAME.as_bytes()).expect_err("unreadable content");
    assert_eq!(error.failure, CandidateFailure::ReaderEnded);
}

/// The header bound is exact:
///  the longest accepted header,
///  one byte more,
///  and a stream ending at the bound.
#[test]
fn header_length_bound_is_exact() {
    assert_eq!(MAX_BATCH_HEADER_BYTES, 128);
    // A missing notice is the echoed request plus eight bytes, so the request length sets the header length.
    let longest: String = "r".repeat(MAX_BATCH_HEADER_BYTES - 8);
    let accepted: String = format!("{longest} missing\n");
    let mut remaining: &[u8] = accepted.as_bytes();
    assert_eq!(
        read_batch_reply(&mut remaining, longest.as_bytes()).expect("longest header"),
        BatchReply::Missing
    );
    let too_long: String = "r".repeat(MAX_BATCH_HEADER_BYTES - 7);
    let refused: String = format!("{too_long} missing\n");
    let mut refused_stream: &[u8] = refused.as_bytes();
    assert_eq!(
        read_batch_reply(&mut refused_stream, too_long.as_bytes())
            .expect_err("overlong header")
            .failure,
        CandidateFailure::ReplyMalformed
    );
    // No line feed ever arrives: the read stops one byte past the bound instead of buffering the stream.
    let endless: Vec<u8> = vec![b'x'; MAX_BATCH_HEADER_BYTES * 4];
    let mut endless_stream: &[u8] = endless.as_slice();
    assert_eq!(
        read_batch_reply(&mut endless_stream, NAME.as_bytes())
            .expect_err("endless header")
            .failure,
        CandidateFailure::ReplyMalformed
    );
    assert_eq!(
        endless_stream.len(),
        MAX_BATCH_HEADER_BYTES * 4 - (MAX_BATCH_HEADER_BYTES + 1)
    );
    // Exactly the bound in bytes and then the end of the stream is an early end, not an overlong header.
    let at_bound: Vec<u8> = vec![b'x'; MAX_BATCH_HEADER_BYTES];
    assert_eq!(
        failure_of(at_bound.as_slice()),
        CandidateFailure::ReplyTruncated
    );
}
