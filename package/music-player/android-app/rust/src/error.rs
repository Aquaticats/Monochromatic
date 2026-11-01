//! One error type for the whole native engine.
//!  Five failure cases (I/O,
//! symphonia decode,
//!  opus decode,
//!  unsupported file/codec,
//!  and AAudio output)
//! all funnel into `PlayerError` so any function can use the `?` operator
//! freely:
//!  a sub-error gets converted into a `PlayerError` and propagated up
//! with no bespoke conversion at the call site.
//!  The `Audio` variant carries
//! AAudio (Android's native low-latency audio output) failures,
//!  formatted to
//! text in `engine_worker.rs`.
//!  (Think of this `//!` block as the file-level
//! docstring that would sit at the very top of a TS module.)

/// What:
///      `use std::fmt;` pulls in the standard-library formatting module.
///  It
///           is the module that defines `Display` (the "pretty,
///  user-facing
///           string" trait),
///  `Formatter` (the buffer we write that string into),
///           and the `write!` macro that targets a `Formatter`.
/// Why:
///       We need it so the `impl fmt::Display` block below,
///  and every
///           `fmt::Formatter` / `fmt::Result` name in this file,
///  is in scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import: toString() is built in
/// ```
use std::fmt;

/// Decode-path failures:
///  wrapped I/O,
///  symphonia,
///  and opus errors,
///  plus an owned
/// message for unsupported files/codecs.
// What:     `#[derive(Debug)]` is an attribute that auto-generates code: it asks
//           the compiler to write a `Debug` implementation for the enum on the
//           next line. `Debug` is the trait behind the `{:?}` format specifier,
//           i.e. the developer-facing dump of a value (as opposed to `Display`,
//           the user-facing one we hand-write below).
// Why:      Errors must be `Debug` so they can flow through `?`/`Result`, be
//           unwrapped, and be logged; the `std::error::Error` bound below also
//           requires it. Deriving saves us writing that boilerplate by hand.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation: every JS value already has a default string form
// ```
#[derive(Debug)]
// What:     `pub enum PlayerError { ... }` declares a public SUM TYPE: a value of
//           this type is exactly ONE of the listed variants at a time, never
//           several. Several variants are tuple-style (`Name(InnerType)`), which
//           means the variant WRAPS one inner value. `pub` makes the type
//           visible to other modules in this crate and to crates that depend on
//           it.
// Why:      One unified error type means the `?` operator can convert any
//           sub-error (io, symphonia, opus) into this single type and propagate
//           it up, with no per-call-site conversion code.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayerError =
//   | { kind: "io"; cause: Error }
//   | { kind: "decode"; cause: Error }
//   | { kind: "opus"; cause: Error }
//   | { kind: "unsupported"; message: string }
//   | { kind: "audio"; message: string };
// ```
pub enum PlayerError {
    /// What:
    ///      `Io(std::io::Error)` is a tuple-style variant:
    ///  the `Io` case wraps
    ///           one inner value of type `std::io::Error` (the standard library's
    ///           filesystem/stream error).
    ///  Siblings the reader might expect here:
    ///           the symphonia / opus error types the other variants wrap;
    ///  we use
    ///           the `std::io` one specifically for OS-level read/open failures.
    /// Why:
    ///       Opening or reading the audio file from disk (or a `content://`
    ///           fd) can fail,
    ///  and we want to carry the original error through.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "io"; cause: Error }
    /// ```
    Io(
        /// What:
        ///      Unnamed field `.0` of the `Io` variant:
        ///  a wrapped `std::io::Error`
        ///           (the standard library's filesystem/stream error;
        ///  siblings the other
        ///           variants wrap:
        ///  a `symphonia` error,
        ///  an `opus::Error`,
        ///  and an owned
        ///           `String`).
        /// Why:
        ///       Carries the real OS read/open failure so `Display` and the `?`
        ///           `From` conversions can surface the original cause.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `cause: Error` payload of { kind: "io" }
        /// ```
        std::io::Error,
    ),
    /// What:
    ///      `Decode(symphonia::core::errors::Error)` is a tuple variant that
    ///           wraps one symphonia error value.
    ///  `symphonia::core::errors::Error`
    ///           is symphonia's own error type (the `::` segments are a module
    ///           path:
    ///  crate `symphonia`,
    ///  module `core`,
    ///  module `errors`,
    ///  type
    ///           `Error`).
    ///  Sibling wrappers:
    ///  `std::io::Error` (above),
    ///  `opus::Error`
    ///           (below).
    /// Why:
    ///       Probing the container format,
    ///  demuxing it,
    ///  or decoding a packet
    ///           via symphonia can fail;
    ///  we keep that original error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "decode"; cause: Error }
    /// ```
    Decode(
        /// What:
        ///      Unnamed field `.0` of the `Decode` variant:
        ///  a wrapped
        ///           `symphonia::core::errors::Error` (siblings:
        ///  `std::io::Error`,
        ///           `opus::Error`,
        ///  owned `String`).
        /// Why:
        ///       Carries the container probe / demux / packet-decode failure for
        ///           display and propagation.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `cause: Error` payload of { kind: "decode" }
        /// ```
        symphonia::core::errors::Error,
    ),
    /// What:
    ///      `Opus(opus::Error)` is a tuple variant wrapping one `opus::Error`
    ///           value (the libopus binding's error type;
    ///  `opus::` is the crate
    ///           path).
    ///  Sibling wrappers are the `std::io` and `symphonia` error
    ///           types above.
    /// Why:
    ///       Decoding an Opus packet via libopus can fail;
    ///  we preserve its
    ///           error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "opus"; cause: Error }
    /// ```
    Opus(
        /// What:
        ///      Unnamed field `.0` of the `Opus` variant:
        ///  a wrapped `opus::Error`
        ///           (the libopus binding's error type;
        ///  siblings:
        ///  `std::io::Error`,
        ///  the
        ///           `symphonia` error,
        ///  owned `String`).
        /// Why:
        ///       Carries the libopus packet-decode failure for display and
        ///           propagation.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `cause: Error` payload of { kind: "opus" }
        /// ```
        opus::Error,
    ),
    /// What:
    ///      `Unsupported(String)` is a tuple variant carrying one OWNED text
    ///           message (an owned,
    ///  heap-allocated,
    ///  growable UTF-8 string).
    ///  The
    ///           type is `String`,
    ///  NOT its sibling `&str` (a borrowed view that
    ///           does not own its bytes),
    ///  because the error value outlives the
    ///           function call that built it and must own its own text.
    /// Why:
    ///       Some files/codecs we simply cannot play;
    ///  this variant carries a
    ///           plain reason.
    ///  In this crate it is built with messages such as
    ///           "no audio track",
    ///  "track has no audio codec parameters",
    ///  and
    ///           "seek:
    ///  track not found".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "unsupported"; message: string }
    /// ```
    Unsupported(
        /// What:
        ///      Unnamed field `.0` of the `Unsupported` variant:
        ///  an OWNED `String`
        ///           message (sibling `&str` would be a borrowed view that cannot outlive
        ///           the call that built it).
        /// Why:
        ///       Holds a plain reason ("no audio track",
        ///  "no codec parameters",
        ///           "seek:
        ///  track not found") with no underlying error object to wrap.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `message: string` payload of { kind: "unsupported" }
        /// ```
        String,
    ),
    /// What:
    ///      `Audio(String)` is a tuple variant carrying one OWNED text message
    ///           (`String`,
    ///  the owned heap UTF-8 buffer;
    ///  sibling `&str` is the
    ///           borrowed view we DON'T use,
    ///  for the same outlive reason as
    ///           `Unsupported`).
    ///  This case holds AAudio (Android's native
    ///           low-latency audio-output API) failures whose native error values
    ///           we flatten to text rather than wrap structurally.
    /// Why:
    ///       Opening,
    ///  building,
    ///  or starting the AAudio output stream can fail;
    ///           in `engine_worker.rs` such an error is formatted to a string and
    ///           handed to this variant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "audio"; message: string }
    /// ```
    Audio(
        /// What:
        ///      Unnamed field `.0` of the `Audio` variant:
        ///  an OWNED `String` message
        ///           (sibling `&str` would dangle once the call returns).
        /// Why:
        ///       AAudio output failures arrive as native error values we flatten to
        ///           one owned text message rather than wrapping structurally.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `message: string` payload of { kind: "audio" }
        /// ```
        String,
    ),
}

