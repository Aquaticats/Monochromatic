//! Exact replies of the server-request policy,
//!  without a server.

use super::{ClientView, Decision, EDIT_REFUSAL, Effect, configuration_section, decide};
use crate::json_value;
use helix_lsp::{jsonrpc, lsp};
use serde_json::{Value, json};

fn params(value: Value) -> jsonrpc::Params {
    return match value {
        Value::Object(map) => jsonrpc::Params::Map(map),
        Value::Array(list) => jsonrpc::Params::Array(list),
        _ => jsonrpc::Params::None,
    };
}

fn settings() -> Value {
    return json_value!({ "scripted": { "flag": true, "nested": { "value": 42 } } });
}

fn decided(method: &str, value: Value) -> Decision {
    let settings = settings();
    let folders = vec![lsp::WorkspaceFolder {
        uri: lsp::Url::parse("file:///project").expect("folder address"),
        name: "project".to_string(),
    }];
    let client = ClientView {
        settings: Some(&settings),
        folders: &folders,
    };
    return decide(method, params(value), &client);
}

/// The reply as it appears on the wire,
///  without the envelope helix-lsp adds.
fn wire(decision: &Decision) -> Value {
    return match &decision.reply {
        Ok(result) => json_value!({ "result": result }),
        Err(error) => json_value!({ "error": error }),
    };
}

#[test]
fn workspace_edit_is_refused_with_a_normal_result() {
    let decision = decided(
        "workspace/applyEdit",
        json_value!({ "label": "scripted edit", "edit": { "changes": { "file:///project/a.rs": [
            { "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } }, "newText": "INJECTED " }
        ] } } }),
    );
    assert_eq!(
        wire(&decision),
        json_value!({ "result": { "applied": false, "failureReason": "read-only client: workspace edits are not applied" } }),
        "a server-initiated workspace edit was not refused"
    );
    assert_eq!(
        EDIT_REFUSAL,
        "read-only client: workspace edits are not applied"
    );
    assert_eq!(decision.effect, Effect::None);
    assert_eq!(decision.policy, "refused");
}

#[test]
fn configuration_returns_one_entry_per_item() {
    let decision = decided(
        "workspace/configuration",
        json_value!({ "items": [{ "section": "scripted" }, { "section": "scripted.nested.value" }, { "section": "missing" }, {}] }),
    );
    assert_eq!(
        wire(&decision),
        json_value!({ "result": [{ "flag": true, "nested": { "value": 42 } }, 42, null, settings()] })
    );
    assert_eq!(decision.policy, "answered");
}

#[test]
fn configuration_section_without_settings_is_null() {
    assert_eq!(configuration_section(None, Some("anything")), Value::Null);
    let settings = settings();
    assert_eq!(configuration_section(Some(&settings), Some("")), settings);
    assert_eq!(
        configuration_section(Some(&settings), Some("scripted.flag.deeper")),
        Value::Null
    );
}

#[test]
fn capability_registration_is_acknowledged_and_only_watchers_are_kept() {
    let decision = decided(
        "client/registerCapability",
        json_value!({ "registrations": [
            { "id": "watch-1", "method": "workspace/didChangeWatchedFiles", "registerOptions": { "watchers": [{ "globPattern": "**/*.scripted" }] } },
            { "id": "save-1", "method": "textDocument/didSave", "registerOptions": {} },
            { "id": "watch-2", "method": "workspace/didChangeWatchedFiles", "registerOptions": { "watchers": 5 } }
        ] }),
    );
    assert_eq!(wire(&decision), json!({ "result": null }));
    let Effect::RegisterWatchers(watchers) = &decision.effect else {
        panic!("registration produced {:?}", decision.effect);
    };
    assert_eq!(watchers.len(), 1);
    assert_eq!(watchers[0].0, "watch-1");
    assert_eq!(watchers[0].1.watchers.len(), 1);
}

#[test]
fn capability_unregistration_is_acknowledged() {
    let decision = decided(
        "client/unregisterCapability",
        json_value!({ "unregisterations": [
            { "id": "watch-1", "method": "workspace/didChangeWatchedFiles" },
            { "id": "save-1", "method": "textDocument/didSave" }
        ] }),
    );
    assert_eq!(wire(&decision), json!({ "result": null }));
    assert_eq!(
        decision.effect,
        Effect::UnregisterWatchers(vec!["watch-1".to_string()])
    );
}

#[test]
fn progress_creation_and_message_requests_get_null() {
    let progress = decided(
        "window/workDoneProgress/create",
        json_value!({ "token": "scripted-progress" }),
    );
    assert_eq!(wire(&progress), json!({ "result": null }));
    assert_eq!(progress.policy, "acknowledged");
    let message = decided(
        "window/showMessageRequest",
        json_value!({ "type": 3, "message": "Pick one", "actions": [{ "title": "Yes" }, { "title": "No" }] }),
    );
    assert_eq!(wire(&message), json!({ "result": null }));
    assert_eq!(message.effect, Effect::None);
}

#[test]
fn workspace_folders_are_the_clients_own() {
    let decision = decided("workspace/workspaceFolders", Value::Null);
    assert_eq!(
        wire(&decision),
        json_value!({ "result": [{ "uri": "file:///project", "name": "project" }] })
    );
}

#[test]
fn show_document_is_refused() {
    let decision = decided(
        "window/showDocument",
        json_value!({ "uri": "https://example.invalid/", "external": true }),
    );
    assert_eq!(wire(&decision), json!({ "result": { "success": false } }));
    assert_eq!(decision.policy, "refused");
}

#[test]
fn diagnostic_refresh_is_acknowledged_and_triggers_a_pull() {
    let decision = decided("workspace/diagnostic/refresh", Value::Null);
    assert_eq!(wire(&decision), json!({ "result": null }));
    assert_eq!(decision.effect, Effect::RefreshDiagnostics);
}

#[test]
fn unknown_method_gets_method_not_found() {
    let decision = decided("scripted/unknownMethod", json_value!({ "any": 1 }));
    assert_eq!(
        wire(&decision),
        json_value!({ "error": { "code": -32601, "message": "Method not found: scripted/unknownMethod" } })
    );
    assert_eq!(decision.policy, "method-not-found");
}

#[test]
fn malformed_parameters_get_invalid_params() {
    let decision = decided("workspace/applyEdit", json_value!({ "edit": 5 }));
    let reply = wire(&decision);
    assert_eq!(reply["error"]["code"], json_value!(-32602));
    let message = reply["error"]["message"].as_str().expect("error message");
    assert!(
        message.starts_with("Malformed workspace/applyEdit: "),
        "unexpected message: {message}"
    );
    assert_eq!(decision.policy, "malformed");
    assert_eq!(decision.effect, Effect::None);
}
