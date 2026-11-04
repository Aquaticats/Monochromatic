//! What:
//!  Controls for the configuration error value.
//! Why:
//!  The message is the whole diagnostic,
//!  so constructing and printing the error must
//!      carry it byte for byte,
//!  including quotes,
//!  newlines and non-ASCII text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(String(new ConfigError('bad "key"'))).toBe('bad "key"');
//! ```

/// Import the error under test.
use super::ConfigError;

/// Printing yields exactly the stored message;
///  an empty message prints nothing.
#[test]
fn display_prints_exactly_the_message() {
    for message in [
        "Unknown configuration key: plugin",
        "bad \"key\"\nsecond line\t\\",
        "naïve ключ 設定",
        "",
    ] {
        let error: ConfigError = ConfigError::new(message);
        assert_eq!(error.message, message);
        // `.to_string()` goes through the `Display` implementation under test.
        assert_eq!(error.to_string(), message);
        assert_eq!(format!("[{error}]"), format!("[{message}]"));
    }
}

/// Errors compare by message,
///  and the standard error interface reports no underlying cause.
#[test]
fn errors_compare_by_message_and_have_no_source() {
    assert_eq!(ConfigError::new("a"), ConfigError::new("a"));
    assert_ne!(ConfigError::new("a"), ConfigError::new("b"));
    let error: ConfigError = ConfigError::new("a");
    // `&dyn Error` views the value through the standard error interface only.
    let general: &dyn std::error::Error = &error;
    assert!(general.source().is_none());
    assert_eq!(general.to_string(), "a");
}
