//! Names the tool in prose only,
//! so the start written below is a comment and not a start.
//!
//! This file is a fixture of `rust-sources-in-the-scan.unit.test.ts`:
//! it writes the start shape in code the scan must not read as code,
//! so the answer for it is no start at all.

// A line comment naming the same start: Command::new("podman").args(["run"])
/* A block comment naming it too:
Command::new("podman").args(["run"]) */
/** A block doc comment naming it:
Command::new("podman").arg("run") */
/* An example with a nested comment:
/* nested */ Command::new("podman").args(["run"]) */

/// The image name this module only words,
/// and the start it would take.
pub const IMAGE: &str = "localhost/x:current";

/// The verb this module only words.
pub const VERB: &str = "run";
