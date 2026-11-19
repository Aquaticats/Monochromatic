//! Parses the CLI target and publishes it atomically to the running service.

/// What
/// Imports Rust's filesystem and input/output modules.
/// Why
/// Keep the command protocol in a file that can be replaced in one operation.
/// TypeScript pseudocode
/// ```ts
/// import * as fs from "node:fs";
/// ```
use std::{fs, io};
/// Borrows paths while clients and the daemon keep ownership of their runtime directory.
use std::path::Path;
/// Provides the method for sending a string to the temporary file.
use std::io::Write;
/// Creates a private temporary file in the destination directory so a rename is atomic.
use tempfile::NamedTempFile;

/// What
/// An enum is a value with exactly one of the listed cases. `u16` holds an unsigned
/// 16-bit integer, compared with wider `u32`, signed `i32`, and index-sized `usize`.
/// Why
/// Whole degrees fit this representation and convert to signed millidegrees without overflow.
/// TypeScript pseudocode
/// ```ts
/// type Target = { kind: "off" } | { kind: "celsius"; degrees: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    /// Release the controller's current restrictions.
    Off,
    /// Keep feedback enabled at the given temperature in degrees Celsius.
    Celsius(u16),
}

/// Runtime filename exchanged between the command and service.
const TARGET_FILE: &str = "target";

/// Millidegrees in one degree, the units Linux hwmon reports.
pub const MILLIDEGREES: i32 = 1_000;

/// What
/// Takes a borrowed string and returns a `Result`, either the target or an owned error string.
/// Why
/// Clap and the service apply the same validation when they receive a target.
/// TypeScript pseudocode
/// ```ts
/// function parseTarget(value: string): Target // throws on invalid input
/// ```
pub fn parse_target(value: &str) -> Result<Target, String> {
    if value == "off" {
        // What
        // `Ok` carries a successful result. `Target::Off` names a case in the enum.
        // Why
        // Off has its own command value on every machine.
        // TypeScript pseudocode
        // return { kind: "off" };
        return Ok(Target::Off);
    }
    // What
    // `parse::<u16>()` selects an unsigned integer parser. `if let Ok` opens only
    // a successful result and binds the integer.
    // Why
    // Reject negative, fractional and out-of-range input before it reaches the daemon.
    // TypeScript pseudocode
    // const degrees = parseUnsigned16(value);
    // if (degrees !== undefined && degrees > 0) return { kind: "celsius", degrees };
    if let Ok(degrees) = value.parse::<u16>()
        && degrees > 0
    {
        // Wrap a valid Celsius value for Clap.
        return Ok(Target::Celsius(degrees));
    }
    // What
    // `Err` carries failure. `.to_owned()` copies the literal into an owned Rust string.
    // Why
    // The caller can print the same diagnostic for command input and damaged runtime state.
    // TypeScript pseudocode
    // throw new Error("expected a positive whole Celsius temperature or off");
    return Err("expected a positive whole Celsius temperature or off".to_owned());
}

/// Store a validated target in one rename in the service's existing directory.
pub fn save_target(runtime: &Path, target: Target) -> io::Result<()> {
    // What
    // `?` returns the current function's error early when an operation fails.
    // Why
    // A command that cannot create its private file must leave the old target usable.
    // TypeScript pseudocode
    // let temporary = openPrivateTemporaryFile(runtime);
    let mut temporary = NamedTempFile::new_in(runtime)?;
    // What
    // `match` selects one branch for each case. `to_string` formats the integer as owned text.
    // Why
    // The daemon sees one simple, canonical token.
    // TypeScript pseudocode
    // const text = target.kind === "off" ? "off" : String(target.degrees);
    let text = match target {
        Target::Off => "off".to_owned(),
        Target::Celsius(degrees) => degrees.to_string(),
    };
    // Supply bytes to the file's write method and stop on a write failure.
    temporary.write_all(text.as_bytes())?;
    // What
    // `map_err` transforms only the error case. A closure written `|error| error.error`
    // extracts the I/O error from tempfile's rename error.
    // Why
    // Readers see either a complete previous target or a complete new one.
    // TypeScript pseudocode
    // renameOrThrowIoError(temporary, join(runtime, TARGET_FILE));
    temporary.persist(runtime.join(TARGET_FILE)).map_err(|error| error.error)?;
    // The rename made the target visible.
    return Ok(());
}

/// Read the desired target. A fresh runtime directory starts in the off state.
pub fn read_target(runtime: &Path) -> io::Result<Target> {
    // Distinguish a missing target from a file we cannot read.
    let text = match fs::read_to_string(runtime.join(TARGET_FILE)) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Target::Off),
        Err(error) => return Err(error),
    };
    // Treat an invalid token as failed control input so the service retains its existing caps.
    return parse_target(text.trim()).map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message));
}
