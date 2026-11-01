//! What:
//!  Typed configuration failures with operation-focused messages.
//! Why:
//!  The CLI can distinguish setup errors from rule findings without parsing diagnostics,
//!  and every message names the binary that rejected the configuration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ConfigError extends Error {}
//! ```

/// Import the running binary's quotable identity for the message suffix.
use crate::run_finish::binary_identity;

/// What:
///  An owned message for a rejected configuration.
/// Why:
///  A String owns its bytes,
///  unlike a borrowed &str,
///  so the error can outlive the input document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConfigError extends Error { readonly message: string; readonly unknownRule?: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError {
    /// Explanation that the caller prefixes with the affected configuration path.
    /// The message always ends with the running binary's identity suffix.
    pub message: String,
    /// Rule identifier the message rejects,
    ///  present only for an unregistered rule name.
    /// Why:
    ///  Path-aware composition adds staleness context for this failure kind
    ///  without parsing the message text.
    pub unknown_rule: Option<String>,
}

/// What:
///  End one configuration diagnostic with the running binary's identity.
/// Why:
///  A startup failure must name the binary that produced it,
///  because a stale resolved binary
///  changes which rules exist without changing the configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function identify(message: string): string { return `${message} (${binaryIdentity()})`; }
/// ```
fn identify(message: &str) -> String {
    return format!("{} ({})", message, binary_identity());
}

/// What:
///  Construction of configuration failures.
/// Why:
///  Call sites keep their validation condition next to its user-facing explanation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ConfigError(message);
/// ```
impl ConfigError {
    /// What:
    ///  Copy borrowed message text into the error's owned storage,
    ///  ending it with the running binary's identity.
    /// Why:
    ///  Temporary formatted messages must remain valid after validation returns,
    ///  and every
    /// configuration rejection must say which binary rejected it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(message: string) { super(identify(message)); }
    /// ```
    pub fn new(message: &str) -> ConfigError {
        // What: Allocate owned strings for the identified message and no rule marker.
        // Why: Rust does not implicitly copy a borrowed string into an owning field,
        //  and a
        // non-rule rejection names no rule identifier.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { message: identify(message) };
        // ```
        return ConfigError {
            message: identify(message),
            unknown_rule: None,
        };
    }

    /// What:
    ///  Build the rejection of one rule identifier the shipped registry lacks.
    /// Why:
    ///  The message text and the retained identifier stay together,
    ///  so path-aware composition
    /// can add staleness context without parsing the message.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static unknownBuiltInRule(id: string): ConfigError;
    /// ```
    pub fn unknown_built_in_rule(id: &str) -> ConfigError {
        // What: Reuse the leaf constructor and record which identifier it rejects.
        // Why: The identity suffix and the marker must agree on one construction path.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { message: identify(`Unknown built-in rule ${id}.`), unknownRule: id };
        // ```
        let error: ConfigError = ConfigError::new(format!("Unknown built-in rule {id}.").as_str());
        return ConfigError {
            message: error.message,
            unknown_rule: Some(String::from(id)),
        };
    }

    /// What:
    ///  Compose a wrapper message around an already-identified diagnostic.
    /// Why:
    ///  The inner message already ends with the running binary's identity suffix,
    ///  so this
    /// constructor deliberately appends no second one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static composed(message: string, unknownRule?: string): ConfigError;
    /// ```
    pub fn composed(message: String, unknown_rule: Option<String>) -> ConfigError {
        // What: Store the composed text and the carried marker without decoration.
        // Why: Wrapping renames the affected input,
        //  never the identity of the running binary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { message, unknownRule };
        // ```
        return ConfigError {
            message,
            unknown_rule,
        };
    }
}

/// What:
///  Supply the standard printable-error interface.
/// Why:
///  Callers can render the message without knowing the error's storage layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return this.message; }
/// ```
impl std::fmt::Display for ConfigError {
    /// Borrow the formatter only while it writes this error's message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// What:
///  Mark this value as a standard error.
/// Why:
///  Application error handling can carry it without converting it to an untyped string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Extending Error supplies this integration in TypeScript.
/// ```
impl std::error::Error for ConfigError {}

/// Identity-suffix controls stay outside release artifacts.
#[cfg(test)]
#[path = "config_error_tests.rs"]
mod tests;
