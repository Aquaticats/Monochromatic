//! What:
//!  The gate that asks the repository link check before a command touches the repository.
//! Why:
//!  Nothing may be created in the wrong place when the link dangles.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gate = repositoryLinkGate({ home, paths });
//! ```

use crate::repository_link_probe::LinkProbe;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// What:
///  What the gate decided.
/// Why:
///  The caller continues or prints the text and exits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkGate = { kind: 'clear' } | { kind: 'stop'; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkGate {
    /// The command goes on.
    Clear,
    /// The command stops with this standard error and status 128.
    Stop(String),
}

/// What:
///  Runs the gate with a probe.
/// Why:
///  Tests count the probe's calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gateWithProbe({ probe, home, paths })
/// ```
pub fn link_gate_with_probe(probe: Arc<dyn LinkProbe + Send + Sync>, home: Option<&Path>, paths: &[PathBuf]) -> LinkGate {
    todo!("{} {home:?} {}", Arc::strong_count(&probe), paths.len())
}

/// Gate controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_gate_tests.rs"]
mod tests;
