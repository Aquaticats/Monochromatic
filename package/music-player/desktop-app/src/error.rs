//! One error type for the whole player.
//!  Decoding,
//!  I/O,
//!  Opus,
//!  and audio-output
//! failures all funnel into `PlayerError` so functions can use `?` freely.

/// What:
///      `use std::fmt;` imports the formatting module (it defines `Display`,
///           `Formatter`,
///  and the `write!` macro target).
/// Why:
///       We implement human-readable error messages via `fmt::Display`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import: toString() is built in
/// ```
use std::fmt;

// What:     `#[derive(Debug)]` auto-implements `{:?}` debug printing for the enum
//           on the next line. Errors must be `Debug` to flow through `?`/`Result`
//           and to be unwrapped/printed.
// Why:      Logging and the `std::error::Error` bound both want `Debug`.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation: every JS value already has a default string form
// ```
#[derive(Debug)]
/// What:
///      `pub enum PlayerError { ... }` a SUM TYPE:
///  a value is exactly one of
///           these failure cases.
///  Several variants WRAP an inner error value (a
///           tuple-style variant `Name(InnerType)`).
/// Why:
///       A single error type lets `?` convert any sub-error into this and
///           propagate it up without bespoke conversions at every call site.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlayerError =
///   | { kind: "io"; cause: Error }
///   | { kind: "decode"; cause: Error }
///   | { kind: "opus"; cause: Error }
///   | { kind: "unsupported"; message: string }
///   | { kind: "audio"; message: string };
/// ```
pub enum PlayerError {
    /// What:
    ///      `Io(std::io::Error)` a tuple variant wrapping a filesystem/stream
    ///           error value (`std::io::Error`;
    ///  sibling here:
    ///  the `symphonia`/`opus`
    ///           error types the other variants wrap).
    /// Why:
    ///       Opening or reading a file can fail.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "io"; cause: Error }
    /// ```
    Io(
        /// What:
        ///      Unnamed field `.0` of the `Io` variant:
        ///  a wrapped
        ///           `std::io::Error` (siblings the other variants wrap:
        ///  a
        ///           `symphonia::core::errors::Error`,
        ///  an `opus::Error`,
        ///  an owned
        ///           `String`).
        /// Why:
        ///       Carries the real OS/stream failure for `Display` and `?`
        ///           propagation.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `cause: Error` payload of { kind: "io" }
        /// ```
        std::io::Error,
    ),
    /// What:
    ///      `Decode(symphonia::core::errors::Error)` wraps a symphonia error.
    /// Why:
    ///       Probing/demuxing/decoding a container can fail.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "decode"; cause: Error }
    /// ```
    Decode(
        /// What:
        ///      Unnamed field `.0` of the `Decode` variant:
        ///  a wrapped
        ///           `symphonia::core::errors::Error` (siblings the other variants
        ///           wrap:
        ///  a `std::io::Error`,
        ///  an `opus::Error`,
        ///  an owned `String`).
        /// Why:
        ///       Carries the real symphonia probe/demux/decode failure for
        ///           `Display` and `?` propagation.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `cause: Error` payload of { kind: "decode" }
        /// ```
        symphonia::core::errors::Error,
    ),
    /// What:
    ///      `Opus(opus::Error)` wraps a libopus error.
    /// Why:
    ///       Opus packet decoding can fail.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "opus"; cause: Error }
    /// ```
    Opus(
        /// What:
        ///      Unnamed field `.0` of the `Opus` variant:
        ///  a wrapped
        ///           `opus::Error` (siblings the other variants wrap:
        ///  a
        ///           `std::io::Error`,
        ///  a `symphonia::core::errors::Error`,
        ///  an owned
        ///           `String`).
        /// Why:
        ///       Carries the real libopus packet-decode failure for `Display` and
        ///           `?` propagation.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `cause: Error` payload of { kind: "opus" }
        /// ```
        opus::Error,
    ),
    /// What:
    ///      `Unsupported(String)` carries an OWNED message describing an
    ///           unsupported file/codec situation.
    ///  `String` (owned,
    ///  heap-allocated)
    ///           not `&str` (a borrowed view) because the error outlives the call
    ///           that made it.
    /// Why:
    ///       Report "no audio track" / ">2 Opus channels" cleanly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "unsupported"; message: string }
    /// ```
    Unsupported(
        /// What:
        ///      Unnamed field `.0` of the `Unsupported` variant:
        ///  an OWNED,
        ///           heap-allocated `String` message (siblings the other variants
        ///           wrap:
        ///  a `std::io::Error`,
        ///  a `symphonia` error,
        ///  an `opus::Error`).
        ///           `String` (owned) not `&str` (a borrowed view that does not own
        ///           its bytes).
        /// Why:
        ///       Carries the "no audio track" / ">2 Opus channels" explanation
        ///           for `Display`;
        ///  `String` (not `&str`) because the error outlives
        ///           the call that built it,
        ///  so a borrowed slice would dangle.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `message: string` payload of { kind: "unsupported" }
        /// ```
        String,
    ),
    /// What:
    ///      `Audio(String)` an OWNED message for audio-output (PipeWire) failures
    ///           whose native error types we flatten to text.
    ///  `String` over `&str`
    ///           for the same outlive reason as `Unsupported`.
    /// Why:
    ///       PipeWire errors come in several types;
    ///  one string is enough here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "audio"; message: string }
    /// ```
    Audio(
        /// What:
        ///      Unnamed field `.0` of the `Audio` variant:
        ///  an OWNED,
        ///           heap-allocated `String` message for audio-output (PipeWire)
        ///           failures whose native error types we flatten to text (siblings
        ///           the other variants wrap:
        ///  a `std::io::Error`,
        ///  a `symphonia`
        ///           error,
        ///  an `opus::Error`).
        ///  `String` (owned) not `&str` (a
        ///           borrowed view).
        /// Why:
        ///       Carries the flattened PipeWire explanation for `Display`;
        ///           `String` (not `&str`) for the same outlive reason as
        ///           `Unsupported` (the error outlives the call,
        ///  so a borrowed slice
        ///           would dangle).
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `message: string` payload of { kind: "audio" }
        /// ```
        String,
    ),
}

