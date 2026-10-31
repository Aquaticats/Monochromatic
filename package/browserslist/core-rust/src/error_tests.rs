//! What:
//!  Tests for the error constructors and their messages.
//! Why:
//!  The "not implemented" prefix and the byte-offset wording are contracts other tests and callers rely on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('BrowserslistError', () => { /* one message per variant */ });
//! ```

use crate::error::{BrowserslistError, data_error, not_implemented};

/// Every variant prints the message shape its doc comment promises.
#[test]
fn messages() {
    assert_eq!(not_implemented("query \"x\"").to_string(), "not implemented: query \"x\"");
    assert_eq!(
        BrowserslistError::Browserslist { message: "Unknown browser Foo".to_string() }.to_string(),
        "Unknown browser Foo"
    );
    assert_eq!(data_error("a.js", 7, "bad").to_string(), "a.js: bad (byte offset 7)");
    assert_eq!(
        BrowserslistError::Data { file: "lock".to_string(), offset: None, message: "bad".to_string() }.to_string(),
        "lock: bad"
    );
    assert_eq!(
        BrowserslistError::Integrity { subject: "caniuse-lite".to_string(), expected: "sha512-A".to_string(), actual: "sha512-B".to_string() }
            .to_string(),
        "caniuse-lite: downloaded bytes hash to sha512-B, but the lock pins sha512-A"
    );
    assert_eq!(
        BrowserslistError::Network { url: "http://x/".to_string(), message: "refused".to_string() }.to_string(),
        "request to http://x/ failed: refused"
    );
    assert_eq!(
        BrowserslistError::Io { path: "/p".to_string(), message: "denied".to_string() }.to_string(),
        "/p: denied"
    );
}

/// The error works as a standard `Error` trait object.
#[test]
fn is_a_std_error() {
    let boxed: Box<dyn std::error::Error> = Box::new(not_implemented("x"));
    assert_eq!(boxed.to_string(), "not implemented: x");
}