/// What:
///      `impl fmt::Display for PlayerError { ... }` attaches the `Display`
///           trait's behaviour TO our `PlayerError` type.
///  `Display` is the
///           "pretty,
///  user-facing string" trait (contrast `Debug`,
///  the developer
///           dump).
///  Writing `impl Trait for Type` is how Rust says "this type now
///           satisfies this trait",
///  adding the trait's method to it.
/// Why:
///       We print these errors to logs and embed them inside other messages,
///           so we need a clean human-readable form for each variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // class PlayerError { toString(): string { switch (this.kind) { ... } } }
/// ```
impl fmt::Display for PlayerError {
    /// What:
    ///      `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` is the
    ///           one method `Display` requires.
    ///  `&self` is a READ-ONLY borrow of
    ///           the error (we look at it,
    ///  we don't take ownership or mutate it).
    ///           `f: &mut fmt::Formatter<'_>` is the output sink borrowed MUTABLY
    ///           (we write characters into it).
    ///  The `'_` inside `Formatter<'_>` is
    ///           an inferred lifetime placeholder (Rust filling in "how long the
    ///           borrow lives" for us).
    ///  The return type `fmt::Result` is an alias
    ///           for `Result<(), fmt::Error>` — success carries the empty tuple
    ///           `()`,
    ///  failure carries a formatting error.
    /// Why:
    ///       This is the single function `Display` demands;
    ///  implementing it is
    ///           what makes `PlayerError` printable.
    /// Gotcha:
    ///    `&mut f` means the buffer is LENT to us to write into;
    ///  this
    ///           function does not own or free it,
    ///  and while we hold the mutable
    ///           borrow no other code may touch that buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { switch (this.kind) { ... } }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match self { ... }` inspects which variant `self` currently
        //           is and runs the matching arm. Each arm pattern (`PlayerError::Io(e)`
        //           etc.) binds the wrapped inner value to a name (`e` or `m`) BY
        //           REFERENCE, because `self` is borrowed (`&self`) rather than
        //           owned, so we may only look at the inner value, not move it out.
        // Why:      Produce a message tailored to whichever failure case this is.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.kind) { ... }
        // ```
        match self {
            // What:     `PlayerError::Io(e) => write!(f, "i/o error: {e}")` is one
            //           match arm. The left side `PlayerError::Io(e)` matches the
            //           `Io` variant and binds its inner `std::io::Error` to `e`.
            //           The right side calls the `write!` MACRO, which formats text
            //           into `f` (the borrowed buffer); `{e}` interpolates `e`'s OWN
            //           `Display` string. The arm's value is `write!`'s `fmt::Result`,
            //           which (being the last expression in `fmt`) becomes `fmt`'s
            //           return value.
            // Why:      Prefix the category ("i/o error: "), then defer to the inner
            //           io error's own message for the detail.
            // Gotcha:   `write!(f, ...)` APPENDS to a buffer and returns a `Result`;
            //           it is NOT `console.log` and prints nothing on its own. With
            //           no trailing `;`, this arm value IS the function's return.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "i/o error: " + e;
            // ```
            PlayerError::Io(e) => return write!(f, "i/o error: {e}"),
            // What:     `PlayerError::Decode(e) => write!(f, "decode error: {e}")`.
            //           Same shape as the `Io` arm: match the `Decode` variant, bind
            //           its inner symphonia error to `e`, then `write!` a prefixed
            //           message into `f` with `{e}` interpolating the inner error.
            // Why:      Surface decode failures with the symphonia error's own detail.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "decode error: " + e;
            // ```
            PlayerError::Decode(e) => return write!(f, "decode error: {e}"),
            // What:     `PlayerError::Opus(e) => write!(f, "opus error: {e}")`. Match
            //           the `Opus` variant, bind its inner `opus::Error` to `e`, and
            //           `write!` a prefixed message; `{e}` interpolates the opus
            //           error's own `Display`.
            // Why:      Surface opus failures with their own detail.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "opus error: " + e;
            // ```
            PlayerError::Opus(e) => return write!(f, "opus error: {e}"),
            // What:     `PlayerError::Unsupported(m) => write!(f, "unsupported: {m}")`.
            //           Match the `Unsupported` variant and bind its inner message to
            //           `m`. Because `self` is borrowed, `m` is a `&String` (a
            //           read-only borrow of the owned message, not a copy). `{m}`
            //           interpolates that text.
            // Why:      Surface the plain explanation we built when constructing the
            //           error (e.g. "no audio track").
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "unsupported: " + m;
            // ```
            PlayerError::Unsupported(m) => return write!(f, "unsupported: {m}"),
            // What:     `PlayerError::Audio(m) => write!(f, "audio output error: {m}")`.
            //           Match the `Audio` variant and bind its inner AAudio message
            //           to `m` (a `&String` borrow, as above). The literal prefix is
            //           the exact string `"audio output error: "`, then `{m}`
            //           interpolates the flattened AAudio error text.
            // Why:      Surface the AAudio (native output) explanation built in
            //           `engine_worker.rs`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "audio output error: " + m;
            // ```
            PlayerError::Audio(m) => return write!(f, "audio output error: {m}"),
        }
    }
}

