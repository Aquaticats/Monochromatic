//! Error type shared by terminal engine operations.

/// What:
///      `use std::fmt;` imports Rust's formatting traits and helper types.
///           The sibling module would be `std::error` for error traits.
/// Why:
///       `Display` needs `fmt::Formatter` and `fmt::Result`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { formatError } from "std/fmt";
/// ```
use std::fmt;

// What:     `#[derive(Debug)]` asks the compiler to generate debug printing for
//           the enum below.
// Why:      Rust's `Error` trait requires `Debug` so failures can be inspected.
//
// In TS you'd write (pseudocode):
// ```ts
// type TerminalError = { kind: "ghostty"; source: Error };
// ```
#[derive(Debug)]
/// What:
///      `pub enum TerminalError` declares a public tagged union.
///  The sibling
///           shape would be a `struct` for one fixed record,
///  but this can grow
///           named variants as more engine error sources appear.
/// Why:
///       The engine currently wraps libghostty-vt errors and keeps room for
///           future PTY or renderer errors without changing callers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TerminalError = { kind: "ghostty"; source: Error };
/// ```
pub enum TerminalError {
    /// What:
    ///      `Ghostty(libghostty_vt::Error)` is an enum variant that wraps one
    ///           libghostty-vt error value.
    ///  `::` is Rust's path separator.
    /// Why:
    ///       Keep the upstream error available as the source.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "ghostty", source }
    /// ```
    Ghostty(
        /// What:
        ///      `libghostty_vt::Error` is the single,
        ///  unnamed `.0` field of this
        ///           tuple variant:
        ///  the owned upstream error value from the
        ///           libghostty-vt crate.
        ///  `::` is Rust's path separator,
        ///  naming the
        ///           `Error` type inside the `libghostty_vt` crate.
        ///  Sibling shape would
        ///           be a type-erased `anyhow::Error`.
        /// Why:
        ///       Keep the concrete `libghostty_vt::Error` (not `anyhow::Error`)
        ///           so the `source()` method can hand the real upstream error back to
        ///           reporters with its exact type intact.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// source: GhosttyError;
        /// ```
        libghostty_vt::Error,
    ),
}

/// What:
///      `impl fmt::Display for TerminalError` teaches Rust how to print the
///           error for humans.
///  The sibling trait is `Debug`,
///  generated above.
/// Why:
///       `anyhow::Error` and `eprintln!` use this text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function terminalErrorToString(error: TerminalError): string { ... }
/// ```
impl fmt::Display for TerminalError {
    /// What:
    ///      `fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result`
    ///           is the required formatting method.
    ///  `&self` borrows the error;
    ///           `&mut` lends the output formatter mutably;
    ///  `'_` lets Rust infer a
    ///           short lifetime.
    /// Why:
    ///       The formatter is where the human-readable message is written.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function toString(error: TerminalError): string {
    ///   return `libghostty-vt error: ${error.source}`;
    /// }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match self` branches on the enum variant by borrowing `self`.
        // Why:      Each variant can print its own message.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (error.kind === "ghostty") return `libghostty-vt error: ${error.source}`;
        // ```
        match self {
            // What:     `TerminalError::Ghostty(source)` extracts the wrapped
            //           upstream error from the enum variant.
            // Why:      Include the underlying libghostty-vt failure in the text.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return `libghostty-vt error: ${source}`;
            // ```
            TerminalError::Ghostty(source) => return write!(formatter, "libghostty-vt error: {source}"),
        }
    }
}

/// What:
///      `impl std::error::Error for TerminalError` marks the enum as a real
///           Rust error type.
///  The sibling trait is `Display`,
///  implemented above.
/// Why:
///       `main` can return `anyhow::Error` and still carry this source chain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TerminalError extends Error {}
/// ```
impl std::error::Error for TerminalError {
    /// What:
    ///      `fn source(&self) -> Option<&(dyn std::error::Error + 'static)>`
    ///           returns a borrowed nested error if one exists.
    ///  `Option` is Rust's
    ///           `value | null`;
    ///  `dyn` means trait object.
    /// Why:
    ///       Error reporters can show the libghostty-vt cause chain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return error.source;
    /// ```
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // What:     `match self` branches on the enum and returns `Some(source)`.
        //           `Some` wraps a present value for `Option`; `&` keeps it borrowed.
        // Why:      Preserve the underlying Ghostty error for diagnostics.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return error.source;
        // ```
        match self {
            // What:     `Some(source)` constructs the present `Option` variant with
            //           a borrowed trait-object-compatible error.
            // Why:      Expose the cause without copying or owning it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return source;
            // ```
            TerminalError::Ghostty(source) => return Some(source),
        }
    }
}

/// What:
///      `impl From<libghostty_vt::Error> for TerminalError` defines an
///           automatic conversion.
///  The sibling manual path is calling the enum
///           variant directly.
/// Why:
///       Engine methods can use `?` on libghostty-vt results.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fromGhostty(source: Error): TerminalError {
///   return { kind: "ghostty", source };
/// }
/// ```
impl From<libghostty_vt::Error> for TerminalError {
    /// What:
    ///      `fn from(source: libghostty_vt::Error) -> Self` consumes the
    ///           upstream error and returns this crate's error enum.
    /// Why:
    ///       This is the hook the `?` operator uses for conversion.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return { kind: "ghostty", source };
    /// ```
    fn from(source: libghostty_vt::Error) -> Self {
        // What:     `TerminalError::Ghostty(source)` constructs the wrapper variant.
        // Why:      Keep the upstream error intact.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "ghostty", source };
        // ```
        return TerminalError::Ghostty(source)
    }
}
