//! The notifications the client sends:
//!  a document opened,
//!  changed,
//!  or closed,
//!  and the end of the session.

/// The loop's state, whose method this is.
use super::Session;
/// Text mirroring for a document the client changed.
use crate::document::apply_change;
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// The notification's own message.
use serde_json::Value;
/// The shared output stream,
///  and the threads the client probe and the registration run on.
use std::{
    sync::Arc,
    thread,
};

/// Notification handling,
///  what each message the client sends does.
impl Session {
    /// React to one notification from the client.
    pub(super) fn notification(
        &mut self,
        method: &str,
        params: &Value,
    ) {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let version = &params["textDocument"]["version"];
        if method == "exit" {
            // A lingering server stays; the client then has to kill it.
            if !self.script.linger {
                std::process::exit(0);
            }
        } else if method == "textDocument/didOpen" {
            let text = params["textDocument"]["text"].as_str().unwrap_or("");
            self.documents.insert(uri.to_string(), text.to_string());
            self.publish(uri, version);
            if self.script.probe && !self.probed {
                self.probed = true;
                let wire = Arc::clone(&self.wire);
                let target = uri.to_string();
                thread::spawn(move || {
                    crate::probe::run(&wire, &target);
                });
            }
        } else if method == "textDocument/didChange" {
            // `entry(...).or_default()` borrows the stored text for editing, creating empty text first if absent.
            let text = self.documents.entry(uri.to_string()).or_default();
            // `as_array()` is `Some` for a JSON array; `into_iter().flatten()` walks its items or nothing.
            for change in params["contentChanges"].as_array().into_iter().flatten() {
                apply_change(text, change, self.script.unit());
            }
            self.publish(uri, version);
        } else if method == "textDocument/didClose" {
            self.documents.remove(uri);
        } else if method == "initialized"
            && let Some(watchers) = self.script.watchers.clone()
        {
            // Registered from its own thread, because the reply arrives through this read loop.
            let wire = Arc::clone(&self.wire);
            thread::spawn(move || {
                let reply = wire.ask(
                    "client/registerCapability",
                    json_value!({ "registrations": [{ "id": "scripted-watchers", "method": "workspace/didChangeWatchedFiles", "registerOptions": { "watchers": watchers } }] }),
                );
                wire.record(json_value!({ "registered-watchers": reply }));
            });
        }
    }
}
