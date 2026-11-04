//! What:
//!  A control for a byte source that reports its end and later yields more bytes.
//! Why:
//!  A pipe stays ended once it ends,
//!  but the reply reader accepts any buffered source,
//!      and a file that is still being written does not.
//!  Without the explicit check that
//!      all declared content bytes arrived,
//!  such a source could have its later bytes
//!      read as the closing line feed,
//!  and shortened content would be accepted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => readBatchReply(growingSource('…blob 6\nabc', '\n'), oid)).toThrow('reply-truncated');
//! ```

/// Import the reader under test and the values it produces.
use super::read_batch_reply;
use crate::candidate_error::{CandidateError, CandidateFailure};
use std::io::{BufRead, Read};

/// A complete SHA-1 object name used as the request.
const NAME: &str = "0123456789abcdef0123456789abcdef01234567";

/// A byte source in two parts that reports its end once between them,
///  like a file that grows.
struct ResumingStream {
    /// Bytes available before the first reported end.
    first: Vec<u8>,
    /// Bytes that appear after the end was reported once.
    second: Vec<u8>,
    /// How many bytes of the current part were already handed out.
    position: usize,
    /// Whether the end between the parts was already reported.
    reported_end: bool,
}

/// Reading hands out the first part,
///  then one end,
///  then the second part.
impl Read for ResumingStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let available: &[u8] = self.fill_buf()?;
        let count: usize = available.len().min(buffer.len());
        buffer[..count].copy_from_slice(&available[..count]);
        self.consume(count);
        return Ok(count);
    }
}

/// Buffered reading follows the same three stages.
impl BufRead for ResumingStream {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if !self.reported_end {
            if self.position < self.first.len() {
                return Ok(&self.first[self.position..]);
            }
            // The first part is used up: report the end once and switch to the second part.
            self.reported_end = true;
            self.position = 0;
            return Ok(&[]);
        }
        return Ok(&self.second[self.position..]);
    }

    fn consume(&mut self, amount: usize) {
        self.position += amount;
    }
}

/// Content that stops short of its declared size is truncated,
///  even when a line feed arrives afterwards.
#[test]
fn content_that_ends_early_is_refused_even_when_more_bytes_follow() {
    let mut stream: ResumingStream = ResumingStream {
        first: format!("{NAME} blob 6\nabc").into_bytes(),
        // Exactly the byte that would close an object, arriving after the end was reported.
        second: b"\n".to_vec(),
        position: 0,
        reported_end: false,
    };
    let error: CandidateError =
        read_batch_reply(&mut stream, NAME.as_bytes()).expect_err("short content");
    assert_eq!(error.failure, CandidateFailure::ReplyTruncated);
    assert!(
        error
            .message
            .contains("ended before the declared object size"),
        "{error}"
    );
    // The later byte was not consumed as a terminator.
    assert!(stream.reported_end);
    assert_eq!(stream.position, 0);
}

/// The same source with its content complete in the first part is accepted,
///  so the control can tell the two apart.
#[test]
fn complete_content_from_the_same_source_is_accepted() {
    let mut stream: ResumingStream = ResumingStream {
        first: format!("{NAME} blob 3\nabc\n").into_bytes(),
        second: b"later".to_vec(),
        position: 0,
        reported_end: false,
    };
    assert!(read_batch_reply(&mut stream, NAME.as_bytes()).is_ok());
    assert!(!stream.reported_end);
}