/// What:
///      `impl std::error::Error for PlayerError {}` marks our type as a
///           STANDARD library error.
///  `std::error::Error` is the common trait every
///           "real" error implements.
///  The body is empty `{}`,
///  which means we accept
///           every default method the trait provides (notably we do NOT override
///           `source()`,
///  so we expose no underlying cause chain).
/// Why:
///       With this,
///  `PlayerError` can be accepted by any caller that wants a
///           generic standard error,
///  and `?` can propagate it into such contexts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlayerError extends Error {}
/// ```
impl std::error::Error for PlayerError {}

/// What:
///      `impl From<std::io::Error> for PlayerError { ... }` defines a CONVERSION
///           recipe:
///  how to build a `PlayerError` FROM a `std::io::Error`.
///  `From<X>`
///           is the standard "convert an `X` into me" trait,
///  and the `<...>` is its
///           generic type argument naming the source type.
///  Implementing it is what
///           lets the `?` operator auto-convert errors.
/// Why:
///       So that `let f = File::open(p)?;` turns an io error into a
///           `PlayerError` automatically right at the `?`,
///  with no manual mapping.
/// Gotcha:
///    Implementing `From<X>` is precisely what makes `?` SILENTLY convert an
///           `X` error into a `PlayerError`.
///  TS has nothing like this;
///  a `throw`
///           rethrows the same object,
///  it never re-types it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implicit: a thrown fs Error is just rethrown unchanged
/// ```
impl From<std::io::Error> for PlayerError {
    /// What:
    ///      `fn from(e: std::io::Error) -> PlayerError` is the single method
    ///           `From` requires.
    ///  It takes the io error `e` BY VALUE (taking
    ///           ownership of it;
    ///  the caller no longer owns it afterward) and
    ///           returns a brand-new `PlayerError`.
    /// Why:
    ///       Turn the raw io error into our `Io` variant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(e: Error): PlayerError { return { kind: "io", cause: e }; }
    /// ```
    fn from(e: std::io::Error) -> PlayerError {
        // What:     `PlayerError::Io(e)` constructs the `Io` variant, wrapping the
        //           owned error `e` inside it. There is no trailing `;`, so this is
        //           the function's tail expression and therefore its return value.
        // Why:      Wrap the io error in our enum and hand it back as a
        //           `PlayerError`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "io", cause: e };
        // ```
        return PlayerError::Io(e)
    }
}

