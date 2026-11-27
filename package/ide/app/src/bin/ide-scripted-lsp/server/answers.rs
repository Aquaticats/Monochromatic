//! What the scripted server answers while it runs:
//!  the start gate and the capabilities of `initialize`,
//!  one written message,
//!  a document's diagnostics,
//!  and a hover answer.

/// The loop's state, whose methods these are.
use super::Session;
/// Position arithmetic for a hover answer.
use crate::document::{
    line_at,
    offset_at,
};
/// How this run answers a hover,
///  and what it announces before anything else.
use crate::script::{
    Hover,
    Script,
};
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// JSON values and the literal-building macro.
use serde_json::{
    Value,
    json,
};
/// The gate file of a held start,
///  the shared output stream,
///  and the thread a delayed answer waits on.
use std::{
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

/// What:
///  Wait until a file exists,
///  checking every 5 ms,
///  for at most one minute.
///  `&Path` lends
///       the path (sibling:
///  owned `PathBuf`).
/// Why:
///  The read loop holds `initialize` until the test allows the start;
///  a test that never
///      creates the file still ends,
///  because the client's start deadline stops the server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitFor(gate: string) { for (let i = 0; i < 12_000 && !existsSync(gate); i++) sleepSync(5); }
/// ```
pub(super) fn wait_for(gate: &Path) {
    for _ in 0..12_000 {
        if gate.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// Capabilities announced in the `initialize` answer.
pub(super) fn capabilities(script: &Script) -> Value {
    // `mut` allows adding optional members below.
    let mut result = json!({ "hoverProvider": true });
    if let Some(encoding) = script.encoding {
        result["positionEncoding"] = json_value!(match encoding {
            crate::script::Unit::Utf8 => "utf-8",
            crate::script::Unit::Utf16 => "utf-16",
            crate::script::Unit::Utf32 => "utf-32",
        });
    }
    if let Some(change) = script.sync {
        result["textDocumentSync"] = json_value!({ "openClose": true, "change": change });
        if script.save {
            result["textDocumentSync"]["save"] = json!(true);
        }
    }
    if script.all_features {
        result["definitionProvider"] = json!(true);
        result["referencesProvider"] = json!(true);
        result["inlayHintProvider"] = json!({ "resolveProvider": true });
    }
    if script.pull_diagnostics {
        result["diagnosticProvider"] = json_value!({ "identifier": "scripted", "interFileDependencies": false, "workspaceDiagnostics": false });
    }
    return result;
}

/// Session behavior,
///  the answers this server writes while it runs.
impl Session {
    /// What:
    ///  Send a message,
    ///  turning a write failure into a line on standard error.
    /// Why:
    ///  A closed pipe ends the loop at the next read;
    ///  one failed write needs no other handling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// send(message: object) { try { wire.send(message); } catch (error) { console.error(error); } }
    /// ```
    pub(super) fn send(
        &self,
        message: Value,
    ) {
        if let Err(error) = self.wire.send(message) {
            eprintln!("scripted language server cannot write: {error}");
        }
    }

    /// Record the server's copy of a document and,
    ///  when configured,
    ///  push diagnostics that quote it.
    pub(super) fn publish(
        &self,
        uri: &str,
        version: &Value,
    ) {
        // `get` returns `Option<&String>`; `map_or` substitutes empty text for an unknown document.
        let text = match self.documents.get(uri) {
            Some(found) => found.as_str(),
            None => "",
        };
        self.wire
            .record(json_value!({ "text": { "uri": uri, "version": version, "text": text } }));
        if !self.script.push_diagnostics {
            return;
        }
        let mut params = json_value!({
            "uri": uri,
            "diagnostics": [{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
                "severity": 2,
                "source": "scripted",
                "message": format!("TEXT:{text}"),
            }],
        });
        if self.script.versioned_diagnostics {
            // `clone` copies the JSON number so the notification owns its own value.
            params["version"] = version.clone();
        }
        self.send(json_value!({ "method": "textDocument/publishDiagnostics", "params": params }));
    }

    /// What:
    ///  Answer a hover request according to the script.
    ///  `&mut self` allows counting requests.
    /// Why:
    ///  Hover is the request tests use for delay,
    ///  crash,
    ///  supersession,
    ///  and silence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hover(id: unknown, params: HoverParams): void
    /// ```
    pub(super) fn hover(
        &mut self,
        id: &Value,
        params: &Value,
    ) {
        self.hovers += 1;
        let modified = self.script.hover == Hover::Modified
            || (self.script.hover == Hover::ModifiedOnce && self.hovers == 1);
        if self.script.hover == Hover::Crash {
            // The configured line comes right before the end, as a crashing server's last words.
            if let Some(line) = &self.script.stderr_at_shutdown {
                eprintln!("{line}");
            }
            std::process::exit(7);
        }
        if self.script.hover == Hover::Silent {
            return;
        }
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let text = match self.documents.get(uri) {
            Some(found) => found.as_str(),
            None => "",
        };
        let position = &params["position"];
        let offset = offset_at(
            text,
            position["line"].as_u64().unwrap_or(0),
            position["character"].as_u64().unwrap_or(0),
            self.script.unit(),
        );
        let line = line_at(text, offset);
        let answer = if modified {
            // The answer a server gives when its own state moved under the request.
            json_value!({ "id": id, "error": { "code": -32801, "message": "content modified" } })
        } else if line.trim().is_empty() {
            json_value!({ "id": id, "result": null })
        } else {
            // `chars().next()` reads the character at the offset, if any; `map_or` renders it as text.
            let character = match text[offset..].chars().next() {
                Some(found) => found.to_string(),
                None => String::new(),
            };
            json_value!({ "id": id, "result": {
                "contents": { "kind": "markdown", "value": format!("line={line} char={character}") },
                "range": { "start": position, "end": position },
            } })
        };
        if self.script.hover_delay == 0 {
            self.send(answer);
            if self.script.push_after_hover {
                // An unversioned push that arrives after an answer, as delayed analysis produces.
                self.publish(uri, &Value::Null);
            }
            return;
        }
        // What: `Arc::clone` copies the shared pointer; `thread::spawn(move || ...)` runs the
        //       closure on a new thread, moving the captured values into it.
        // Why: The answer was computed from the text at request time and is sent late, so a
        //      change notification can overtake it, which is the stale-reply case.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // setTimeout(() => wire.send(answer), hoverDelay);
        // ```
        let wire = Arc::clone(&self.wire);
        let delay = Duration::from_millis(self.script.hover_delay);
        thread::spawn(move || {
            thread::sleep(delay);
            if let Err(error) = wire.send(answer) {
                eprintln!("scripted language server cannot write a delayed answer: {error}");
            }
        });
    }
}
