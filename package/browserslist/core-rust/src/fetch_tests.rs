//! What:
//!  Tests for the bounded GET against a fake loopback server.
//! Why:
//!  Status handling,
//! size limits,
//! URL policy and use from inside an async runtime are the behaviors the cache
//!  and update paths rely on,
//! and none of them may touch the real registry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('getBytes', () => { /* 200, non-200, too large, URL policy */ });
//! ```

use crate::error::BrowserslistError;
use crate::fake_registry::FakeRegistry;
use crate::fetch::get_bytes;

/// A 200 answer returns its body.
#[test]
fn returns_the_body() {
    let server = FakeRegistry::start(vec![("/hello", 200, b"world".to_vec())]);
    assert_eq!(get_bytes(&format!("{}/hello", server.url()), 1024).expect("fetches"), b"world".to_vec());
    assert_eq!(server.requests(), vec!["/hello".to_string()]);
}

/// Any other status,
/// including a redirect,
/// is an error that names the status and the URL.
#[test]
fn rejects_non_200_answers() {
    let server = FakeRegistry::start(vec![("/redirect", 302, Vec::new()), ("/error", 500, b"boom".to_vec())]);
    for path in ["/redirect", "/error", "/missing"] {
        let url = format!("{}{path}", server.url());
        let error = get_bytes(&url, 1024).expect_err("rejected");
        let BrowserslistError::Network { url: reported, message } = error else {
            panic!("expected a network error");
        };
        assert_eq!(reported, url);
        assert!(message.contains("not 200"), "{message}");
    }
}

/// A body over the limit is refused instead of buffered.
#[test]
fn enforces_the_size_limit() {
    let server = FakeRegistry::start(vec![("/big", 200, vec![1_u8; 5000])]);
    let message = get_bytes(&format!("{}/big", server.url()), 100).expect_err("too big").to_string();
    assert!(message.contains("within 100 bytes"), "{message}");
    assert_eq!(get_bytes(&format!("{}/big", server.url()), 5000).expect("exact limit").len(), 5000);
}

/// Only https,
/// or http to loopback,
/// is fetched;
/// nothing is sent for the refused URLs.
#[test]
fn applies_the_url_policy() {
    for url in ["http://example.com/x", "ftp://127.0.0.1/x", "file:///etc/passwd", "not a url", "/relative", "http://127.0.0.1.example.com/x"] {
        let message = get_bytes(url, 10).expect_err("refused").to_string();
        assert!(message.starts_with(&format!("request to {url} failed: ")), "{message}");
    }
}

/// A closed port fails fast with a network error.
#[test]
fn reports_connection_failures() {
    let error = get_bytes("http://127.0.0.1:1/x", 10).expect_err("connection refused");
    assert!(matches!(error, BrowserslistError::Network { .. }));
}

/// The call also works from inside a running async runtime (it uses its own thread).
#[test]
fn works_inside_an_async_runtime() {
    let server = FakeRegistry::start(vec![("/ok", 200, b"fine".to_vec())]);
    let runtime = tokio::runtime::Builder::new_current_thread().build().expect("runtime");
    let url = format!("{}/ok", server.url());
    let bytes = runtime.block_on(async { return get_bytes(&url, 10) }).expect("fetches");
    assert_eq!(bytes, b"fine".to_vec());
}
