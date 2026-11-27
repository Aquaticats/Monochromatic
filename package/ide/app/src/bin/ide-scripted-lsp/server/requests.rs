//! The requests the client sends:
//!  `initialize`,
//!  `shutdown`,
//!  the five feature requests,
//!  and the error every other method answers with.

/// The loop's state, whose method this is.
use super::Session;
/// The answers this dispatch hands off to.
use super::answers::{
    capabilities,
    wait_for,
};
/// The script's start behavior,
///  and how it answers each request.
use crate::script::{
    Init,
    Step,
    step_for,
};
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// The request's own message.
use serde_json::Value;
/// The thread a slow start sleeps on.
use std::{
    thread,
    time::Duration,
};

/// Request dispatch,
///  the one place that names what each method answers.
impl Session {
    /// Answer one request from the client.
    pub(super) fn request(
        &mut self,
        id: &Value,
        method: &str,
        params: &Value,
    ) {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        if method == "initialize" {
            match self.script.init {
                Init::Hang => {}
                Init::Exit => std::process::exit(3),
                Init::Error => self.send(
                    json_value!({ "id": id, "error": { "code": -32603, "message": "scripted initialize failure" } }),
                ),
                Init::Ok => {
                    // A gated start waits for the test's file, for at most one minute.
                    if let Some(gate) = &self.script.init_gate {
                        wait_for(gate);
                    }
                    // A slow start: the client must not send anything else in the meantime.
                    thread::sleep(Duration::from_millis(self.script.init_delay));
                    self.send(json_value!({ "id": id, "result": {
                        "capabilities": capabilities(&self.script),
                        "serverInfo": { "name": "ide-scripted-lsp" },
                    } }));
                }
            }
        } else if method == "shutdown" {
            // `if let Some` writes the configured line only when one was given.
            if let Some(line) = &self.script.stderr_at_shutdown {
                eprintln!("{line}");
            }
            self.send(json_value!({ "id": id, "result": null }));
        } else if method == "textDocument/hover" {
            self.hover(id, params);
        } else if method == "textDocument/definition"
            && let Some(result) = &self.script.definition
        {
            // A scripted result is sent exactly as the test wrote it.
            self.send(json_value!({ "id": id, "result": result }));
        } else if method == "textDocument/references"
            && let Some(result) = &self.script.references
        {
            self.send(json_value!({ "id": id, "result": result }));
        } else if method == "textDocument/definition" {
            let range = json_value!({ "start": { "line": 2, "character": 0 }, "end": { "line": 2, "character": 6 } });
            let mut targets = vec![
                json_value!({ "uri": uri, "range": range }),
                json_value!({ "uri": "untitled:Untitled-1", "range": range }),
                json_value!({ "uri": "jdt://contents/java.base/java/lang/String.class", "range": range }),
                json_value!({ "uri": "file:///definitely/missing/file.rs", "range": range }),
            ];
            if let Some(outside) = &self.script.outside {
                targets.push(json_value!({
                    "uri": format!("file://{}", outside.display()),
                    "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
                }));
            }
            self.send(json_value!({ "id": id, "result": targets }));
        } else if method == "textDocument/references" {
            self.send(json_value!({ "id": id, "error": { "code": -32603, "message": "scripted internal failure" } }));
        } else if method == "textDocument/inlayHint" {
            self.inlay_hints += 1;
            // `&self.script.hint_steps` lends the scripted list; the count picks this request's step.
            let step = step_for(&self.script.hint_steps, self.inlay_hints);
            // `&mut self.held_hint` lends the slot of a held-back answer, which `respond` fills or empties.
            let first_line = params["range"]["start"]["line"].as_u64().unwrap_or(0);
            crate::inlay::respond(&self.wire, id, step, &mut self.held_hint, first_line);
        } else if method == "textDocument/diagnostic" {
            self.pulls += 1;
            // A scripted silence leaves this pull unanswered; the client's timeout decides.
            if step_for(&self.script.pull_steps, self.pulls) == Step::Silent {
                return;
            }
            let text = match self.documents.get(uri) {
                Some(found) => found.as_str(),
                None => "",
            };
            self.send(json_value!({ "id": id, "result": { "kind": "full", "items": [{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
                "severity": 1,
                "source": "scripted-pull",
                "message": format!("PULLED:{text}"),
            }] } }));
        } else {
            self.send(json_value!({ "id": id, "error": { "code": -32601, "message": format!("scripted server has no {method}") } }));
        }
    }
}
