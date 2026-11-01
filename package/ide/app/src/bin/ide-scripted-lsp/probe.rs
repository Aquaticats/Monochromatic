//! Every request a server can send to the client,
//!  sent once so a test can check each reply.

/// The shared output stream,
///  report file,
///  and reply table.
use crate::framing::Wire;
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// JSON values and the literal-building macro.
use serde_json::{Value, json};

/// What:
///  Send one request,
///  wait for its reply,
///  and record both.
///  `&Wire` lends the shared stream;
///       `&str` borrows the method name;
///  `params` is moved in.
/// Why:
///  The report line is what the test asserts:
///  an exact reply for each method,
///  and none missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function probeOne(wire: Wire, method: string, params: unknown) {
///   wire.record({ probe: { method, response: await wire.ask(method, params) } });
/// }
/// ```
fn probe_one(wire: &Wire, method: &str, params: Value) {
    let response = wire.ask(method, params);
    wire.record(json_value!({ "probe": { "method": method, "response": response } }));
}

/// What:
///  Send a notification,
///  reporting a write failure on standard error.
/// Why:
///  Notifications have no reply;
///  the client must simply not stall or fail on them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notify(wire: Wire, method: string, params: unknown) { wire.send({ method, params }); }
/// ```
fn notify(wire: &Wire, method: &str, params: Value) {
    if let Err(error) = wire.send(json_value!({ "method": method, "params": params })) {
        eprintln!("scripted language server cannot notify the client: {error}");
    }
}

/// What:
///  Run the whole probe against the document at `uri`.
/// Why:
///  It covers the edit request the read-only client must refuse,
///  every request it must
///      answer,
///  an unknown method,
///  malformed parameters,
///  and notifications it must tolerate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function run(wire: Wire, uri: string): Promise<void>
/// ```
pub fn run(wire: &Wire, uri: &str) {
    probe_one(
        wire,
        "window/workDoneProgress/create",
        json_value!({ "token": "scripted-progress" }),
    );
    notify(
        wire,
        "$/progress",
        json_value!({ "token": "scripted-progress", "value": { "kind": "begin", "title": "Scripted indexing" } }),
    );
    notify(
        wire,
        "$/progress",
        json_value!({ "token": "scripted-progress", "value": { "kind": "end" } }),
    );
    probe_one(
        wire,
        "workspace/configuration",
        json_value!({ "items": [{ "section": "scripted" }, { "section": "scripted.nested.value" }, { "section": "missing" }] }),
    );
    probe_one(
        wire,
        "client/registerCapability",
        json_value!({ "registrations": [
            { "id": "watch-1", "method": "workspace/didChangeWatchedFiles", "registerOptions": { "watchers": [{ "globPattern": "**/*.scripted" }] } },
            { "id": "save-1", "method": "textDocument/didSave", "registerOptions": {} }
        ] }),
    );
    probe_one(wire, "workspace/workspaceFolders", Value::Null);
    probe_one(
        wire,
        "window/showMessageRequest",
        json_value!({ "type": 3, "message": "Pick one", "actions": [{ "title": "Yes" }, { "title": "No" }] }),
    );
    probe_one(
        wire,
        "window/showDocument",
        json_value!({ "uri": "https://example.invalid/", "external": true }),
    );
    // The one request that would change a project file if the client applied it.
    probe_one(
        wire,
        "workspace/applyEdit",
        json_value!({ "label": "scripted edit", "edit": { "changes": { uri: [
            { "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } }, "newText": "INJECTED " }
        ] } } }),
    );
    probe_one(wire, "workspace/applyEdit", json_value!({ "edit": 5 }));
    probe_one(wire, "workspace/diagnostic/refresh", Value::Null);
    probe_one(wire, "scripted/unknownMethod", json_value!({ "any": 1 }));
    notify(
        wire,
        "scripted/unknownNotification",
        json_value!({ "any": 1 }),
    );
    notify(
        wire,
        "window/showMessage",
        json_value!({ "type": 3, "message": "scripted message" }),
    );
    notify(
        wire,
        "window/logMessage",
        json_value!({ "type": 4, "message": "scripted log line" }),
    );
    wire.record(json!({ "probe-done": true }));
}
