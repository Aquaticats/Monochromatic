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

/// Symlink-safe reads of the real filesystem.
pub mod real_fs;

/// The virtual view rules plan against.
pub mod view;

/// Glob expansion over the view.
pub mod view_glob;

/// The rule-kind interface.
pub mod rule;

/// Evaluation of value expressions.
pub mod value_eval;

/// Format-preserving TOML edits.
mod toml_ops;

/// The `concat` kind.
pub mod kind_concat;

/// The `mode-only` kind.
pub mod kind_mode_only;

/// The `mise-path-list` kind.
pub mod kind_mise_path_list;

/// The `package-licenses` kind.
pub mod kind_package_licenses;

/// The `mirror` kind.
pub mod kind_mirror;

/// The `toml-keys` kind.
pub mod kind_toml_keys;

/// The rule registry and the placeholder for unlinked kinds.
pub mod registry;

/// Planning and whole-run validation.
pub mod plan;

/// Drift detection.
pub mod drift;

/// Publishing through hidden staging names.
pub mod publish;

/// Paired actual and expected trees for diff tools.
pub mod expected_dir;

/// `explain`.
pub mod explain;

/// Running one command.
pub mod run;

/// JSON object construction without the `json!` macro.
pub mod json_build;

/// JSON-RPC message lines.
pub mod rpc_protocol;

/// Runtime directory and socket naming.
pub mod runtime_dir;

/// Build and worktree identities for the handshake.
pub mod identity;

/// Per-worktree and per-user endpoints.
pub mod endpoint;

/// Server startup lock and spawn.
pub mod launch;

/// The server.
pub mod server;

/// One server connection.
pub mod server_connection;

/// Server-side watch.
pub mod watch;

/// The client.
pub mod client;

#[cfg(test)]
mod test_support;

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

#[cfg(test)]
#[path = "kinds_tests.rs"]
mod kinds_tests;

#[cfg(test)]
#[path = "run_tests.rs"]
mod run_tests;

#[cfg(test)]
#[path = "schema_tests.rs"]
mod schema_tests;

#[cfg(test)]
#[path = "server_tests.rs"]
mod server_tests;
