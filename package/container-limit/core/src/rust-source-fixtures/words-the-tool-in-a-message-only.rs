//! Words the tool in a message a program prints,
//! and starts nothing.
//!
//! This file is a fixture of `rust-sources-in-the-scan.unit.test.ts`.
//! A string that carries `podman run` is a command line in a `mise.toml` or a workflow,
//! where the scan reads it as a start;
//! here the same words are a message and a note,
//! and the answer the scan must give is no start at all.

/// The message a run prints when a call of the tool it names fails.
pub fn failure(reason: &str) -> String {
    return format!("podman run failed: {reason}");
}

/// The command line a person would type,
/// which this module never runs.
pub const HINT: &str = "podman run --rm localhost/x:current";
