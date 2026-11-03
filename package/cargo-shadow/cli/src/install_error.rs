//! What:
//! The error of an install and the helpers that classify a failure into it.
//! Why:
//! `install.rs` holds the transaction;
//! the error type,
//! its wording and the classification of messages (a registry the policy cannot judge is a refusal,
//! not an
//! operational failure) were split out to keep each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-error.ts
//! export class InstallError extends Error { kind: 'registry' | 'io' | 'refused' | 'unsettled' | ... }
//! ```

/// Imports `fmt` for the `Display` implementation.
use std::fmt;
/// Imports the borrowed path type.
use std::path::Path;

/// Imports the registry failure that converts into an install error.
use crate::http::RegistryError;
/// Imports the snapshot type and the marker of the unsupported-registry message.
use crate::crates::LockSnapshot;
/// Imports the marker text of the unsupported-registry message.
use crate::lockfile::UNSUPPORTED_REGISTRY_MARK;

/// Why an install did not complete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallError {
    /// A frozen install found stale lockfiles.
    Stale {
        /// The repo-relative crate roots whose lockfile is stale.
        stale_dirs: Vec<String>,
    },
    /// Cargo could not bring a crate's lockfile in line.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup failed.
    Registry {
        /// What failed.
        message: String,
    },
    /// A file could not be read or written,
    /// or cargo could not be started.
    Io {
        /// What failed.
        message: String,
    },
    /// A lockfile holds something the policy cannot judge (a registry other than crates.io):
    /// a refusal that stops the command,
    /// not an operational problem.
    Refused {
        /// What the lockfile holds and why it is refused.
        message: String,
    },
    /// The step failed and a lockfile that this run's cargo wrote could not be put back:
    /// building it would run unapproved dependencies,
    /// so the command must stop.
    Unsettled {
        /// Which lockfiles still hold this run's unapproved output.
        message: String,
    },
}

/// What:
/// `impl fmt::Display for InstallError` lets an install failure be printed with `{}`.
/// Why:
/// The shadow and the CLI print the error to the user,
/// and each variant carries its own remedy text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InstallError extends Error { toString() { ... } }
/// ```
impl fmt::Display for InstallError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the message of whichever variant the error is (a `match` over the cases).
    /// Why:
    /// A stale-lock failure names every stale root,
    /// a cargo failure shows cargo's output and the offline hint,
    /// so the user can act without reading the code.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { switch (this.kind) { case "stale": ...; case "cargo": ...; } }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      The message of an `InstallError` depends on which case failed (stale roots, cargo, registry, I/O), and the compiler checks that no case lacks a message.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        match self {
            InstallError::Stale { stale_dirs } => {
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      `writeln!` writes each line of the stale-lock message into the formatter, so the stale roots print one per line.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                writeln!(
                    formatter,
                    "Frozen install failed: {} Cargo.lock file(s) are present but need an update to satisfy their manifests.",
                    stale_dirs.len()
                // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
                // Why:      A failed write into the formatter ends the message at once and is reported to the caller of `fmt`.
                // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const value = await expr;   // throws to the caller on failure
                // ```
                )?;
                for dir in stale_dirs {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Macro `writeln!`: builds or prints text (or a list) from a template.
                    writeln!(formatter, "- {dir}")?;
                }
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Nothing was written. Frozen is the default in CI, as in pnpm; `--locked` or `--frozen` on the cargo command asks for it too.")?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(
                    formatter,
                    "To update them, run `cargo check` (or `cargo-shadow install --no-frozen-lockfile`) outside CI and commit the Cargo.lock changes."
                );
            }
            InstallError::Cargo { crate_dir, command, output } => {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "`{command}` failed in {crate_dir}; every lockfile this run changed was put back (a file another process wrote meanwhile is kept and named in the warnings).")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "The install runs cargo offline, so it can pick only releases the local registry index and cache already hold.")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "When cargo says an index entry or package is missing, fetch it once with network access (`cargo fetch` in the crate)")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "and run the install again, or run `cargo update`, which resolves online under the release-age and trust policy.")?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "When cargo reports conflicting requirements, fix the manifest or the rust-catalog.toml floor that it names.");
            }
            InstallError::Registry { message } => {
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "A registry lookup failed; every lockfile this run changed was put back (a file another process wrote meanwhile is kept and named in the warnings): {message}");
            }
            InstallError::Refused { message } => {
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "{message}");
            }
            InstallError::Unsettled { message } => {
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "{message}");
            }
            InstallError::Io { message } => {
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "{message}");
            }
        }
    }
}

