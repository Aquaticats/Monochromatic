//! What:
//!  `monochromatic-file-enforcer` is the engine behind the `file-enforcer` command: it reads the JSONC config, plans
//!  every rule against a virtual view of the worktree, validates the whole run, publishes generated files one by one,
//!  and serves all of that over a per-worktree Unix socket.
//! Why:
//!  The engine has no command-line parsing so that other tools (meow) can link it; the `file-enforcer` binary in
//!  `package/file-enforcer/cli` is a thin client over it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { runRules, startServer } from '@monochromatic-dev/file-enforcer-core-rust';
//! ```

/// Diagnostics: a failure or warning before it becomes an event.
pub mod diagnostic;

/// Events: the JSON object each output line is, and the sinks that receive them.
pub mod event;

/// Config types shared by the decoder and the derived schema.
pub mod config_types;

/// Value expressions and rule building blocks.
pub mod config_parts;

/// Field readers that collect decode problems.
pub mod config_reader;

/// Per-kind rule decoding.
mod config_decode_rules;

/// Top-level config decoding.
pub mod config_decode;

/// JSONC file loading with positioned errors.
pub mod config_load;

/// The derived JSON Schema.
pub mod schema;

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