/// What:
///      `impl From<symphonia::core::errors::Error> for PlayerError { ... }`
///           defines the conversion FROM a symphonia error INTO a `PlayerError`,
///           the same `From` trait as above but with symphonia's error type as the
///           `<...>` source argument.
/// Why:
///       So `?` on any symphonia call can produce a `PlayerError` automatically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implicit rethrow of a decoder error
/// ```
impl From<symphonia::core::errors::Error> for PlayerError {
    /// What:
    ///      `fn from(e: symphonia::core::errors::Error) -> PlayerError` takes the
    ///           symphonia error `e` BY VALUE (taking ownership) and returns a new
    ///           `PlayerError`.
    /// Why:
    ///       Turn the raw symphonia error into our `Decode` variant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(e: Error): PlayerError { return { kind: "decode", cause: e }; }
    /// ```
    fn from(e: symphonia::core::errors::Error) -> PlayerError {
        // What:     `PlayerError::Decode(e)` constructs the `Decode` variant wrapping
        //           the owned symphonia error `e`. No trailing `;`, so this tail
        //           expression is the return value.
        // Why:      Wrap and return the symphonia error as our `PlayerError`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "decode", cause: e };
        // ```
        return PlayerError::Decode(e)
    }
}

/// What:
///      `impl From<opus::Error> for PlayerError { ... }` defines the conversion
///           FROM an `opus::Error` INTO a `PlayerError`,
///  again the `From` trait with
///           the opus error type as the `<...>` source argument.
/// Why:
///       So `?` on any opus call can produce a `PlayerError` automatically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implicit rethrow of an opus error
/// ```
impl From<opus::Error> for PlayerError {
    /// What:
    ///      `fn from(e: opus::Error) -> PlayerError` takes the opus error `e` BY
    ///           VALUE (taking ownership) and returns a new `PlayerError`.
    /// Why:
    ///       Turn the raw opus error into our `Opus` variant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(e: Error): PlayerError { return { kind: "opus", cause: e }; }
    /// ```
    fn from(e: opus::Error) -> PlayerError {
        // What:     `PlayerError::Opus(e)` constructs the `Opus` variant wrapping the
        //           owned opus error `e`. No trailing `;`, so this tail expression is
        //           the return value.
        // Why:      Wrap and return the opus error as our `PlayerError`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "opus", cause: e };
        // ```
        return PlayerError::Opus(e)
    }
}