/// What:
///      `impl fmt::Display for PlayerError { ... }` provides the user-facing
///           message.
///  `Display` is the "pretty print" trait (vs `Debug`'s
///           developer view).
///  Implementing a trait `for` a type adds that trait's
///           methods to it.
/// Why:
///       We print these errors to stderr and into other messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // class PlayerError { toString(): string { switch (this.kind) { ... } } }
/// ```
impl fmt::Display for PlayerError {
    /// What:
    ///      `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`.
    ///  `&self`
    ///           is a read-only borrow of the error;
    ///  `f: &mut fmt::Formatter<'_>` is
    ///           the output sink borrowed MUTABLY (we write into it);
    ///  `'_` is an
    ///           inferred lifetime placeholder;
    ///  the return `fmt::Result` is an alias
    ///           for `Result<(), fmt::Error>`.
    /// Why:
    ///       The single method `Display` requires.
    /// Gotcha:
    ///    `&mut f` means we are LENT the buffer to write into;
    ///  the function
    ///           does not own or free it,
    ///  and no other code may touch it meanwhile.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { switch (this.kind) { ... } }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match self { ... }` dispatches on which variant this is. Each
        //           arm binds the inner value by reference (`e`/`m`) because `self`
        //           is borrowed, not owned.
        // Why:      Produce a message tailored to the case.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.kind) { ... }
        // ```
        match self {
            // What:     `PlayerError::Io(e) => write!(f, "i/o error: {e}")`. The
            //           `write!` MACRO formats into `f`; `{e}` interpolates the inner
            //           error's OWN `Display`. The arm's value is the macro's
            //           `fmt::Result`, returned from `fmt`.
            // Why:      Prefix the category, then defer to the inner message.
            // Gotcha:   `write!(f, ...)` appends to a buffer and returns a `Result`;
            //           it is NOT `console.log`. The trailing arm value IS the return.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "i/o error: " + e;
            // ```
            PlayerError::Io(e) => return write!(f, "i/o error: {e}"),
            // What:     `PlayerError::Decode(e) => write!(f, "decode error: {e}")`.
            //           Same shape: bind the inner symphonia error `e`, write a
            //           prefixed message.
            // Why:      Surface decode failures with their own detail.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "decode error: " + e;
            // ```
            PlayerError::Decode(e) => return write!(f, "decode error: {e}"),
            // What:     `PlayerError::Opus(e) => write!(f, "opus error: {e}")`. Bind
            //           the inner opus error `e` and write a prefixed message.
            // Why:      Surface opus failures with their own detail.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "opus error: " + e;
            // ```
            PlayerError::Opus(e) => return write!(f, "opus error: {e}"),
            // What:     `PlayerError::Unsupported(m) => write!(f, "unsupported: {m}")`.
            //           `m` is the `&String` message borrowed from the variant.
            // Why:      Surface the explanation we built when constructing the error.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "unsupported: " + m;
            // ```
            PlayerError::Unsupported(m) => return write!(f, "unsupported: {m}"),
            // What:     `PlayerError::Audio(m) => write!(f, "audio error: {m}")`. `m`
            //           is the borrowed audio message.
            // Why:      Surface the flattened audio-output explanation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "audio error: " + m;
            // ```
            PlayerError::Audio(m) => return write!(f, "audio error: {m}"),
        }
    }
}

