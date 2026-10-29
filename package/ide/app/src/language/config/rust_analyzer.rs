//! rust-analyzer hears about file changes from the IDE instead of watching the project itself.
//!
//! Helix's built-in definition sets `files.watcher = "server"`,
//!  because Helix watches no files for its
//! servers.
//!  rust-analyzer then asks notify for a recursive watch of every workspace package folder,
//!  which
//! watches every folder below it,
//!  `node_modules` included,
//!  whatever `files.excludeDirs` says (measured:
//! 3081 watches on a 40-crate workspace with one 2001-folder `node_modules`).
//!  The IDE watches the
//! project's source folders for every server and forwards changes,
//!  so rust-analyzer is told to register
//! watchers with the client (`files.watcher = "client"`) and takes no watches of its own.

/// Helix's typed configuration.
use helix_core::syntax::config::Configuration;
/// What:
///  `Value` is any JSON value,
///  and `Value::from` wraps a string in one;
///  `json!` builds a JSON value from literal syntax.
/// Why:
///  The server's settings table is JSON.
///  A string is wrapped with `Value::from` because `json!` unwraps a `Result` when it expands a string,
///  and the repository's clippy configuration bans that call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settings = {};
/// ```
use serde_json::{Value, json};

/// Name of Helix's rust-analyzer definition.
const SERVER: &str = "rust-analyzer";

/// What:
///  Set `files.watcher = "client"` in rust-analyzer's settings,
///  keeping every other key.
/// Why:
///  Set in code after the definitions are merged,
///  because a merged `files` table replaces the whole
///      table,
///  so a key set only in TOML could be lost together with its siblings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchThroughClient(configuration: Configuration): void
/// ```
pub(super) fn watch_through_client(configuration: &mut Configuration) {
    // What: `get_mut` borrows the definition for modification, or gives `None` when there is none.
    // Why: A configuration without rust-analyzer needs nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const definition = configuration.languageServer[SERVER]; if (!definition) return;
    // ```
    let Some(definition) = configuration.language_server.get_mut(SERVER) else {
        return;
    };
    // `get_or_insert` gives the existing settings, or stores an empty table first.
    let settings = definition.config.get_or_insert(json!({}));
    // What: `is_object` checks the JSON shape; indexing a JSON object with `["files"]` inserts a missing key.
    // Why: Indexing a value of another shape would stop the program, so a malformed table is replaced.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (typeof settings !== 'object') settings = {}; settings.files ??= {};
    // ```
    if !settings.is_object() {
        tracing::warn!(
            "rust-analyzer's settings are not a table; replacing them so it watches through the IDE"
        );
        *settings = json!({});
    }
    if !settings["files"].is_object() {
        settings["files"] = json!({});
    }
    settings["files"]["watcher"] = Value::from("client");
}

/// The setting rust-analyzer is given,
///  with and without merged definitions.
#[cfg(test)]
#[path = "rust_analyzer_tests.rs"]
mod tests;
