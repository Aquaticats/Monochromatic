//! What:
//!  Tests that run the real `file-enforcer` binary:
//! JSON-lines output,
//! exit codes,
//! the auto-started server.
//! Why:
//!  The command line,
//! the server lifecycle and the JSON-lines contract only exist in the built executable,
//! so these
//!  tests start it as a user would,
//! in a scratch worktree with its own runtime directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('file-enforcer binary', () => { it('prints help as one JSON line', () => { ... }); });
//! ```
//!
//! Each module below covers one command or one mechanism,
//! and every test it holds runs the built binary.

mod support;

/// The binary tests for the command line itself.
#[path = "binary/cli_contract.rs"]
mod cli_contract;

/// The binary tests for the server a command starts on demand.
#[path = "binary/server_lifecycle.rs"]
mod server_lifecycle;

/// The binary test for where the server keeps its socket.
#[path = "binary/runtime_directory.rs"]
mod runtime_directory;

/// The binary tests for the two ways a config file can be wrong.
#[path = "binary/config_errors.rs"]
mod config_errors;

/// The binary tests for what a check writes and which drift it reports.
#[path = "binary/check_outputs.rs"]
mod check_outputs;

/// The binary test for the watch command.
#[path = "binary/watch.rs"]
mod watch;

/// The binary tests for the rule kinds that need a linked library.
#[path = "binary/rule_kinds.rs"]
mod rule_kinds;
