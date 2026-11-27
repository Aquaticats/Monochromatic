//! What:
//!  The name a removal's refusal is known by in this folder and outside it:
//!  `OriginRefusal`,
//!  re-exported from the file that now holds it.
//! Why:
//!  The refusal names the identities a removal planned,
//!  and the file that plans them answers a plan that lacks one with
//!  `OriginRefusal::PlanIncomplete`,
//!  so the two files named each other.
//!  The type now sits in `planned_origin.rs` and this file names it for the
//!  callers that reach it here:
//!  the folder's own files,
//!  the two removals outside it,
//!  and the controls this file's sidecar declares.
//!  That leaves one direction of use between the two files
//!  (`AGENTS.md` rule LYF;
//!  `doc/planning/cli-git-source-layout-target-tree.md`,
//!  "Loops between files,
//!  and how each is broken (2026-11-26)",
//!  whose break moved each cause into the file that plans the identities it
//!  names).
//!  The cause carries the planned tuple rather than a sentence built where the
//!  object was read,
//!  so the words live in one set of files and the same cause reads the same at
//!  any call site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export { OriginRefusal } from './planned_origin.ts';
//! ```

pub use super::planned_origin::OriginRefusal;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_refusal_tests.rs"]
mod tests;