/// What:
/// `impl From<RegistryError> for InstallError` lets a `RegistryError` convert into an `InstallError` automatically where `?` is used.
/// Why:
/// The policy checks fail with a registry error,
/// and the install step reports everything as an install error without wrapping each call by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS: catch (e) { throw new InstallError(e) }
/// ```
impl From<RegistryError> for InstallError {
    /// What:
    /// `fn from(error)` wraps the registry error's message in the `Registry` variant.
    /// Why:
    /// The variant tells the user the lockfiles were restored because a lookup failed,
    /// not because cargo did.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(error: RegistryError): InstallError { return new InstallError("registry", error.message); }
    /// ```
    fn from(error: RegistryError) -> InstallError {
        return InstallError::Registry { message: error.message };
    }
}

/// What:
/// `fn io_error(error) -> InstallError` wraps any displayable error as the `Io` variant of the install error.
/// Why:
/// `.map_err(io_error)` converts file and parse errors of the install steps without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ioError = (error: unknown) => new InstallError("io", String(error))
/// ```
pub(crate) fn io_error(error: impl fmt::Display) -> InstallError {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    return io_message(error.to_string());
}

/// What:
/// `pub(crate) fn io_message(message) -> InstallError` wraps a message text as the `Io` variant.
/// Why:
/// Helpers that already return text (`Result<_, String>`) convert with `.map_err(io_message)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ioMessage = (message: string) => new InstallError("io", message)
/// ```
pub(crate) fn io_message(message: String) -> InstallError {
    // A registry other than crates.io is a refusal of the policy, not an operational failure.
    if message.contains(UNSUPPORTED_REGISTRY_MARK) {
        return InstallError::Refused { message };
    }
    return InstallError::Io { message };
}

/// What:
/// `fn registry_error(message) -> InstallError` wraps a message text as the `Registry` variant.
/// Why:
/// A failed policy lookup is reported as a registry failure,
/// so the user knows the lockfiles were restored because crates.io could not be asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registryError = (message: string) => new InstallError("registry", message)
/// ```
pub(crate) fn registry_error(message: String) -> InstallError {
    if message.contains(UNSUPPORTED_REGISTRY_MARK) {
        return InstallError::Refused { message };
    }
    return InstallError::Registry { message };
}

/// What:
/// `fn invalid_lock_toml(message: String) -> InstallError` turns a TOML parse message of a `Cargo.lock` into an install error.
/// Why:
/// A lockfile that cannot be parsed stops the install with a message that names the file kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invalidLockToml = (message: string) => new InstallError(`a Cargo.lock is not valid TOML: ${message}`);
/// ```
pub(crate) fn invalid_lock_toml(message: String) -> InstallError {
    // Macro `format!`: builds or prints text (or a list) from a template.
    // A registry other than crates.io is a refusal of the policy, not a parse problem.
    if message.contains(UNSUPPORTED_REGISTRY_MARK) {
        return InstallError::Refused { message };
    }
    return InstallError::Io { message: format!("a Cargo.lock could not be read for its registry versions: {message}") };
}

/// What:
/// `fn unapproved_roots(root: &Path, produced: &[LockSnapshot]) -> Vec<String>` lists the crate roots whose live lockfile still holds
/// what this run's cargo wrote.
/// Why:
/// After a rollback the output of this run must be gone;
/// a root where it is still live (or cannot be read) is unsettled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unapprovedRoots(root: string, produced: LockSnapshot[]): string[]
/// ```
pub(crate) fn unapproved_roots(root: &Path, produced: &[LockSnapshot]) -> Vec<String> {
    // `let mut`: a binding that may change.
    // Typed binding: `roots: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut roots: Vec<String> = Vec::new();
    for output in produced {
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&root.join`: lends the value without copying or giving it away.
        match crate::file_owner::observe(&root.join(&output.crate_dir).join(crate::constants::LOCKFILE_NAME)) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            // `.clone()`: an independent owned copy.
            Ok(Some(live)) if live.text == output.source => roots.push(output.crate_dir.clone()),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(_) => {}
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.clone()`: an independent owned copy.
            Err(_) => roots.push(output.crate_dir.clone()),
        }
    }
    return roots;
}
