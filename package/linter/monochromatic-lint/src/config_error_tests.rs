//! What:
//!  Controls for the identity suffix every configuration rejection carries.
//! Why:
//!  A startup failure must name the running binary,
//!  and a wrapper must never repeat that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('config error', () => { /* identity suffix, unknown-rule marker, composition */ });
//! ```

/// Import the error type and its constructors under test.
use super::ConfigError;

/// The identity suffix the running build appends,
///  read from the manifest so a release bump cannot leave it stale.
fn identity_suffix() -> String {
    return format!("(monochromatic-lint {})", env!("CARGO_PKG_VERSION"));
}

/// Every leaf rejection message ends with the running binary's identity,
///  and prints the same text it stores.
#[test]
fn every_rejection_message_names_the_running_binary() {
    let error: ConfigError = ConfigError::new("Rejected.");
    assert_eq!(
        error.message,
        format!("Rejected. {}", identity_suffix()).as_str()
    );
    assert_eq!(error.to_string(), error.message);
    assert_eq!(error.unknown_rule, None);
}

/// The unknown-rule rejection names the rule,
///  the running binary,
///  and retains the identifier for path-aware composition.
#[test]
fn the_unknown_rule_rejection_names_the_rule_and_the_running_binary() {
    let error: ConfigError = ConfigError::unknown_built_in_rule("rust/no-anonymous-functions");
    assert_eq!(
        error.message,
        format!(
            "Unknown built-in rule rust/no-anonymous-functions. {}",
            identity_suffix()
        )
        .as_str()
    );
    assert_eq!(
        error.unknown_rule,
        Some(String::from("rust/no-anonymous-functions"))
    );
}

/// A wrapper around an already-identified diagnostic keeps its text verbatim,
///  one identity suffix,
///  and the marker it carries.
#[test]
fn composition_reuses_the_identity_suffix_already_in_the_message() {
    let inner: ConfigError = ConfigError::unknown_built_in_rule("rust/avoidable-closure");
    let wrapped: ConfigError = ConfigError::composed(
        format!("Configuration /repo/config.jsonc: {}", inner.message),
        inner.unknown_rule,
    );
    assert_eq!(
        wrapped.message,
        format!(
            "Configuration /repo/config.jsonc: Unknown built-in rule rust/avoidable-closure. {}",
            identity_suffix()
        )
        .as_str()
    );
    assert_eq!(wrapped.message.matches('(').count(), 1);
    assert_eq!(
        wrapped.unknown_rule,
        Some(String::from("rust/avoidable-closure"))
    );
}
