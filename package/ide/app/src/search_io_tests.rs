//! Protocol limits are checked before unbounded allocation and stderr remains drained after truncation.

/// Private helpers are exercised with the same async read traits used for child-process pipes.
use super::{MAX_DIAGNOSTIC_BYTES, diagnostic, record};
/// Shared limit defines the exact accepted and rejected record boundary.
use crate::search::MAX_SEARCH_RECORD;

/// Tokio supplies a test-local runtime;
///  borrowed byte slices implement its async read interfaces.
#[tokio::test]
async fn delimiters_and_unterminated_final_records_preserve_bytes() {
    let mut bytes: &[u8] = b"first\0line\nbreak\0last";
    assert_eq!(
        record(&mut bytes, 0).await.expect("first record"),
        Some(b"first".to_vec())
    );
    assert_eq!(
        record(&mut bytes, 0).await.expect("newline filename"),
        Some(b"line\nbreak".to_vec())
    );
    assert_eq!(
        record(&mut bytes, 0).await.expect("final record"),
        Some(b"last".to_vec())
    );
    assert!(record(&mut bytes, 0).await.expect("EOF").is_none());
    let mut empty_line: &[u8] = b"\n";
    assert_eq!(
        record(&mut empty_line, b'\n').await.expect("empty record"),
        Some(Vec::new())
    );
}

/// One delimiter beyond the payload limit is valid;
///  one additional payload byte is not.
#[tokio::test]
async fn exact_record_limit_has_an_observed_boundary() {
    let mut exact = vec![b'x'; MAX_SEARCH_RECORD];
    exact.push(b'\n');
    let mut accepted = exact.as_slice();
    assert_eq!(
        record(&mut accepted, b'\n')
            .await
            .expect("exact limit")
            .expect("record")
            .len(),
        MAX_SEARCH_RECORD
    );
    let mut oversized = vec![b'x'; MAX_SEARCH_RECORD + 1];
    oversized.push(b'\n');
    let mut rejected = oversized.as_slice();
    assert!(record(&mut rejected, b'\n').await.is_err());
    let mut unterminated = &oversized[..MAX_SEARCH_RECORD + 1];
    assert!(record(&mut unterminated, b'\n').await.is_err());
}

/// Truncating retained stderr must not stop consumption and leave a child blocked behind its pipe.
#[tokio::test]
async fn diagnostics_are_bounded_and_fully_drained() {
    let bytes = vec![b'x'; MAX_DIAGNOSTIC_BYTES + 17];
    let mut source = bytes.as_slice();
    let captured = diagnostic(&mut source).await.expect("bounded diagnostic");
    assert!(source.is_empty());
    assert!(captured.starts_with(&"x".repeat(MAX_DIAGNOSTIC_BYTES)));
    assert!(captured.ends_with("truncated after 65536 bytes."));
    let exact = vec![b'y'; MAX_DIAGNOSTIC_BYTES];
    assert_eq!(
        diagnostic(exact.as_slice())
            .await
            .expect("exact diagnostic limit"),
        "y".repeat(MAX_DIAGNOSTIC_BYTES)
    );
    assert_eq!(diagnostic(&b""[..]).await.expect("empty diagnostic"), "");
    assert_eq!(
        diagnostic(&b"bad\xff"[..])
            .await
            .expect("non-UTF-8 diagnostic"),
        "bad\u{fffd}"
    );
}
