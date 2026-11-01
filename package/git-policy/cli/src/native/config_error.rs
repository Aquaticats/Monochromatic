//! What:
//!  The one failure type for rejected `cli-git.config.jsonc` content or files.
//! Why:
//!  Callers render a single configuration diagnostic without parsing message text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ConfigError extends Error {}
//! ```

/// What:
///  `ConfigError` owns the explanation of one rejected configuration.
///       `String` is a heap-allocated,
///  growable UTF-8 buffer this struct owns.
///       Sibling the reader might expect:
///  `&str`,
///  a borrowed view that owns nothing.
/// Why:
///   The error outlives the parsed document and the temporary formatted text,
///       so a borrowed `&str` would dangle;
///  `String` keeps the bytes alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConfigError extends Error { readonly message: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError {
    /// Complete diagnostic naming the affected key or file and the accepted values.
    pub message: String,
}

/// What:
///  `impl ConfigError { ... }` attaches functions to the struct,
///  like class statics.
/// Why:
///   Validation sites keep their condition next to its user-facing explanation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ConfigError(message);
/// ```
impl ConfigError {
    /// What:
    ///  Build an error from borrowed text.
    ///  `&str` lends the caller's bytes;
    ///       `String::from` copies them into storage the error owns.
    /// Why:
    ///   Messages are usually `format!` temporaries that end with the caller's statement.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(message: string) { super(message); }
    /// ```
    pub fn new(message: &str) -> ConfigError {
        // What: `return ConfigError { message: ... };` builds the struct and hands it back.
        // Why:  Rust never copies a borrowed string into an owning field implicitly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { message };
        // ```
        return ConfigError {
            message: String::from(message),
        };
    }
}

/// What:
///  `impl std::fmt::Display for ConfigError` supplies Rust's "print me" interface.
///       `::` walks module paths,
///  like `std.fmt.Display` would in TS.
/// Why:
///   The executable writes the message with ordinary formatting,
///  not field access.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return this.message; }
/// ```
impl std::fmt::Display for ConfigError {
    /// What:
    ///  `&self` borrows this error read-only;
    ///  `&mut` lends the formatter for writing;
    ///       `'_` is an unnamed lifetime the compiler fills in.
    /// Why:
    ///   Formatting writes into the caller's buffer without allocating another string.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// format(out: Writer): void { out.write(this.message); }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `.as_str()` lends the owned message as a borrowed view for the writer.
        return formatter.write_str(self.message.as_str());
    }
}

/// What:
///  An empty `impl` marks the type as a standard error value.
/// Why:
///   Generic error handling can carry it without converting it to a bare string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `extends Error` already provides this in TypeScript.
/// ```
impl std::error::Error for ConfigError {}

/// Message controls stay out of the release executable.
#[cfg(test)]
#[path = "config_error_tests.rs"]
mod tests;
