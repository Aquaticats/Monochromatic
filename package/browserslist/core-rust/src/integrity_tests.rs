//! What:
//!  Tests for npm integrity parsing,
//! verification and cache keys.
//! Why:
//!  A known SHA-512 vector pins the digest code,
//! and malformed strings must fail instead of weakening the check.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('integrity', () => { /* known vector, mismatch, malformed */ });
//! ```

use crate::error::BrowserslistError;
use crate::integrity::{cache_key, integrity_of, parse_sha512, verify};

/// The SHA-512 of the three bytes `abc` (FIPS 180-2 test vector) in npm's spelling.
const ABC_INTEGRITY: &str =
    "sha512-3a81oZNherrMQXNJriBBMRLm+k6JqX6iCp7u5ktV05ohkpkqJ0/BqDa6PCOj/uu9RU1EI2Q86A4qmslPpUyknw==";

/// The vector matches,
/// and `integrity_of` reproduces it.
#[test]
fn known_vector() {
    assert_eq!(integrity_of(b"abc"), ABC_INTEGRITY);
    assert_eq!(verify("pkg", ABC_INTEGRITY, b"abc"), Ok(()));
    assert_eq!(parse_sha512(ABC_INTEGRITY).expect("parses").len(), 64);
}

/// Changed bytes fail with both digests in the error.
#[test]
fn mismatch_reports_both_digests() {
    let error = verify("pkg", ABC_INTEGRITY, b"abd").expect_err("mismatch");
    let BrowserslistError::Integrity { subject, expected, actual } = error else {
        panic!("expected an integrity error");
    };
    assert_eq!(subject, "pkg");
    assert_eq!(expected, ABC_INTEGRITY);
    assert_eq!(actual, integrity_of(b"abd"));
}

/// Other algorithms,
/// bad base64,
/// wrong length,
/// and several entries are all refused.
#[test]
fn rejects_malformed_strings() {
    for text in [
        "",
        "sha1-AAAA",
        "sha512-",
        "sha512-%%%",
        "sha512-AAAA",
        &format!("{ABC_INTEGRITY} {ABC_INTEGRITY}"),
        &format!("{ABC_INTEGRITY}\n"),
        &ABC_INTEGRITY.to_uppercase(),
    ] {
        let message = parse_sha512(text).expect_err("rejected").to_string();
        assert!(message.contains("sha512-<base64>"), "{text:?}: {message}");
        assert!(verify("pkg", text, b"abc").is_err(), "{text:?}");
    }
}

/// The cache key is 128 lowercase hex digits and differs for different digests.
#[test]
fn cache_keys() {
    let key = cache_key(ABC_INTEGRITY).expect("key");
    assert_eq!(key.len(), 128);
    assert!(key.bytes().all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    assert!(key.starts_with("ddaf35a193617aba"));
    assert_ne!(key, cache_key(&integrity_of(b"abd")).expect("key"));
    assert!(cache_key("nope").is_err());
}
