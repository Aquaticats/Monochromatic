//! Server-to-client requests:
//!  every one is answered by policy,
//!  and every edit is refused.

use crate::support::{self, PRODUCT_TIMEOUT, Probe};
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
use ide_app::language::reply::{RequestKind, RequestOutcome};
use serde_json::{Value, json};

/// True when the scripted server reported the end of its probe.
fn probe_done(lines: &[Value]) -> bool {
    for line in lines {
        if line["probe-done"] == true {
            return true;
        }
    }
    return false;
}

/// The reply the scripted server recorded for a method;
///  the second `workspace/applyEdit` is the malformed one.
fn replies(lines: &[Value], method: &str) -> Vec<Value> {
    let mut found = Vec::new();
    for line in lines {
        if line["probe"]["method"] == method {
            found.push(line["probe"]["response"].clone());
        }
    }
    return found;
}

/// The server sends every request it can;
///  the client refuses the edit,
///  answers the rest,
///  and goes on.
#[test]
fn server_requests_get_policy_replies_and_edits_change_nothing() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "policy::server_requests_get_policy_replies_and_edits_change_nothing",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("PROBE", "1"), ("PULL", "1")], PRODUCT_TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    let file = root.join("guarded.scripted");
    let original = "original bytes \u{e9}\nsecond line\n";
    probe.open(&file, original);
    probe.until_ready();
    let mut probe_wait = support::ReportWait::new(&root, "the end of the server's probe");
    while !probe_done(probe_wait.lines()) {
        probe_wait.again();
    }
    let lines = probe_wait.into_lines();
    let edits = replies(&lines, "workspace/applyEdit");
    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0],
        json_value!({ "result": { "applied": false, "failureReason": "read-only client: workspace edits are not applied" } }),
        "a server-initiated workspace edit was not refused"
    );
    assert_eq!(edits[1]["error"]["code"], -32602, "{}", edits[1]);
    assert_eq!(
        std::fs::read(&file).expect("guarded file"),
        original.as_bytes(),
        "a server-initiated edit changed the file"
    );
    assert_eq!(
        replies(&lines, "window/workDoneProgress/create"),
        [json!({ "result": null })]
    );
    assert_eq!(
        replies(&lines, "workspace/configuration"),
        [json_value!({ "result": [{ "flag": true, "nested": { "value": 42 } }, 42, null] })]
    );
    assert_eq!(
        replies(&lines, "client/registerCapability"),
        [json!({ "result": null })]
    );
    assert_eq!(
        replies(&lines, "workspace/workspaceFolders"),
        [
            json_value!({ "result": [{ "uri": format!("file://{}", root.display()), "name": "project" }] })
        ]
    );
    assert_eq!(
        replies(&lines, "window/showMessageRequest"),
        [json!({ "result": null })]
    );
    assert_eq!(
        replies(&lines, "window/showDocument"),
        [json!({ "result": { "success": false } })]
    );
    assert_eq!(
        replies(&lines, "workspace/diagnostic/refresh"),
        [json!({ "result": null })]
    );
    assert_eq!(
        replies(&lines, "scripted/unknownMethod"),
        [
            json_value!({ "error": { "code": -32601, "message": "Method not found: scripted/unknownMethod" } })
        ]
    );
    let mut unanswered: Vec<&Value> = Vec::new();
    for line in lines.iter() {
        if line["probe"]["response"]["unanswered"] == true {
            unanswered.push(line);
        }
    }
    assert!(
        unanswered.is_empty(),
        "a server request was left unanswered: {unanswered:?}"
    );
    // The refresh request makes the client pull diagnostics a second time.
    let mut pull_wait = support::ReportWait::new(&root, "a second diagnostics pull");
    while support::count_received(pull_wait.lines(), "textDocument/diagnostic") < 2 {
        pull_wait.again();
    }
    let pulls = pull_wait.into_lines();
    assert!(!pulls.is_empty());
    // The session continues: requests still work and the registered watcher is notified of the reload.
    let number = probe.request(RequestKind::Hover, 2);
    assert!(matches!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Hover(_)
    ));
    probe.reload("reloaded by the test\n");
    let mut watched_wait = support::ReportWait::new(&root, "a watched-file notification");
    while support::count_received(watched_wait.lines(), "workspace/didChangeWatchedFiles") == 0 {
        watched_wait.again();
    }
    let watched = watched_wait.into_lines();
    let event = support::first_received(&watched, "workspace/didChangeWatchedFiles")
        .expect("watched-file notification");
    assert_eq!(
        event["params"]["changes"][0]["uri"],
        format!("file://{}", file.display()).as_str()
    );
    assert_eq!(event["params"]["changes"][0]["type"], 2);
}