/// What:
///      `impl std::error::Error for PlayerError { fn source(..) }` marks the type as a STANDARD
///           error and overrides `source()`, the cause-chain hook, to hand back the wrapped error of
///           the `Io`, `Decode` and `Opus` variants. `Unsupported` and `Audio` hold only a message,
///           so their `source()` is `None`.
/// Why:
///       Lets `PlayerError` interoperate with `anyhow::Error` and `?` in callers
///           that want a generic error, and lets a chain walker (the `anyhow` startup report, a log
///           of `Error::source` links) reach the wrapped `std::io::Error`, symphonia error or opus
///           error instead of only the flattened `Display` line.
/// Gotcha:
///    `Display` for the three wrapper variants also prints the wrapped error, so a chain printed
///           in full shows that text twice: once inside the `PlayerError` line, once as its cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlayerError extends Error { cause?: Error }
/// ```
impl std::error::Error for PlayerError {
    /// What:
    ///      `fn source(&self) -> Option<&(dyn std::error::Error + 'static)>` borrows the wrapped
    ///           error as a trait object; `'static` because the wrapped types own all their data.
    /// Why:
    ///       The default returns `None`, which cuts the chain at `PlayerError`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get cause(): Error | undefined { return this.kind === "io" ? this.inner : undefined; }
    /// ```
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // What:     `match self { ... }` over the five variants. The three wrapper variants bind the
        //           inner error `e` and return `Some(e)`; the two message variants return `None`.
        //           Every variant is listed, with no `_` arm, so a new variant fails to compile here
        //           until its cause is decided.
        // Why:      Forward exactly the causes the variants hold.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.kind) { case "io": case "decode": case "opus": return this.cause; default: return undefined; }
        // ```
        return match self {
            PlayerError::Io(e) => Some(e),
            PlayerError::Decode(e) => Some(e),
            PlayerError::Opus(e) => Some(e),
            PlayerError::Unsupported(_) | PlayerError::Audio(_) => None,
        };
    }
}

/// What:
///      `impl From<std::io::Error> for PlayerError { ... }` defines how to CONVERT
///           an io error into our error.
///  The `From` trait powers the `?` operator's
///           automatic error conversion.
/// Why:
///       So `let f = File::open(p)?;` turns an io error into a `PlayerError`
///           automatically at the `?`.
/// Gotcha:
///    implementing `From<X>` is what makes `?` silently convert an `X` error
///           into a `PlayerError`;
///  there is no TS equivalent (a `throw` just rethrows).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implicit: a thrown fs Error is just rethrown unchanged
/// ```
impl From<std::io::Error> for PlayerError {
    /// What:
    ///      `fn from(e: std::io::Error) -> PlayerError` takes OWNERSHIP of the io
    ///           error (by value) and returns it wrapped.
    /// Why:
    ///       Build the `Io` variant from the raw error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(e: Error): PlayerError { return { kind: "io", cause: e }; }
    /// ```
    fn from(e: std::io::Error) -> PlayerError {
        // What:     `PlayerError::Io(e)` constructs the `Io` variant wrapping `e`.
        //           Tail expression -> return value.
        // Why:      Wrap and return as our error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "io", cause: e };
        // ```
        return PlayerError::Io(e)
    }
}

/// What:
///      `impl From<symphonia::core::errors::Error> for PlayerError { ... }` the
///           `From` conversion for symphonia errors.
/// Why:
///       Enables `?` on symphonia calls to produce a `PlayerError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implicit rethrow of a decoder error
/// ```
impl From<symphonia::core::errors::Error> for PlayerError {
    /// What:
    ///      `fn from(e: symphonia::core::errors::Error) -> PlayerError` takes the
    ///           symphonia error by value.
    /// Why:
    ///       Build the `Decode` variant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(e: Error): PlayerError { return { kind: "decode", cause: e }; }
    /// ```
    fn from(e: symphonia::core::errors::Error) -> PlayerError {
        // What:     `PlayerError::Decode(e)` wraps into the `Decode` variant. Tail
        //           expression -> return value.
        // Why:      Return as our error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "decode", cause: e };
        // ```
        return PlayerError::Decode(e)
    }
}

/// What:
///      `impl From<opus::Error> for PlayerError { ... }` the `From` conversion
///           for opus errors.
/// Why:
///       Enables `?` on opus calls to produce a `PlayerError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implicit rethrow of an opus error
/// ```
impl From<opus::Error> for PlayerError {
    /// What:
    ///      `fn from(e: opus::Error) -> PlayerError` takes the opus error by value.
    /// Why:
    ///       Build the `Opus` variant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(e: Error): PlayerError { return { kind: "opus", cause: e }; }
    /// ```
    fn from(e: opus::Error) -> PlayerError {
        // What:     `PlayerError::Opus(e)` wraps into the `Opus` variant. Tail
        //           expression -> return value.
        // Why:      Return as our error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "opus", cause: e };
        // ```
        return PlayerError::Opus(e)
    }
}

/// Cause-chain tests for `PlayerError`.
#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
