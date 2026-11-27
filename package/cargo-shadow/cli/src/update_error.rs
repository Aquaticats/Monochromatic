//! What:
//! The failure of a catalog update and the text it prints,
//! and the two conversions that every step of an update maps its own error through.
//! Why:
//! The update's steps all fail into one type,
//! and the cargo variant carries the remedies for a release-age or toolchain problem,
//! so the type and its wording stay in one file that no step owns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update_error.ts: the UpdateError class, its message, and the two helpers that wrap any failure in it
//! ```

/// What:
/// `use std::fmt;` imports the standard library's formatting module,
/// which holds `Display` (a type that can be printed with `{}`),
/// `Formatter` (the printer that a `Display` implementation writes into) and `Result` (the success-or-failure value of one write).
/// Why:
/// The error's message is written with `writeln!` into a `Formatter`,
/// and `other_text` accepts any `Display` value,
/// so this module names the items both need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has no separate formatting module: String(value) and template literals are the whole of it.
/// ```
use std::fmt;

/// Why an update did not complete;
/// every lockfile was restored first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateError {
    /// Cargo could not resolve a crate.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup,
    /// file operation or parse failed.
    Other {
        /// What failed.
        message: String,
    },
}

/// What:
/// `impl fmt::Display for UpdateError` lets an update failure be printed with `{}`.
/// Why:
/// The shadow and the CLI print the error,
/// and a cargo failure carries the remedies for a release-age or toolchain problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UpdateError extends Error { toString() { ... } }
/// ```
impl fmt::Display for UpdateError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the message of whichever variant the error is (a `match` over the cases).
    /// Why:
    /// A cargo failure shows cargo's output plus the release-age and toolchain remedies,
    /// so the user can act without reading the code.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.kind === "cargo" ? this.cargoMessage() : this.message; }
    /// ```
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      The message of an `UpdateError` differs for a cargo failure (command, output, remedies) and for the catch-all, and the compiler checks both are covered.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        match self {
            UpdateError::Cargo {
                crate_dir,
                command,
                output,
            } => {
                // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
                // Why:      A failed write into the formatter ends the message and is reported to the caller of `fmt`.
                // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const value = await expr;   // throws to the caller on failure
                // ```
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      `writeln!` writes the failed command and crate on the first line and each remedy on its own line.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                writeln!(
                    formatter,
                    "`{command}` failed in {crate_dir}; every lockfile this run changed was put back (a file another process wrote meanwhile is kept and named in the warnings)."
                )?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(
                    formatter,
                    "Fix the cause cargo names. When it is that no release old enough satisfies a requirement,"
                )?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(
                    formatter,
                    "either wait, add the crate to minimumReleaseAgeExclude in rust-catalog.toml,"
                )?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(
                    formatter,
                    "or set minimumReleaseAgeStrict = false to fall back to a younger release."
                )?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(
                    formatter,
                    "When cargo says `-Z` needs the nightly channel, set RUSTUP_TOOLCHAIN to the nightly toolchain.lock.json pins."
                );
            }
            // Macro `write!`: builds or prints text (or a list) from a template.
            UpdateError::Other {
                message,
            } => return write!(formatter, "{message}"),
        }
    }
}

/// What:
/// `fn other(message) -> UpdateError` wraps a message in the catch-all update error.
/// Why:
/// `map_err(other)` turns a plain text failure into the error type of the function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const other = (message: string) => new UpdateError(message)
/// ```
pub(crate) fn other(message: String) -> UpdateError {
    return UpdateError::Other {
        message,
    };
}

/// What:
/// `fn other_text(error) -> UpdateError` wraps any displayable error as the catch-all update error (`impl Display`:
/// any type that prints with `{}`).
/// Why:
/// `.map_err(other_text)` converts the I/O and parse errors of the update steps without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const otherText = (error: unknown) => new UpdateError(String(error))
/// ```
pub(crate) fn other_text(error: impl fmt::Display) -> UpdateError {
    return other(error.to_string());
}
